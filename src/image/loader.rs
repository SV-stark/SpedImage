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
        } else if format_type == ImageFormatType::Raw {
            return Err(eyre!("RAW support is temporarily disabled."));
            /*
            let raw_img = imagepipe::simple_decode_8bit(path, 0, 0)
                .map_err(|e| eyre!("Imagepipe RAW decode failed for {path:?}: {e:?}"))?;
            
            Ok((
                vec![ImageData {
                    path: path.to_path_buf(),
                    rgba_data: raw_img.data,
                    width: raw_img.width as u32,
                    height: raw_img.height as u32,
                    format: format_type,
                    file_size_bytes: std::fs::metadata(path)?.len(),
                    frame_delay_ms: 0,
                    exif_info: None,
                    exif_loaded: false,
                    histogram: None,
                }],
                format_type,
            ))
            */
        } else {
            let mut img = Image::open(path)
                .map_err(|e| eyre!("Failed to open image {path:?}: {e:?}"))?;
            
            // Ensure we are in RGBA8
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;
            
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
        use gif::DecodeOptions;
        let file = std::fs::File::open(path)?;
        let mut options = DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options.read_info(file)
            .map_err(|e| eyre!("Failed to read GIF info: {e:?}"))?;

        let mut image_frames = Vec::new();
        let file_size = std::fs::metadata(path)?.len();
        let (w, h) = (decoder.width() as u32, decoder.height() as u32);

        // A GIF frame might not cover the full canvas, so we need a canvas to compose them.
        let mut canvas = vec![0u8; (w * h * 4) as usize];

        while let Ok(Some(frame)) = decoder.read_next_frame() {
            let delay_ms = frame.delay as u32 * 10;
            
            // For simplicity, we just update the canvas with the new frame data.
            // Note: Proper GIF disposal methods are complex, this is a basic implementation.
            let line_len = frame.width as usize * 4;
            for (i, line) in frame.buffer.chunks_exact(line_len).enumerate() {
                let y = frame.top as usize + i;
                if y < h as usize {
                    let canvas_start = (y * w as usize + frame.left as usize) * 4;
                    let canvas_end = canvas_start + line_len;
                    if canvas_end <= canvas.len() {
                        canvas[canvas_start..canvas_end].copy_from_slice(line);
                    }
                }
            }

            image_frames.push(ImageData {
                path: path.to_path_buf(),
                rgba_data: canvas.clone(),
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
