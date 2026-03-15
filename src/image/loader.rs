use color_eyre::eyre::{eyre, Result};
use std::path::Path;
use zune_image::image::Image;
use zune_image::traits::OperationsTrait;

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

        if ext == "gif" {
            Self::load_gif(path)
        } else {
            let mut img = Image::open(path)
                .map_err(|e| eyre!("Failed to open image {path:?}: {e:?}"))?;
            
            // Ensure we are in RGBA8
            img.convert_color(zune_image::colorspace::ColorSpace::RGBA)?;
            
            let (w, h) = img.dimensions();
            let rgba = img.flatten_to_u8()[0].clone();

            Ok((
                vec![ImageData {
                    path: path.to_path_buf(),
                    rgba_data: rgba,
                    width: w as u32,
                    height: h as u32,
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
        // Fallback to image crate for GIF animation for now as it's well-integrated
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
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
