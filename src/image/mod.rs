mod loader;
mod metadata;
mod processing;
mod types;

pub use loader::ImageLoader;
pub use processing::ImageProcessor;
pub use types::{ImageData, ImageError, ImageFormatType};

use color_eyre::eyre::Result;

pub struct ImageBackend;

impl ImageBackend {
    /// Load an image from path (returns multiple frames for GIFs)
    pub fn load(path: &std::path::Path) -> Result<Vec<ImageData>> {
        let (frames, _format) = ImageLoader::load(path)?;
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

    /// Check if format is supported
    pub fn is_supported(path: &std::path::Path) -> bool {
        ImageProcessor::is_supported(path)
    }

    /// Supported extensions list
    pub fn supported_extensions() -> Vec<&'static str> {
        ImageProcessor::supported_extensions()
    }

    /// Save an image to disk
    pub fn save(
        path: &std::path::Path,
        rgba_data: &[u8],
        w: u32,
        h: u32,
    ) -> Result<()> {
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
