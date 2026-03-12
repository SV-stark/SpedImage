use anyhow::Result;
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

                use std::num::NonZeroU32;
                let dst_w_nz = NonZeroU32::new(dst_w).unwrap();
                let dst_h_nz = NonZeroU32::new(dst_h).unwrap();

                // High-quality downsampling using fast_image_resize
                let src_image = fast_image_resize::Image::from_vec_u8(
                    NonZeroU32::new(img.width).unwrap(),
                    NonZeroU32::new(img.height).unwrap(),
                    img.rgba_data,
                    fast_image_resize::PixelType::U8x4,
                )
                .map_err(|e| anyhow::anyhow!("Failed to create source image for resize: {e}"))?;

                let mut dst_image = fast_image_resize::Image::new(
                    dst_w_nz,
                    dst_h_nz,
                    fast_image_resize::PixelType::U8x4,
                );

                // In fast_image_resize 3.0, it's Convolution(FilterType::Lanczos3)
                use fast_image_resize::{FilterType, ResizeAlg, Resizer};
                let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
                resizer
                    .resize(&src_image.view(), &mut dst_image.view_mut())
                    .map_err(|e| anyhow::anyhow!("Resize failed: {e}"))?;

                img.width = dst_w;
                img.height = dst_h;
                img.rgba_data = dst_image.into_vec();
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
        let mut exts = vec![
            "jpg", "jpeg", "png", "gif", "bmp", "tga", "tiff", "webp", "ico",
        ];

        #[cfg(feature = "raw")]
        exts.extend_from_slice(&["raw", "dng", "cr2", "nef", "arw", "orf", "raf"]);

        #[cfg(feature = "svg")]
        exts.push("svg");

        #[cfg(feature = "heif")]
        exts.extend_from_slice(&["heic", "heif", "avif"]);

        exts
    }

    pub fn save(path: &Path, image: &image::DynamicImage, quality: u8) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => {
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
                encoder.encode_image(image)?;
            }
            _ => {
                image.save(path)?;
            }
        }
        Ok(())
    }
}
