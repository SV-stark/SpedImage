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
    pub fn is_supported(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let exts = Self::supported_extensions();
        exts.contains(&ext.as_str())
    }

    /// Get list of supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec![
            "jpg", "jpeg", "png", "gif", "bmp", "tga", "tiff", "tif", "webp", "ico", "heic",
            "heif", "avif", "jxl", "svg", "qoi", "exr", "arw", "cr2", "nef", "dng", "orf", "raf",
            "srw",
        ]
    }

    pub fn rotate_rgba(rgba: &[u8], width: u32, height: u32, deg: i32) -> (Vec<u8>, u32, u32) {
        let deg = deg.rem_euclid(360);
        match deg {
            90 => {
                let mut out = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = ((y * width + x) * 4) as usize;
                        let dst_x = height - 1 - y;
                        let dst_y = x;
                        let dst_idx = ((dst_y * height + dst_x) * 4) as usize;
                        out[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
                (out, height, width)
            }
            180 => {
                let mut out = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = ((y * width + x) * 4) as usize;
                        let dst_x = width - 1 - x;
                        let dst_y = height - 1 - y;
                        let dst_idx = ((dst_y * width + dst_x) * 4) as usize;
                        out[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
                (out, width, height)
            }
            270 => {
                let mut out = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = ((y * width + x) * 4) as usize;
                        let dst_x = y;
                        let dst_y = width - 1 - x;
                        let dst_idx = ((dst_y * height + dst_x) * 4) as usize;
                        out[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
                (out, height, width)
            }
            _ => (rgba.to_vec(), width, height),
        }
    }

    pub fn flip_rgba(rgba: &mut [u8], width: u32, height: u32, flip_h: bool, flip_v: bool) {
        if !flip_h && !flip_v {
            return;
        }
        let w = width as usize;
        let h = height as usize;

        if flip_h {
            for y in 0..h {
                let row_start = y * w * 4;
                for x in 0..w / 2 {
                    let left = row_start + x * 4;
                    let right = row_start + (w - 1 - x) * 4;
                    for c in 0..4 {
                        rgba.swap(left + c, right + c);
                    }
                }
            }
        }

        if flip_v {
            for y in 0..h / 2 {
                let top_start = y * w * 4;
                let bot_start = (h - 1 - y) * w * 4;
                for x in 0..w * 4 {
                    rgba.swap(top_start + x, bot_start + x);
                }
            }
        }
    }

    pub fn crop_rgba(rgba: &[u8], width: u32, height: u32, rect: [f32; 4]) -> (Vec<u8>, u32, u32) {
        let (rx, ry, rw, rh) = (
            rect[0].clamp(0.0, 1.0),
            rect[1].clamp(0.0, 1.0),
            rect[2].clamp(0.01, 1.0),
            rect[3].clamp(0.01, 1.0),
        );

        let cx = ((rx * width as f32) as u32).min(width.saturating_sub(1));
        let cy = ((ry * height as f32) as u32).min(height.saturating_sub(1));
        let cw = ((rw * width as f32).round() as u32).clamp(1, width - cx);
        let ch = ((rh * height as f32).round() as u32).clamp(1, height - cy);

        let mut out = vec![0u8; (cw * ch * 4) as usize];
        for y in 0..ch {
            let src_start = (((cy + y) * width + cx) * 4) as usize;
            let src_end = src_start + (cw * 4) as usize;
            let dst_start = (y * cw * 4) as usize;
            let dst_end = dst_start + (cw * 4) as usize;
            if src_end <= rgba.len() && dst_end <= out.len() {
                out[dst_start..dst_end].copy_from_slice(&rgba[src_start..src_end]);
            }
        }

        (out, cw, ch)
    }

    pub fn apply_adjustments_cpu(
        rgba: &[u8],
        width: u32,
        height: u32,
        adjustments: &crate::render::ImageAdjustments,
    ) -> (Vec<u8>, u32, u32) {
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
        assert!(exts.contains(&"avif"));
        assert!(exts.contains(&"jxl"));
        assert!(exts.contains(&"svg"));
    }

    #[test]
    fn test_supported_extensions_includes_raw_formats() {
        let exts = ImageProcessor::supported_extensions();
        assert!(exts.contains(&"arw"));
        assert!(exts.contains(&"cr2"));
        assert!(exts.contains(&"nef"));
        assert!(exts.contains(&"dng"));
        assert!(exts.contains(&"orf"));
        assert!(exts.contains(&"raf"));
        assert!(exts.contains(&"srw"));
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
}
