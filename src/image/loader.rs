use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;
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

        let (exif_info, orientation, gps_coords, color_space) = if ext != "svg" && ext != "gif" {
            crate::image::extract_exif_and_orientation(path)
        } else {
            (None, None, None, None)
        };

        let (mut image_frames, format_type) = if ext == "gif" {
            Self::load_gif(path, max_w, max_h)?
        } else if ext == "svg" {
            Self::load_svg(path)?
        } else if ext == "jxl" {
            Self::load_jxl(path)?
        } else if ext == "heic" || ext == "heif" || ext == "avif" {
            Self::load_heic(path, format_type)?
        } else if ext == "tiff" || ext == "tif" {
            Self::load_tiff(path)?
        } else if format_type == ImageFormatType::Raw {
            Self::load_raw(path)?
        } else {
            let file = std::fs::File::open(path)
                .map_err(|e| eyre!("Failed to open image file {path:?}: {e:?}"))?;
            let mmap = unsafe {
                memmap2::Mmap::map(&file)
                    .map_err(|e| eyre!("Failed to memory map image {path:?}: {e:?}"))?
            };
            let cursor = std::io::Cursor::new(&mmap[..]);

            let mut img = Image::read(cursor, zune_core::options::DecoderOptions::default())
                .map_err(|e| eyre!("Failed to decode image {path:?}: {e:?}"))?;

            // Ensure we are in RGBA8
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

            let (w, h) = img.dimensions();
            let mut rgba = img.flatten_to_u8()[0].clone();

            // Apply color management if ICC profile exists
            if let Some(icc) = img.metadata().icc_chunk() {
                let res = Self::apply_color_profile(&mut rgba, icc);
                if let Err(e) = res {
                    tracing::warn!("Failed to apply color profile: {:?}", e);
                }
            }

            (
                vec![ImageData {
                    path: path.to_path_buf(),
                    rgba_data: Arc::new(rgba),
                    width: w as u32,
                    height: h as u32,
                    format: format_type,
                    file_size_bytes: std::fs::metadata(path)?.len(),
                    frame_delay_ms: 0,
                    exif_info: exif_info.clone(),
                    exif_loaded: true,
                    histogram: None,
                    is_downsampled: false,
                    gps_coords,
                    color_space,
                }],
                format_type,
            )
        };

        for frame in &mut image_frames {
            frame.exif_info = exif_info.clone();
            frame.gps_coords = gps_coords;
            frame.color_space = color_space;
        }

        // Post-process to auto-rotate based on EXIF orientation tag
        if let Some(orientation) = orientation {
            let deg = match orientation {
                3 => Some(180),
                6 => Some(90),
                8 => Some(270),
                _ => None,
            };
            if let Some(d) = deg {
                for frame in &mut image_frames {
                    let (rotated_rgba, rotated_w, rotated_h) =
                        rotate_rgba(&frame.rgba_data, frame.width, frame.height, d);
                    frame.rgba_data = Arc::new(rotated_rgba);
                    frame.width = rotated_w;
                    frame.height = rotated_h;
                }
            }
        }

        Ok((image_frames, format_type))
    }

    fn apply_color_profile(rgba: &mut [u8], icc_data: &[u8]) -> Result<()> {
        let in_profile = qcms::Profile::new_from_slice(icc_data, false)
            .ok_or_else(|| eyre!("Failed to parse ICC profile"))?;

        let out_profile = qcms::Profile::new_sRGB();

        let transform = qcms::Transform::new(
            &in_profile,
            &out_profile,
            qcms::DataType::RGBA8,
            qcms::Intent::Perceptual,
        )
        .ok_or_else(|| eyre!("Failed to create color transform"))?;

        transform.apply(rgba);
        Ok(())
    }

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

        use rayon::prelude::*;
        rgba.par_chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, rgba_pixel)| {
                let pixel = &fb[i * num_channels..(i + 1) * num_channels];
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

                rgba_pixel[0] = r;
                rgba_pixel[1] = g;
                rgba_pixel[2] = b;
                rgba_pixel[3] = a;
            });

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: ImageFormatType::Jxl,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Jxl,
        ))
    }

    fn load_svg(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use resvg::tiny_skia;
        use resvg::usvg;

        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let svg_data = &mmap[..];

        let rtree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
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
                rgba_data: Arc::new(pixmap.data().to_vec()),
                width,
                height,
                format: ImageFormatType::Svg,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Svg,
        ))
    }

    fn load_tiff(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use tiff::decoder::{Decoder, DecodingResult};
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);
        let mut decoder =
            Decoder::new(cursor).map_err(|e| eyre!("TIFF decoder init failed: {e:?}"))?;

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
                rgba_data: Arc::new(rgba_data),
                width,
                height,
                format: ImageFormatType::Tiff,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Tiff,
        ))
    }

    fn load_heic(
        path: &Path,
        format_type: ImageFormatType,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let data = &mmap[..];

        let info = heic::ImageInfo::from_bytes(data)
            .map_err(|e| eyre!("Failed to parse HEIC/AVIF header: {:?}", e))?;

        let layout = heic::PixelLayout::Rgba8;
        let buffer_size = info
            .output_buffer_size(layout)
            .ok_or_else(|| eyre!("Could not determine HEIC/AVIF buffer size"))?;

        let mut rgba = vec![0u8; buffer_size];

        let (width, height) = heic::DecoderConfig::new()
            .decode_request(data)
            .with_output_layout(layout)
            .decode_into(&mut rgba)
            .map_err(|e| eyre!("HEIC/AVIF decode failed: {:?}", e))?;

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: format_type,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                gps_coords: None,
                color_space: None,
            }],
            format_type,
        ))
    }

    fn load_gif(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use gif::DecodeOptions;
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);

        let mut options = DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(cursor)
            .map_err(|e| eyre!("Failed to read GIF info: {e:?}"))?;

        let mut image_frames = Vec::new();
        let file_size = std::fs::metadata(path)?.len();
        let (w, h) = (decoder.width() as u32, decoder.height() as u32);

        let mut dst_w = w;
        let mut dst_h = h;
        if let (Some(mw), Some(mh)) = (max_w, max_h)
            && (w > mw || h > mh)
        {
            let ratio = (w as f32 / mw as f32).max(h as f32 / mh as f32);
            dst_w = (w as f32 / ratio).round() as u32;
            dst_h = (h as f32 / ratio).round() as u32;
            dst_w = dst_w.max(1);
            dst_h = dst_h.max(1);
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
                rgba_data: Arc::new(final_rgba),
                width: dst_w,
                height: dst_h,
                format: ImageFormatType::Gif,
                file_size_bytes: file_size,
                frame_delay_ms: delay_ms,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled,
                gps_coords: None,
                color_space: None,
            });
        }

        Ok((image_frames, ImageFormatType::Gif))
    }

    fn load_raw(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let image = rawloader::decode_file(path)
            .map_err(|e| eyre!("Failed to decode RAW file: {:?}", e))?;

        let width = image.width;
        let height = image.height;

        let raw_data = match &image.data {
            rawloader::RawImageData::Integer(data) => data,
            _ => return Err(eyre!("Unsupported non-integer RAW image data")),
        };

        // Do a fast half-size downsampled demosaicing (RGGB binned pixels)
        let out_w = width / 2;
        let out_h = height / 2;

        if out_w == 0 || out_h == 0 {
            return Err(eyre!("RAW image dimensions are too small"));
        }

        let mut rgba = vec![0u8; out_w * out_h * 4];

        let black = image.blacklevels[0] as f32;
        let white = image.whitelevels[0] as f32;
        let range = (white - black).max(1.0);

        use rayon::prelude::*;
        rgba.par_chunks_exact_mut(out_w * 4)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..out_w {
                    let r_idx = (y * 2) * width + (x * 2);
                    let g1_idx = (y * 2) * width + (x * 2 + 1);
                    let g2_idx = (y * 2 + 1) * width + (x * 2);
                    let b_idx = (y * 2 + 1) * width + (x * 2 + 1);

                    if r_idx < raw_data.len()
                        && g1_idx < raw_data.len()
                        && g2_idx < raw_data.len()
                        && b_idx < raw_data.len()
                    {
                        let r_raw = raw_data[r_idx];
                        let g1_raw = raw_data[g1_idx];
                        let g2_raw = raw_data[g2_idx];
                        let b_raw = raw_data[b_idx];

                        let g_raw = ((g1_raw as u32 + g2_raw as u32) / 2) as u16;

                        let r = (((r_raw as f32 - black) / range).clamp(0.0, 1.0) * 255.0) as u8;
                        let g = (((g_raw as f32 - black) / range).clamp(0.0, 1.0) * 255.0) as u8;
                        let b = (((b_raw as f32 - black) / range).clamp(0.0, 1.0) * 255.0) as u8;

                        let out_idx = x * 4;
                        row[out_idx] = r;
                        row[out_idx + 1] = g;
                        row[out_idx + 2] = b;
                        row[out_idx + 3] = 255;
                    }
                }
            });

        let file_size = std::fs::metadata(path)?.len();

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width: out_w as u32,
                height: out_h as u32,
                format: ImageFormatType::Raw,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: true,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Raw,
        ))
    }
}

fn rotate_rgba(rgba: &[u8], width: u32, height: u32, degrees: u32) -> (Vec<u8>, u32, u32) {
    use rayon::prelude::*;
    let w = width as usize;
    let h = height as usize;

    if degrees == 90 {
        let mut out = vec![0u8; rgba.len()];
        out.par_chunks_exact_mut(h * 4)
            .enumerate()
            .for_each(|(dst_y, row)| {
                let x = dst_y;
                for dst_x in 0..h {
                    let y = h - 1 - dst_x;
                    let src_idx = (y * w + x) * 4;
                    let dst_idx = dst_x * 4;
                    if src_idx + 3 < rgba.len() && dst_idx + 3 < row.len() {
                        row[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
            });
        (out, height, width)
    } else if degrees == 180 {
        let mut out = vec![0u8; rgba.len()];
        out.par_chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(|(dst_y, row)| {
                let y = h - 1 - dst_y;
                for dst_x in 0..w {
                    let x = w - 1 - dst_x;
                    let src_idx = (y * w + x) * 4;
                    let dst_idx = dst_x * 4;
                    if src_idx + 3 < rgba.len() && dst_idx + 3 < row.len() {
                        row[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
            });
        (out, width, height)
    } else if degrees == 270 {
        let mut out = vec![0u8; rgba.len()];
        out.par_chunks_exact_mut(h * 4)
            .enumerate()
            .for_each(|(dst_y, row)| {
                let x = w - 1 - dst_y;
                for dst_x in 0..h {
                    let y = dst_x;
                    let src_idx = (y * w + x) * 4;
                    let dst_idx = dst_x * 4;
                    if src_idx + 3 < rgba.len() && dst_idx + 3 < row.len() {
                        row[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
                    }
                }
            });
        (out, height, width)
    } else {
        (rgba.to_vec(), width, height)
    }
}
