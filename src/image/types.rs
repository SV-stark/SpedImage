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
            | Self::Svg => true,
            _ => false,
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
