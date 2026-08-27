use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;
use zune_image::image::Image;

use super::types::{ImageData, ImageFormatType};

/// How many frames to decode from animated formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameLimit {
    /// Decode every frame.
    All,
    /// Stop after the first frame (thumbnails).
    First,
}

/// Optional work toggles applied while decoding.
#[derive(Debug, Clone, Copy)]
pub struct LoadOptions {
    pub frames: FrameLimit,
    /// When false, EXIF orientation is reported via `ImageData::orientation_deg`
    /// instead of being baked into the pixel buffer (the GPU rotates instead).
    pub bake_orientation: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            frames: FrameLimit::All,
            bake_orientation: true,
        }
    }
}

pub struct ImageLoader;

/// Map an EXIF orientation value to a rotation in degrees.
pub(crate) fn exif_orientation_to_degrees(o: u32) -> Option<u16> {
    match o {
        3 => Some(180),
        6 => Some(90),
        8 => Some(270),
        _ => None,
    }
}

impl ImageLoader {
    /// Load an image from a file path
    pub fn load(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        Self::load_with(path, max_w, max_h, LoadOptions::default())
    }

    /// Load an image with explicit decode options
    pub fn load_with(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        opts: LoadOptions,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format_type = ImageFormatType::from_extension(&ext);

        let orientation = if ext != "svg" && ext != "gif" {
            crate::image::extract_orientation(path)
        } else {
            None
        };

        let (image_frames, format_type) = if ext == "gif" {
            Self::load_gif(path, max_w, max_h, opts)?
        } else if ext == "jpg" || ext == "jpeg" {
            Self::load_jpeg(path, max_w, max_h, opts, orientation)?
        } else if ext == "svg" {
            Self::load_svg(path, max_w, max_h)?
        } else if ext == "jxl" {
            Self::load_jxl(path)?
        } else if ext == "qoi" {
            Self::load_qoi(path)?
        } else if ext == "exr" {
            Self::load_exr(path)?
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

            let (target_mw, target_mh) = match (max_w, max_h) {
                (Some(mw), Some(mh)) => {
                    let deg = orientation.and_then(|o| match o {
                        3 => Some(180),
                        6 => Some(90),
                        8 => Some(270),
                        _ => None,
                    });
                    if matches!(deg, Some(90) | Some(270)) {
                        (mh, mw)
                    } else {
                        (mw, mh)
                    }
                }
                _ => (0, 0),
            };

            let mut img = Image::read(cursor, zune_core::options::DecoderOptions::default())
                .map_err(|e| eyre!("Failed to decode image {path:?}: {e:?}"))?;

            // Ensure we are in RGBA8
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

            let (src_w, src_h) = (img.dimensions().0 as u32, img.dimensions().1 as u32);
            let icc = img.metadata().icc_chunk().map(|s| s.to_vec());
            // Take ownership instead of cloning: saves a full-buffer copy per image.
            let mut rgba = img.flatten_to_u8().swap_remove(0);

            let mut final_w = src_w;
            let mut final_h = src_h;
            let mut is_downsampled = false;

            if (max_w.is_some() || max_h.is_some()) && (src_w > target_mw || src_h > target_mh) {
                let ratio = (src_w as f32 / target_mw as f32).max(src_h as f32 / target_mh as f32);
                let dst_w = ((src_w as f32 / ratio).round() as u32).max(1);
                let dst_h = ((src_h as f32 / ratio).round() as u32).max(1);

                use fast_image_resize as fr;
                let src_image = fr::images::ImageRef::new(src_w, src_h, &rgba, fr::PixelType::U8x4)
                    .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;
                let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
                let mut resizer = super::processing::ImageProcessor::create_simd_resizer();
                resizer
                    .resize(&src_image, &mut dst_image, None)
                    .map_err(|e| eyre!("Resize failed: {e:?}"))?;

                rgba = dst_image.into_vec();
                final_w = dst_w;
                final_h = dst_h;
                is_downsampled = true;
            }

            // Apply color profile to downsampled buffer in parallel
            if let Some(ref icc_bytes) = icc
                && let Err(e) = Self::apply_color_profile(&mut rgba, icc_bytes)
            {
                tracing::warn!("Failed to apply color profile: {:?}", e);
            }

            // Apply EXIF rotation: baked into the buffer, or deferred to the GPU.
            let mut orientation_deg = 0u16;
            if let Some(o) = orientation
                && let Some(d) = exif_orientation_to_degrees(o)
            {
                if opts.bake_orientation {
                    let (rotated_rgba, rotated_w, rotated_h) =
                        rotate_rgba(&rgba, final_w, final_h, d as u32);
                    rgba = rotated_rgba;
                    final_w = rotated_w;
                    final_h = rotated_h;
                } else {
                    orientation_deg = d;
                }
            }

            (
                vec![ImageData {
                    path: path.to_path_buf(),
                    rgba_data: Arc::new(rgba),
                    width: final_w,
                    height: final_h,
                    format: format_type,
                    file_size_bytes: std::fs::metadata(path)?.len(),
                    frame_delay_ms: 0,
                    exif_info: None,
                    exif_loaded: false,
                    histogram: None,
                    is_downsampled,
                    orientation_deg,
                    gps_coords: None,
                    color_space: None,
                }],
                format_type,
            )
        };

        Ok((image_frames, format_type))
    }

    fn load_jpeg(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
        opts: LoadOptions,
        orientation: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)
            .map_err(|e| eyre!("Failed to open JPEG file {path:?}: {e:?}"))?;
        let mmap = unsafe {
            memmap2::Mmap::map(&file)
                .map_err(|e| eyre!("Failed to memory map JPEG {path:?}: {e:?}"))?
        };

        let decoded =
            libjpeg_turbo_rs::decompress_to(&mmap[..], libjpeg_turbo_rs::PixelFormat::Rgba)
                .map_err(|e| eyre!("Failed to decode JPEG {path:?}: {e:?}"))?;

        let src_w = decoded.width as u32;
        let src_h = decoded.height as u32;
        let mut rgba = decoded.data;

        let (target_mw, target_mh) = match (max_w, max_h) {
            (Some(mw), Some(mh)) => {
                let deg = orientation.and_then(exif_orientation_to_degrees);
                if matches!(deg, Some(90) | Some(270)) {
                    (mh, mw)
                } else {
                    (mw, mh)
                }
            }
            _ => (0, 0),
        };

        let mut final_w = src_w;
        let mut final_h = src_h;
        let mut is_downsampled = false;

        if (max_w.is_some() || max_h.is_some())
            && target_mw > 0
            && target_mh > 0
            && (src_w > target_mw || src_h > target_mh)
        {
            let ratio = (src_w as f32 / target_mw as f32).max(src_h as f32 / target_mh as f32);
            let dst_w = ((src_w as f32 / ratio).round() as u32).max(1);
            let dst_h = ((src_h as f32 / ratio).round() as u32).max(1);

            use fast_image_resize as fr;
            let src_image = fr::images::ImageRef::new(src_w, src_h, &rgba, fr::PixelType::U8x4)
                .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;
            let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
            let mut resizer = super::processing::ImageProcessor::create_simd_resizer();
            resizer
                .resize(&src_image, &mut dst_image, None)
                .map_err(|e| eyre!("Resize failed: {e:?}"))?;

            rgba = dst_image.into_vec();
            final_w = dst_w;
            final_h = dst_h;
            is_downsampled = true;
        }

        // Apply EXIF rotation: baked into the buffer, or deferred to the GPU.
        let mut orientation_deg = 0u16;
        if let Some(o) = orientation
            && let Some(d) = exif_orientation_to_degrees(o)
        {
            if opts.bake_orientation {
                let (rotated_rgba, rotated_w, rotated_h) =
                    rotate_rgba(&rgba, final_w, final_h, d as u32);
                rgba = rotated_rgba;
                final_w = rotated_w;
                final_h = rotated_h;
            } else {
                orientation_deg = d;
            }
        }

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width: final_w,
                height: final_h,
                format: ImageFormatType::Jpeg,
                file_size_bytes: file.metadata()?.len(),
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: false,
                histogram: None,
                is_downsampled,
                orientation_deg,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Jpeg,
        ))
    }

    fn apply_color_profile(rgba: &mut [u8], icc_data: &[u8]) -> Result<()> {
        let in_profile = qcms::Profile::new_from_slice(icc_data, false)
            .ok_or_else(|| eyre!("Failed to parse ICC profile"))?;

        // Identity transform: skip the expensive per-pixel pass entirely.
        if in_profile.is_sRGB() {
            return Ok(());
        }

        let out_profile = qcms::Profile::new_sRGB();

        let transform = qcms::Transform::new(
            &in_profile,
            &out_profile,
            qcms::DataType::RGBA8,
            qcms::Intent::Perceptual,
        )
        .ok_or_else(|| eyre!("Failed to create color transform"))?;

        use rayon::prelude::*;
        let chunk_size = 16384 * 4;
        rgba.par_chunks_mut(chunk_size).for_each(|chunk| {
            transform.apply(chunk);
        });

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
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Jxl,
        ))
    }

    fn load_svg(
        path: &Path,
        max_w: Option<u32>,
        max_h: Option<u32>,
    ) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use resvg::tiny_skia;
        use resvg::usvg;

        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let svg_data = &mmap[..];

        let rtree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
            .map_err(|e| eyre!("Failed to parse SVG: {e:?}"))?;

        let size = rtree.size();
        let mut width = size.width() as u32;
        let mut height = size.height() as u32;

        // Rasterize directly at the requested size instead of rendering at
        // intrinsic resolution and resizing on the CPU afterwards.
        let mut transform = tiny_skia::Transform::default();
        if let (Some(mw), Some(mh)) = (max_w, max_h)
            && (width > mw || height > mh)
            && width > 0
            && height > 0
        {
            let ratio = (width as f32 / mw as f32).max(height as f32 / mh as f32);
            let scale = 1.0 / ratio;
            width = ((width as f32 * scale).round() as u32).max(1);
            height = ((height as f32 * scale).round() as u32).max(1);
            transform = tiny_skia::Transform::from_scale(scale, scale);
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            eyre!(
                "Failed to create pixmap for SVG rendering (size: {}x{})",
                width,
                height
            )
        })?;

        resvg::render(&rtree, transform, &mut pixmap.as_mut());

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
                orientation_deg: 0,
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
                img.flatten_to_u8().swap_remove(0)
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
                orientation_deg: 0,
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
                orientation_deg: 0,
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
        opts: LoadOptions,
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
        let mut prev_canvas = canvas.clone();
        let mut prev_disposal = gif::DisposalMethod::Keep;
        let mut prev_frame_rect = (0usize, 0usize, 0usize, 0usize);

        // Create resizer once outside the frame loop
        let is_downsampled = dst_w != w || dst_h != h;
        let mut resizer = if is_downsampled {
            Some(super::processing::ImageProcessor::create_simd_resizer())
        } else {
            None
        };

        const MAX_GIF_FRAMES: usize = 500;

        while let Ok(Some(frame)) = decoder.read_next_frame() {
            if image_frames.len() >= MAX_GIF_FRAMES {
                break;
            }

            // Apply disposal method from the PREVIOUS frame before rendering the current frame
            match prev_disposal {
                gif::DisposalMethod::Background => {
                    let (fl, ft, fw, fh) = prev_frame_rect;
                    for row in 0..fh {
                        let y = ft + row;
                        if y < h as usize {
                            let start = (y * w as usize + fl) * 4;
                            let end = start + fw * 4;
                            if end <= canvas.len() {
                                canvas[start..end].fill(0);
                            }
                        }
                    }
                }
                gif::DisposalMethod::Previous => {
                    canvas.copy_from_slice(&prev_canvas);
                }
                _ => {}
            }

            if frame.dispose == gif::DisposalMethod::Previous {
                prev_canvas.copy_from_slice(&canvas);
            }

            let delay_ms = (frame.delay as u32 * 10).max(10);
            let fl = frame.left as usize;
            let ft = frame.top as usize;
            let fw = frame.width as usize;
            let fh = frame.height as usize;

            let line_len = fw * 4;
            for (i, line) in frame.buffer.chunks_exact(line_len).enumerate() {
                let y = ft + i;
                if y < h as usize {
                    let canvas_start = (y * w as usize + fl) * 4;
                    for (p, pixel) in line.as_chunks::<4>().0.iter().enumerate() {
                        let dst_idx = canvas_start + p * 4;
                        if dst_idx + 4 <= canvas.len() {
                            let alpha = pixel[3];
                            if alpha > 0 {
                                canvas[dst_idx..dst_idx + 4].copy_from_slice(pixel);
                            }
                        }
                    }
                }
            }

            prev_disposal = frame.dispose;
            prev_frame_rect = (fl, ft, fw, fh);

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
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            });

            // Thumbnail fast-path: one composited frame is enough.
            if opts.frames == FrameLimit::First {
                break;
            }
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

        // Do a fast half-size downsampled demosaicing with proper CFA mapping
        let out_w = width / 2;
        let out_h = height / 2;

        if out_w == 0 || out_h == 0 {
            return Err(eyre!("RAW image dimensions are too small"));
        }

        let mut rgba = vec![0u8; out_w * out_h * 4];

        let black = image.blacklevels[0] as f32;
        let white = image.whitelevels[0] as f32;
        let range = (white - black).max(1.0);

        // Extract camera white balance coefficients (fallback to 1.0)
        let wb_r = if image.wb_coeffs[0] > 0.01 {
            image.wb_coeffs[0]
        } else {
            1.0
        };
        let wb_g = if image.wb_coeffs[1] > 0.01 {
            image.wb_coeffs[1]
        } else {
            1.0
        };
        let wb_b = if image.wb_coeffs[2] > 0.01 {
            image.wb_coeffs[2]
        } else {
            1.0
        };
        let r_gain = (wb_r / wb_g).clamp(0.5, 4.0);
        let b_gain = (wb_b / wb_g).clamp(0.5, 4.0);

        let cfa_name = image.cfa.name.to_ascii_uppercase();

        use rayon::prelude::*;
        rgba.par_chunks_exact_mut(out_w * 4)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..out_w {
                    let p00_idx = (y * 2) * width + (x * 2);
                    let p01_idx = (y * 2) * width + (x * 2 + 1);
                    let p10_idx = (y * 2 + 1) * width + (x * 2);
                    let p11_idx = (y * 2 + 1) * width + (x * 2 + 1);

                    if p11_idx < raw_data.len() {
                        let p00 = raw_data[p00_idx] as f32;
                        let p01 = raw_data[p01_idx] as f32;
                        let p10 = raw_data[p10_idx] as f32;
                        let p11 = raw_data[p11_idx] as f32;

                        // Map CFA pattern to R, G, B channels
                        let (r_raw, g_raw, b_raw) = match cfa_name.as_str() {
                            "BGGR" => (p11, (p01 + p10) * 0.5, p00),
                            "GRBG" => (p01, (p00 + p11) * 0.5, p10),
                            "GBRG" => (p10, (p00 + p11) * 0.5, p01),
                            _ => (p00, (p01 + p10) * 0.5, p11), // Default RGGB
                        };

                        // Normalize with black/white levels and apply white balance
                        let r_norm = (((r_raw - black) / range).max(0.0) * r_gain).clamp(0.0, 1.0);
                        let g_norm = ((g_raw - black) / range).clamp(0.0, 1.0);
                        let b_norm = (((b_raw - black) / range).max(0.0) * b_gain).clamp(0.0, 1.0);

                        // Apply standard sRGB gamma (pow 1/2.2) so linear RAW is properly exposed
                        let r = (r_norm.powf(1.0 / 2.2) * 255.0).round() as u8;
                        let g = (g_norm.powf(1.0 / 2.2) * 255.0).round() as u8;
                        let b = (b_norm.powf(1.0 / 2.2) * 255.0).round() as u8;

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
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Raw,
        ))
    }

    fn load_qoi(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        let file = std::fs::File::open(path)
            .map_err(|e| eyre!("Failed to open QOI file {path:?}: {e:?}"))?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let (header, decoded) =
            qoi::decode_to_vec(&mmap[..]).map_err(|e| eyre!("QOI decode error: {e:?}"))?;
        let file_size = mmap.len() as u64;

        let rgba_data = match header.channels {
            qoi::Channels::Rgb => {
                let mut rgba =
                    Vec::with_capacity(header.width as usize * header.height as usize * 4);
                for rgb in decoded.as_chunks::<3>().0 {
                    rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
                rgba
            }
            qoi::Channels::Rgba => decoded,
        };

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba_data),
                width: header.width,
                height: header.height,
                format: ImageFormatType::Qoi,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Qoi,
        ))
    }

    fn load_exr(path: &Path) -> Result<(Vec<ImageData>, ImageFormatType)> {
        use exr::prelude::*;
        let image = read_first_rgba_layer_from_file(
            path,
            |resolution, _| {
                let pixel_count = resolution.width() * resolution.height();
                Vec::with_capacity(pixel_count * 4)
            },
            |vec, _position, (r, g, b, a): (f32, f32, f32, f32)| {
                let u8_r = fast_srgb8::f32_to_srgb8(r.clamp(0.0, 1.0));
                let u8_g = fast_srgb8::f32_to_srgb8(g.clamp(0.0, 1.0));
                let u8_b = fast_srgb8::f32_to_srgb8(b.clamp(0.0, 1.0));
                let u8_a = (a.clamp(0.0, 1.0) * 255.0) as u8;
                vec.extend_from_slice(&[u8_r, u8_g, u8_b, u8_a]);
            },
        )
        .map_err(|e| eyre!("Failed to decode EXR file: {e:?}"))?;

        let width = image.layer_data.size.width() as u32;
        let height = image.layer_data.size.height() as u32;
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let rgba = image.layer_data.channel_data.pixels;

        Ok((
            vec![ImageData {
                path: path.to_path_buf(),
                rgba_data: Arc::new(rgba),
                width,
                height,
                format: ImageFormatType::Exr,
                file_size_bytes: file_size,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            }],
            ImageFormatType::Exr,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exif_orientation_mapping() {
        assert_eq!(exif_orientation_to_degrees(1), None);
        assert_eq!(exif_orientation_to_degrees(2), None);
        assert_eq!(exif_orientation_to_degrees(3), Some(180));
        assert_eq!(exif_orientation_to_degrees(6), Some(90));
        assert_eq!(exif_orientation_to_degrees(8), Some(270));
        assert_eq!(exif_orientation_to_degrees(0), None);
        assert_eq!(exif_orientation_to_degrees(9), None);
    }

    /// Build a tiny 3-frame animated GIF in the temp dir.
    fn write_test_gif(path: &Path) {
        use gif::Frame;
        use std::fs::File;

        let mut encoder = gif::Encoder::new(
            File::create(path).expect("create gif"),
            4,
            4,
            &[0, 0, 0, 255, 255, 255],
        )
        .expect("encoder");
        encoder.set_repeat(gif::Repeat::Infinite).ok();
        for i in 0..3u8 {
            let buffer = vec![i; 16];
            encoder
                .write_frame(&Frame {
                    width: 4,
                    height: 4,
                    delay: 2,
                    dispose: gif::DisposalMethod::Keep,
                    transparent: None,
                    needs_user_input: false,
                    top: 0,
                    left: 0,
                    interlaced: false,
                    palette: None,
                    buffer: std::borrow::Cow::Owned(buffer),
                })
                .expect("write frame");
        }
    }

    #[test]
    fn test_gif_first_frame_limit_decodes_one_frame() {
        let dir = std::env::temp_dir().join(format!("spedimage_test_gif_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("anim.gif");
        write_test_gif(&path);

        // First-frame mode (thumbnails / streaming display path).
        let (frames, format) = ImageLoader::load_with(
            &path,
            None,
            None,
            LoadOptions {
                frames: FrameLimit::First,
                bake_orientation: true,
            },
        )
        .expect("load first");
        assert_eq!(format, ImageFormatType::Gif);
        assert_eq!(
            frames.len(),
            1,
            "FrameLimit::First must stop after one frame"
        );

        // Default mode still returns every frame.
        let (frames, _) = ImageLoader::load(&path, None, None).expect("load all");
        assert_eq!(frames.len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_options_default_bakes_orientation() {
        let opts = LoadOptions::default();
        assert!(opts.bake_orientation);
        assert_eq!(opts.frames, FrameLimit::All);
    }
}
