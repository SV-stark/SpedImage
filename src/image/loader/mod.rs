mod codecs;

use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;
use zune_image::image::Image;

use super::types::{ImageData, ImageFormatType};

/// How many frames to decode from animated formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameLimit {
    /// Decode every frame.
    All,
    /// Stop after the first frame (thumbnails).
    First,
}

/// Optional work toggles applied while decoding.
#[derive(Debug, Clone, Copy)]
pub struct LoadOptions {
    pub frames: FrameLimit,
    /// When false, EXIF orientation is reported via `ImageData::orientation_deg`
    /// instead of being baked into the pixel buffer (the GPU rotates instead).
    pub bake_orientation: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            frames: FrameLimit::All,
            bake_orientation: true,
        }
    }
}

pub struct ImageLoader;

/// Map an EXIF orientation value to a rotation in degrees.
///
/// Only the pure rotations are representable on the GPU; the mirrored values
/// (2, 4, 5, 7) are baked into the pixels by [`exif_orientation_transform`].
pub(crate) fn exif_orientation_to_degrees(o: u32) -> Option<u16> {
    match o {
        3 => Some(180),
        6 => Some(90),
        8 => Some(270),
        _ => None,
    }
}

/// Full EXIF orientation transform as `(rotate_cw_degrees, mirror_h, mirror_v)`.
///
/// The mirror is applied to the stored pixels *before* the rotation, which is
/// what distinguishes orientation 5 (main-diagonal transpose) from 7
/// (anti-diagonal transverse).
pub(crate) fn exif_orientation_transform(o: u32) -> Option<(u32, bool, bool)> {
    match o {
        2 => Some((0, true, false)),
        3 => Some((180, false, false)),
        4 => Some((0, false, true)),
        5 => Some((270, true, false)),
        6 => Some((90, false, false)),
        7 => Some((90, true, false)),
        8 => Some((270, false, false)),
        _ => None,
    }
}

/// Apply an EXIF orientation to a decoded buffer.
pub(crate) fn apply_orientation(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    orientation: u32,
    bake: bool,
) -> (Vec<u8>, u32, u32, u16) {
    let Some((deg, mirror_h, mirror_v)) = exif_orientation_transform(orientation) else {
        return (rgba, width, height, 0);
    };

    // A rotation the GPU can do for free stays deferred; a mirror cannot.
    if !bake && let Some(d) = exif_orientation_to_degrees(orientation) {
        return (rgba, width, height, d);
    }

    let mut buf = rgba;
    let (mut w, mut h) = (width, height);
    super::processing::ImageProcessor::flip_rgba(&mut buf, w, h, mirror_h, mirror_v);
    if deg != 0 {
        let (rotated, rw, rh) =
            super::processing::ImageProcessor::rotate_rgba(&buf, w, h, deg as i32);
        buf = rotated;
        w = rw;
        h = rh;
    }
    (buf, w, h, 0)
}

/// Resolve the decode budget into a `(target_w, target_h)` box.
pub(crate) fn decode_box(max_w: Option<u32>, max_h: Option<u32>) -> Option<(u32, u32)> {
    match (max_w, max_h) {
        (Some(mw), Some(mh)) if mw > 0 && mh > 0 => Some((mw, mh)),
        _ => None,
    }
}

/// Swap the box bounds for sideways EXIF orientations so the decoded buffer is
/// measured in the same axis order as the display.
pub(crate) fn orient_box(
    box_wh: Option<(u32, u32)>,
    orientation: Option<u32>,
) -> Option<(u32, u32)> {
    let (mw, mh) = box_wh?;
    if orientation
        .and_then(exif_orientation_to_degrees)
        .is_some_and(|d| d == 90 || d == 270)
    {
        Some((mh, mw))
    } else {
        Some((mw, mh))
    }
}

/// Largest DCT decode downscale (1/8, 1/4, 1/2, 1/1) that still leaves the frame
/// at least as large as the final fit target.
pub(crate) fn pick_dct_scale(
    src_w: u32,
    src_h: u32,
    target_w: u32,
    target_h: u32,
) -> libjpeg_turbo_rs::ScalingFactor {
    let denom = match decode_box(Some(target_w), Some(target_h)) {
        Some(_) => {
            let ratio = (src_w as f64 / target_w as f64).max(src_h as f64 / target_h as f64);
            let mut d = 8u32;
            while ratio < d as f64 && d > 1 {
                d /= 2;
            }
            d
        }
        None => 1,
    };
    libjpeg_turbo_rs::ScalingFactor::new(1, denom)
}

/// Aspect-fit `rgba` into the `(target_w, target_h)` box.
pub(crate) fn downsample_to_fit(
    rgba: Vec<u8>,
    src_w: u32,
    src_h: u32,
    box_wh: Option<(u32, u32)>,
) -> Result<(Vec<u8>, u32, u32, bool)> {
    let Some((target_w, target_h)) = box_wh else {
        return Ok((rgba, src_w, src_h, false));
    };
    if src_w == 0 || src_h == 0 || (src_w <= target_w && src_h <= target_h) {
        return Ok((rgba, src_w, src_h, false));
    }

    let ratio = (src_w as f64 / target_w as f64).max(src_h as f64 / target_h as f64);
    let dst_w = ((src_w as f64 / ratio).round() as u32).max(1);
    let dst_h = ((src_h as f64 / ratio).round() as u32).max(1);

    use fast_image_resize as fr;
    let src_image = fr::images::Image::from_vec_u8(src_w, src_h, rgba, fr::PixelType::U8x4)
        .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;
    let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);

    super::processing::ImageProcessor::create_simd_resizer()
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| eyre!("Resize failed: {e:?}"))?;

    Ok((dst_image.into_vec(), dst_w, dst_h, true))
}

pub(crate) fn apply_color_profile(rgba: &mut [u8], icc_data: &[u8]) -> Result<()> {
    let in_profile = qcms::Profile::new_from_slice(icc_data, false)
        .ok_or_else(|| eyre!("Failed to parse ICC profile"))?;

    // Identity transform: skip the expensive per-pixel pass entirely.
    if in_profile.is_sRGB() {
        return Ok(());
    }

    let out_profile = qcms::Profile::new_sRGB();

    let transform = qcms::Transform::new(
        &in_profile,
        &out_profile,
        qcms::DataType::RGBA8,
        qcms::Intent::Perceptual,
    )
    .ok_or_else(|| eyre!("Failed to create color transform"))?;

    use rayon::prelude::*;
    let chunk_size = 16384 * 4;
    rgba.par_chunks_mut(chunk_size).for_each(|chunk| {
        transform.apply(chunk);
    });

    Ok(())
}

impl ImageLoader {
    /// Load an image from a file path
    pub fn load(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        Self::load_with(path, max_w, max_h, LoadOptions::default())
    }

    /// Load an image with explicit decode options
    pub fn load_with(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        opts: LoadOptions,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format_type = ImageFormatType::from_extension(&ext);
        let is_jpeg = ext == "jpg" || ext == "jpeg";

        let exif = if is_jpeg || ext == "svg" || ext == "gif" {
            crate::image::metadata::ExifMeta::default()
        } else {
            crate::image::metadata::read_exif_meta(path)
        };
        let orientation = exif.orientation;

        let (image_frames, format_type) = if ext == "gif" {
            codecs::other::load_gif(path, max_w, max_h, opts)?
        } else if is_jpeg {
            codecs::jpeg::load_jpeg(path, max_w, max_h, opts)?
        } else if ext == "svg" {
            codecs::other::load_svg(path, max_w, max_h)?
        } else if ext == "jxl" {
            codecs::other::load_jxl(path, max_w, max_h)?
        } else if ext == "qoi" {
            codecs::other::load_qoi(path, max_w, max_h)?
        } else if ext == "ff" || ext == "farbfeld" {
            codecs::other::load_farbfeld(path, max_w, max_h)?
        } else if ext == "exr" {
            codecs::other::load_exr(path, max_w, max_h)?
        } else if ext == "heic" || ext == "heif" {
            codecs::heic::load_heic(path, max_w, max_h, format_type, &exif, opts)?
        } else if ext == "webp" {
            codecs::other::load_webp(path, max_w, max_h, &exif, opts)?
        } else if ext == "tiff" || ext == "tif" {
            codecs::tiff::load_tiff(path, max_w, max_h, &exif, opts)?
        } else if ext == "ico" || ext == "cur" {
            codecs::other::load_ico(path, max_w, max_h)?
        } else if format_type == ImageFormatType::Raw {
            codecs::raw::load_raw(path, max_w, max_h, &exif, opts)?
        } else if format_type == ImageFormatType::Avif {
            return Err(eyre!(
                "AVIF is not supported: it is an HEIF container holding AV1, \
                 which needs an AV1 decoder. Convert it to HEIC or PNG first."
            ));
        } else {
            let file = std::fs::File::open(path)
                .map_err(|e| eyre!("Failed to open image file {path:?}: {e:?}"))?;
            let mmap = unsafe {
                memmap2::Mmap::map(&file)
                    .map_err(|e| eyre!("Failed to memory map image {path:?}: {e:?}"))?
            };
            let cursor = std::io::Cursor::new(&mmap[..]);

            let target_box = orient_box(decode_box(max_w, max_h), orientation);

            let mut img = Image::read(cursor, zune_core::options::DecoderOptions::default())
                .map_err(|e| eyre!("Failed to decode image {path:?}: {e:?}"))?;

            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

            let (src_w, src_h) = (img.dimensions().0 as u32, img.dimensions().1 as u32);
            if src_w == 0 || src_h == 0 {
                return Err(eyre!("Image {path:?} reports a zero dimension"));
            }
            let icc = img.metadata().icc_chunk().map(|s| s.to_vec());
            let rgba = img.flatten_to_u8().swap_remove(0);

            let meta = &exif;

            let (mut rgba, final_w, final_h, is_downsampled) =
                downsample_to_fit(rgba, src_w, src_h, target_box)?;

            if let Some(ref icc_bytes) = icc
                && let Err(e) = apply_color_profile(&mut rgba, icc_bytes)
            {
                tracing::warn!("Failed to apply color profile: {:?}", e);
            }

            let (rgba, final_w, final_h, orientation_deg) = match orientation {
                Some(o) => apply_orientation(rgba, final_w, final_h, o, opts.bake_orientation),
                None => (rgba, final_w, final_h, 0),
            };

            (
                vec![ImageData {
                    path: path.to_path_buf(),
                    rgba_data: Arc::new(rgba),
                    width: final_w,
                    height: final_h,
                    format: format_type,
                    file_size_bytes: std::fs::metadata(path)?.len(),
                    frame_delay_ms: 0,
                    exif_info: None,
                    exif_loaded: false,
                    histogram: None,
                    is_downsampled,
                    orientation_deg,
                    gps_coords: meta.gps_coords,
                    color_space: meta.color_space,
                }],
                format_type,
            )
        };

        Ok((image_frames, format_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::{parse_exif_block, read_exif_meta};

    #[test]
    fn test_exif_orientation_mapping() {
        assert_eq!(exif_orientation_to_degrees(1), None);
        assert_eq!(exif_orientation_to_degrees(2), None);
        assert_eq!(exif_orientation_to_degrees(3), Some(180));
        assert_eq!(exif_orientation_to_degrees(6), Some(90));
        assert_eq!(exif_orientation_to_degrees(8), Some(270));
        assert_eq!(exif_orientation_to_degrees(0), None);
        assert_eq!(exif_orientation_to_degrees(9), None);
    }

    fn write_test_gif(path: &Path) {
        use gif::Frame;
        use std::fs::File;

        let mut encoder = gif::Encoder::new(
            File::create(path).expect("create gif"),
            4,
            4,
            &[0, 0, 0, 255, 255, 255],
        )
        .expect("encoder");
        encoder.set_repeat(gif::Repeat::Infinite).ok();
        for i in 0..3u8 {
            let buffer = vec![i; 16];
            encoder
                .write_frame(&Frame {
                    width: 4,
                    height: 4,
                    delay: 2,
                    dispose: gif::DisposalMethod::Keep,
                    transparent: None,
                    needs_user_input: false,
                    top: 0,
                    left: 0,
                    interlaced: false,
                    palette: None,
                    buffer: std::borrow::Cow::Owned(buffer),
                })
                .expect("write frame");
        }
    }

    #[test]
    fn test_gif_first_frame_limit_decodes_one_frame() {
        let dir = std::env::temp_dir().join(format!("spedimage_test_gif_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("anim.gif");
        write_test_gif(&path);

        let (frames, format) = ImageLoader::load_with(
            &path,
            None,
            None,
            LoadOptions {
                frames: FrameLimit::First,
                bake_orientation: true,
            },
        )
        .expect("load first");
        assert_eq!(format, ImageFormatType::Gif);
        assert_eq!(
            frames.len(),
            1,
            "FrameLimit::First must stop after one frame"
        );

        let (frames, _) = ImageLoader::load(&path, None, None).expect("load all");
        assert_eq!(frames.len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn exif_fixture(orientation: u16, color_space: u16) -> Vec<u8> {
        fn entry(tag: u16, typ: u16, value: u32) -> [u8; 12] {
            let mut e = [0u8; 12];
            e[0..2].copy_from_slice(&tag.to_le_bytes());
            e[2..4].copy_from_slice(&typ.to_le_bytes());
            e[4..8].copy_from_slice(&1u32.to_le_bytes());
            e[8..10].copy_from_slice(&(value as u16).to_le_bytes());
            e
        }
        fn ifd(entries: &[[u8; 12]], next_ifd: u32) -> Vec<u8> {
            let mut v = Vec::new();
            v.extend_from_slice(&(entries.len() as u16).to_le_bytes());
            for e in entries {
                v.extend_from_slice(e);
            }
            v.extend_from_slice(&next_ifd.to_le_bytes());
            v
        }

        let ifd0 = ifd(
            &[entry(0x0112, 3, orientation as u32), entry(0x8769, 4, 0)],
            0,
        );
        let exif_ifd_ofs = 8 + ifd0.len() as u32;
        let exif_ifd = ifd(&[entry(0xA001, 3, color_space as u32)], 0);

        let mut block = Vec::new();
        block.extend_from_slice(b"II");
        block.extend_from_slice(&42u16.to_le_bytes());
        block.extend_from_slice(&8u32.to_le_bytes());
        block.extend_from_slice(&ifd0);
        let ptr_at = 8 + 2 + 12 + 8;
        block[ptr_at..ptr_at + 4].copy_from_slice(&exif_ifd_ofs.to_le_bytes());
        block.extend_from_slice(&exif_ifd);
        block
    }

    #[test]
    fn test_exif_meta_parses_orientation_and_colorspace() {
        let meta = parse_exif_block(&exif_fixture(6, 2)).expect("parse exif");
        assert_eq!(meta.orientation, Some(6));
        assert_eq!(meta.color_space, Some(2));

        let mut with_prefix = b"Exif\0\0".to_vec();
        with_prefix.extend_from_slice(&exif_fixture(3, 1));
        let meta = parse_exif_block(&with_prefix).expect("parse prefixed exif");
        assert_eq!(meta.orientation, Some(3));
        assert_eq!(meta.color_space, Some(1));

        let (out, w, h, deg) = apply_orientation(vec![0u8; 16], 2, 2, 6, false);
        assert_eq!((w, h, deg), (2, 2, 90));
        assert!(out.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_exif_meta_is_default_for_garbage() {
        assert!(parse_exif_block(b"not exif at all").is_none());
        assert!(parse_exif_block(b"").is_none());
        let meta = read_exif_meta(std::path::Path::new("__no_such_file__.jpg"));
        assert!(meta.orientation.is_none());
        assert!(meta.gps_coords.is_none());
    }

    #[test]
    fn test_load_options_default_bakes_orientation() {
        let opts = LoadOptions::default();
        assert!(opts.bake_orientation);
        assert_eq!(opts.frames, FrameLimit::All);
    }

    #[test]
    fn test_decode_box_rejects_partial_or_zero_bounds() {
        assert_eq!(decode_box(None, None), None);
        assert_eq!(decode_box(Some(80), None), None);
        assert_eq!(decode_box(None, Some(80)), None);
        assert_eq!(decode_box(Some(0), Some(80)), None);
        assert_eq!(decode_box(Some(80), Some(60)), Some((80, 60)));
    }

    #[test]
    fn test_downsample_to_fit_is_identity_without_a_full_box() {
        let data = vec![7u8; 4 * 2 * 4];
        let (out, w, h, downsampled) = downsample_to_fit(data.clone(), 4, 2, None).unwrap();
        assert_eq!((w, h), (4, 2));
        assert!(!downsampled);
        assert_eq!(out, data);
    }

    #[test]
    fn test_downsample_to_fit_fits_inside_the_box() {
        let data = vec![3u8; 400 * 200 * 4];
        let (out, w, h, downsampled) = downsample_to_fit(data, 400, 200, Some((100, 100))).unwrap();
        assert!((w, h) == (100, 50), "got {w}x{h}");
        assert!(downsampled);
        assert_eq!(out.len(), (100 * 50 * 4) as usize);
    }

    #[test]
    fn test_downsample_to_fit_leaves_small_images_alone() {
        let data = vec![1u8; 40 * 20 * 4];
        let (out, w, h, downsampled) = downsample_to_fit(data, 40, 20, Some((800, 600))).unwrap();
        assert_eq!((w, h), (40, 20));
        assert!(!downsampled);
        assert_eq!(out.len(), 40 * 20 * 4);
    }

    #[test]
    fn test_orient_box_swaps_for_sideways_exif() {
        assert_eq!(orient_box(Some((800, 600)), Some(3)), Some((800, 600)));
        assert_eq!(orient_box(Some((800, 600)), Some(6)), Some((600, 800)));
        assert_eq!(orient_box(Some((800, 600)), Some(8)), Some((600, 800)));
        assert_eq!(orient_box(Some((800, 600)), Some(2)), Some((800, 600)));
        assert_eq!(orient_box(None, Some(6)), None);
    }

    #[test]
    fn test_dct_scale_never_forces_an_upscale() {
        for &(src_w, src_h) in &[(6000u32, 4000u32), (4000, 3000), (1024, 768), (300, 200)] {
            for &(tw, th) in &[(80u32, 80u32), (1920, 1080), (1, 1)] {
                let s = pick_dct_scale(src_w, src_h, tw, th);
                let dec_w = s.scale_dim(src_w as usize) as u32;
                let dec_h = s.scale_dim(src_h as usize) as u32;
                let ratio = (src_w as f64 / tw as f64).max(src_h as f64 / th as f64);
                if ratio > 1.0 {
                    let dst_w = (src_w as f64 / ratio).round() as u32;
                    let dst_h = (src_h as f64 / ratio).round() as u32;
                    assert!(
                        dec_w >= dst_w && dec_h >= dst_h,
                        "scale 1/{s_denom} upscales {src_w}x{src_h} for box {tw}x{th}: got {dec_w}x{dec_h}, need >= {dst_w}x{dst_h}",
                        s_denom = s.denom
                    );
                }
            }
        }
    }

    #[test]
    fn test_dct_scale_picks_the_largest_reduction() {
        assert_eq!(pick_dct_scale(6000, 4000, 80, 80).denom, 8);
        assert_eq!(pick_dct_scale(6000, 4000, 1920, 1080).denom, 2);
        assert_eq!(pick_dct_scale(640, 480, 1920, 1080).denom, 1);
    }

    #[test]
    fn test_exif_orientation_transform_covers_mirrored_values() {
        assert_eq!(exif_orientation_transform(1), None);
        assert_eq!(exif_orientation_transform(3), Some((180, false, false)));
        assert_eq!(exif_orientation_transform(6), Some((90, false, false)));
        assert_eq!(exif_orientation_transform(8), Some((270, false, false)));
        assert_eq!(exif_orientation_transform(2), Some((0, true, false)));
        assert_eq!(exif_orientation_transform(4), Some((0, false, true)));
        assert_eq!(exif_orientation_transform(5), Some((270, true, false)));
        assert_eq!(exif_orientation_transform(7), Some((90, true, false)));
        assert_eq!(exif_orientation_transform(99), None);
    }

    #[test]
    fn test_apply_orientation_defers_only_pure_rotations() {
        let data: Vec<u8> = (0..16u32).map(|v| v as u8).collect();
        let (out, w, h, deg) = apply_orientation(data.clone(), 2, 2, 6, false);
        assert_eq!((w, h, deg), (2, 2, 90));
        assert_eq!(out, data, "deferred rotation must not touch pixels");

        let (out, w, h, deg) = apply_orientation(data.clone(), 2, 2, 5, false);
        assert_eq!(deg, 0, "a mirror must never be deferred to the GPU");
        assert_eq!((w, h), (2, 2), "transpose keeps dimensions");
        assert_ne!(out, data);
    }

    #[test]
    fn test_apply_orientation_transposes_and_transverses() {
        let data: Vec<u8> = vec![1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255];
        let (out, w, h, _) = apply_orientation(data.clone(), 2, 2, 5, true);
        assert_eq!((w, h), (2, 2));
        assert_eq!(&out[0..4], &[1, 0, 0, 255]);
        assert_eq!(&out[4..8], &[3, 0, 0, 255]);
        assert_eq!(&out[8..12], &[2, 0, 0, 255]);
        assert_eq!(&out[12..16], &[4, 0, 0, 255]);

        let (out, w, h, _) = apply_orientation(data.clone(), 2, 2, 7, true);
        assert_eq!((w, h), (2, 2));
        assert_eq!(out[0], 4);
        assert_eq!(out[4], 2);
        assert_eq!(out[8], 3);
        assert_eq!(out[12], 1);

        let (out, _, _, _) = apply_orientation(data.clone(), 2, 2, 2, true);
        assert_eq!((out[0], out[4], out[8], out[12]), (2, 1, 4, 3));

        let (out, w, h, _) = apply_orientation(data.clone(), 2, 2, 8, true);
        assert_eq!((w, h), (2, 2));
        assert_eq!(out[0], 2);
    }

    #[test]
    fn test_apply_orientation_sideways_swaps_dimensions() {
        let data: Vec<u8> = (0..12).map(|v| v as u8).collect();
        let (_, w, h, deg) = apply_orientation(data.clone(), 3, 1, 6, true);
        assert_eq!((w, h, deg), (1, 3, 0));
        let (_, w, h, deg) = apply_orientation(data.clone(), 3, 1, 6, false);
        assert_eq!((w, h, deg), (3, 1, 90));
    }

    #[test]
    fn test_apply_orientation_identity_for_missing_tag() {
        let data: Vec<u8> = (0..16u32).map(|v| v as u8).collect();
        for o in [0, 1, 9, 42] {
            let (out, w, h, deg) = apply_orientation(data.clone(), 2, 2, o, true);
            assert_eq!((w, h, deg), (2, 2, 0));
            assert_eq!(out, data);
        }
    }
}
