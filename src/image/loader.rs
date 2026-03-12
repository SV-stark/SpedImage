use anyhow::Result;
use image::GenericImageView;
use std::io::BufReader;
use std::path::Path;

use super::types::{ImageData, ImageFormatType};

pub struct ImageLoader;

impl ImageLoader {
    /// Load an image from a file path
    pub fn load(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format_type = ImageFormatType::from_extension(&ext);

        match format_type {
            #[cfg(feature = "heif")]
            ImageFormatType::Heic | ImageFormatType::Avif => {
                let (rgba, w, h) = Self::load_heif(path)?;
                Ok((
                    vec![ImageData {
                        path: path.to_path_buf(),
                        rgba_data: rgba,
                        width: w,
                        height: h,
                        format: format_type,
                        file_size_bytes: std::fs::metadata(path)?.len(),
                        frame_delay_ms: 0,
                        exif_info: None,
                        exif_loaded: false,
                        histogram: None,
                    }],
                    format_type,
                ))
            }
            #[cfg(feature = "raw")]
            ImageFormatType::Raw => {
                let (rgba, w, h) = Self::load_raw(path)?;
                Ok((
                    vec![ImageData {
                        path: path.to_path_buf(),
                        rgba_data: rgba,
                        width: w,
                        height: h,
                        format: format_type,
                        file_size_bytes: std::fs::metadata(path)?.len(),
                        frame_delay_ms: 0,
                        exif_info: None,
                        exif_loaded: false,
                        histogram: None,
                    }],
                    format_type,
                ))
            }
            #[cfg(feature = "svg")]
            ImageFormatType::Svg => {
                let (rgba, w, h) = Self::load_svg(path)?;
                Ok((
                    vec![ImageData {
                        path: path.to_path_buf(),
                        rgba_data: rgba,
                        width: w,
                        height: h,
                        format: format_type,
                        file_size_bytes: std::fs::metadata(path)?.len(),
                        frame_delay_ms: 0,
                        exif_info: None,
                        exif_loaded: false,
                        histogram: None,
                    }],
                    format_type,
                ))
            }
            _ => {
                let format = image::ImageFormat::from_path(path).unwrap_or(image::ImageFormat::Png);

                if format == image::ImageFormat::Gif {
                    Self::load_gif(path)
                } else {
                    let img = image::open(path)
                        .map_err(|e| anyhow::anyhow!("Failed to open image {path:?}: {e}"))?;
                    let (w, h) = img.dimensions();
                    let rgba = img.to_rgba8().into_raw();
                    Ok((
                        vec![ImageData {
                            path: path.to_path_buf(),
                            rgba_data: rgba,
                            width: w,
                            height: h,
                            format: format_type,
                            file_size_bytes: std::fs::metadata(path)?.len(),
                            frame_delay_ms: 0,
                            exif_info: None,
                            exif_loaded: false,
                            histogram: None,
                        }],
                        format_type,
                    ))
                }
            }
        }
    }

    #[cfg(feature = "heif")]
    fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

        let lib_heif = LibHeif::new();
        let ctx = HeifContext::read_from_file(
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
        )
        .map_err(|e| anyhow::anyhow!("Failed to open HEIF context: {e}"))?;

        let handle = ctx
            .primary_image_handle()
            .map_err(|e| anyhow::anyhow!("Failed to get primary image handle: {e}"))?;

        let image = lib_heif
            .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
            .map_err(|e| anyhow::anyhow!("Failed to decode HEIF image: {e}"))?;

        let width = image.width();
        let height = image.height();

        let planes = image.planes();
        let interleaved = planes.interleaved.ok_or_else(|| {
            anyhow::anyhow!("Failed to get interleaved plane data from HEIF image")
        })?;

        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        let rgb_data = interleaved.data;
        let stride = interleaved.stride;

        for y in 0..height {
            let row_start = y as usize * stride;
            for x in 0..width {
                let pixel_start = row_start + (x as usize * 3);
                if pixel_start + 2 < rgb_data.len() {
                    rgba_data.push(rgb_data[pixel_start]); // R
                    rgba_data.push(rgb_data[pixel_start + 1]); // G
                    rgba_data.push(rgb_data[pixel_start + 2]); // B
                    rgba_data.push(255); // A
                } else {
                    rgba_data.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
        }

        Ok((rgba_data, width, height))
    }

    #[cfg(feature = "raw")]
    fn load_raw(_path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        anyhow::bail!("RAW loading temporarily disabled due to environment limitations")
    }

    #[cfg(feature = "svg")]
    fn load_svg(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let opt = resvg::usvg::Options::default();
        let data = std::fs::read(path)?;
        let tree = resvg::usvg::Tree::from_data(&data, &opt)
            .map_err(|e| anyhow::anyhow!("Failed to parse SVG: {e}"))?;

        let pixmap_size = tree.size();
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(pixmap_size.width() as u32, pixmap_size.height() as u32)
                .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap for SVG"))?;

        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );

        let width = pixmap.width();
        let height = pixmap.height();
        let rgba = pixmap.take();

        Ok((rgba, width, height))
    }

    fn load_gif(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = image::codecs::gif::GifDecoder::new(reader)?;
        let frames = image::AnimationDecoder::into_frames(decoder).collect_frames()?;

        let mut image_frames = Vec::with_capacity(frames.len());
        let file_size = std::fs::metadata(path)?.len();

        for frame in frames {
            let delay = std::time::Duration::from(frame.delay());
            let delay_ms = delay.as_millis() as u32;
            let buffer = frame.into_buffer();
            let (w, h) = buffer.dimensions();

            image_frames.push(ImageData {
                path: path.to_path_buf(),
                rgba_data: buffer.into_raw(),
                width: w,
                height: h,
                format: ImageFormatType::Gif,
                file_size_bytes: file_size,
                frame_delay_ms: delay_ms,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
            });
        }

        Ok((image_frames, ImageFormatType::Gif))
    }
}
