use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use zune_image::image::Image;

use super::types::{ImageData, ImageFormatType};

pub struct ImageLoader;

impl ImageLoader {
    /// Load an image from a file path
    pub fn load(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format_type = ImageFormatType::from_extension(&ext);

        if ext == "gif" {
            Self::load_gif(path, max_w, max_h)
        } else if ext == "svg" {
            Self::load_svg(path)
        } else if ext == "jxl" {
            Self::load_jxl(path)
        } else if ext == "tiff" || ext == "tif" {
            Self::load_tiff(path)
        } else if format_type == ImageFormatType::Raw {
            Err(eyre!("RAW support is temporarily disabled."))
        } else {
            let mut img =
                Image::open(path).map_err(|e| eyre!("Failed to open image {path:?}: {e:?}"))?;

            // Ensure we are in RGBA8
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

            let (w, h) = img.dimensions();
            let rgba = img.flatten_to_u8()[0].clone();

            // Apply color management if ICC profile exists
            /*
            if let Some(icc) = icc_profile {
                if let Err(e) = Self::apply_color_profile(&mut rgba, &icc) {
                    tracing::warn!("Failed to apply color profile: {:?}", e);
                }
            }
            */

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
                    is_downsampled: false,
                }],
                format_type,
            ))
        }
    }

    /*
    fn apply_color_profile(rgba: &mut [u8], icc_data: &[u8]) -> Result<()> {
        let mut in_profile = qcms::Profile::from_slice(icc_data)
            .map_err(|_| eyre!("Failed to parse ICC profile"))?;

        let mut out_profile = qcms::Profile::new_sRGB();

        let transform = qcms::Transform::new(
            &mut in_profile,
            &mut out_profile,
            qcms::DataType::RGBA8,
            qcms::Intent::Perceptual,
        )
        .map_err(|_| eyre!("Failed to create color transform"))?;

        transform.apply_inplace(rgba);
        Ok(())
    }
    */

    fn load_jxl(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use jxl_oxide::JxlImage;

        let image = JxlImage::builder()
            .open(path)
            .map_err(|e| eyre!("Failed to open JXL: {e:?}"))?;

        let (width, height) = (image.width(), image.height());
        let render = image
            .render_frame(0)
            .map_err(|e| eyre!("Failed to render JXL frame: {e:?}"))?;

        let frame_buffer = render.image_all_channels();
        let fb = frame_buffer.buf();
        let mut rgba = vec![0u8; width as usize * height as usize * 4];

        let num_channels = frame_buffer.channels();
        for (i, pixel) in fb.chunks_exact(num_channels).enumerate() {
            let r = (pixel[0].clamp(0.0, 1.0) * 255.0) as u8;
            let g = if num_channels > 1 {
                (pixel[1].clamp(0.0, 1.0) * 255.0) as u8
            } else {
                r
            };
            let b = if num_channels > 2 {
                (pixel[2].clamp(0.0, 1.0) * 255.0) as u8
            } else {
                r
            };
            let a = if num_channels > 3 {
                (pixel[3].clamp(0.0, 1.0) * 255.0) as u8
            } else {
                255
            };

            rgba[i * 4] = r;
            rgba[i * 4 + 1] = g;
            rgba[i * 4 + 2] = b;
            rgba[i * 4 + 3] = a;
        }

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: rgba,
                width,
                height,
                format: ImageFormatType::Jxl,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled: false,
            }],
            ImageFormatType::Jxl,
        ))
    }

    fn load_svg(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use resvg::tiny_skia;
        use resvg::usvg;

        let svg_data = std::fs::read(path)?;

        let rtree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
            .map_err(|e| eyre!("Failed to parse SVG: {e:?}"))?;

        let size = rtree.size();
        let width = size.width() as u32;
        let height = size.height() as u32;

        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            eyre!(
                "Failed to create pixmap for SVG rendering (size: {}x{})",
                width,
                height
            )
        })?;

        resvg::render(
            &rtree,
            tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: pixmap.data().to_vec(),
                width,
                height,
                format: ImageFormatType::Svg,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled: false,
            }],
            ImageFormatType::Svg,
        ))
    }

    fn load_tiff(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use tiff::decoder::{Decoder, DecodingResult};
        let file = std::fs::File::open(path)?;
        let mut decoder =
            Decoder::new(file).map_err(|e| eyre!("TIFF decoder init failed: {e:?}"))?;

        let (width, height) = decoder
            .dimensions()
            .map_err(|e| eyre!("Failed to get TIFF dimensions: {e:?}"))?;

        let img_res = decoder
            .read_image()
            .map_err(|e| eyre!("TIFF decode failed: {e:?}"))?;

        let rgba_data = match img_res {
            DecodingResult::U8(v) => {
                use zune_core::colorspace::ColorSpace;
                use zune_image::image::Image;

                let colortype = decoder
                    .colortype()
                    .map_err(|e| eyre!("Failed to get TIFF colortype: {e:?}"))?;

                let input_space = match colortype {
                    tiff::ColorType::RGB(8) => ColorSpace::RGB,
                    tiff::ColorType::RGBA(8) => ColorSpace::RGBA,
                    tiff::ColorType::Gray(8) => ColorSpace::Luma,
                    _ => return Err(eyre!("Unsupported TIFF color type: {:?}", colortype)),
                };

                let mut img = Image::from_u8(&v, width as usize, height as usize, input_space);
                img.convert_color(ColorSpace::RGBA)?;
                img.flatten_to_u8()[0].clone()
            }
            _ => return Err(eyre!("Unsupported TIFF bit depth")),
        };

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data,
                width,
                height,
                format: ImageFormatType::Tiff,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled: false,
            }],
            ImageFormatType::Tiff,
        ))
    }

    fn load_gif(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use gif::DecodeOptions;
        let file = std::fs::File::open(path)?;
        let mut options = DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(file)
            .map_err(|e| eyre!("Failed to read GIF info: {e:?}"))?;

        let mut image_frames = Vec::new();
        let file_size = std::fs::metadata(path)?.len();
        let (w, h) = (decoder.width() as u32, decoder.height() as u32);

        let mut dst_w = w;
        let mut dst_h = h;
        if let (Some(mw), Some(mh)) = (max_w, max_h) {
            if w > mw || h > mh {
                let ratio = (w as f32 / mw as f32).max(h as f32 / mh as f32);
                dst_w = (w as f32 / ratio).round() as u32;
                dst_h = (h as f32 / ratio).round() as u32;
                dst_w = dst_w.max(1);
                dst_h = dst_h.max(1);
            }
        }

        // A GIF frame might not cover the full canvas, so we need a canvas to compose them.
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        // Create resizer once outside the frame loop
        let is_downsampled = dst_w != w || dst_h != h;
        let mut resizer = if is_downsampled {
            use fast_image_resize as fr;
            Some(fr::Resizer::new())
        } else {
            None
        };

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

            let is_downsampled = dst_w != w || dst_h != h;
            let final_rgba = if is_downsampled {
                use fast_image_resize as fr;

                let src_image = fr::images::ImageRef::new(w, h, &canvas, fr::PixelType::U8x4)
                    .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;

                let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);

                if let Some(ref mut r) = resizer {
                    r.resize(&src_image, &mut dst_image, None)
                        .map_err(|e| eyre!("Resize failed: {e:?}"))?;
                }

                dst_image.into_vec()
            } else {
                canvas.clone()
            };

            image_frames.push(ImageData {
                path: path.to_path_buf(),
                rgba_data: final_rgba,
                width: dst_w,
                height: dst_h,
                format: ImageFormatType::Gif,
                file_size_bytes: file_size,
                frame_delay_ms: delay_ms,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled,
            });
        }

        Ok((image_frames, ImageFormatType::Gif))
    }
}
