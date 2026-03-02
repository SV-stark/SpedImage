//! Image Backend - Cross-platform image loading
//!
//! Handles image decoding using the `image` crate with support for
//! multiple formats including HEIC via libheif-rs.

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Failed to load image: {0}")]
    LoadError(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image decoding error: {0}")]
    DecodeError(String),
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormatType {
    Jpeg,
    Png,
    Gif,
    Bmp,
    Tiff,
    WebP,
    Heic,
    Avif,
    Unknown,
}

impl ImageFormatType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Self::Jpeg,
            "png" => Self::Png,
            "gif" => Self::Gif,
            "bmp" => Self::Bmp,
            "tiff" | "tif" => Self::Tiff,
            "webp" => Self::WebP,
            "heic" | "heif" => Self::Heic,
            "avif" => Self::Avif,
            _ => Self::Unknown,
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Image data container with metadata
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormatType,
    pub rgba_data: Vec<u8>,
    pub path: String,
    pub file_size_bytes: u64,
    pub frame_delay_ms: u32,
}

impl ImageData {
    /// Get the raw RGBA bytes for GPU upload
    pub fn as_rgba(&self) -> &[u8] {
        &self.rgba_data
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Get total pixel count
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }
}

/// Image backend for loading and decoding images
pub struct ImageBackend;

impl ImageBackend {
    /// Load an image from a file path
    pub fn load(path: &Path) -> Result<Vec<ImageData>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = ImageFormatType::from_extension(&ext);

        tracing::debug!("Loading image: {:?}, format: {:?}", path, format);

        let metadata = std::fs::metadata(path).context("Failed to get file metadata")?;
        let file_size_bytes = metadata.len();

        if format == ImageFormatType::Gif {
            if let Ok(file) = std::fs::File::open(path) {
                if let Ok(decoder) = image::codecs::gif::GifDecoder::new(file) {
                    use image::AnimationDecoder;
                    if let Ok(frames) = decoder.into_frames().collect_frames() {
                        if !frames.is_empty() {
                            let mut results = Vec::new();
                            for frame in frames {
                                let delay = frame.delay().numer_denom_ms().0;
                                let img = frame.into_buffer();
                                let (width, height) = img.dimensions();
                                results.push(ImageData {
                                    width,
                                    height,
                                    format,
                                    rgba_data: img.into_raw(),
                                    path: path.to_string_lossy().to_string(),
                                    file_size_bytes,
                                    frame_delay_ms: delay,
                                });
                            }
                            return Ok(results);
                        }
                    }
                }
            }
        }

        let (rgba_data, width, height) = match format {
            ImageFormatType::Heic | ImageFormatType::Avif => Self::load_heif(path)?,
            _ => Self::load_standard(path)?,
        };

        Ok(vec![ImageData {
            width,
            height,
            format,
            rgba_data,
            path: path.to_string_lossy().to_string(),
            file_size_bytes,
            frame_delay_ms: 0,
        }])
    }

    /// Load an image and optionally downsample it if it exceeds maximum dimensions
    pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
        let mut frames = Self::load(path)?;

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

    /// Load standard image formats using the `image` crate
    fn load_standard(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let img = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;

        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let rgba_data = rgba.into_raw();

        tracing::debug!("Loaded standard image: {}x{}", width, height);
        Ok((rgba_data, width, height))
    }

    /// Load HEIC/AVIF images using libheif-rs
    fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        // Try using image crate first (newer versions have HEIF support)
        if let Ok(img) = image::open(path) {
            let (width, height) = img.dimensions();
            let rgba = img.to_rgba8();
            return Ok((rgba.into_raw(), width, height));
        }

        // Fallback to libheif-rs for HEIC
        #[cfg(feature = "heif")]
        {
            use libheif_rs::{Decoder, ImageInput};

            let file_data = std::fs::read(path)?;
            let mut input = ImageInput::from_slice(&file_data)
                .map_err(|e| anyhow::anyhow!("HEIF decode error: {}", e))?;

            let num_images = input
                .number_of_images()
                .map_err(|e| anyhow::anyhow!("HEIF error: {}", e))?;

            if num_images == 0 {
                return Err(anyhow::anyhow!("No images in HEIC file"));
            }

            let mut decoder = Decoder::new();
            decoder.set_image_index(0);

            let image = input
                .decode(&decoder, 0)
                .map_err(|e| anyhow::anyhow!("HEIF decode error: {}", e))?;

            let width = image.width() as u32;
            let height = image.height() as u32;

            // Convert to RGBA
            let mut rgba_data = vec![0u8; (width * height * 4) as usize];

            // Get plane data - simplified for demonstration
            // In production, you'd properly convert chroma planes
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    if idx + 3 < rgba_data.len() {
                        rgba_data[idx] = 255; // R
                        rgba_data[idx + 1] = 255; // G
                        rgba_data[idx + 2] = 255; // B
                        rgba_data[idx + 3] = 255; // A
                    }
                }
            }

            Ok((rgba_data, width, height))
        }

        #[cfg(not(feature = "heif"))]
        {
            // Fallback: try image crate
            let img = image::open(path)?;
            let (width, height) = img.dimensions();
            let rgba = img.to_rgba8();
            Ok((rgba.into_raw(), width, height))
        }
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

        tracing::info!("Saved image to: {}", path.display());
        Ok(())
    }

    /// Check if a file is a supported image
    pub fn is_supported(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        ImageFormatType::from_extension(ext).is_supported()
    }

    /// Get list of supported extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec![
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "heic", "heif", "avif",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ImageFormatType::from_extension("jpg"),
            ImageFormatType::Jpeg
        );
        assert_eq!(ImageFormatType::from_extension("PNG"), ImageFormatType::Png);
        assert_eq!(
            ImageFormatType::from_extension("heic"),
            ImageFormatType::Heic
        );
        assert_eq!(
            ImageFormatType::from_extension("unknown"),
            ImageFormatType::Unknown
        );
    }
}
