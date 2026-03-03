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
            // RAW requires the `raw` feature, SVG requires `svg`
            #[cfg(not(feature = "raw"))]
            Self::Raw => false,
            #[cfg(not(feature = "svg"))]
            Self::Svg => false,
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
                let reader = std::io::BufReader::new(file);
                if let Ok(decoder) = image::codecs::gif::GifDecoder::new(reader) {
                    use image::AnimationDecoder;
                    let frames: Vec<image::Frame> =
                        decoder.into_frames().filter_map(|f| f.ok()).collect();
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

        let (rgba_data, width, height) = match format {
            ImageFormatType::Heic | ImageFormatType::Avif => Self::load_heif(path)?,
            #[cfg(feature = "raw")]
            ImageFormatType::Raw => Self::load_raw(path)?,
            #[cfg(feature = "svg")]
            ImageFormatType::Svg => Self::load_svg(path)?,
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

    /// Load HEIC/AVIF images using built-in OS codecs (WIC on Windows)
    #[cfg(windows)]
    fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        // First try the image crate in case it handles it
        if let Ok(img) = image::open(path) {
            let (width, height) = img.dimensions();
            let rgba = img.to_rgba8();
            return Ok((rgba.into_raw(), width, height));
        }

        // Fallback to Windows Imaging Component (WIC)
        use windows::core::HSTRING;
        use windows::Win32::Foundation::GENERIC_READ;
        use windows::Win32::Graphics::Imaging::*;
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
        };

        // Initialize COM (it might already be initialized by winit, but safe to re-call)
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        let result = (|| -> windows::core::Result<(Vec<u8>, u32, u32)> {
            unsafe {
                let factory: IWICImagingFactory =
                    CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;

                let path_hstring = HSTRING::from(path.as_os_str());
                let decoder = factory.CreateDecoderFromFilename(
                    &path_hstring,
                    None,
                    GENERIC_READ,
                    WICDecodeMetadataCacheOnDemand,
                )?;

                let frame = decoder.GetFrame(0)?;

                let mut width = 0;
                let mut height = 0;
                frame.GetSize(&mut width, &mut height)?;

                let converter = factory.CreateFormatConverter()?;
                converter.Initialize(
                    &frame,
                    &GUID_WICPixelFormat32bppRGBA,
                    WICBitmapDitherTypeNone,
                    None,
                    0.0,
                    WICBitmapPaletteTypeCustom,
                )?;

                let stride = width * 4;
                let size = stride * height;
                let mut buffer: Vec<u8> = vec![0; size as usize];

                converter.CopyPixels(std::ptr::null(), stride, &mut buffer)?;

                Ok((buffer, width, height))
            }
        })();

        match result {
            Ok(data) => {
                tracing::debug!("Loaded HEIC via WIC: {}x{}", data.1, data.2);
                Ok(data)
            }
            Err(e) => {
                let msg = format!(
                    "Failed to decode HEIC using Windows WIC: {}. You may need to install the 'HEVC Video Extensions' and 'HEIF Image Extensions' from the Microsoft Store.", 
                    e
                );
                tracing::error!("{}", msg);
                Err(anyhow::anyhow!(msg))
            }
        }
    }

    #[cfg(not(windows))]
    fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        // Fallback: try image crate, but provide custom error if it fails
        let res = image::open(path);
        match res {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let rgba = img.to_rgba8();
                Ok((rgba.into_raw(), width, height))
            }
            Err(_) => Err(anyhow::anyhow!(
                "HEIC/AVIF support is currently only available natively on Windows. \
                     Please use JPEG or PNG on this platform, or install an HEIF decoder."
            )),
        }
    }

    // -------------------------------------------------------------------------
    // RAW camera format loading
    // -------------------------------------------------------------------------

    /// Load RAW camera files using the `rawler` crate.
    /// Falls back to Windows WIC on Windows if rawler cannot decode the format.
    #[cfg(feature = "raw")]
    fn load_raw(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        tracing::debug!("Loading RAW file: {:?}", path);

        let result: anyhow::Result<(Vec<u8>, u32, u32)> = (|| {
            let raw_image =
                rawler::decode_file(path).map_err(|e| anyhow::anyhow!("rawler decode: {}", e))?;

            let width = raw_image.width;
            let height = raw_image.height;

            // rawler gives us 16-bit CFA mosaic data (Bayer pattern).
            // We do a minimal debayer by treating R/G/B channels via the CFA pattern,
            // then scale 16-bit → 8-bit via a simple linear mapping.
            let rgba: Vec<u8> = match raw_image.data {
                rawler::RawImageData::Integer(ref pixels) => {
                    // Find actual max to normalise instead of assuming 65535
                    let max_val = pixels.iter().copied().max().unwrap_or(1).max(1) as f32;
                    let cfa = &raw_image.camera.cfa;
                    let mut out = vec![0u8; (width * height * 4) as usize];
                    for y in 0..height {
                        for x in 0..width {
                            let idx = (y * width + x) as usize;
                            let raw = pixels[idx];
                            let channel = cfa.color_at(x, y);
                            let v = (raw as f32 / max_val * 255.0).clamp(0.0, 255.0) as u8;
                            let dest = idx * 4;
                            // Assign to the correct RGB channel; others stay 0 until
                            // averaged by neighbouring pixels in a full debayer.
                            // For this fast-path we just display each channel visually.
                            match channel {
                                0 => {
                                    out[dest] = v;
                                    out[dest + 3] = 255;
                                } // R
                                1 => {
                                    out[dest + 1] = v;
                                    out[dest + 3] = 255;
                                } // G
                                2 => {
                                    out[dest + 2] = v;
                                    out[dest + 3] = 255;
                                } // B
                                _ => {
                                    out[dest] = v;
                                    out[dest + 1] = v;
                                    out[dest + 2] = v;
                                    out[dest + 3] = 255;
                                }
                            }
                        }
                    }
                    out
                }
                rawler::RawImageData::Float(ref pixels) => pixels
                    .iter()
                    .flat_map(|&v| {
                        let b = (v.clamp(0.0, 1.0) * 255.0) as u8;
                        [b, b, b, 255u8]
                    })
                    .collect(),
            };

            tracing::debug!("RAW decoded: {}x{}", width, height);
            Ok((rgba, width as u32, height as u32))
        })();

        match result {
            Ok(data) => Ok(data),
            Err(rawler_err) => {
                tracing::warn!("rawler failed ({}), trying WIC fallback", rawler_err);
                // Windows fallback: WIC can handle DNG/ARW/NEF with proper codec installed
                #[cfg(windows)]
                {
                    if let Ok(data) = Self::load_heif(path) {
                        return Ok(data);
                    }
                }
                Err(anyhow::anyhow!(
                    "Failed to load RAW file. \
                    On Windows, install the \"Microsoft Raw Image Extension\" from the Store. \
                    Error: {}",
                    rawler_err
                ))
            }
        }
    }

    // -------------------------------------------------------------------------
    // SVG vector graphics loading
    // -------------------------------------------------------------------------

    /// Rasterize an SVG file to RGBA using `resvg`.
    /// SVGs are rendered at their native viewBox size, capped at 2000px on the
    /// longest axis so they look sharp on 4K screens without wasting VRAM.
    #[cfg(feature = "svg")]
    fn load_svg(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        tracing::debug!("Loading SVG file: {:?}", path);

        let svg_data = std::fs::read(path)?;

        // resvg re-exports usvg and tiny_skia — use those to avoid version mismatches.
        use resvg::tiny_skia;
        use resvg::usvg;

        // Parse SVG
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_data(&svg_data, &opts)
            .map_err(|e| anyhow::anyhow!("SVG parse error: {}", e))?;

        // Determine output size, cap at 2000px on longest edge
        const MAX_SIDE: f32 = 2000.0;
        let svg_size = tree.size();
        let (svg_w, svg_h) = (svg_size.width(), svg_size.height());
        let scale = (MAX_SIDE / svg_w.max(svg_h)).min(1.0);
        let out_w = (svg_w * scale).round() as u32;
        let out_h = (svg_h * scale).round() as u32;

        if out_w == 0 || out_h == 0 {
            return Err(anyhow::anyhow!("SVG has zero-size viewBox"));
        }

        // Create a transparent pixmap and render into it
        let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h).ok_or_else(|| {
            anyhow::anyhow!("Failed to allocate SVG pixmap ({}x{})", out_w, out_h)
        })?;

        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        tracing::debug!("SVG rasterized at {}x{} (scale {:.2})", out_w, out_h, scale);

        // tiny-skia produces premultiplied RGBA (BGRA on some platforms).
        // Un-premultiply alpha for correct GPU texture sampling.
        let mut rgba = pixmap.take();
        for px in rgba.chunks_exact_mut(4) {
            let a = px[3];
            if a > 0 && a < 255 {
                let inv = 255.0 / a as f32;
                px[0] = (px[0] as f32 * inv).min(255.0) as u8;
                px[1] = (px[1] as f32 * inv).min(255.0) as u8;
                px[2] = (px[2] as f32 * inv).min(255.0) as u8;
            }
        }

        Ok((rgba, out_w, out_h))
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
    fn test_load_and_downsample() {
        let temp_dir = std::env::temp_dir();
        let test_img_path = temp_dir.join("test_spedimage_downsample.png");

        // Create a 1000x1000 test image
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 1000));
        img.save(&test_img_path).unwrap();

        // Downsample to 500x500
        let result = ImageBackend::load_and_downsample(&test_img_path, 500, 500).unwrap();
        assert_eq!(result.len(), 1);
        let data = &result[0];

        assert_eq!(data.width, 500);
        assert_eq!(data.height, 500);
        assert_eq!(data.rgba_data.len(), 500 * 500 * 4);

        // Downsample to 400x300 (should keep aspect ratio -> 300x300)
        let result = ImageBackend::load_and_downsample(&test_img_path, 400, 300).unwrap();
        let data = &result[0];

        // 1000x1000 mapped to fit within 400x300 -> 300x300
        assert_eq!(data.width, 300);
        assert_eq!(data.height, 300);

        std::fs::remove_file(&test_img_path).unwrap();
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
            path: "/test/image.png".to_string(),
            file_size_bytes: 1024,
            frame_delay_ms: 0,
        };

        assert_eq!(data.as_rgba().len(), 800 * 600 * 4);
        assert!((data.aspect_ratio() - 800.0 / 600.0).abs() < 0.001);
        assert_eq!(data.pixel_count(), 800 * 600);
    }

    #[test]
    fn test_save_jpeg() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_spedimage_save.jpg");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_png() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_spedimage_save.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_bmp() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_spedimage_save.bmp");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_is_supported() {
        let temp_dir = std::env::temp_dir();
        let jpg_path = temp_dir.join("test.jpg");
        let png_path = temp_dir.join("test.png");
        let unknown_path = temp_dir.join("test.xyz");

        std::fs::File::create(&jpg_path).unwrap();
        std::fs::File::create(&png_path).unwrap();
        std::fs::File::create(&unknown_path).unwrap();

        assert!(ImageBackend::is_supported(&jpg_path));
        assert!(ImageBackend::is_supported(&png_path));
        assert!(!ImageBackend::is_supported(&unknown_path));

        std::fs::remove_file(&jpg_path).unwrap();
        std::fs::remove_file(&png_path).unwrap();
        std::fs::remove_file(&unknown_path).unwrap();
    }

    #[test]
    fn test_supported_extensions() {
        let exts = ImageBackend::supported_extensions();
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"tiff"));
        assert!(exts.contains(&"webp"));
        assert!(exts.contains(&"heic"));
        assert!(exts.contains(&"avif"));

        #[cfg(feature = "raw")]
        {
            assert!(exts.contains(&"dng"));
            assert!(exts.contains(&"arw"));
            assert!(exts.contains(&"cr2"));
        }

        #[cfg(feature = "svg")]
        assert!(exts.contains(&"svg"));
    }

    #[test]
    fn test_load_standard_image() {
        let temp_dir = std::env::temp_dir();
        let test_img_path = temp_dir.join("test_load.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        img.save(&test_img_path).unwrap();

        let result = ImageBackend::load(&test_img_path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].width, 100);
        assert_eq!(result[0].height, 100);

        std::fs::remove_file(&test_img_path).unwrap();
    }

    #[test]
    fn test_gif_loading() {
        let temp_dir = std::env::temp_dir();
        let test_gif_path = temp_dir.join("test_animated.gif");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10));
        img.save(&test_gif_path).unwrap();

        let result = ImageBackend::load(&test_gif_path).unwrap();
        assert!(!result.is_empty());

        std::fs::remove_file(&test_gif_path).unwrap();
    }
}
