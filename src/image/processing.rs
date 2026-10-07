use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::loader::{ImageLoader, LoadOptions};
use super::types::ImageData;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Create a fast_image_resize Resizer configured with automatic system SIMD CPU extension detection (AVX2/SSE4.1/NEON).
    pub fn create_simd_resizer() -> fast_image_resize::Resizer {
        fast_image_resize::Resizer::new()
    }

    /// Load an image from file and downsample it if needed for the current display resolution.
    /// This is used for background loading and prefetching.
    pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
        Self::load_and_downsample_with(path, max_w, max_h, LoadOptions::default())
    }

    /// Like [`Self::load_and_downsample`] with explicit decode options.
    pub fn load_and_downsample_with(
        path: &Path,
        max_w: u32,
        max_h: u32,
        opts: LoadOptions,
    ) -> Result<Vec<ImageData>> {
        let (frames, _format) = ImageLoader::load_with(path, Some(max_w), Some(max_h), opts)?;
        let mut processed = Vec::with_capacity(frames.len());

        use fast_image_resize as fr;
        let mut resizer = Self::create_simd_resizer();

        for frame in frames {
            let mut img = frame;
            let (w, h) = (img.width, img.height);

            // Calculate target size (maintain aspect ratio)
            if w > max_w || h > max_h {
                let ratio = (w as f32 / max_w as f32).max(h as f32 / max_h as f32);
                let dst_w = (w as f32 / ratio).round() as u32;
                let dst_h = (h as f32 / ratio).round() as u32;

                let dst_w = dst_w.max(1);
                let dst_h = dst_h.max(1);

                let src_image = fr::images::ImageRef::new(
                    img.width,
                    img.height,
                    &img.rgba_data,
                    fr::PixelType::U8x4,
                )
                .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;

                let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);

                resizer
                    .resize(&src_image, &mut dst_image, None)
                    .map_err(|e| eyre!("Resize failed: {e:?}"))?;

                img.width = dst_w;
                img.height = dst_h;
                img.rgba_data = Arc::new(dst_image.into_vec());
                img.is_downsampled = true;
            }

            processed.push(img);
        }

        Ok(processed)
    }

    /// Return true if the file extension is one of the supported formats.
    ///
    /// Allocation-free: this runs once per directory entry, so building a
    /// lowercase `String` and a `Vec` per file dominated large-folder scans.
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                Self::SUPPORTED_EXTENSIONS
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }

    /// Extensions the viewer accepts, without any per-call allocation.
    ///
    /// This is the list the file browser filters on, so it has to match what
    /// `ImageFormatType::is_supported` says the loaders can decode. `.avif` is
    /// absent on purpose: AVIF needs an AV1 decoder and there is none here, so
    /// listing it would show files that can only fail. `.tga` is absent for the
    /// same reason and no longer accepted here: there is still no Targa
    /// decoder, and listing it only produced files that were visible in the
    /// browser and then failed to open.
    pub const SUPPORTED_EXTENSIONS: &'static [&'static str] = &[
        // Still images with a dedicated loader.
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "ico", "cur", "heic", "heif",
        "jxl", "svg", "qoi", "exr",
        // Reached through zune-image's magic-byte sniffing.
        "psd", "psb", "hdr", "ppm", "pgm", "pbm", "pnm", "pam", "pfm", "ff", "farbfeld",
        // RAW: one entry per family `rawloader` can sniff.
        "arw", "cr2", "crw", "nef", "nrw", "dng", "orf", "raf", "sr2", "srf", "srw", "pef", "mrw",
        "kdc", "dcr", "rw2",
    ];

    /// Get list of supported file extensions
    pub fn supported_extensions() -> &'static [&'static str] {
        Self::SUPPORTED_EXTENSIONS
    }
    pub fn rotate_rgba(rgba: &[u8], width: u32, height: u32, deg: i32) -> (Vec<u8>, u32, u32) {
        use rayon::prelude::*;
        let (w, h) = (width as usize, height as usize);
        if w == 0 || h == 0 || rgba.len() < w * h * 4 {
            return (rgba.to_vec(), width, height);
        }

        let deg = deg.rem_euclid(360);
        let mut out = vec![0u8; rgba.len()];
        // Each destination row is written independently, so the passes below
        // are embarrassingly parallel (a single-threaded loop made every
        // rotate/flip on a 24 MP export visibly stutter).
        match deg {
            90 => out
                .par_chunks_exact_mut(h * 4)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    for (dst_x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                        let src = ((h - 1 - dst_x) * w + dst_y) * 4;
                        px.copy_from_slice(&rgba[src..src + 4]);
                    }
                }),
            180 => out
                .par_chunks_exact_mut(w * 4)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    let y = h - 1 - dst_y;
                    for (dst_x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                        let src = (y * w + (w - 1 - dst_x)) * 4;
                        px.copy_from_slice(&rgba[src..src + 4]);
                    }
                }),
            270 => out
                .par_chunks_exact_mut(h * 4)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    for (dst_x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                        let src = (dst_x * w + (w - 1 - dst_y)) * 4;
                        px.copy_from_slice(&rgba[src..src + 4]);
                    }
                }),
            _ => return (rgba.to_vec(), width, height),
        }
        let (nw, nh) = if deg == 180 {
            (width, height)
        } else {
            (height, width)
        };
        (out, nw, nh)
    }

    pub fn flip_rgba(rgba: &mut [u8], width: u32, height: u32, flip_h: bool, flip_v: bool) {
        use rayon::prelude::*;
        if !flip_h && !flip_v {
            return;
        }
        let (w, h) = (width as usize, height as usize);
        if w == 0 || h == 0 || rgba.len() < w * h * 4 {
            return;
        }

        if flip_h {
            let half = w / 2;
            // Each row is an independent mirror, so the rows run in parallel.
            rgba.par_chunks_exact_mut(w * 4).for_each(|row| {
                for x in 0..half {
                    let l = x * 4;
                    let r = (w - 1 - x) * 4;
                    row.swap(l, r);
                    row.swap(l + 1, r + 1);
                    row.swap(l + 2, r + 2);
                    row.swap(l + 3, r + 3);
                }
            });
        }

        if flip_v {
            // Pair row `y` with row `h - 1 - y`. Splitting in half only works
            // for even heights, so the bottom half is walked in reverse; with
            // an odd height the extra middle row stays put on its own.
            let row_bytes = w * 4;
            let top_rows = h.div_ceil(2);
            let (top, bottom) = rgba.split_at_mut(top_rows * row_bytes);
            let bottom = bottom.par_rchunks_exact_mut(row_bytes);
            top.par_chunks_exact_mut(row_bytes)
                .zip(bottom)
                .for_each(|(a, b)| a.swap_with_slice(b));
        }
    }

    pub fn crop_rgba(rgba: &[u8], width: u32, height: u32, rect: [f32; 4]) -> (Vec<u8>, u32, u32) {
        use rayon::prelude::*;
        if width == 0 || height == 0 || rgba.len() < (width as usize) * (height as usize) * 4 {
            return (rgba.to_vec(), width, height);
        }

        let (rx, ry, rw, rh) = (
            rect[0].clamp(0.0, 1.0),
            rect[1].clamp(0.0, 1.0),
            rect[2].clamp(0.01, 1.0),
            rect[3].clamp(0.01, 1.0),
        );

        let cx = ((rx * width as f32) as u32).min(width - 1);
        let cy = ((ry * height as f32) as u32).min(height - 1);
        let cw = ((rw * width as f32).round() as u32).max(1).min(width - cx);
        let ch = ((rh * height as f32).round() as u32)
            .max(1)
            .min(height - cy);

        let mut out = vec![0u8; (cw * ch * 4) as usize];
        out.par_chunks_exact_mut((cw * 4) as usize)
            .enumerate()
            .for_each(|(y, row)| {
                let src_start = (((cy + y as u32) * width + cx) * 4) as usize;
                let end = src_start + (cw * 4) as usize;
                if end <= rgba.len() {
                    row.copy_from_slice(&rgba[src_start..end]);
                }
            });

        (out, cw, ch)
    }

    pub fn apply_adjustments_cpu(
        rgba: &[u8],
        width: u32,
        height: u32,
        adjustments: &crate::render::ImageAdjustments,
    ) -> (Vec<u8>, u32, u32) {
        // A buffer shorter than `width * height * 4` (a truncated decode, a
        // clipboard image with mismatched metadata) would otherwise be handed
        // to the GPU upload as a short buffer. Drop the ragged tail so the
        // result is always whole pixels.
        let rgba = &rgba[..rgba.len().min(width as usize * height as usize * 4)];

        // 1. Crop
        let (mut data, mut w, mut h) = if let Some(crop) = adjustments.crop_rect_actual {
            Self::crop_rgba(rgba, width, height, crop)
        } else {
            (rgba.to_vec(), width, height)
        };

        // 2. Rotation
        let deg = (adjustments.rotation.to_degrees().round() as i32 % 360 + 360) % 360;
        if deg != 0 {
            let (rot_data, rw, rh) = Self::rotate_rgba(&data, w, h, deg);
            data = rot_data;
            w = rw;
            h = rh;
        }

        // 3. Flips
        Self::flip_rgba(
            &mut data,
            w,
            h,
            adjustments.flip_horizontal,
            adjustments.flip_vertical,
        );

        // 4. Color / exposure adjustments per pixel (parallelized with rayon)
        let has_wb = adjustments.temperature.abs() > 0.001 || adjustments.tint.abs() > 0.001;
        let has_shadows_highlights =
            adjustments.shadows.abs() > 0.001 || adjustments.highlights.abs() > 0.001;
        let has_clarity = adjustments.clarity.abs() > 0.001;
        let has_bc = (adjustments.brightness - 1.0).abs() > 0.001
            || (adjustments.contrast - 1.0).abs() > 0.001;
        let has_sat = (adjustments.saturation - 1.0).abs() > 0.001;
        let has_hdr = adjustments.hdr_toning;

        if has_wb || has_shadows_highlights || has_clarity || has_bc || has_sat || has_hdr {
            use rayon::prelude::*;
            data.par_chunks_exact_mut(4).for_each(|pixel| {
                let mut r = pixel[0] as f32 / 255.0;
                let mut g = pixel[1] as f32 / 255.0;
                let mut b = pixel[2] as f32 / 255.0;

                // White balance
                if has_wb {
                    r += adjustments.temperature * 0.22 + adjustments.tint * 0.12;
                    g += adjustments.temperature * 0.06 - adjustments.tint * 0.22;
                    b += -adjustments.temperature * 0.22 + adjustments.tint * 0.12;
                    r = r.clamp(0.0, 1.0);
                    g = g.clamp(0.0, 1.0);
                    b = b.clamp(0.0, 1.0);
                }

                // Shadows & Highlights
                if has_shadows_highlights {
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let shadow_mask = (1.0 - lum / 0.55).clamp(0.0, 1.0);
                    let highlight_mask = ((lum - 0.45) / 0.55).clamp(0.0, 1.0);
                    r += r * (adjustments.shadows * shadow_mask * 0.5)
                        + (1.0 - r) * (adjustments.highlights * highlight_mask * 0.5);
                    g += g * (adjustments.shadows * shadow_mask * 0.5)
                        + (1.0 - g) * (adjustments.highlights * highlight_mask * 0.5);
                    b += b * (adjustments.shadows * shadow_mask * 0.5)
                        + (1.0 - b) * (adjustments.highlights * highlight_mask * 0.5);
                    r = r.clamp(0.0, 1.0);
                    g = g.clamp(0.0, 1.0);
                    b = b.clamp(0.0, 1.0);
                }

                // Clarity
                if has_clarity {
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let midtone_mask = (1.0 - (lum - 0.5).abs() * 2.0).max(0.0);
                    r += (r - 0.5) * (adjustments.clarity * 0.4 * midtone_mask);
                    g += (g - 0.5) * (adjustments.clarity * 0.4 * midtone_mask);
                    b += (b - 0.5) * (adjustments.clarity * 0.4 * midtone_mask);
                    r = r.clamp(0.0, 1.0);
                    g = g.clamp(0.0, 1.0);
                    b = b.clamp(0.0, 1.0);
                }

                // Brightness & Contrast
                if has_bc {
                    r = (r - 0.5) * adjustments.contrast + 0.5 + (adjustments.brightness - 1.0);
                    g = (g - 0.5) * adjustments.contrast + 0.5 + (adjustments.brightness - 1.0);
                    b = (b - 0.5) * adjustments.contrast + 0.5 + (adjustments.brightness - 1.0);
                }

                // Saturation
                if has_sat {
                    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                    r = gray + (r - gray) * adjustments.saturation;
                    g = gray + (g - gray) * adjustments.saturation;
                    b = gray + (b - gray) * adjustments.saturation;
                }

                // HDR Filmic Toning
                if has_hdr {
                    let mut rx = (r * 1.6).max(0.0);
                    let mut gx = (g * 1.6).max(0.0);
                    let mut bx = (b * 1.6).max(0.0);
                    rx = rx / (1.0 + rx);
                    gx = gx / (1.0 + gx);
                    bx = bx / (1.0 + bx);
                    r = rx * rx * (3.0 - 2.0 * rx);
                    g = gx * gx * (3.0 - 2.0 * gx);
                    b = bx * bx * (3.0 - 2.0 * bx);
                }

                pixel[0] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
                pixel[1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
                pixel[2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
            });
        }

        (data, w, h)
    }

    pub fn save(path: &Path, rgba_data: &[u8], w: u32, h: u32) -> Result<()> {
        use zune_image::image::Image;
        let img = Image::from_u8(
            rgba_data,
            w as usize,
            h as usize,
            zune_core::colorspace::ColorSpace::RGBA,
        );
        img.save(path)
            .map_err(|e| eyre!("Failed to save image: {e:?}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageFormatType;
    use std::path::PathBuf;

    #[test]
    fn test_supported_extensions_includes_common_formats() {
        let exts = ImageProcessor::supported_extensions();
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"tiff"));
        assert!(exts.contains(&"tif"));
        assert!(exts.contains(&"webp"));
        assert!(exts.contains(&"heic"));
        assert!(exts.contains(&"heif"));
        assert!(exts.contains(&"jxl"));
        assert!(exts.contains(&"svg"));
    }

    /// The browser filters on `SUPPORTED_EXTENSIONS`, not on
    /// `ImageFormatType::is_supported`, so an extension in the list that
    /// `is_supported` rejects is shown to the user and then fails to open.
    /// AVIF hit exactly that, and so did `tga` and `ico`, which sat in the list
    /// with no decoder behind them until they were either implemented or
    /// removed. The list is now fully consistent, so there is no exemption.
    #[test]
    fn test_supported_extensions_agree_with_is_supported() {
        for ext in ImageProcessor::supported_extensions() {
            let format = ImageFormatType::from_extension(ext);
            assert!(
                format.is_supported(),
                "{ext} is in SUPPORTED_EXTENSIONS but is_supported() says no, \
                 so it would be listed in the browser and fail to open"
            );
        }
    }

    /// The reverse direction: a format the loaders can decode must also be
    /// reachable from the file browser, or it exists but users never see it.
    #[test]
    fn test_every_supported_extension_maps_back_into_the_list() {
        let listed = ImageProcessor::supported_extensions();
        // Everything `from_extension` calls supported has to appear in the
        // browser's list, extension for extension, or the two drift apart
        // again in the direction that hides working formats.
        for ext in [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "ico", "cur", "heic",
            "heif", "jxl", "svg", "qoi", "exr", "psd", "psb", "hdr", "ppm", "pgm", "pbm", "pnm",
            "pam", "pfm", "ff", "farbfeld", "arw", "cr2", "crw", "nef", "nrw", "dng", "orf", "raf",
            "sr2", "srf", "srw", "pef", "mrw", "kdc", "dcr", "rw2",
        ] {
            assert!(
                listed.contains(&ext),
                "{ext} decodes but is missing from SUPPORTED_EXTENSIONS, so the \
                 file browser would hide it"
            );
        }
    }

    /// `tga` used to be listed without a decoder behind it.
    #[test]
    fn test_tga_is_not_listed_in_the_browser() {
        assert!(!ImageProcessor::supported_extensions().contains(&"tga"));
    }

    #[test]
    fn test_avif_is_not_listed_in_the_browser() {
        assert!(!ImageProcessor::supported_extensions().contains(&"avif"));
        assert!(!ImageProcessor::supported_extensions().contains(&"avis"));
        // ...but still recognised, so an explicit open can explain itself.
        assert_eq!(
            ImageFormatType::from_extension("avif"),
            ImageFormatType::Avif
        );
    }

    #[test]
    fn test_supported_extensions_includes_raw_formats() {
        let exts = ImageProcessor::supported_extensions();
        // One entry per camera family `rawloader` can sniff. `.pef`, `.crw`,
        // `.mrw`, `.kdc`, `.dcr`, `.rw2`, `.nrw`, `.srf` and `.sr2` were all
        // missing while the README advertised their formats, so those files
        // were invisible in the browser.
        for ext in [
            "arw", "cr2", "crw", "nef", "nrw", "dng", "orf", "raf", "sr2", "srf", "srw", "pef",
            "mrw", "kdc", "dcr", "rw2",
        ] {
            assert!(exts.contains(&ext), "RAW extension {ext} is not listed");
            assert_eq!(ImageFormatType::from_extension(ext), ImageFormatType::Raw);
        }
    }

    #[test]
    fn test_is_supported_positive() {
        assert!(ImageProcessor::is_supported(&PathBuf::from("photo.jpg")));
        assert!(ImageProcessor::is_supported(&PathBuf::from("photo.PNG")));
        assert!(ImageProcessor::is_supported(&PathBuf::from("photo.Gif")));
        assert!(ImageProcessor::is_supported(&PathBuf::from("photo.webp")));
        assert!(ImageProcessor::is_supported(&PathBuf::from("photo.svg")));
    }

    #[test]
    fn test_is_supported_negative() {
        assert!(!ImageProcessor::is_supported(&PathBuf::from(
            "document.txt"
        )));
        assert!(!ImageProcessor::is_supported(&PathBuf::from("data.csv")));
        assert!(!ImageProcessor::is_supported(&PathBuf::from("archive.zip")));
        assert!(!ImageProcessor::is_supported(&PathBuf::from("noextension")));
    }

    #[test]
    fn test_is_supported_empty_extension() {
        assert!(!ImageProcessor::is_supported(&PathBuf::from(".")));
        assert!(!ImageProcessor::is_supported(&PathBuf::from("Makefile")));
    }

    #[test]
    fn test_rotate_rgba_90_180_270() {
        // 2x3 image:
        // [1, 2]
        // [3, 4]
        // [5, 6]
        let w = 2u32;
        let h = 3u32;
        let mut data = vec![0u8; (w * h * 4) as usize];
        for i in 0..(w * h) {
            data[(i * 4) as usize] = (i + 1) as u8;
            data[(i * 4 + 3) as usize] = 255;
        }

        // Rotate 90 CW -> 3x2:
        // [5, 3, 1]
        // [6, 4, 2]
        let (rot90, w90, h90) = ImageProcessor::rotate_rgba(&data, w, h, 90);
        assert_eq!(w90, 3);
        assert_eq!(h90, 2);
        assert_eq!(rot90[0], 5); // (0,0) is 5
        assert_eq!(rot90[4], 3); // (1,0) is 3
        assert_eq!(rot90[8], 1); // (2,0) is 1
        assert_eq!(rot90[12], 6); // (0,1) is 6
        assert_eq!(rot90[16], 4); // (1,1) is 4
        assert_eq!(rot90[20], 2); // (2,1) is 2

        // Rotate 180 -> 2x3:
        // [6, 5]
        // [4, 3]
        // [2, 1]
        let (rot180, w180, h180) = ImageProcessor::rotate_rgba(&data, w, h, 180);
        assert_eq!(w180, 2);
        assert_eq!(h180, 3);
        assert_eq!(rot180[0], 6);
        assert_eq!(rot180[4], 5);
        assert_eq!(rot180[8], 4);
        assert_eq!(rot180[12], 3);
        assert_eq!(rot180[16], 2);
        assert_eq!(rot180[20], 1);

        // Rotate 270 -> 3x2:
        // [2, 4, 6]
        // [1, 3, 5]
        let (rot270, w270, h270) = ImageProcessor::rotate_rgba(&data, w, h, 270);
        assert_eq!(w270, 3);
        assert_eq!(h270, 2);
        assert_eq!(rot270[0], 2);
        assert_eq!(rot270[4], 4);
        assert_eq!(rot270[8], 6);
        assert_eq!(rot270[12], 1);
        assert_eq!(rot270[16], 3);
        assert_eq!(rot270[20], 5);
    }

    #[test]
    fn test_flip_rgba() {
        let mut data = vec![10, 0, 0, 255, 20, 0, 0, 255, 30, 0, 0, 255, 40, 0, 0, 255];
        // Flip H:
        // [20, 10]
        // [40, 30]
        ImageProcessor::flip_rgba(&mut data, 2, 2, true, false);
        assert_eq!(data[0], 20);
        assert_eq!(data[4], 10);
        assert_eq!(data[8], 40);
        assert_eq!(data[12], 30);

        // Flip V:
        // [40, 30]
        // [20, 10]
        ImageProcessor::flip_rgba(&mut data, 2, 2, false, true);
        assert_eq!(data[0], 40);
        assert_eq!(data[4], 30);
        assert_eq!(data[8], 20);
        assert_eq!(data[12], 10);
    }

    #[test]
    fn test_crop_rgba() {
        let data = vec![
            1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255, 5, 0, 0, 255, 6, 0, 0, 255, 7,
            0, 0, 255, 8, 0, 0, 255, 9, 0, 0, 255, 10, 0, 0, 255, 11, 0, 0, 255, 12, 0, 0, 255, 13,
            0, 0, 255, 14, 0, 0, 255, 15, 0, 0, 255, 16, 0, 0, 255,
        ];
        // Crop middle 2x2: x=0.25..0.75, y=0.25..0.75
        let (crop, cw, ch) = ImageProcessor::crop_rgba(&data, 4, 4, [0.25, 0.25, 0.5, 0.5]);
        assert_eq!(cw, 2);
        assert_eq!(ch, 2);
        assert_eq!(crop[0], 6);
        assert_eq!(crop[4], 7);
        assert_eq!(crop[8], 10);
        assert_eq!(crop[12], 11);
    }

    #[test]
    fn test_apply_adjustments_cpu_brightness_contrast() {
        let data = vec![100, 150, 200, 255];
        let adj = crate::render::ImageAdjustments {
            brightness: 1.2,
            contrast: 1.1,
            ..Default::default()
        };
        let (out, w, h) = ImageProcessor::apply_adjustments_cpu(&data, 1, 1, &adj);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert!(out[0] > 100);
        assert!(out[1] > 150);
        assert_eq!(out[3], 255);
    }

    // --- degenerate geometry regressions -------------------------------
    //
    // `crop_rgba` used to call `clamp(1, width - cx)`, which panics when the
    // image has a zero dimension (min > max).

    #[test]
    fn test_zero_dimension_images_do_not_panic() {
        let empty: Vec<u8> = Vec::new();
        let (out, w, h) = ImageProcessor::crop_rgba(&empty, 0, 0, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!((w, h), (0, 0));
        assert!(out.is_empty());

        let (out, w, h) = ImageProcessor::rotate_rgba(&empty, 0, 10, 90);
        assert_eq!((w, h), (0, 10));
        assert!(out.is_empty());

        let mut buf: Vec<u8> = Vec::new();
        ImageProcessor::flip_rgba(&mut buf, 4, 0, true, true);
        assert!(buf.is_empty());

        let mut buf: Vec<u8> = Vec::new();
        ImageProcessor::flip_rgba(&mut buf, 0, 4, true, true);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_truncated_buffer_is_treated_as_noop() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let (out, w, h) = ImageProcessor::crop_rgba(&data, 4, 4, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!((w, h), (4, 4));
        assert_eq!(out, data);
    }

    #[test]
    fn test_flip_rgba_both_axes() {
        let data: Vec<u8> = (1..=16u8).collect();

        let mut buf = data.clone();
        ImageProcessor::flip_rgba(&mut buf, 2, 2, true, true);
        // Mirror H then mirror V == rotate 180.
        let (rot, _, _) = ImageProcessor::rotate_rgba(&data, 2, 2, 180);
        assert_eq!(buf, rot);
    }

    #[test]
    fn test_flip_rgba_odd_height_leaves_middle_row_in_place() {
        // 1x3: only the top and bottom rows may swap.
        let mut buf: Vec<u8> = (1..=12u8).collect();
        ImageProcessor::flip_rgba(&mut buf, 1, 3, false, true);
        assert_eq!(buf[0], 9);
        assert_eq!(buf[4], 5);
        assert_eq!(buf[8], 1);
    }

    #[test]
    fn test_apply_adjustments_cpu_trailing_partial_pixel_is_dropped_not_misread() {
        // `par_chunks_exact_mut(4)` silently ignores a ragged tail; the output
        // must still be a whole number of pixels for the GPU upload.
        let data: Vec<u8> = vec![100, 150, 200, 255, 50];
        let adj = crate::render::ImageAdjustments {
            brightness: 1.5,
            ..Default::default()
        };
        let (out, w, h) = ImageProcessor::apply_adjustments_cpu(&data, 1, 1, &adj);
        assert_eq!((w, h), (1, 1));
        // The stray trailing byte must not reach the encoder/GPU upload.
        assert_eq!(out.len(), 4);
        // brightness 1.5 adds 0.5 to the normalized red channel: 100/255+0.5.
        let expected = ((100.0f32 / 255.0 + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
        assert_eq!(out[0], expected, "the real pixel is still adjusted");
        assert_eq!(out[3], 255, "alpha is untouched");
    }

    #[test]
    fn test_apply_adjustments_cpu_passes_through_when_no_adjustments() {
        let data = vec![10u8, 20, 30, 40, 50, 60, 70, 80];
        let (out, w, h) = ImageProcessor::apply_adjustments_cpu(
            &data,
            2,
            1,
            &crate::render::ImageAdjustments::default(),
        );
        assert_eq!((w, h), (2, 1));
        assert_eq!(out, data, "identity adjustments must not alter pixels");
    }

    #[test]
    fn test_apply_adjustments_cpu_crop_then_rotate_swaps_dimensions() {
        let data: Vec<u8> = (0..(4u32 * 2 * 4)).map(|i| i as u8).collect();
        let adj = crate::render::ImageAdjustments {
            crop_rect_actual: Some([0.0, 0.0, 0.5, 1.0]), // 2x2 crop
            rotation: std::f32::consts::FRAC_PI_2,        // then 90 CW -> 2x2
            ..Default::default()
        };
        let (out, w, h) = ImageProcessor::apply_adjustments_cpu(&data, 4, 2, &adj);
        assert_eq!((w, h), (2, 2));
        assert_eq!(out.len(), 2 * 2 * 4);
    }

    #[test]
    fn test_supported_extensions_is_a_shared_static() {
        // Returning a `Vec` forced a heap allocation per directory entry.
        let a = ImageProcessor::supported_extensions();
        let b = ImageProcessor::supported_extensions();
        assert!(std::ptr::eq(a.as_ptr(), b.as_ptr()));
        assert!(a.len() > 20);
    }

    #[test]
    fn test_crop_rgba_is_stable_under_repeated_calls() {
        let data: Vec<u8> = (0..(64 * 64 * 4)).map(|i| (i % 251) as u8).collect();
        let (first, w1, h1) = ImageProcessor::crop_rgba(&data, 64, 64, [0.1, 0.2, 0.5, 0.5]);
        let (second, w2, h2) = ImageProcessor::crop_rgba(&data, 64, 64, [0.1, 0.2, 0.5, 0.5]);
        assert_eq!((w1, h1), (w2, h2));
        assert_eq!(first, second);
        assert_eq!(first.len(), (w1 * h1 * 4) as usize);
    }
}
