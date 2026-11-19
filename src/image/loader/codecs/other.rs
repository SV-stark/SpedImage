use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;
use zune_image::image::Image;

use super::super::{
    FrameLimit, LoadOptions, apply_color_profile, apply_orientation, decode_box, downsample_to_fit,
    orient_box,
};
use crate::image::types::{ImageData, ImageFormatType};

pub(crate) fn load_jxl(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
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
    let (rgba, width, height, is_downsampled) =
        downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

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
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Jxl,
    ))
}

/// Decode a still WebP frame.
///
/// Uses `image-webp` directly rather than zune-image's WebP codec: that
/// codec only compiles when zune's `jpeg-xl` feature is also on, because of
/// an ungated `jxl_oxide` import in it, and `image-webp` is what it wraps
/// anyway.
pub(crate) fn load_webp(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
    exif: &crate::image::metadata::ExifMeta,
    opts: LoadOptions,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };

    // `WebPDecoder` needs a Seek, and an mmap-backed slice cannot seek.
    let cursor = std::io::Cursor::new(&mmap[..]);
    let mut decoder = image_webp::WebPDecoder::new(cursor)
        .map_err(|e| eyre!("Failed to parse WebP header: {e:?}"))?;

    let (src_w, src_h) = decoder.dimensions();
    if src_w == 0 || src_h == 0 {
        return Err(eyre!("WebP {path:?} reports a zero dimension"));
    }

    // The ICC profile is read before the pixel pass, while the decoder is
    // still positioned at the header.
    let icc = decoder.icc_profile().ok().flatten();

    // `image-webp` emits RGB (3 bytes/px) when the file has no alpha channel
    // and RGBA (4 bytes/px) when it does, so the buffer has to be widened
    // to RGBA before anything downstream sees it.
    let has_alpha = decoder.has_alpha();
    let buffer_size = decoder
        .output_buffer_size()
        .ok_or_else(|| eyre!("Could not determine the WebP buffer size"))?;
    let file_size = std::fs::metadata(path)?.len();
    let target_box = orient_box(decode_box(max_w, max_h), exif.orientation);

    let widen_and_finish = |decoded: Vec<u8>, delay_ms: u32| -> Result<ImageData> {
        let pixels = src_w as usize * src_h as usize;
        let rgba = if has_alpha {
            decoded
        } else {
            let mut out = Vec::with_capacity(pixels * 4);
            for px in decoded.as_chunks::<3>().0 {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        };

        let (mut rgba, width, height, is_downsampled) =
            downsample_to_fit(rgba, src_w, src_h, target_box)?;

        if let Some(ref icc_bytes) = icc
            && let Err(e) = apply_color_profile(&mut rgba, icc_bytes)
        {
            tracing::warn!("Failed to apply WebP color profile: {e:?}");
        }

        let (rgba, width, height, orientation_deg) = match exif.orientation {
            Some(o) => apply_orientation(rgba, width, height, o, opts.bake_orientation),
            None => (rgba, width, height, 0),
        };

        Ok(ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width,
            height,
            format: ImageFormatType::WebP,
            file_size_bytes: file_size,
            frame_delay_ms: delay_ms,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled,
            orientation_deg,
            gps_coords: exif.gps_coords,
            color_space: exif.color_space,
        })
    };

    if decoder.is_animated() && opts.frames == FrameLimit::All {
        let total = (decoder.num_frames() as usize).clamp(1, 120);
        let mut frames = Vec::with_capacity(total);
        for _ in 0..total {
            let mut buf = vec![0u8; buffer_size];
            match decoder.read_frame(&mut buf) {
                Ok(delay_ms) => frames.push(widen_and_finish(buf, delay_ms.max(20))?),
                Err(_) if !frames.is_empty() => break,
                Err(e) => return Err(eyre!("Animated WebP decode failed: {e:?}")),
            }
        }
        return Ok((frames, ImageFormatType::WebP));
    }

    let mut decoded = vec![0u8; buffer_size];
    decoder
        .read_image(&mut decoded)
        .map_err(|e| eyre!("WebP decode failed: {e:?}"))?;

    Ok((vec![widen_and_finish(decoded, 0)?], ImageFormatType::WebP))
}

pub(crate) fn load_svg(
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
    let orig_w = width;
    let orig_h = height;

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

    let is_downsampled = width != orig_w || height != orig_h;

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
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Svg,
    ))
}

pub(crate) fn load_gif(
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
    if let Some((mw, mh)) = decode_box(max_w, max_h)
        && (w > mw || h > mh)
    {
        let ratio = (w as f64 / mw as f64).max(h as f64 / mh as f64);
        dst_w = ((w as f64 / ratio).round() as u32).max(1);
        dst_h = ((h as f64 / ratio).round() as u32).max(1);
    }

    // A GIF frame might not cover the full canvas, so we need a canvas to compose them.
    let mut canvas = vec![0u8; (w * h * 4) as usize];
    let mut prev_canvas = canvas.clone();
    let mut prev_disposal = gif::DisposalMethod::Keep;
    let mut prev_frame_rect = (0usize, 0usize, 0usize, 0usize);

    // Create resizer once outside the frame loop
    let is_downsampled = dst_w != w || dst_h != h;
    let mut resizer = if is_downsampled {
        Some(crate::image::processing::ImageProcessor::create_simd_resizer())
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
                    if y >= h as usize || fl >= w as usize {
                        break;
                    }
                    let span = fw.min(w as usize - fl);
                    let start = (y * w as usize + fl) * 4;
                    let end = start + span * 4;
                    canvas[start..end].fill(0);
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

        if fw > 0 && fh > 0 {
            let line_len = fw * 4;
            for (i, line) in frame.buffer.chunks_exact(line_len).enumerate() {
                let y = ft + i;
                if y >= h as usize {
                    break;
                }
                let row_base = y * w as usize;
                let copy_pixels = fw.min(w as usize - fl);
                if copy_pixels == 0 {
                    break;
                }
                let canvas_start = (row_base + fl) * 4;
                let copy_bytes = copy_pixels * 4;
                if line.len() < copy_bytes {
                    break;
                }
                for (p, pixel) in line[..copy_bytes].as_chunks::<4>().0.iter().enumerate() {
                    let dst_idx = canvas_start + p * 4;
                    if pixel[3] > 0 {
                        canvas[dst_idx..dst_idx + 4].copy_from_slice(pixel);
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

/// Decode a Windows icon (`.ico`) or cursor (`.cur`).
pub(crate) fn load_ico(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    const HEADER_LEN: usize = 6;
    const DIR_ENTRY_LEN: usize = 16;

    let file =
        std::fs::File::open(path).map_err(|e| eyre!("Failed to open icon file {path:?}: {e:?}"))?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let data = &mmap[..];

    if data.len() < HEADER_LEN {
        return Err(eyre!("Icon file {path:?} is too short to hold a header"));
    }
    let reserved = u16::from_le_bytes([data[0], data[1]]);
    let kind = u16::from_le_bytes([data[2], data[3]]);
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;

    if reserved != 0 || (kind != 1 && kind != 2) {
        return Err(eyre!(
            "Not an icon or cursor: reserved={reserved} type={kind}"
        ));
    }
    if count == 0 {
        return Err(eyre!("Icon file {path:?} declares zero images"));
    }

    let dim = |v: u8| if v == 0 { 256u32 } else { v as u32 };
    let mut best: Option<(u32, usize, usize)> = None;
    for i in 0..count {
        let start = HEADER_LEN + i * DIR_ENTRY_LEN;
        let Some(entry) = data.get(start..start + DIR_ENTRY_LEN) else {
            break;
        };
        let size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        let area = dim(entry[0]) * dim(entry[1]);
        let better = best.is_none_or(|(best_area, _, _)| area > best_area);
        if better {
            best = Some((area, offset, size));
        }
    }

    let (_, offset, size) =
        best.ok_or_else(|| eyre!("Icon file {path:?} has no readable directory entry"))?;
    let payload = data
        .get(offset..offset.saturating_add(size))
        .ok_or_else(|| {
            eyre!("Icon directory points outside the file: offset={offset} size={size}")
        })?;
    if payload.is_empty() {
        return Err(eyre!("Icon entry at {offset} is empty"));
    }

    let is_png = payload.starts_with(&[0x89, b'P', b'N', b'G']);
    let mut bmp: Vec<u8>;
    let bytes: &[u8] = if is_png {
        payload
    } else {
        let ihsize = payload
            .get(0..4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| eyre!("Icon DIB at {offset} is too short for a header"))?;
        let height_at = match ihsize {
            12 => Some(6usize),
            40 | 52 | 56 | 64 | 108 | 124 => Some(8),
            _ => None,
        }
        .ok_or_else(|| eyre!("Icon DIB uses unsupported header size {ihsize}"))?;

        let declared = payload
            .get(height_at..height_at + 4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .unwrap_or(0);
        let real_height = (declared / 2).max(0);

        bmp = Vec::with_capacity(payload.len() + 14);
        let off_bits = 14u32.saturating_add(ihsize);
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&((payload.len() + 14) as u32).to_le_bytes());
        bmp.extend_from_slice(&0u16.to_le_bytes());
        bmp.extend_from_slice(&0u16.to_le_bytes());
        bmp.extend_from_slice(&off_bits.to_le_bytes());
        bmp.extend_from_slice(payload);
        bmp[14 + height_at..14 + height_at + 4].copy_from_slice(&real_height.to_le_bytes());
        &bmp
    };

    let cursor = std::io::Cursor::new(bytes);
    let mut img = Image::read(cursor, zune_core::options::DecoderOptions::default())
        .map_err(|e| eyre!("Failed to decode icon payload in {path:?}: {e:?}"))?;
    img.convert_color(zune_core::colorspace::ColorSpace::RGBA)?;

    let (src_w, src_h) = (img.dimensions().0 as u32, img.dimensions().1 as u32);
    if src_w == 0 || src_h == 0 {
        return Err(eyre!("Icon {path:?} reports a zero dimension"));
    }
    let mut rgba = img.flatten_to_u8().swap_remove(0);

    if rgba.len() >= 4 && rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
        for px in rgba.as_chunks_mut::<4>().0.iter_mut() {
            px[3] = 255;
        }
    }

    let file_size = mmap.len() as u64;
    let (rgba, width, height, is_downsampled) =
        downsample_to_fit(rgba, src_w, src_h, decode_box(max_w, max_h))?;

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width,
            height,
            format: ImageFormatType::Ico,
            file_size_bytes: file_size,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: true,
            histogram: None,
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Ico,
    ))
}

/// Decode a farbfeld image: an 8-byte magic, two big-endian `u32` dimensions,
/// then 8 bytes per pixel as four big-endian `u16` samples.
pub(crate) fn load_farbfeld(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    const MAGIC: &[u8; 8] = b"farbfeld";

    let file = std::fs::File::open(path)
        .map_err(|e| eyre!("Failed to open farbfeld file {path:?}: {e:?}"))?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let data = &mmap[..];

    let data = data
        .strip_prefix(MAGIC)
        .ok_or_else(|| eyre!("Not a farbfeld file: {path:?}"))?;
    let dims = data
        .get(0..8)
        .ok_or_else(|| eyre!("Farbfeld file {path:?} is truncated in its header"))?;
    let width = u32::from_be_bytes([dims[0], dims[1], dims[2], dims[3]]);
    let height = u32::from_be_bytes([dims[4], dims[5], dims[6], dims[7]]);
    if width == 0 || height == 0 {
        return Err(eyre!("Farbfeld file {path:?} reports a zero dimension"));
    }

    let pixels = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(8))
        .ok_or_else(|| eyre!("Farbfeld dimensions {width}x{height} overflow"))?;
    let body = data
        .get(8..8 + pixels)
        .ok_or_else(|| eyre!("Farbfeld file {path:?} is truncated in its pixel data"))?;

    let mut rgba = Vec::with_capacity(pixels / 2);
    for px in body.as_chunks::<8>().0 {
        rgba.extend_from_slice(&[px[0], px[2], px[4], px[6]]);
    }

    let file_size = mmap.len() as u64;
    let (rgba, width, height, is_downsampled) =
        downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width,
            height,
            format: ImageFormatType::Farbfeld,
            file_size_bytes: file_size,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: true,
            histogram: None,
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Farbfeld,
    ))
}

pub(crate) fn load_qoi(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let file =
        std::fs::File::open(path).map_err(|e| eyre!("Failed to open QOI file {path:?}: {e:?}"))?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let (header, decoded) =
        qoi::decode_to_vec(&mmap[..]).map_err(|e| eyre!("QOI decode error: {e:?}"))?;
    let file_size = mmap.len() as u64;

    let rgba_data = match header.channels {
        qoi::Channels::Rgb => {
            let mut rgba = Vec::with_capacity(header.width as usize * header.height as usize * 4);
            for rgb in decoded.as_chunks::<3>().0 {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
            rgba
        }
        qoi::Channels::Rgba => decoded,
    };

    let (rgba_data, width, height, is_downsampled) = downsample_to_fit(
        rgba_data,
        header.width,
        header.height,
        decode_box(max_w, max_h),
    )?;

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba_data),
            width,
            height,
            format: ImageFormatType::Qoi,
            file_size_bytes: file_size,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: true,
            histogram: None,
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Qoi,
    ))
}

pub(crate) fn load_exr(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
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
    let (rgba, width, height, is_downsampled) =
        downsample_to_fit(rgba, width, height, decode_box(max_w, max_h))?;

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
            is_downsampled,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        }],
        ImageFormatType::Exr,
    ))
}
