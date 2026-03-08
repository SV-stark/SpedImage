use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat};
use std::path::Path;

use super::loader::ImageLoader;
use super::types::{ImageData, ImageFormatType};

pub struct ImageProcessor;

impl ImageProcessor {
    /// Load an image and optionally downsample it if it exceeds maximum dimensions
    pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
        let mut frames = ImageLoader::load(path)?;

        // Fast path: if the first frame is already small enough, just return
        if let Some(first) = frames.first() {
            if first.width <= max_w && first.height <= max_h {
                return Ok(frames);
            }
        } else {
            return Ok(frames);
        }

        use fast_image_resize as fir;
        let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));

        for data in frames.iter_mut() {
            let src = fir::Image::from_vec_u8(
                std::num::NonZeroU32::new(data.width).unwrap(),
                std::num::NonZeroU32::new(data.height).unwrap(),
                std::mem::take(&mut data.rgba_data),
                fir::PixelType::U8x4,
            )?;

            let scale_w = max_w as f32 / data.width as f32;
            let scale_h = max_h as f32 / data.height as f32;
            let scale = scale_w.min(scale_h).min(1.0);

            let dst_w = (data.width as f32 * scale).round() as u32;
            let dst_h = (data.height as f32 * scale).round() as u32;

            let mut dst = fir::Image::new(
                std::num::NonZeroU32::new(dst_w).unwrap(),
                std::num::NonZeroU32::new(dst_h).unwrap(),
                fir::PixelType::U8x4,
            );
            resizer.resize(&src.view(), &mut dst.view_mut())?;

            data.rgba_data = dst.into_vec();
            data.width = dst_w;
            data.height = dst_h;
        }

        Ok(frames)
    }

    /// Save image to file
    pub fn save(path: &Path, image: &DynamicImage, quality: u8) -> Result<()> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => {
                let rgb = image.to_rgb8();
                let mut file = std::fs::File::create(path)?;
                let encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
                rgb.write_with_encoder(encoder)
                    .context("Failed to write JPEG")?;
            }
            "png" => {
                image.save_with_format(path, ImageFormat::Png)?;
            }
            _ => {
                image.save(path)?;
            }
        }

        let p = path.display();
        tracing::info!("Saved image to: {p}");
        Ok(())
    }

    /// Check if a file is a supported image
    pub fn is_supported(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        ImageFormatType::from_extension(ext).is_supported()
    }

    /// Get list of supported extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        let mut exts = vec![
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "heic", "heif", "avif",
        ];
        #[cfg(feature = "raw")]
        exts.extend_from_slice(&[
            "cr2", "cr3", "crw", "nef", "nrw", "arw", "srf", "sr2", "raf", "orf", "rw2", "dng",
            "mrw", "pef", "3fr", "rwl", "raw", "rw1",
        ]);
        #[cfg(feature = "svg")]
        exts.push("svg");
        exts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_and_downsample() {
        let temp_dir = std::env::temp_dir();
        let test_img_path = temp_dir.join("test_spedimage_downsample.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 1000));
        img.save(&test_img_path).unwrap();

        let result = ImageProcessor::load_and_downsample(&test_img_path, 500, 500).unwrap();
        assert_eq!(result.len(), 1);
        let data = &result[0];

        assert_eq!(data.width, 500);
        assert_eq!(data.height, 500);
        assert_eq!(data.rgba_data.len(), 500 * 500 * 4);

        let result = ImageProcessor::load_and_downsample(&test_img_path, 400, 300).unwrap();
        let data = &result[0];

        assert_eq!(data.width, 300);
        assert_eq!(data.height, 300);

        std::fs::remove_file(&test_img_path).unwrap();
    }

    #[test]
    fn test_save_jpeg() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_jpeg.jpg");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageProcessor::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_png() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_png.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageProcessor::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_bmp() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_bmp.bmp");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageProcessor::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_is_supported() {
        let temp_dir = std::env::temp_dir();
        let jpg_path = temp_dir.join("test_is_supported_1.jpg");
        let png_path = temp_dir.join("test_is_supported_2.png");
        let unknown_path = temp_dir.join("test_is_supported_3.xyz");

        std::fs::File::create(&jpg_path).unwrap();
        std::fs::File::create(&png_path).unwrap();
        std::fs::File::create(&unknown_path).unwrap();

        assert!(ImageProcessor::is_supported(&jpg_path));
        assert!(ImageProcessor::is_supported(&png_path));
        assert!(!ImageProcessor::is_supported(&unknown_path));

        std::fs::remove_file(&jpg_path).unwrap();
        std::fs::remove_file(&png_path).unwrap();
        std::fs::remove_file(&unknown_path).unwrap();
    }

    #[test]
    fn test_supported_extensions() {
        let exts = ImageProcessor::supported_extensions();
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
    }
}
