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
    /// RAW camera formats (Canon, Nikon, Sony, Fuji, Adobe DNG, etc.)
    Raw,
    /// SVG vector graphics
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
            // RAW formats: Canon, Nikon, Sony, Fuji, Olympus, Panasonic, Leica, Adobe
            "cr2" | "cr3" | "crw"  // Canon
            | "nef" | "nrw"        // Nikon
            | "arw" | "srf" | "sr2" // Sony
            | "raf"                // Fujifilm
            | "orf"                // Olympus
            | "rw2"                // Panasonic
            | "dng"                // Adobe DNG (universal)
            | "mrw"                // Minolta
            | "pef"                // Pentax
            | "3fr"                // Hasselblad
            | "rwl"                // Leica
            | "raw" | "rw1"        // Generic
            => Self::Raw,
            "svg" => Self::Svg,
            _ => Self::Unknown,
        }
    }

    pub fn is_supported(&self) -> bool {
        match self {
            Self::Unknown => false,
            // RAW requires the `raw` feature, SVG requires `svg`, HEIC/AVIF require `heif`
            #[cfg(not(feature = "raw"))]
            Self::Raw => false,
            #[cfg(not(feature = "svg"))]
            Self::Svg => false,
            #[cfg(not(feature = "heif"))]
            Self::Heic => false,
            #[cfg(not(feature = "heif"))]
            Self::Avif => false,
            _ => true,
        }
    }
}

/// Image data container with metadata
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormatType,
    pub rgba_data: Vec<u8>,
    pub path: std::path::PathBuf,
    pub file_size_bytes: u64,
    pub frame_delay_ms: u32,
    pub exif_info: Option<String>,
    pub histogram: Option<([u32; 256], [u32; 256], [u32; 256])>,
    pub exif_loaded: bool,
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

        for chunk in self.rgba_data.chunks_exact(4) {
            r_hist[chunk[0] as usize] += 1;
            g_hist[chunk[1] as usize] += 1;
            b_hist[chunk[2] as usize] += 1;
        }

        self.histogram = Some((r_hist, g_hist, b_hist));
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
        #[cfg(feature = "svg")]
        assert_eq!(ImageFormatType::from_extension("svg"), ImageFormatType::Svg);
        #[cfg(feature = "raw")]
        {
            assert_eq!(ImageFormatType::from_extension("dng"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("arw"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("cr2"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("nef"), ImageFormatType::Raw);
        }
        assert_eq!(
            ImageFormatType::from_extension("unknown"),
            ImageFormatType::Unknown
        );
    }

    #[test]
    fn test_format_is_supported() {
        assert!(ImageFormatType::Jpeg.is_supported());
        assert!(ImageFormatType::Png.is_supported());
        assert!(ImageFormatType::Gif.is_supported());
        assert!(ImageFormatType::Bmp.is_supported());
        assert!(ImageFormatType::Tiff.is_supported());
        assert!(ImageFormatType::WebP.is_supported());
        assert!(ImageFormatType::Heic.is_supported());
        assert!(ImageFormatType::Avif.is_supported());
        assert!(!ImageFormatType::Unknown.is_supported());

        #[cfg(feature = "raw")]
        assert!(ImageFormatType::Raw.is_supported());
        #[cfg(not(feature = "raw"))]
        assert!(!ImageFormatType::Raw.is_supported());

        #[cfg(feature = "svg")]
        assert!(ImageFormatType::Svg.is_supported());
        #[cfg(not(feature = "svg"))]
        assert!(!ImageFormatType::Svg.is_supported());
    }

    #[test]
    fn test_image_data_methods() {
        let data = ImageData {
            width: 800,
            height: 600,
            format: ImageFormatType::Png,
            rgba_data: vec![0u8; 800 * 600 * 4],
            path: std::path::PathBuf::from("/test/image.png"),
            file_size_bytes: 1024,
            frame_delay_ms: 0,
            exif_info: None,
            histogram: None,
            exif_loaded: false,
        };

        assert_eq!(data.as_rgba().len(), 800 * 600 * 4);
        assert!((data.aspect_ratio() - 800.0 / 600.0).abs() < 0.001);
        assert_eq!(data.pixel_count(), 800 * 600);
    }
}
