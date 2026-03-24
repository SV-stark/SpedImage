use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use super::loader::ImageLoader;
use super::types::ImageData;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Load an image from file and downsample it if needed for the current display resolution.
    /// This is used for background loading and prefetching.
    pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
        let (frames, _format) = ImageLoader::load(path, Some(max_w), Some(max_h))?;
        let mut processed = Vec::with_capacity(frames.len());

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

                // Use fast_image_resize for high-performance SIMD resizing
                use fast_image_resize as fr;

                let src_image = fr::images::ImageRef::new(
                    img.width,
                    img.height,
                    &img.rgba_data,
                    fr::PixelType::U8x4,
                )
                .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;

                let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
                let mut resizer = fr::Resizer::new();

                resizer
                    .resize(&src_image, &mut dst_image, None)
                    .map_err(|e| eyre!("Resize failed: {e:?}"))?;

                img.width = dst_w;
                img.height = dst_h;
                img.rgba_data = dst_image.into_vec();
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
            "jpg", "jpeg", "png", "gif", "bmp", "tga", "tiff", "webp", "ico", "avif", "jxl", "svg",
            "arw", "cr2", "nef", "dng", "orf", "raf", "srw",
        ]
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
