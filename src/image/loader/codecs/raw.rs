use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::super::{
    LoadOptions, apply_orientation, decode_box, downsample_to_fit, orient_box, pick_dct_scale,
};
use crate::image::types::{ImageData, ImageFormatType};

/// Scan a RAW file for embedded JPEG previews (`FF D8 FF ... FF D9`) and
/// return the largest valid JPEG slice. Most CR2/NEF/ARW/DNG/ORF/RAF files
/// embed a full- or half-size JPEG preview that decodes in <15 ms instead of
/// 300–1500 ms for full Bayer demosaicing.
fn find_largest_embedded_jpeg(data: &[u8]) -> Option<&[u8]> {
    let mut best: Option<&[u8]> = None;
    let mut i = 0;
    let len = data.len();

    while i + 3 < len {
        if data[i] == 0xFF && data[i + 1] == 0xD8 && data[i + 2] == 0xFF {
            // Find matching EOI (0xFF 0xD9)
            let mut j = i + 3;
            while j + 1 < len {
                if data[j] == 0xFF && data[j + 1] == 0xD9 {
                    let end = j + 2;
                    let slice = &data[i..end];
                    // Ignore tiny EXIF thumbnails (< 8 KB) if larger previews exist,
                    // but still keep the largest slice overall.
                    if best.is_none_or(|b| slice.len() > b.len())
                        && libjpeg_turbo_rs::Decoder::new(slice).is_ok()
                    {
                        best = Some(slice);
                    }
                    i = end;
                    break;
                }
                j += 1;
            }
            if j + 1 >= len {
                break;
            }
        } else {
            i += 1;
        }
    }

    best.filter(|s| s.len() >= 4096)
}

fn try_load_raw_embedded_jpeg(
    path: &Path,
    max_w: u32,
    max_h: u32,
    exif: &crate::image::metadata::ExifMeta,
    opts: LoadOptions,
) -> Option<(Vec<ImageData>, ImageFormatType)> {
    let file = std::fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    let mmap = unsafe { memmap2::Mmap::map(&file).ok()? };
    let jpeg_slice = find_largest_embedded_jpeg(&mmap[..])?;

    let mut decoder = libjpeg_turbo_rs::Decoder::new(jpeg_slice).ok()?;
    let header = decoder.header();
    let src_w = header.width() as u32;
    let src_h = header.height() as u32;
    if src_w < 160 || src_h < 120 {
        return None;
    }

    let orientation = decoder
        .exif_orientation()
        .map(u32::from)
        .or(exif.orientation);
    let target_box = orient_box(decode_box(Some(max_w), Some(max_h)), orientation);
    if let Some((tw, th)) = target_box {
        decoder.set_scale(pick_dct_scale(src_w, src_h, tw, th));
    }
    decoder.set_output_format(libjpeg_turbo_rs::PixelFormat::Rgba);

    let mut buf = vec![0u8; decoder.output_buffer_size().ok()?];
    let info = decoder.decode_image_into(&mut buf).ok()?;
    buf.truncate(info.bytes_written);

    let (rgba, final_w, final_h, _) =
        downsample_to_fit(buf, info.width as u32, info.height as u32, target_box).ok()?;
    let (rgba, final_w, final_h, orientation_deg) = match orientation {
        Some(o) => apply_orientation(rgba, final_w, final_h, o, opts.bake_orientation),
        None => (rgba, final_w, final_h, 0),
    };

    Some((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width: final_w,
            height: final_h,
            format: ImageFormatType::Raw,
            file_size_bytes: file_size,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            // Mark downsampled so progressive refinement can still demosaic the
            // full RAW sensor data if the user zooms in.
            is_downsampled: true,
            orientation_deg,
            gps_coords: exif.gps_coords,
            color_space: exif.color_space,
        }],
        ImageFormatType::Raw,
    ))
}

pub(crate) fn load_raw(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
    exif: &crate::image::metadata::ExifMeta,
    opts: LoadOptions,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    // Fast path for previews and thumbnails: extract embedded JPEG preview.
    if let (Some(mw), Some(mh)) = (max_w, max_h)
        && let Some(preview) = try_load_raw_embedded_jpeg(path, mw, mh, exif, opts)
    {
        return Ok(preview);
    }

    let image =
        rawloader::decode_file(path).map_err(|e| eyre!("Failed to decode RAW file: {:?}", e))?;

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
            exif_loaded: false,
            histogram: None,
            is_downsampled: true,
            orientation_deg: 0,
            gps_coords: exif.gps_coords,
            color_space: exif.color_space,
        }],
        ImageFormatType::Raw,
    ))
}
