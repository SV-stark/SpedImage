use std::sync::Arc;
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
    Jxl,
    Raw,
    Svg,
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
            "jxl" => Self::Jxl,
            "svg" => Self::Svg,
            "arw" | "cr2" | "nef" | "dng" | "orf" | "raf" | "srw" => Self::Raw,
            _ => Self::Unknown,
        }
    }

    pub fn is_supported(&self) -> bool {
        match self {
            Self::Unknown => false,
            // Core formats + RAW via imagepipe
            Self::Jpeg
            | Self::Png
            | Self::Gif
            | Self::Bmp
            | Self::Tiff
            | Self::WebP
            | Self::Raw
            | Self::Avif
            | Self::Heic
            | Self::Jxl
            | Self::Svg => true,
        }
    }
}

/// Image data container with metadata
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormatType,
    pub rgba_data: Arc<Vec<u8>>,
    pub path: std::path::PathBuf,
    pub file_size_bytes: u64,
    pub frame_delay_ms: u32,
    pub exif_info: Option<String>,
    pub histogram: Option<([u32; 256], [u32; 256], [u32; 256])>,
    pub exif_loaded: bool,
    pub is_downsampled: bool,
}

impl ImageData {
    /// Get the raw RGBA bytes for GPU upload
    pub fn as_rgba(&self) -> &[u8] {
        &self.rgba_data
    }

    /// Load EXIF data lazily on demand
    pub fn load_exif(&mut self) {
        if self.exif_loaded {
            return;
        }
        self.exif_info = crate::image::metadata::extract_exif_lazy(&self.path);
        self.exif_loaded = true;
    }

    pub fn compute_histogram(&mut self) {
        if self.histogram.is_some() {
            return;
        }
        if self.rgba_data.is_empty() {
            return;
        }
        let mut r_hist = [0u32; 256];
        let mut g_hist = [0u32; 256];
        let mut b_hist = [0u32; 256];
        Self::compute_rgb_histogram(&self.rgba_data, &mut r_hist, &mut g_hist, &mut b_hist);
        self.histogram = Some((r_hist, g_hist, b_hist));
    }

    pub fn compute_rgb_histogram(
        rgba: &[u8],
        r_hist: &mut [u32; 256],
        g_hist: &mut [u32; 256],
        b_hist: &mut [u32; 256],
    ) {
        use rayon::prelude::*;

        // Process in parallel using chunk sizes of 4096 pixels (16KB)
        let chunk_size = 4096 * 4;
        let (r, g, b) = rgba
            .par_chunks_exact(chunk_size)
            .map(|chunk| {
                let mut local_r = [0u32; 256];
                let mut local_g = [0u32; 256];
                let mut local_b = [0u32; 256];
                for pixel in chunk.chunks_exact(4) {
                    local_r[pixel[0] as usize] += 1;
                    local_g[pixel[1] as usize] += 1;
                    local_b[pixel[2] as usize] += 1;
                }
                (local_r, local_g, local_b)
            })
            .reduce(
                || ([0u32; 256], [0u32; 256], [0u32; 256]),
                |mut acc, local| {
                    for i in 0..256 {
                        acc.0[i] += local.0[i];
                        acc.1[i] += local.1[i];
                        acc.2[i] += local.2[i];
                    }
                    acc
                },
            );

        let processed_len = (rgba.len() / chunk_size) * chunk_size;
        let remainder = &rgba[processed_len..];

        let mut final_r = r;
        let mut final_g = g;
        let mut final_b = b;
        for pixel in remainder.chunks_exact(4) {
            final_r[pixel[0] as usize] += 1;
            final_g[pixel[1] as usize] += 1;
            final_b[pixel[2] as usize] += 1;
        }

        r_hist.copy_from_slice(&final_r);
        g_hist.copy_from_slice(&final_g);
        b_hist.copy_from_slice(&final_b);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(
            ImageFormatType::from_extension("jpg"),
            ImageFormatType::Jpeg
        );
        assert_eq!(
            ImageFormatType::from_extension("jpeg"),
            ImageFormatType::Jpeg
        );
        assert_eq!(ImageFormatType::from_extension("png"), ImageFormatType::Png);
        assert_eq!(ImageFormatType::from_extension("gif"), ImageFormatType::Gif);
        assert_eq!(ImageFormatType::from_extension("bmp"), ImageFormatType::Bmp);
        assert_eq!(
            ImageFormatType::from_extension("tiff"),
            ImageFormatType::Tiff
        );
        assert_eq!(
            ImageFormatType::from_extension("tif"),
            ImageFormatType::Tiff
        );
        assert_eq!(
            ImageFormatType::from_extension("webp"),
            ImageFormatType::WebP
        );
        assert_eq!(
            ImageFormatType::from_extension("heic"),
            ImageFormatType::Heic
        );
        assert_eq!(
            ImageFormatType::from_extension("heif"),
            ImageFormatType::Heic
        );
        assert_eq!(
            ImageFormatType::from_extension("avif"),
            ImageFormatType::Avif
        );
        assert_eq!(ImageFormatType::from_extension("jxl"), ImageFormatType::Jxl);
        assert_eq!(ImageFormatType::from_extension("svg"), ImageFormatType::Svg);
        assert_eq!(ImageFormatType::from_extension("arw"), ImageFormatType::Raw);
        assert_eq!(ImageFormatType::from_extension("cr2"), ImageFormatType::Raw);
        assert_eq!(ImageFormatType::from_extension("nef"), ImageFormatType::Raw);
        assert_eq!(ImageFormatType::from_extension("dng"), ImageFormatType::Raw);
        assert_eq!(
            ImageFormatType::from_extension("txt"),
            ImageFormatType::Unknown
        );
    }

    #[test]
    fn test_image_format_case_insensitive() {
        assert_eq!(
            ImageFormatType::from_extension("JPG"),
            ImageFormatType::Jpeg
        );
        assert_eq!(ImageFormatType::from_extension("PNG"), ImageFormatType::Png);
        assert_eq!(ImageFormatType::from_extension("GIF"), ImageFormatType::Gif);
    }

    #[test]
    fn test_image_format_is_supported() {
        assert!(ImageFormatType::Jpeg.is_supported());
        assert!(ImageFormatType::Png.is_supported());
        assert!(ImageFormatType::Gif.is_supported());
        assert!(ImageFormatType::Bmp.is_supported());
        assert!(ImageFormatType::Tiff.is_supported());
        assert!(ImageFormatType::WebP.is_supported());
        assert!(ImageFormatType::Heic.is_supported());
        assert!(ImageFormatType::Avif.is_supported());
        assert!(ImageFormatType::Jxl.is_supported());
        assert!(ImageFormatType::Svg.is_supported());
        assert!(ImageFormatType::Raw.is_supported());
        assert!(!ImageFormatType::Unknown.is_supported());
    }

    #[test]
    fn test_image_data_aspect_ratio() {
        let img = ImageData {
            width: 1920,
            height: 1080,
            format: ImageFormatType::Png,
            rgba_data: Arc::new(vec![0; 1920 * 1080 * 4]),
            path: std::path::PathBuf::from("test.png"),
            file_size_bytes: 1000,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled: false,
        };
        let ratio = img.aspect_ratio();
        assert!((ratio - 1920.0 / 1080.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_image_data_pixel_count() {
        let img = ImageData {
            width: 100,
            height: 200,
            format: ImageFormatType::Png,
            rgba_data: Arc::new(vec![0; 100 * 200 * 4]),
            path: std::path::PathBuf::from("test.png"),
            file_size_bytes: 1000,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled: false,
        };
        assert_eq!(img.pixel_count(), 20000);
    }

    #[test]
    fn test_image_data_aspect_ratio_square() {
        let img = ImageData {
            width: 512,
            height: 512,
            format: ImageFormatType::Png,
            rgba_data: Arc::new(vec![0; 512 * 512 * 4]),
            path: std::path::PathBuf::from("test.png"),
            file_size_bytes: 500,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled: false,
        };
        assert!((img.aspect_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_image_data_empty_histogram() {
        let mut img = ImageData {
            width: 2,
            height: 2,
            format: ImageFormatType::Png,
            rgba_data: Arc::new(vec![
                255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255,
            ]),
            path: std::path::PathBuf::from("test.png"),
            file_size_bytes: 100,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled: false,
        };
        assert!(img.histogram.is_none());
        img.compute_histogram();
        assert!(img.histogram.is_some());
        let (r, g, b) = img.histogram.unwrap();
        assert_eq!(r[255], 2);
        assert_eq!(g[255], 2);
        assert_eq!(b[0], 4); // all 4 pixels have 0 blue
    }
}
