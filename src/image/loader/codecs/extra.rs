//! Zero-dependency pure-Rust decoders for TGA, DDS (RGBA/BC1/BC2/BC3), and PCX.

use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::super::{decode_box, downsample_to_fit};
use crate::image::types::{ImageData, ImageFormatType};

const MAX_DIM: u32 = 32_768;

fn finish_image(
    path: &Path,
    rgba: Vec<u8>,
    dims: (u32, u32),
    bounds: (Option<u32>, Option<u32>),
    format: ImageFormatType,
    file_size: u64,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let (rgba, width, height, is_downsampled) =
        downsample_to_fit(rgba, dims.0, dims.1, decode_box(bounds.0, bounds.1))?;
    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width,
            height,
            format,
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
        format,
    ))
}

// ---------------------------------------------------------------------------
// Truevision Targa (.tga)
// ---------------------------------------------------------------------------

pub(crate) fn load_tga(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let data = std::fs::read(path)?;
    let file_size = data.len() as u64;
    let (rgba, w, h) = decode_tga_bytes(&data)?;
    finish_image(
        path,
        rgba,
        (w, h),
        (max_w, max_h),
        ImageFormatType::Tga,
        file_size,
    )
}

pub(crate) fn decode_tga_bytes(data: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    if data.len() < 18 {
        return Err(eyre!("TGA file is truncated (shorter than 18-byte header)"));
    }

    let id_len = data[0] as usize;
    let color_map_type = data[1];
    let image_type = data[2];
    let cmap_origin = u16::from_le_bytes([data[3], data[4]]) as usize;
    let cmap_len = u16::from_le_bytes([data[5], data[6]]) as usize;
    let cmap_depth = data[7];
    let width = u16::from_le_bytes([data[12], data[13]]) as u32;
    let height = u16::from_le_bytes([data[14], data[15]]) as u32;
    let bpp = data[16];
    let descriptor = data[17];

    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Err(eyre!("Invalid TGA dimensions: {width}x{height}"));
    }

    let is_rle = matches!(image_type, 9..=11);
    let base_type = if is_rle { image_type - 8 } else { image_type };
    if !matches!(base_type, 1..=3) {
        return Err(eyre!("Unsupported TGA image type: {image_type}"));
    }

    let bytes_per_pixel = match bpp {
        8 => 1usize,
        15 | 16 => 2,
        24 => 3,
        32 => 4,
        _ => return Err(eyre!("Unsupported TGA bit depth: {bpp}")),
    };

    let mut pos = 18usize
        .checked_add(id_len)
        .ok_or_else(|| eyre!("TGA header offset overflow"))?;
    if pos > data.len() {
        return Err(eyre!("TGA image ID extends past EOF"));
    }

    // Parse optional color map (palette)
    let cmap_entry_bytes = match cmap_depth {
        0 => 0usize,
        15 | 16 => 2,
        24 => 3,
        32 => 4,
        _ if color_map_type == 1 => {
            return Err(eyre!("Unsupported TGA palette depth: {cmap_depth}"));
        }
        _ => 0,
    };
    let palette: Vec<[u8; 4]> = if color_map_type == 1 && cmap_len > 0 {
        let cmap_bytes = cmap_len
            .checked_mul(cmap_entry_bytes)
            .ok_or_else(|| eyre!("TGA palette size overflow"))?;
        let cmap_end = pos
            .checked_add(cmap_bytes)
            .ok_or_else(|| eyre!("TGA palette end overflow"))?;
        if cmap_end > data.len() {
            return Err(eyre!("TGA palette extends past EOF"));
        }
        let mut pal = Vec::with_capacity(cmap_len);
        for chunk in data[pos..cmap_end].chunks_exact(cmap_entry_bytes) {
            pal.push(tga_sample_to_rgba(chunk, cmap_depth, 2));
        }
        pos = cmap_end;
        pal
    } else {
        Vec::new()
    };

    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| eyre!("TGA pixel count overflow"))?;
    let mut raw = Vec::with_capacity(pixel_count * 4);

    let decode_one = |sample: &[u8]| -> Result<[u8; 4]> {
        if base_type == 1 {
            let idx = match bytes_per_pixel {
                1 => sample[0] as usize,
                2 => u16::from_le_bytes([sample[0], sample[1]]) as usize,
                _ => return Err(eyre!("Unsupported indexed TGA bpp: {bpp}")),
            };
            let rel = idx.saturating_sub(cmap_origin);
            Ok(*palette.get(rel).unwrap_or(&[0, 0, 0, 255]))
        } else {
            Ok(tga_sample_to_rgba(sample, bpp, base_type))
        }
    };

    if !is_rle {
        let needed = pixel_count
            .checked_mul(bytes_per_pixel)
            .ok_or_else(|| eyre!("TGA payload size overflow"))?;
        if pos + needed > data.len() {
            return Err(eyre!("TGA pixel payload is truncated"));
        }
        for chunk in data[pos..pos + needed].chunks_exact(bytes_per_pixel) {
            raw.extend_from_slice(&decode_one(chunk)?);
        }
    } else {
        let mut decoded_px = 0usize;
        while decoded_px < pixel_count {
            if pos >= data.len() {
                return Err(eyre!("Truncated TGA RLE stream"));
            }
            let header = data[pos];
            pos += 1;
            let run_len = (header & 0x7F) as usize + 1;
            if decoded_px + run_len > pixel_count {
                return Err(eyre!("TGA RLE packet exceeds image pixel count"));
            }
            if (header & 0x80) != 0 {
                if pos + bytes_per_pixel > data.len() {
                    return Err(eyre!("Truncated TGA RLE pixel"));
                }
                let px = decode_one(&data[pos..pos + bytes_per_pixel])?;
                pos += bytes_per_pixel;
                for _ in 0..run_len {
                    raw.extend_from_slice(&px);
                }
            } else {
                let raw_bytes = run_len * bytes_per_pixel;
                if pos + raw_bytes > data.len() {
                    return Err(eyre!("Truncated TGA raw packet"));
                }
                for chunk in data[pos..pos + raw_bytes].chunks_exact(bytes_per_pixel) {
                    raw.extend_from_slice(&decode_one(chunk)?);
                }
                pos += raw_bytes;
            }
            decoded_px += run_len;
        }
    }

    // Handle origin orientation flags: bit 5 = top-to-bottom, bit 4 = right-to-left.
    let top_origin = (descriptor & 0x20) != 0;
    let right_origin = (descriptor & 0x10) != 0;
    if !top_origin || right_origin {
        crate::image::ImageProcessor::flip_rgba(&mut raw, width, height, right_origin, !top_origin);
    }

    Ok((raw, width, height))
}

fn tga_sample_to_rgba(s: &[u8], bpp: u8, base_type: u8) -> [u8; 4] {
    if base_type == 3 {
        // Grayscale
        let l = s[0];
        let a = if bpp == 16 && s.len() >= 2 { s[1] } else { 255 };
        return [l, l, l, a];
    }
    match bpp {
        15 | 16 => {
            let v = u16::from_le_bytes([s[0], s[1]]);
            let r = (((v >> 10) & 0x1F) * 255 / 31) as u8;
            let g = (((v >> 5) & 0x1F) * 255 / 31) as u8;
            let b = ((v & 0x1F) * 255 / 31) as u8;
            [r, g, b, 255]
        }
        24 => [s[2], s[1], s[0], 255],
        32 => [s[2], s[1], s[0], s[3]],
        _ => [s[0], s[0], s[0], 255],
    }
}

// ---------------------------------------------------------------------------
// DirectDraw Surface (.dds) — Uncompressed RGBA/BGRA/RGB & BC1 / BC2 / BC3
// ---------------------------------------------------------------------------

pub(crate) fn load_dds(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let data = std::fs::read(path)?;
    let file_size = data.len() as u64;
    let (rgba, w, h) = decode_dds_bytes(&data)?;
    finish_image(
        path,
        rgba,
        (w, h),
        (max_w, max_h),
        ImageFormatType::Dds,
        file_size,
    )
}

#[derive(Clone, Copy)]
enum DdsFormat {
    Bc1,
    Bc2,
    Bc3,
    Rgba8,
    Bgra8,
    Rgb8,
    Bgr8,
}

pub(crate) fn decode_dds_bytes(data: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    if data.len() < 128 || &data[0..4] != b"DDS " {
        return Err(eyre!("Invalid or truncated DDS header"));
    }
    let header_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if header_size != 124 {
        return Err(eyre!("Unexpected DDS header size: {header_size}"));
    }

    let height = u32::from_le_bytes(data[12..16].try_into().unwrap());
    let width = u32::from_le_bytes(data[16..20].try_into().unwrap());
    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Err(eyre!("Invalid DDS dimensions: {width}x{height}"));
    }

    let pf_flags = u32::from_le_bytes(data[80..84].try_into().unwrap());
    let fourcc = &data[84..88];
    let rgb_bits = u32::from_le_bytes(data[88..92].try_into().unwrap());
    let r_mask = u32::from_le_bytes(data[92..96].try_into().unwrap());

    let mut payload_offset = 128usize;
    let fmt = if (pf_flags & 0x4) != 0 {
        match fourcc {
            b"DXT1" => DdsFormat::Bc1,
            b"DXT3" => DdsFormat::Bc2,
            b"DXT5" => DdsFormat::Bc3,
            b"DX10" => {
                if data.len() < 148 {
                    return Err(eyre!("Truncated DDS DX10 extended header"));
                }
                payload_offset = 148;
                let dxgi = u32::from_le_bytes(data[128..132].try_into().unwrap());
                match dxgi {
                    28 | 29 => DdsFormat::Rgba8,      // R8G8B8A8_UNORM / SRGB
                    87 | 91 | 88 => DdsFormat::Bgra8, // B8G8R8A8_UNORM / SRGB / B8G8R8X8
                    71 | 72 => DdsFormat::Bc1,        // BC1_UNORM / SRGB
                    74 | 75 => DdsFormat::Bc2,        // BC2_UNORM / SRGB
                    77 | 78 => DdsFormat::Bc3,        // BC3_UNORM / SRGB
                    _ => return Err(eyre!("Unsupported DDS DXGI format: {dxgi}")),
                }
            }
            _ => {
                return Err(eyre!(
                    "Unsupported DDS FourCC: {:?}",
                    String::from_utf8_lossy(fourcc)
                ));
            }
        }
    } else if (pf_flags & 0x40) != 0 {
        match (rgb_bits, r_mask) {
            (32, 0x00FF_0000) => DdsFormat::Bgra8,
            (32, _) => DdsFormat::Rgba8,
            (24, 0x00FF_0000) => DdsFormat::Bgr8,
            (24, _) => DdsFormat::Rgb8,
            _ => return Err(eyre!("Unsupported uncompressed DDS bit depth: {rgb_bits}")),
        }
    } else {
        return Err(eyre!("Unsupported DDS pixel format flags: {pf_flags:#x}"));
    };

    let src = &data[payload_offset..];
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| eyre!("DDS pixel count overflow"))?;
    let mut out = vec![0u8; pixel_count * 4];

    match fmt {
        DdsFormat::Rgba8 => {
            if src.len() < pixel_count * 4 {
                return Err(eyre!("Truncated DDS RGBA8 payload"));
            }
            out.copy_from_slice(&src[..pixel_count * 4]);
        }
        DdsFormat::Bgra8 => {
            if src.len() < pixel_count * 4 {
                return Err(eyre!("Truncated DDS BGRA8 payload"));
            }
            for (dst, s) in out
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src.as_chunks::<4>().0)
            {
                *dst = [s[2], s[1], s[0], s[3]];
            }
        }
        DdsFormat::Rgb8 | DdsFormat::Bgr8 => {
            if src.len() < pixel_count * 3 {
                return Err(eyre!("Truncated DDS 24-bit payload"));
            }
            let bgr = matches!(fmt, DdsFormat::Bgr8);
            for (dst, s) in out
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src.as_chunks::<3>().0)
            {
                *dst = if bgr {
                    [s[2], s[1], s[0], 255]
                } else {
                    [s[0], s[1], s[2], 255]
                };
            }
        }
        DdsFormat::Bc1 => decode_bc_blocks(src, &mut out, width, height, 8, decode_bc1_block)?,
        DdsFormat::Bc2 => decode_bc_blocks(src, &mut out, width, height, 16, decode_bc2_block)?,
        DdsFormat::Bc3 => decode_bc_blocks(src, &mut out, width, height, 16, decode_bc3_block)?,
    }

    Ok((out, width, height))
}

fn decode_bc_blocks(
    src: &[u8],
    out: &mut [u8],
    width: u32,
    height: u32,
    block_bytes: usize,
    decoder: fn(&[u8]) -> [[u8; 4]; 16],
) -> Result<()> {
    let bw = (width as usize).div_ceil(4);
    let bh = (height as usize).div_ceil(4);
    let needed = bw
        .checked_mul(bh)
        .and_then(|n| n.checked_mul(block_bytes))
        .ok_or_else(|| eyre!("DDS block count overflow"))?;
    if src.len() < needed {
        return Err(eyre!("Truncated DDS compressed payload"));
    }
    let w = width as usize;
    let h = height as usize;
    for by in 0..bh {
        for bx in 0..bw {
            let offset = (by * bw + bx) * block_bytes;
            let pixels = decoder(&src[offset..offset + block_bytes]);
            for py in 0..4 {
                let y = by * 4 + py;
                if y >= h {
                    break;
                }
                for px in 0..4 {
                    let x = bx * 4 + px;
                    if x >= w {
                        break;
                    }
                    let dst = (y * w + x) * 4;
                    out[dst..dst + 4].copy_from_slice(&pixels[py * 4 + px]);
                }
            }
        }
    }
    Ok(())
}

fn rgb565_to_rgb8(c: u16) -> [u8; 3] {
    let r = (((c >> 11) & 0x1F) * 255 / 31) as u8;
    let g = (((c >> 5) & 0x3F) * 255 / 63) as u8;
    let b = ((c & 0x1F) * 255 / 31) as u8;
    [r, g, b]
}

fn decode_bc1_block(b: &[u8]) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([b[0], b[1]]);
    let c1 = u16::from_le_bytes([b[2], b[3]]);
    let rgb0 = rgb565_to_rgb8(c0);
    let rgb1 = rgb565_to_rgb8(c1);
    let mut lut = [[0u8; 4]; 4];
    lut[0] = [rgb0[0], rgb0[1], rgb0[2], 255];
    lut[1] = [rgb1[0], rgb1[1], rgb1[2], 255];
    if c0 > c1 {
        lut[2] = [
            ((2 * rgb0[0] as u16 + rgb1[0] as u16) / 3) as u8,
            ((2 * rgb0[1] as u16 + rgb1[1] as u16) / 3) as u8,
            ((2 * rgb0[2] as u16 + rgb1[2] as u16) / 3) as u8,
            255,
        ];
        lut[3] = [
            ((rgb0[0] as u16 + 2 * rgb1[0] as u16) / 3) as u8,
            ((rgb0[1] as u16 + 2 * rgb1[1] as u16) / 3) as u8,
            ((rgb0[2] as u16 + 2 * rgb1[2] as u16) / 3) as u8,
            255,
        ];
    } else {
        lut[2] = [
            ((rgb0[0] as u16 + rgb1[0] as u16) / 2) as u8,
            ((rgb0[1] as u16 + rgb1[1] as u16) / 2) as u8,
            ((rgb0[2] as u16 + rgb1[2] as u16) / 2) as u8,
            255,
        ];
        lut[3] = [0, 0, 0, 0];
    }
    let indices = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
    let mut out = [[0u8; 4]; 16];
    for (i, px) in out.iter_mut().enumerate() {
        *px = lut[((indices >> (i * 2)) & 0x3) as usize];
    }
    out
}

fn decode_bc2_block(b: &[u8]) -> [[u8; 4]; 16] {
    let mut out = decode_bc1_block(&b[8..16]);
    let alpha_bits = u64::from_le_bytes(b[0..8].try_into().unwrap());
    for (i, px) in out.iter_mut().enumerate() {
        let a4 = ((alpha_bits >> (i * 4)) & 0xF) as u8;
        px[3] = (a4 << 4) | a4;
    }
    out
}

fn decode_bc3_block(b: &[u8]) -> [[u8; 4]; 16] {
    let mut out = decode_bc1_block(&b[8..16]);
    let a0 = b[0] as u16;
    let a1 = b[1] as u16;
    let mut alut = [0u8; 8];
    alut[0] = a0 as u8;
    alut[1] = a1 as u8;
    if a0 > a1 {
        for i in 1..=6u16 {
            alut[1 + i as usize] = (((7 - i) * a0 + i * a1) / 7) as u8;
        }
    } else {
        for i in 1..=4u16 {
            alut[1 + i as usize] = (((5 - i) * a0 + i * a1) / 5) as u8;
        }
        alut[6] = 0;
        alut[7] = 255;
    }
    let mut bits = 0u64;
    for i in 0..6 {
        bits |= (b[2 + i] as u64) << (i * 8);
    }
    for (i, px) in out.iter_mut().enumerate() {
        px[3] = alut[((bits >> (i * 3)) & 0x7) as usize];
    }
    out
}

// ---------------------------------------------------------------------------
// ZSoft PCX (.pcx) — 8-bit VGA palette, 24-bit RGB, 4-bit EGA, grayscale
// ---------------------------------------------------------------------------

pub(crate) fn load_pcx(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let data = std::fs::read(path)?;
    let file_size = data.len() as u64;
    let (rgba, w, h) = decode_pcx_bytes(&data)?;
    finish_image(
        path,
        rgba,
        (w, h),
        (max_w, max_h),
        ImageFormatType::Pcx,
        file_size,
    )
}

pub(crate) fn decode_pcx_bytes(data: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    if data.len() < 128 || data[0] != 0x0A {
        return Err(eyre!("Invalid or truncated PCX header"));
    }
    let encoding = data[2];
    let bpp = data[3];
    let x_min = u16::from_le_bytes([data[4], data[5]]) as u32;
    let y_min = u16::from_le_bytes([data[6], data[7]]) as u32;
    let x_max = u16::from_le_bytes([data[8], data[9]]) as u32;
    let y_max = u16::from_le_bytes([data[10], data[11]]) as u32;
    let planes = data[65] as usize;
    let bytes_per_line = u16::from_le_bytes([data[66], data[67]]) as usize;

    if x_max < x_min || y_max < y_min {
        return Err(eyre!("Invalid PCX bounding box"));
    }
    let width = x_max - x_min + 1;
    let height = y_max - y_min + 1;
    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Err(eyre!("Invalid PCX dimensions: {width}x{height}"));
    }
    if planes == 0 || bytes_per_line == 0 {
        return Err(eyre!("Invalid PCX plane layout"));
    }

    let scanline_len = planes
        .checked_mul(bytes_per_line)
        .ok_or_else(|| eyre!("PCX scanline overflow"))?;
    let total_decoded = scanline_len
        .checked_mul(height as usize)
        .ok_or_else(|| eyre!("PCX buffer overflow"))?;

    let mut scanlines = vec![0u8; total_decoded];
    let mut src_pos = 128usize;
    let mut dst_pos = 0usize;

    if encoding == 1 {
        while dst_pos < total_decoded {
            if src_pos >= data.len() {
                return Err(eyre!("Truncated PCX RLE data"));
            }
            let b = data[src_pos];
            src_pos += 1;
            if (b & 0xC0) == 0xC0 {
                let count = (b & 0x3F) as usize;
                if src_pos >= data.len() {
                    return Err(eyre!("Truncated PCX RLE value"));
                }
                let val = data[src_pos];
                src_pos += 1;
                let end = (dst_pos + count).min(total_decoded);
                scanlines[dst_pos..end].fill(val);
                dst_pos = end;
            } else {
                scanlines[dst_pos] = b;
                dst_pos += 1;
            }
        }
    } else if encoding == 0 {
        if src_pos + total_decoded > data.len() {
            return Err(eyre!("Truncated uncompressed PCX data"));
        }
        scanlines.copy_from_slice(&data[src_pos..src_pos + total_decoded]);
    } else {
        return Err(eyre!("Unsupported PCX encoding: {encoding}"));
    }

    let w = width as usize;
    let h = height as usize;
    let mut out = vec![0u8; w * h * 4];

    match (bpp, planes) {
        (8, 3) => {
            if bytes_per_line < w {
                return Err(eyre!("PCX bytes_per_line smaller than width"));
            }
            for y in 0..h {
                let row = &scanlines[y * scanline_len..(y + 1) * scanline_len];
                let r_plane = &row[0..bytes_per_line];
                let g_plane = &row[bytes_per_line..2 * bytes_per_line];
                let b_plane = &row[2 * bytes_per_line..3 * bytes_per_line];
                for x in 0..w {
                    let dst = (y * w + x) * 4;
                    out[dst] = r_plane[x];
                    out[dst + 1] = g_plane[x];
                    out[dst + 2] = b_plane[x];
                    out[dst + 3] = 255;
                }
            }
        }
        (8, 1) => {
            if bytes_per_line < w {
                return Err(eyre!("PCX bytes_per_line smaller than width"));
            }
            let has_vga_palette = data.len() >= 128 + 769 && data[data.len() - 769] == 0x0C;
            let pal = if has_vga_palette {
                Some(&data[data.len() - 768..])
            } else {
                None
            };
            for y in 0..h {
                let row = &scanlines[y * scanline_len..(y + 1) * scanline_len];
                for (x, &idx_byte) in row.iter().enumerate().take(w) {
                    let idx = idx_byte as usize;
                    let dst = (y * w + x) * 4;
                    if let Some(p) = pal {
                        out[dst] = p[idx * 3];
                        out[dst + 1] = p[idx * 3 + 1];
                        out[dst + 2] = p[idx * 3 + 2];
                    } else {
                        out[dst] = idx as u8;
                        out[dst + 1] = idx as u8;
                        out[dst + 2] = idx as u8;
                    }
                    out[dst + 3] = 255;
                }
            }
        }
        (1, 4) => {
            // Classic 16-color EGA palette in header bytes 16..64
            let ega = &data[16..64];
            for y in 0..h {
                let row = &scanlines[y * scanline_len..(y + 1) * scanline_len];
                for x in 0..w {
                    let byte_idx = x / 8;
                    let bit = 7 - (x % 8);
                    let mut idx = 0usize;
                    for p in 0..4 {
                        if byte_idx < bytes_per_line
                            && ((row[p * bytes_per_line + byte_idx] >> bit) & 1) != 0
                        {
                            idx |= 1 << p;
                        }
                    }
                    let dst = (y * w + x) * 4;
                    out[dst] = ega[idx * 3];
                    out[dst + 1] = ega[idx * 3 + 1];
                    out[dst + 2] = ega[idx * 3 + 2];
                    out[dst + 3] = 255;
                }
            }
        }
        _ => return Err(eyre!("Unsupported PCX format: {bpp} bpp, {planes} planes")),
    }

    Ok((out, width, height))
}
