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

        let format = image::ImageFormat::from_path(path).unwrap_or(image::ImageFormat::Png);

        if format == image::ImageFormat::Gif {
            Self::load_gif(path)
        } else {
            let file = std::fs::File::open(path)?;
            let reader = BufReader::new(file);
            let img = image::load(reader, format)
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
