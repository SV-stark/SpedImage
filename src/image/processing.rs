use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use super::loader::ImageLoader;
use super::types::ImageData;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Load an image from file and downsample it if needed for the current display resolution.
    /// This is used for background loading and prefetching.
    pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
        let (frames, _format) = ImageLoader::load(path)?;
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

                // Use zune-image for resizing
                use zune_image::image::Image;
                use zune_imageprocs::resize::{Resize, ResizeMethod};
                use zune_image::traits::OperationsTrait;
                
                let mut z_img = Image::from_u8(&img.rgba_data, img.width as usize, img.height as usize, zune_core::colorspace::ColorSpace::RGBA);
                let resize = Resize::new(dst_w as usize, dst_h as usize, ResizeMethod::Lanczos3);
                resize.execute(&mut z_img)
                    .map_err(|e| eyre!("Zune resize failed: {e:?}"))?;
                
                let (new_w, new_h) = z_img.dimensions();
                img.width = new_w as u32;
                img.height = new_h as u32;
                img.rgba_data = z_img.flatten_to_u8()[0].clone();
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
            "jpg", "jpeg", "png", "gif", "bmp", "tga", "tiff", "webp", "ico",
            "avif", "svg",
            "arw", "cr2", "nef", "dng", "orf", "raf", "srw",
        ]
    }

    pub fn save(path: &Path, rgba_data: &[u8], w: u32, h: u32) -> Result<()> {
        use zune_image::image::Image;
        let mut img = Image::from_u8(rgba_data, w as usize, h as usize, zune_core::colorspace::ColorSpace::RGBA);
        img.save(path)
            .map_err(|e| eyre!("Failed to save image: {e:?}"))?;
        Ok(())
    }
}
