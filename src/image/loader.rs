use anyhow::{Context, Result};
use image::GenericImageView;
use std::path::Path;

use super::types::{ImageData, ImageFormatType};

pub struct ImageLoader;

impl ImageLoader {
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
                                path: path.to_path_buf(),
                                file_size_bytes,
                                frame_delay_ms: delay,
                                exif_info: None,
                                histogram: None,
                                exif_loaded: false,
                            });
                        }
                        return Ok(results);
                    }
                }
            }
        }

        // Note: EXIF is now lazy-loaded on demand via load_exif()

        let (rgba_data, width, height) = match format {
            ImageFormatType::Heic | ImageFormatType::Avif => Self::load_heif(path)?,
            #[cfg(feature = "raw")]
            ImageFormatType::Raw => Self::load_raw(path)?,
            #[cfg(feature = "svg")]
            ImageFormatType::Svg => Self::load_svg(path)?,
            _ => Self::load_standard(path)?,
        };

        // Note: EXIF is lazy-loaded on demand via load_exif()
        Ok(vec![ImageData {
            width,
            height,
            format,
            rgba_data,
            path: path.to_path_buf(),
            file_size_bytes,
            frame_delay_ms: 0,
            exif_info: None,
            histogram: None,
            exif_loaded: false,
        }])
    }

    /// Load standard image formats using the `image` crate
    fn load_standard(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let img = image::open(path).with_context(|| {
            let p = path.display();
            format!("Failed to open image: {p}")
        })?;

        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let rgba_data = rgba.into_raw();

        tracing::debug!("Loaded standard image: {width}x{height}");
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
                let (w, h) = (data.1, data.2);
                tracing::debug!("Loaded HEIC via WIC: {w}x{h}");
                Ok(data)
            }
            Err(e) => {
                let msg = format!(
                    "Failed to decode HEIC using Windows WIC: {e}. You may need to install the 'HEVC Video Extensions' and 'HEIF Image Extensions' from the Microsoft Store.", 
                );
                tracing::error!("{msg}");
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

    /// Load RAW camera files using the `rawler` crate.
    #[cfg(feature = "raw")]
    fn load_raw(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        tracing::debug!("Loading RAW file: {:?}", path);

        let result: anyhow::Result<(Vec<u8>, u32, u32)> = (|| {
            let raw_image =
                rawler::decode_file(path).map_err(|e| anyhow::anyhow!("rawler decode: {e}"))?;

            let width = raw_image.width;
            let height = raw_image.height;

            let rgba: Vec<u8> = match raw_image.data {
                rawler::RawImageData::Integer(ref pixels) => {
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
                            match channel {
                                0 => {
                                    out[dest] = v;
                                    out[dest + 3] = 255;
                                }
                                1 => {
                                    out[dest + 1] = v;
                                    out[dest + 3] = 255;
                                }
                                2 => {
                                    out[dest + 2] = v;
                                    out[dest + 3] = 255;
                                }
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

            tracing::debug!("RAW decoded: {width}x{height}");
            Ok((rgba, width as u32, height as u32))
        })();

        match result {
            Ok(data) => Ok(data),
            Err(rawler_err) => {
                tracing::warn!("rawler failed ({rawler_err}), trying WIC fallback");
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

    /// Rasterize an SVG file to RGBA using `resvg`.
    #[cfg(feature = "svg")]
    fn load_svg(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        tracing::debug!("Loading SVG file: {:?}", path);

        let svg_data = std::fs::read(path)?;

        use resvg::tiny_skia;
        use resvg::usvg;

        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_data(&svg_data, &opts)
            .map_err(|e| anyhow::anyhow!("SVG parse error: {e}"))?;

        const MAX_SIDE: f32 = 2000.0;
        let svg_size = tree.size();
        let (svg_w, svg_h) = (svg_size.width(), svg_size.height());
        let scale = (MAX_SIDE / svg_w.max(svg_h)).min(1.0);
        let out_w = (svg_w * scale).round() as u32;
        let out_h = (svg_h * scale).round() as u32;

        if out_w == 0 || out_h == 0 {
            return Err(anyhow::anyhow!("SVG has zero-size viewBox"));
        }

        let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate SVG pixmap ({out_w}x{out_h})"))?;

        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        tracing::debug!("SVG rasterized at {out_w}x{out_h} (scale {scale:.2})");

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
}
