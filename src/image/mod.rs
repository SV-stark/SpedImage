mod gif_stream;
mod loader;
mod metadata;
mod processing;
mod save;
mod types;

pub use gif_stream::{GifFrameMsg, GifStreamMsg, stream_gif};
pub use loader::{FrameLimit, ImageLoader, LoadOptions};
pub use metadata::{
    ExifMeta, extract_exif_and_orientation, extract_orientation, parse_exif_block,
    parse_exif_datetime, read_date_taken, read_exif_meta,
};
pub use processing::ImageProcessor;
pub use save::{
    JPEG_QUALITY, SaveFormat, SaveOutcome, edited_output_path, encode, reset_exif_orientation,
    save_edited,
};
pub use types::{ImageData, ImageError, ImageFormatType};

use color_eyre::eyre::Result;

pub struct ImageBackend;

impl ImageBackend {
    /// Load an image from path (returns multiple frames for GIFs)
    pub fn load(path: &std::path::Path) -> Result<Vec<ImageData>> {
        let (frames, _format) = ImageLoader::load(path, None, None)?;
        Ok(frames)
    }

    /// Load and downsample for preview
    pub fn load_and_downsample(
        path: &std::path::Path,
        max_w: u32,
        max_h: u32,
    ) -> Result<Vec<ImageData>> {
        ImageProcessor::load_and_downsample(path, max_w, max_h)
    }

    /// Load and downsample with explicit decode options
    pub fn load_and_downsample_with(
        path: &std::path::Path,
        max_w: u32,
        max_h: u32,
        opts: crate::image::LoadOptions,
    ) -> Result<Vec<ImageData>> {
        ImageProcessor::load_and_downsample_with(path, max_w, max_h, opts)
    }

    /// Check if format is supported
    pub fn is_supported(path: &std::path::Path) -> bool {
        ImageProcessor::is_supported(path)
    }

    /// Supported extensions list
    pub fn supported_extensions() -> &'static [&'static str] {
        ImageProcessor::supported_extensions()
    }

    /// Apply adjustments on CPU
    pub fn apply_adjustments_cpu(
        rgba_data: &[u8],
        w: u32,
        h: u32,
        adjustments: &crate::render::ImageAdjustments,
    ) -> (Vec<u8>, u32, u32) {
        ImageProcessor::apply_adjustments_cpu(rgba_data, w, h, adjustments)
    }

    /// Save an image to disk as PNG, JPEG or WebP, chosen by `path`'s extension.
    pub fn save(path: &std::path::Path, rgba_data: &[u8], w: u32, h: u32) -> Result<()> {
        ImageProcessor::save(path, rgba_data, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_backend_is_supported() {
        assert!(ImageBackend::is_supported(&PathBuf::from("test.jpg")));
        assert!(ImageBackend::is_supported(&PathBuf::from("test.PNG")));
        assert!(!ImageBackend::is_supported(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_load_nonexistent() {
        let result = ImageBackend::load(&PathBuf::from("nonexistent_file_123.jpg"));
        assert!(result.is_err());
    }
}
