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
///
/// Mirrored orientations always bake (the GPU path can only express a
/// rotation); pure rotations are baked only when `bake` is set, otherwise they
/// are reported through [`ImageData::orientation_deg`] and applied by the shader.
///
/// Takes the buffer by value so the deferred path — the common one, since the
/// preview load sets `bake_orientation: false` — hands the `Vec` straight back
/// instead of cloning a full-resolution frame.
fn apply_orientation(
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
///
/// Both bounds are required: a partial box used to divide by zero and collapse
/// the image to 1x1. Returns `None` when the caller wants full resolution.
fn decode_box(max_w: Option<u32>, max_h: Option<u32>) -> Option<(u32, u32)> {
    match (max_w, max_h) {
        (Some(mw), Some(mh)) if mw > 0 && mh > 0 => Some((mw, mh)),
        _ => None,
    }
}

/// Swap the box bounds for sideways EXIF orientations so the decoded buffer is
/// measured in the same axis order as the display.
fn orient_box(box_wh: Option<(u32, u32)>, orientation: Option<u32>) -> Option<(u32, u32)> {
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
///
/// Guarantees the subsequent `fast_image_resize` pass is always a *downscale*,
/// so output quality is unchanged while the decode does less work.
///
/// Measured on a 6000x4000 baseline JPEG: entropy decoding dominates, so a
/// scale factor of 8 only cuts total decode time to roughly 70% rather than the
/// 64x the IDCT saving alone would suggest. It also shrinks the output buffer
/// from 96 MB to 1.5 MB, which is what actually makes the preview and thumbnail
/// paths cheap. See `benches/image_processing.rs`.
fn pick_dct_scale(
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
///
/// Returns the buffer unchanged (no copy) when no bound is set, when the source
/// already fits, or when the decoder already produced something small enough.
fn downsample_to_fit(
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
    if dst_w == src_w && dst_h == src_h {
        return Ok((rgba, src_w, src_h, false));
    }

    use fast_image_resize as fr;
    let src_image = fr::images::ImageRef::new(src_w, src_h, &rgba, fr::PixelType::U8x4)
        .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;
    let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
    super::processing::ImageProcessor::create_simd_resizer()
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| eyre!("Resize failed: {e:?}"))?;

    Ok((dst_image.into_vec(), dst_w, dst_h, true))
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

        // JPEG reads orientation / GPS / color space straight out of the APP1
        // marker it already parsed for decoding, so it skips this pass. Every
        // other container is scanned exactly once, here, and the result is
        // reused for both the transform and the metadata fields.
        let exif = if is_jpeg || ext == "svg" || ext == "gif" {
            crate::image::metadata::ExifMeta::default()
        } else {
            crate::image::metadata::read_exif_meta(path)
        };
        let orientation = exif.orientation;

        let (image_frames, format_type) = if ext == "gif" {
            Self::load_gif(path, max_w, max_h, opts)?
        } else if is_jpeg {
            Self::load_jpeg(path, max_w, max_h, opts)?
        } else if ext == "svg" {
            Self::load_svg(path, max_w, max_h)?
        } else if ext == "jxl" {
            Self::load_jxl(path, max_w, max_h)?
        } else if ext == "qoi" {
            Self::load_qoi(path, max_w, max_h)?
        } else if ext == "exr" {
            Self::load_exr(path, max_w, max_h)?
        } else if ext == "heic" || ext == "heif" {
            Self::load_heic(path, max_w, max_h, format_type)?
        } else if ext == "webp" {
            Self::load_webp(path, max_w, max_h)?
        } else if ext == "tiff" || ext == "tif" {
            Self::load_tiff(path, max_w, max_h)?
        } else if format_type == ImageFormatType::Raw {
            Self::load_raw(path)?
        } else if format_type == ImageFormatType::Avif {
            // `is_supported` already filters these out of the browser, so this
            // only fires on an explicit open. Say why rather than letting the
            // generic path report a confusing "unknown format".
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

            // Ensure we are in RGBA8
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

            let (src_w, src_h) = (img.dimensions().0 as u32, img.dimensions().1 as u32);
            if src_w == 0 || src_h == 0 {
                return Err(eyre!("Image {path:?} reports a zero dimension"));
            }
            let icc = img.metadata().icc_chunk().map(|s| s.to_vec());
            // Take ownership instead of cloning: saves a full-buffer copy per image.
            let rgba = img.flatten_to_u8().swap_remove(0);

            let meta = &exif;

            let (mut rgba, final_w, final_h, is_downsampled) =
                downsample_to_fit(rgba, src_w, src_h, target_box)?;

            // Apply color profile to the (already reduced) buffer in parallel.
            if let Some(ref icc_bytes) = icc
                && let Err(e) = Self::apply_color_profile(&mut rgba, icc_bytes)
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

    /// Decode a JPEG, using libjpeg-turbo's scaled IDCT so thumbnails and
    /// previews never pay for a full-resolution decode of a 24 MP photo.
    fn load_jpeg(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        opts: LoadOptions,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)
            .map_err(|e| eyre!("Failed to open JPEG file {path:?}: {e:?}"))?;
        let mmap = unsafe {
            memmap2::Mmap::map(&file)
                .map_err(|e| eyre!("Failed to memory map JPEG {path:?}: {e:?}"))?
        };

        // Header parse only: gives dimensions, EXIF orientation, ICC and EXIF
        // blocks with no pixel work.
        let mut decoder = libjpeg_turbo_rs::Decoder::new(&mmap[..])
            .map_err(|e| eyre!("Failed to parse JPEG {path:?}: {e:?}"))?;

        let header = decoder.header();
        let src_w = header.width() as u32;
        let src_h = header.height() as u32;
        if src_w == 0 || src_h == 0 {
            return Err(eyre!("JPEG {path:?} reports a zero dimension"));
        }

        let orientation = decoder.exif_orientation().map(u32::from);
        let target_box = orient_box(decode_box(max_w, max_h), orientation);

        // Largest DCT reduction that still leaves room for a downscale-only
        // CPU pass, so the resize never has to upscale. Also keeps the
        // intermediate buffer small: a 24 MP RGBA frame is 96 MB at 1/1 but
        // only 1.5 MB at 1/8.
        let scale = match target_box {
            Some((tw, th)) => pick_dct_scale(src_w, src_h, tw, th),
            None => libjpeg_turbo_rs::ScalingFactor::new(1, 1),
        };
        decoder.set_scale(scale);
        decoder.set_output_format(libjpeg_turbo_rs::PixelFormat::Rgba);

        let mut buf = vec![0u8; decoder.output_buffer_size()?];
        let info = decoder.decode_image_into(&mut buf)?;
        let dec_w = info.width as u32;
        let dec_h = info.height as u32;
        // `output_buffer_size` is an upper bound on some paths; trim to what
        // was actually written before the buffer is reused as an RGBA image.
        buf.truncate(info.bytes_written);

        let icc = info.icc_profile.clone();
        let meta = info
            .exif_data
            .as_deref()
            .and_then(crate::image::metadata::parse_exif_block)
            .unwrap_or_default();

        let mut rgba = buf;
        let (mut final_w, mut final_h) = (dec_w, dec_h);
        let is_downsampled = scale.denom > 1;

        if let Some(box_wh) = target_box
            && (dec_w, dec_h) != (src_w, src_h)
        {
            let (r, w, h, _downsampled) = downsample_to_fit(rgba, dec_w, dec_h, Some(box_wh))?;
            rgba = r;
            final_w = w;
            final_h = h;
        }

        // ICC transform runs on the reduced buffer, so it is proportionally cheaper.
        if let Some(ref icc_bytes) = icc
            && let Err(e) = Self::apply_color_profile(&mut rgba, icc_bytes)
        {
            tracing::warn!("Failed to apply color profile for {path:?}: {e:?}");
        }

        let (rgba, final_w, final_h, orientation_deg) = match orientation {
            Some(o) => apply_orientation(rgba, final_w, final_h, o, opts.bake_orientation),
            None => (rgba, final_w, final_h, 0),
        };

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width: final_w,
                height: final_h,
                format: ImageFormatType::Jpeg,
                file_size_bytes: file.metadata()?.len(),
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled,
                orientation_deg,
                gps_coords: meta.gps_coords,
                color_space: meta.color_space,
            }],
            ImageFormatType::Jpeg,
        ))
    }

    fn apply_color_profile(rgba: &mut [u8], icc_data: &[u8]) -> Result<()> {
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

    fn load_jxl(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use jxl_oxide::JxlImage;

        let image = JxlImage::builder()
            .open(path)
            .map_err(|e| eyre!("Failed to open JXL: {e:?}"))?;

        let (width, height) = (image.width(), image.height());
        let render = image
            .render_frame(0)
            .map_err(|e| eyre!("Failed to render JXL frame: {e:?}"))?;

        let frame_buffer = render.image_all_channels();
        let fb = frame_buffer.buf();
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        let num_channels = frame_buffer.channels();

        use rayon::prelude::*;
        rgba.par_chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, rgba_pixel)| {
                let pixel = &fb[i * num_channels..(i + 1) * num_channels];
                let r = (pixel[0].clamp(0.0, 1.0) * 255.0) as u8;
                let g = if num_channels > 1 {
                    (pixel[1].clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    r
                };
                let b = if num_channels > 2 {
                    (pixel[2].clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    r
                };
                let a = if num_channels > 3 {
                    (pixel[3].clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    255
                };

                rgba_pixel[0] = r;
                rgba_pixel[1] = g;
                rgba_pixel[2] = b;
                rgba_pixel[3] = a;
            });

        let file_size = std::fs::metadata(path)?.len();
        let (rgba, width, height, is_downsampled) =
            downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: ImageFormatType::Jxl,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Jxl,
        ))
    }

    /// Decode a still WebP frame.
    ///
    /// Uses `image-webp` directly rather than zune-image's WebP codec: that
    /// codec only compiles when zune's `jpeg-xl` feature is also on, because of
    /// an ungated `jxl_oxide` import in it, and `image-webp` is what it wraps
    /// anyway.
    fn load_webp(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };

        // `WebPDecoder` needs a Seek, and an mmap-backed slice cannot seek.
        let cursor = std::io::Cursor::new(&mmap[..]);
        let mut decoder = image_webp::WebPDecoder::new(cursor)
            .map_err(|e| eyre!("Failed to parse WebP header: {e:?}"))?;

        let (src_w, src_h) = decoder.dimensions();
        if src_w == 0 || src_h == 0 {
            return Err(eyre!("WebP {path:?} reports a zero dimension"));
        }

        // The ICC profile is read before the pixel pass, while the decoder is
        // still positioned at the header.
        let icc = decoder.icc_profile().ok().flatten();

        // `image-webp` emits RGB (3 bytes/px) when the file has no alpha channel
        // and RGBA (4 bytes/px) when it does, so the buffer has to be widened
        // to RGBA before anything downstream sees it.
        let has_alpha = decoder.has_alpha();
        let buffer_size = decoder
            .output_buffer_size()
            .ok_or_else(|| eyre!("Could not determine the WebP buffer size"))?;
        let mut decoded = vec![0u8; buffer_size];
        decoder
            .read_image(&mut decoded)
            .map_err(|e| eyre!("WebP decode failed: {e:?}"))?;

        let pixels = src_w as usize * src_h as usize;
        let rgba = if has_alpha {
            decoded
        } else {
            let mut out = Vec::with_capacity(pixels * 4);
            for px in decoded.as_chunks::<3>().0 {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        };

        let file_size = std::fs::metadata(path)?.len();
        let (rgba, width, height, is_downsampled) =
            downsample_to_fit(rgba, src_w, src_h, decode_box(max_w, max_h))?;

        let mut rgba = rgba;
        if let Some(icc) = icc
            && let Err(e) = Self::apply_color_profile(&mut rgba, &icc)
        {
            tracing::warn!("Failed to apply WebP color profile: {e:?}");
        }

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: ImageFormatType::WebP,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::WebP,
        ))
    }

    fn load_svg(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use resvg::tiny_skia;
        use resvg::usvg;

        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let svg_data = &mmap[..];

        let rtree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
            .map_err(|e| eyre!("Failed to parse SVG: {e:?}"))?;

        let size = rtree.size();
        let mut width = size.width() as u32;
        let mut height = size.height() as u32;

        // Rasterize directly at the requested size instead of rendering at
        // intrinsic resolution and resizing on the CPU afterwards.
        let mut transform = tiny_skia::Transform::default();
        if let (Some(mw), Some(mh)) = (max_w, max_h)
            && (width > mw || height > mh)
            && width > 0
            && height > 0
        {
            let ratio = (width as f32 / mw as f32).max(height as f32 / mh as f32);
            let scale = 1.0 / ratio;
            width = ((width as f32 * scale).round() as u32).max(1);
            height = ((height as f32 * scale).round() as u32).max(1);
            transform = tiny_skia::Transform::from_scale(scale, scale);
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            eyre!(
                "Failed to create pixmap for SVG rendering (size: {}x{})",
                width,
                height
            )
        })?;

        resvg::render(&rtree, transform, &mut pixmap.as_mut());

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(pixmap.data().to_vec()),
                width,
                height,
                format: ImageFormatType::Svg,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Svg,
        ))
    }

    fn load_tiff(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use tiff::decoder::{Decoder, DecodingResult};
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);
        let mut decoder =
            Decoder::new(cursor).map_err(|e| eyre!("TIFF decoder init failed: {e:?}"))?;

        let (width, height) = decoder
            .dimensions()
            .map_err(|e| eyre!("Failed to get TIFF dimensions: {e:?}"))?;

        let img_res = decoder
            .read_image()
            .map_err(|e| eyre!("TIFF decode failed: {e:?}"))?;

        let rgba_data = match img_res {
            DecodingResult::U8(v) => {
                use zune_core::colorspace::ColorSpace;
                use zune_image::image::Image;

                let colortype = decoder
                    .colortype()
                    .map_err(|e| eyre!("Failed to get TIFF colortype: {e:?}"))?;

                let input_space = match colortype {
                    tiff::ColorType::RGB(8) => ColorSpace::RGB,
                    tiff::ColorType::RGBA(8) => ColorSpace::RGBA,
                    tiff::ColorType::Gray(8) => ColorSpace::Luma,
                    _ => return Err(eyre!("Unsupported TIFF color type: {:?}", colortype)),
                };

                let mut img = Image::from_u8(&v, width as usize, height as usize, input_space);
                img.convert_color(ColorSpace::RGBA)?;
                img.flatten_to_u8().swap_remove(0)
            }
            _ => return Err(eyre!("Unsupported TIFF bit depth")),
        };

        let file_size = std::fs::metadata(path)?.len();
        let (rgba_data, width, height, is_downsampled) =
            downsample_to_fit(rgba_data, width, height, decode_box(max_w, max_h))?;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba_data),
                width,
                height,
                format: ImageFormatType::Tiff,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Tiff,
        ))
    }

    /// Decode a HEIC/HEIF still image.
    ///
    /// AVIF is a HEIF container holding AV1, which `heic-rs` recognises and
    /// refuses by name rather than mis-decoding. `ImageFormatType::is_supported`
    /// already excludes AVIF, so this only runs on an explicit open.
    fn load_heic(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        format_type: ImageFormatType,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let data = &mmap[..];

        let options = heic_rs::DecodeOptions::default().with_layout(heic_rs::PixelLayout::Rgba8);
        let image =
            heic_rs::decode(data, &options).map_err(|e| eyre!("HEIC/HEIF decode failed: {}", e))?;

        let rgba = image.data;
        let (width, height) = (image.width, image.height);
        if rgba.len() != width as usize * height as usize * 4 {
            return Err(eyre!(
                "HEIC/HEIF decoder returned {} bytes for {width}x{height}",
                rgba.len()
            ));
        }

        let file_size = std::fs::metadata(path)?.len();
        let (rgba, width, height, is_downsampled) =
            downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: format_type,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            format_type,
        ))
    }

    fn load_gif(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        opts: LoadOptions,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use gif::DecodeOptions;
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);

        let mut options = DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(cursor)
            .map_err(|e| eyre!("Failed to read GIF info: {e:?}"))?;

        let mut image_frames = Vec::new();
        let file_size = std::fs::metadata(path)?.len();
        let (w, h) = (decoder.width() as u32, decoder.height() as u32);

        let mut dst_w = w;
        let mut dst_h = h;
        if let Some((mw, mh)) = decode_box(max_w, max_h)
            && (w > mw || h > mh)
        {
            let ratio = (w as f64 / mw as f64).max(h as f64 / mh as f64);
            dst_w = ((w as f64 / ratio).round() as u32).max(1);
            dst_h = ((h as f64 / ratio).round() as u32).max(1);
        }

        // A GIF frame might not cover the full canvas, so we need a canvas to compose them.
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        let mut prev_canvas = canvas.clone();
        let mut prev_disposal = gif::DisposalMethod::Keep;
        let mut prev_frame_rect = (0usize, 0usize, 0usize, 0usize);

        // Create resizer once outside the frame loop
        let is_downsampled = dst_w != w || dst_h != h;
        let mut resizer = if is_downsampled {
            Some(super::processing::ImageProcessor::create_simd_resizer())
        } else {
            None
        };

        const MAX_GIF_FRAMES: usize = 500;

        while let Ok(Some(frame)) = decoder.read_next_frame() {
            if image_frames.len() >= MAX_GIF_FRAMES {
                break;
            }

            // Apply disposal method from the PREVIOUS frame before rendering the current frame
            match prev_disposal {
                gif::DisposalMethod::Background => {
                    let (fl, ft, fw, fh) = prev_frame_rect;
                    for row in 0..fh {
                        let y = ft + row;
                        // A frame rect can sit outside the logical screen in a
                        // malformed GIF; clamp instead of trusting it.
                        if y >= h as usize || fl >= w as usize {
                            break;
                        }
                        let span = fw.min(w as usize - fl);
                        let start = (y * w as usize + fl) * 4;
                        let end = start + span * 4;
                        canvas[start..end].fill(0);
                    }
                }
                gif::DisposalMethod::Previous => {
                    canvas.copy_from_slice(&prev_canvas);
                }
                _ => {}
            }

            if frame.dispose == gif::DisposalMethod::Previous {
                prev_canvas.copy_from_slice(&canvas);
            }

            let delay_ms = (frame.delay as u32 * 10).max(10);
            let fl = frame.left as usize;
            let ft = frame.top as usize;
            let fw = frame.width as usize;
            let fh = frame.height as usize;

            // A zero-width sub-frame is legal in a malformed GIF and would
            // panic `chunks_exact(0)`.
            if fw > 0 && fh > 0 {
                let line_len = fw * 4;
                for (i, line) in frame.buffer.chunks_exact(line_len).enumerate() {
                    let y = ft + i;
                    if y >= h as usize {
                        break;
                    }
                    let row_base = y * w as usize;
                    let copy_pixels = fw.min(w as usize - fl);
                    if copy_pixels == 0 {
                        break;
                    }
                    let canvas_start = (row_base + fl) * 4;
                    let copy_bytes = copy_pixels * 4;
                    if line.len() < copy_bytes {
                        break;
                    }
                    for (p, pixel) in line[..copy_bytes].as_chunks::<4>().0.iter().enumerate() {
                        let dst_idx = canvas_start + p * 4;
                        // `gif` already zeroed fully transparent pixels.
                        if pixel[3] > 0 {
                            canvas[dst_idx..dst_idx + 4].copy_from_slice(pixel);
                        }
                    }
                }
            }

            prev_disposal = frame.dispose;
            prev_frame_rect = (fl, ft, fw, fh);

            let is_downsampled = dst_w != w || dst_h != h;
            let final_rgba = if is_downsampled {
                use fast_image_resize as fr;

                let src_image = fr::images::ImageRef::new(w, h, &canvas, fr::PixelType::U8x4)
                    .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;

                let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);

                if let Some(ref mut r) = resizer {
                    r.resize(&src_image, &mut dst_image, None)
                        .map_err(|e| eyre!("Resize failed: {e:?}"))?;
                }

                dst_image.into_vec()
            } else {
                canvas.clone()
            };

            image_frames.push(ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(final_rgba),
                width: dst_w,
                height: dst_h,
                format: ImageFormatType::Gif,
                file_size_bytes: file_size,
                frame_delay_ms: delay_ms,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            });

            // Thumbnail fast-path: one composited frame is enough.
            if opts.frames == FrameLimit::First {
                break;
            }
        }

        Ok((image_frames, ImageFormatType::Gif))
    }

    fn load_raw(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let image = rawloader::decode_file(path)
            .map_err(|e| eyre!("Failed to decode RAW file: {:?}", e))?;

        let width = image.width;
        let height = image.height;

        let raw_data = match &image.data {
            rawloader::RawImageData::Integer(data) => data,
            _ => return Err(eyre!("Unsupported non-integer RAW image data")),
        };

        // Do a fast half-size downsampled demosaicing with proper CFA mapping
        let out_w = width / 2;
        let out_h = height / 2;

        if out_w == 0 || out_h == 0 {
            return Err(eyre!("RAW image dimensions are too small"));
        }

        let mut rgba = vec![0u8; out_w * out_h * 4];

        let black = image.blacklevels[0] as f32;
        let white = image.whitelevels[0] as f32;
        let range = (white - black).max(1.0);

        // Extract camera white balance coefficients (fallback to 1.0)
        let wb_r = if image.wb_coeffs[0] > 0.01 {
            image.wb_coeffs[0]
        } else {
            1.0
        };
        let wb_g = if image.wb_coeffs[1] > 0.01 {
            image.wb_coeffs[1]
        } else {
            1.0
        };
        let wb_b = if image.wb_coeffs[2] > 0.01 {
            image.wb_coeffs[2]
        } else {
            1.0
        };
        let r_gain = (wb_r / wb_g).clamp(0.5, 4.0);
        let b_gain = (wb_b / wb_g).clamp(0.5, 4.0);

        let cfa_name = image.cfa.name.to_ascii_uppercase();

        use rayon::prelude::*;
        rgba.par_chunks_exact_mut(out_w * 4)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..out_w {
                    let p00_idx = (y * 2) * width + (x * 2);
                    let p01_idx = (y * 2) * width + (x * 2 + 1);
                    let p10_idx = (y * 2 + 1) * width + (x * 2);
                    let p11_idx = (y * 2 + 1) * width + (x * 2 + 1);

                    if p11_idx < raw_data.len() {
                        let p00 = raw_data[p00_idx] as f32;
                        let p01 = raw_data[p01_idx] as f32;
                        let p10 = raw_data[p10_idx] as f32;
                        let p11 = raw_data[p11_idx] as f32;

                        // Map CFA pattern to R, G, B channels
                        let (r_raw, g_raw, b_raw) = match cfa_name.as_str() {
                            "BGGR" => (p11, (p01 + p10) * 0.5, p00),
                            "GRBG" => (p01, (p00 + p11) * 0.5, p10),
                            "GBRG" => (p10, (p00 + p11) * 0.5, p01),
                            _ => (p00, (p01 + p10) * 0.5, p11), // Default RGGB
                        };

                        // Normalize with black/white levels and apply white balance
                        let r_norm = (((r_raw - black) / range).max(0.0) * r_gain).clamp(0.0, 1.0);
                        let g_norm = ((g_raw - black) / range).clamp(0.0, 1.0);
                        let b_norm = (((b_raw - black) / range).max(0.0) * b_gain).clamp(0.0, 1.0);

                        // Apply standard sRGB gamma (pow 1/2.2) so linear RAW is properly exposed
                        let r = (r_norm.powf(1.0 / 2.2) * 255.0).round() as u8;
                        let g = (g_norm.powf(1.0 / 2.2) * 255.0).round() as u8;
                        let b = (b_norm.powf(1.0 / 2.2) * 255.0).round() as u8;

                        let out_idx = x * 4;
                        row[out_idx] = r;
                        row[out_idx + 1] = g;
                        row[out_idx + 2] = b;
                        row[out_idx + 3] = 255;
                    }
                }
            });

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width: out_w as u32,
                height: out_h as u32,
                format: ImageFormatType::Raw,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: true,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Raw,
        ))
    }

    fn load_qoi(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)
            .map_err(|e| eyre!("Failed to open QOI file {path:?}: {e:?}"))?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let (header, decoded) =
            qoi::decode_to_vec(&mmap[..]).map_err(|e| eyre!("QOI decode error: {e:?}"))?;
        let file_size = mmap.len() as u64;

        let rgba_data = match header.channels {
            qoi::Channels::Rgb => {
                let mut rgba =
                    Vec::with_capacity(header.width as usize * header.height as usize * 4);
                for rgb in decoded.as_chunks::<3>().0 {
                    rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
                rgba
            }
            qoi::Channels::Rgba => decoded,
        };

        let (rgba_data, width, height, is_downsampled) = downsample_to_fit(
            rgba_data,
            header.width,
            header.height,
            decode_box(max_w, max_h),
        )?;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba_data),
                width,
                height,
                format: ImageFormatType::Qoi,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Qoi,
        ))
    }

    fn load_exr(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use exr::prelude::*;
        let image = read_first_rgba_layer_from_file(
            path,
            |resolution, _| {
                let pixel_count = resolution.width() * resolution.height();
                Vec::with_capacity(pixel_count * 4)
            },
            |vec, _position, (r, g, b, a): (f32, f32, f32, f32)| {
                let u8_r = fast_srgb8::f32_to_srgb8(r.clamp(0.0, 1.0));
                let u8_g = fast_srgb8::f32_to_srgb8(g.clamp(0.0, 1.0));
                let u8_b = fast_srgb8::f32_to_srgb8(b.clamp(0.0, 1.0));
                let u8_a = (a.clamp(0.0, 1.0) * 255.0) as u8;
                vec.extend_from_slice(&[u8_r, u8_g, u8_b, u8_a]);
            },
        )
        .map_err(|e| eyre!("Failed to decode EXR file: {e:?}"))?;

        let width = image.layer_data.size.width() as u32;
        let height = image.layer_data.size.height() as u32;
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let rgba = image.layer_data.channel_data.pixels;
        let (rgba, width, height, is_downsampled) =
            downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: ImageFormatType::Exr,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Exr,
        ))
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

    /// Build a tiny 3-frame animated GIF in the temp dir.
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

        // First-frame mode (thumbnails / streaming display path).
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

        // Default mode still returns every frame.
        let (frames, _) = ImageLoader::load(&path, None, None).expect("load all");
        assert_eq!(frames.len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Build a little-endian EXIF/TIFF block with Orientation in IFD0 and
    /// ColorSpace in the Exif sub-IFD (where the spec puts it, and where
    /// `exif::Tag::ColorSpace`'s context requires it to be).
    fn exif_fixture(orientation: u16, color_space: u16) -> Vec<u8> {
        fn entry(tag: u16, typ: u16, value: u32) -> [u8; 12] {
            let mut e = [0u8; 12];
            e[0..2].copy_from_slice(&tag.to_le_bytes());
            e[2..4].copy_from_slice(&typ.to_le_bytes());
            e[4..8].copy_from_slice(&1u32.to_le_bytes()); // count
            e[8..10].copy_from_slice(&(value as u16).to_le_bytes()); // SHORT, left-justified
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

        // Layout: header(8) | IFD0 | ExifIFD
        let ifd0 = ifd(
            &[
                entry(0x0112, 3, orientation as u32), // Orientation, PRIMARY
                entry(0x8769, 4, 0),                  // ExifIFDPointer, offset patched below
            ],
            0,
        );
        let exif_ifd_ofs = 8 + ifd0.len() as u32;
        let exif_ifd = ifd(&[entry(0xA001, 3, color_space as u32)], 0);

        let mut block = Vec::new();
        block.extend_from_slice(b"II");
        block.extend_from_slice(&42u16.to_le_bytes());
        block.extend_from_slice(&8u32.to_le_bytes());
        block.extend_from_slice(&ifd0);
        // Patch the ExifIFDPointer value field now that the offset is known.
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

        // With the "Exif\0\0" identifier still attached, as APP1 carries it.
        let mut with_prefix = b"Exif\0\0".to_vec();
        with_prefix.extend_from_slice(&exif_fixture(3, 1));
        let meta = parse_exif_block(&with_prefix).expect("parse prefixed exif");
        assert_eq!(meta.orientation, Some(3));
        assert_eq!(meta.color_space, Some(1));

        // Orientation 6 must route to a GPU rotation, not a bake.
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

    // --- decode budget regressions -------------------------------------

    #[test]
    fn test_decode_box_rejects_partial_or_zero_bounds() {
        // A partial box used to divide by zero and collapse the image to 1x1.
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
        // 400x200 into a 100x100 box -> ratio 4 -> 100x50.
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
        // The chosen DCT factor must leave the frame at least as large as the
        // aspect-fit result, otherwise the CPU pass would have to upscale.
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
        // 6000x4000 thumbnail: needs 1/8 to stay above 80px.
        assert_eq!(pick_dct_scale(6000, 4000, 80, 80).denom, 8);
        // 6000x4000 preview in a 1920x1080 window: 1/4 is too small, so 1/2.
        assert_eq!(pick_dct_scale(6000, 4000, 1920, 1080).denom, 2);
        // Image already fits: no DCT scaling at all.
        assert_eq!(pick_dct_scale(640, 480, 1920, 1080).denom, 1);
    }

    // --- EXIF orientation regressions ----------------------------------

    #[test]
    fn test_exif_orientation_transform_covers_mirrored_values() {
        // Pure rotations stay on the GPU.
        assert_eq!(exif_orientation_transform(1), None);
        assert_eq!(exif_orientation_transform(3), Some((180, false, false)));
        assert_eq!(exif_orientation_transform(6), Some((90, false, false)));
        assert_eq!(exif_orientation_transform(8), Some((270, false, false)));
        // Mirrored values were previously dropped entirely, showing the photo
        // unrotated. 5 is the main-diagonal transpose, 7 the anti-diagonal.
        assert_eq!(exif_orientation_transform(2), Some((0, true, false)));
        assert_eq!(exif_orientation_transform(4), Some((0, false, true)));
        assert_eq!(exif_orientation_transform(5), Some((270, true, false)));
        assert_eq!(exif_orientation_transform(7), Some((90, true, false)));
        assert_eq!(exif_orientation_transform(99), None);
    }

    #[test]
    fn test_apply_orientation_defers_only_pure_rotations() {
        // 2x2 RGBA, one byte per channel.
        let data: Vec<u8> = (0..16u32).map(|v| v as u8).collect();
        let (out, w, h, deg) = apply_orientation(data.clone(), 2, 2, 6, false);
        assert_eq!((w, h, deg), (2, 2, 90));
        assert_eq!(out, data, "deferred rotation must not touch pixels");

        // Mirrored: cannot be expressed by `pre_rotation`, so it must bake.
        let (out, w, h, deg) = apply_orientation(data.clone(), 2, 2, 5, false);
        assert_eq!(deg, 0, "a mirror must never be deferred to the GPU");
        assert_eq!((w, h), (2, 2), "transpose keeps dimensions");
        assert_ne!(out, data);
    }

    #[test]
    fn test_apply_orientation_transposes_and_transverses() {
        // 2x2 where the red channel is a unique pixel id:
        //   1 2
        //   3 4
        let data: Vec<u8> = vec![1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255];
        // Orientation 5 == transpose: [[1,2],[3,4]] -> [[1,3],[2,4]]
        let (out, w, h, _) = apply_orientation(data.clone(), 2, 2, 5, true);
        assert_eq!((w, h), (2, 2));
        assert_eq!(&out[0..4], &[1, 0, 0, 255]);
        assert_eq!(&out[4..8], &[3, 0, 0, 255]);
        assert_eq!(&out[8..12], &[2, 0, 0, 255]);
        assert_eq!(&out[12..16], &[4, 0, 0, 255]);

        // Orientation 7 == transverse (anti-diagonal flip):
        // [[1,2],[3,4]] -> [[4,2],[3,1]]
        let (out, w, h, _) = apply_orientation(data.clone(), 2, 2, 7, true);
        assert_eq!((w, h), (2, 2));
        assert_eq!(out[0], 4);
        assert_eq!(out[4], 2);
        assert_eq!(out[8], 3);
        assert_eq!(out[12], 1);

        // Orientation 2 == mirror horizontal: [[1,2],[3,4]] -> [[2,1],[4,3]]
        let (out, _, _, _) = apply_orientation(data.clone(), 2, 2, 2, true);
        assert_eq!((out[0], out[4], out[8], out[12]), (2, 1, 4, 3));

        // Orientation 8 == rotate 270 CW: dimensions swap.
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
