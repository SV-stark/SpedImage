//! Writing edited images back to disk.
//!
//! Three output formats are supported, each through a crate that is already in
//! the dependency tree for decoding:
//!
//! - PNG via `zune-image` (its `png` feature carries the encoder),
//! - JPEG via `libjpeg-turbo-rs`, which ships an encoder and a lossless
//!   DCT-domain transform beside the decoder,
//! - WebP via `image-webp`, which can only *encode* lossless (VP8L).
//!
//! Anything else (HEIC, RAW, SVG, TIFF, ...) cannot be written, so an edited
//! copy of such a file is saved as PNG instead of asking an encoder that does
//! not exist.

use color_eyre::eyre::{Result, WrapErr, eyre};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::loader::{FrameLimit, ImageLoader, LoadOptions, exif_orientation_transform};
use super::processing::ImageProcessor;
use super::types::ImageData;
use crate::render::ImageAdjustments;

/// JPEG quality used when an edit has to be re-encoded.
pub const JPEG_QUALITY: u8 = 92;

/// A format this build can encode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    Png,
    Jpeg,
    WebP,
    Qoi,
    Bmp,
}

impl SaveFormat {
    pub const ALL: [SaveFormat; 5] = [
        SaveFormat::Png,
        SaveFormat::Jpeg,
        SaveFormat::WebP,
        SaveFormat::Qoi,
        SaveFormat::Bmp,
    ];

    pub fn from_extension(ext: &str) -> Option<Self> {
        if ext.eq_ignore_ascii_case("png") {
            Some(Self::Png)
        } else if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
            Some(Self::Jpeg)
        } else if ext.eq_ignore_ascii_case("webp") {
            Some(Self::WebP)
        } else if ext.eq_ignore_ascii_case("qoi") {
            Some(Self::Qoi)
        } else if ext.eq_ignore_ascii_case("bmp") {
            Some(Self::Bmp)
        } else {
            None
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Canonical extension for new file names.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Qoi => "qoi",
            Self::Bmp => "bmp",
        }
    }

    /// Every extension the save dialog should accept for this format.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Png => &["png"],
            Self::Jpeg => &["jpg", "jpeg"],
            Self::WebP => &["webp"],
            Self::Qoi => &["qoi"],
            Self::Bmp => &["bmp"],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG Image",
            Self::Jpeg => "JPEG Image",
            Self::WebP => "WebP Image (lossless)",
            Self::Qoi => "QOI Image (lossless)",
            Self::Bmp => "BMP Image (uncompressed)",
        }
    }
}

/// Where Ctrl+S writes an edited copy of `source`.
///
/// Keeps the source extension when this build can encode it, so a JPEG stays
/// a JPEG. Otherwise (HEIC, RAW, SVG, TIFF, ...) the copy becomes a PNG, which
/// is lossless and always writable.
pub fn edited_output_path(source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string());
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| SaveFormat::from_extension(e).is_some())
        .map(str::to_owned)
        .unwrap_or_else(|| SaveFormat::Png.extension().to_string());
    source.with_file_name(format!("{stem}_edited.{ext}"))
}

/// Encode an RGBA8 buffer into `format` using the default JPEG quality.
pub fn encode(format: SaveFormat, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    encode_with_quality(format, rgba, width, height, JPEG_QUALITY)
}

/// Encode an RGBA8 buffer into `format` with an explicit JPEG quality (`1..=100`).
pub fn encode_with_quality(
    format: SaveFormat,
    rgba: &[u8],
    width: u32,
    height: u32,
    jpeg_quality: u8,
) -> Result<Vec<u8>> {
    let expected = width as usize * height as usize * 4;
    if width == 0 || height == 0 {
        return Err(eyre!("Cannot save an image with a zero dimension"));
    }
    if rgba.len() < expected {
        return Err(eyre!(
            "Pixel buffer holds {} bytes, {width}x{height} needs {expected}",
            rgba.len()
        ));
    }
    let rgba = &rgba[..expected];

    match format {
        SaveFormat::Png => {
            let img = zune_image::image::Image::from_u8(
                rgba,
                width as usize,
                height as usize,
                zune_core::colorspace::ColorSpace::RGBA,
            );
            img.write_to_vec(zune_image::codecs::ImageFormat::PNG)
                .map_err(|e| eyre!("PNG encode failed: {e:?}"))
        }
        SaveFormat::Jpeg => {
            // JPEG has no alpha channel. Dropping it would expose whatever RGB
            // sits under transparent pixels (usually black), so composite onto
            // white the way browsers and Explorer previews show transparency.
            let rgb = flatten_onto_white(rgba);
            libjpeg_turbo_rs::Encoder::new(
                &rgb,
                width as usize,
                height as usize,
                libjpeg_turbo_rs::PixelFormat::Rgb,
            )
            .quality(jpeg_quality.clamp(1, 100))
            .encode()
            .map_err(|e| eyre!("JPEG encode failed: {e:?}"))
        }
        SaveFormat::WebP => {
            use image_webp::{ColorType, WebPEncoder};
            let mut out = Vec::new();
            let pixels = rgba.as_chunks::<4>().0;
            let opaque = pixels.iter().all(|p| p[3] == 255);
            if opaque {
                // VP8L stores no alpha plane for RGB input, which is smaller.
                let rgb: Vec<u8> = pixels.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
                WebPEncoder::new(&mut out).encode(&rgb, width, height, ColorType::Rgb8)
            } else {
                WebPEncoder::new(&mut out).encode(rgba, width, height, ColorType::Rgba8)
            }
            .map_err(|e| eyre!("WebP encode failed: {e:?}"))?;
            Ok(out)
        }
        SaveFormat::Qoi => {
            qoi::encode_to_vec(rgba, width, height).map_err(|e| eyre!("QOI encode failed: {e}"))
        }
        SaveFormat::Bmp => encode_bmp_bgra(rgba, width, height),
    }
}

/// Pure-Rust 32-bit BGRA BMP encoder (standard 14-byte file header + 40-byte
/// BITMAPINFOHEADER, bottom-up scanlines). Zero dependencies, lossless, fast.
fn encode_bmp_bgra(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let pixel_bytes = (width as u64)
        .checked_mul(height as u64)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| eyre!("BMP dimensions overflow"))?;
    let file_size = 54u64
        .checked_add(pixel_bytes)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| eyre!("BMP exceeds 4 GiB format limit"))?;

    let mut out = Vec::with_capacity(file_size as usize);
    // BITMAPFILEHEADER (14 bytes)
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved
    out.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset
    // BITMAPINFOHEADER (40 bytes)
    out.extend_from_slice(&40u32.to_le_bytes()); // header size
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&(height as i32).to_le_bytes()); // positive = bottom-up
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bpp
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB (uncompressed)
    out.extend_from_slice(&(pixel_bytes as u32).to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes()); // ~72 DPI X
    out.extend_from_slice(&2835i32.to_le_bytes()); // ~72 DPI Y
    out.extend_from_slice(&0u32.to_le_bytes()); // colors used
    out.extend_from_slice(&0u32.to_le_bytes()); // important colors

    let row_stride = width as usize * 4;
    for row in rgba.chunks_exact(row_stride).rev() {
        for px in row.as_chunks::<4>().0 {
            out.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }
    Ok(out)
}

/// Composite straight-alpha RGBA onto white, producing packed RGB.
fn flatten_onto_white(rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len() / 4 * 3);
    for p in rgba.as_chunks::<4>().0 {
        let a = p[3] as u32;
        if a == 255 {
            out.extend_from_slice(&p[..3]);
        } else {
            let inv = 255 - a;
            for &c in &p[..3] {
                out.push(((c as u32 * a + 255 * inv + 127) / 255) as u8);
            }
        }
    }
    out
}

/// Encode `rgba` in the format named by `path`'s extension and write it.
pub fn save(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<()> {
    let format = format_for_output(path)?;
    let bytes = encode(format, rgba, width, height)?;
    write_atomically(path, &bytes)
}

fn format_for_output(path: &Path) -> Result<SaveFormat> {
    SaveFormat::from_path(path).ok_or_else(|| {
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_else(|| "a file with no extension".to_string());
        eyre!("Cannot save as {ext}: use a .png, .jpg, .webp, .qoi or .bmp file name")
    })
}

/// Write through a temporary sibling and rename it into place, so a failed
/// encode or a full disk never leaves a truncated file at `path`.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let file_name = path
        .file_name()
        .ok_or_else(|| eyre!("Save path {path:?} has no file name"))?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(file_name);
    tmp_name.push(".spedimage-tmp");
    let tmp = path.with_file_name(tmp_name);

    std::fs::write(&tmp, bytes).wrap_err_with(|| format!("Failed to write {}", tmp.display()))?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(eyre!("Failed to move the saved file into place: {e}"));
    }
    Ok(())
}

/// What a save actually did, for the status message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
    pub path: PathBuf,
    /// True when a JPEG was rotated/flipped in the DCT domain, so no pixel was
    /// re-encoded and the original EXIF and ICC data were carried over.
    pub lossless: bool,
}

/// Save `source` with `adjustments` applied to `output`.
///
/// `displayed` is the buffer currently on screen. It is reused when it already
/// holds the full-resolution pixels; a downsampled preview is never written.
///
/// JPEG-to-JPEG saves whose only edits are 90° rotations and flips go through
/// libjpeg-turbo's lossless transform, the same operation `jpegtran` performs.
/// Everything else is decoded at full resolution, edited, and re-encoded.
pub fn save_edited(
    source: &Path,
    output: &Path,
    adjustments: &ImageAdjustments,
    displayed: Option<&ImageData>,
) -> Result<SaveOutcome> {
    save_edited_with_quality(source, output, adjustments, displayed, JPEG_QUALITY)
}

/// Export parameters for [`save_edited_with_options`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveOptions {
    pub jpeg_quality: u8,
    /// Strip EXIF, GPS, and ICC chunks on save (privacy export).
    pub strip_metadata: bool,
    /// Optional longest-edge cap in pixels; `None` keeps full resolution.
    pub max_dimension: Option<u32>,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            jpeg_quality: JPEG_QUALITY,
            strip_metadata: false,
            max_dimension: None,
        }
    }
}

/// Save `source` with `adjustments` applied to `output` using `jpeg_quality`,
/// preserving EXIF (with orientation reset to 1) and ICC metadata where supported.
pub fn save_edited_with_quality(
    source: &Path,
    output: &Path,
    adjustments: &ImageAdjustments,
    displayed: Option<&ImageData>,
    jpeg_quality: u8,
) -> Result<SaveOutcome> {
    save_edited_with_options(
        source,
        output,
        adjustments,
        displayed,
        SaveOptions {
            jpeg_quality,
            ..SaveOptions::default()
        },
    )
}

/// Save `source` with `adjustments` applied to `output` according to `options`
/// (JPEG quality, optional metadata stripping, optional SIMD longest-edge resize).
pub fn save_edited_with_options(
    source: &Path,
    output: &Path,
    adjustments: &ImageAdjustments,
    displayed: Option<&ImageData>,
    options: SaveOptions,
) -> Result<SaveOutcome> {
    let format = format_for_output(output)?;
    let max_dim = options.max_dimension.filter(|&d| d > 0);

    if format == SaveFormat::Jpeg && max_dim.is_none() && lossless_eligible(source, adjustments) {
        match lossless_jpeg(source, adjustments) {
            Ok(Some(mut bytes)) => {
                if options.strip_metadata {
                    bytes = strip_jpeg_metadata(bytes);
                }
                write_atomically(output, &bytes)?;
                return Ok(SaveOutcome {
                    path: output.to_path_buf(),
                    lossless: true,
                });
            }
            Ok(None) => {}
            // Typically a partial edge MCU (`perfect` refused it). Re-encoding
            // still produces a correct file, just not a lossless one.
            Err(e) => tracing::debug!("Lossless JPEG transform declined for {source:?}: {e}"),
        }
    }

    let (rgba, width, height) = full_resolution_pixels(source, displayed)?;
    let (mut edited, mut w, mut h) =
        ImageProcessor::apply_adjustments_cpu(&rgba, width, height, adjustments);

    if let Some(cap) = max_dim
        && (w > cap || h > cap)
    {
        let (resized, rw, rh) = resize_rgba_to_fit(&edited, w, h, cap)?;
        edited = resized;
        w = rw;
        h = rh;
    }

    let encoded = encode_with_quality(format, &edited, w, h, options.jpeg_quality)?;
    let bytes = if options.strip_metadata {
        encoded
    } else {
        let (mut exif, icc) = extract_source_metadata(source);
        if let Some(ref mut exif_bytes) = exif {
            normalize_exif_orientation(exif_bytes);
        }
        embed_metadata(format, encoded, exif, icc)
    };
    write_atomically(output, &bytes)?;
    Ok(SaveOutcome {
        path: output.to_path_buf(),
        lossless: false,
    })
}

/// Strip EXIF and ICC chunks from an already-encoded JPEG without touching DCT coefficients.
fn strip_jpeg_metadata(encoded: Vec<u8>) -> Vec<u8> {
    use img_parts::{ImageEXIF, ImageICC};
    if let Ok(mut jpeg) = img_parts::jpeg::Jpeg::from_bytes(encoded.clone().into()) {
        jpeg.set_exif(None);
        jpeg.set_icc_profile(None);
        return jpeg.encoder().bytes().to_vec();
    }
    encoded
}

/// Downscale an RGBA8 buffer so its longest edge is at most `max_dim`,
/// preserving aspect ratio with SIMD `fast_image_resize`.
fn resize_rgba_to_fit(
    rgba: &[u8],
    width: u32,
    height: u32,
    max_dim: u32,
) -> Result<(Vec<u8>, u32, u32)> {
    use fast_image_resize as fr;
    let max_dim = max_dim.max(1);
    let longest = width.max(height);
    if longest <= max_dim {
        return Ok((rgba.to_vec(), width, height));
    }
    let scale = max_dim as f64 / longest as f64;
    let dst_w = ((width as f64 * scale).round() as u32).max(1);
    let dst_h = ((height as f64 * scale).round() as u32).max(1);

    let src = fr::images::ImageRef::new(width, height, rgba, fr::PixelType::U8x4)
        .map_err(|e| eyre!("Export resize src failed: {e:?}"))?;
    let mut dst = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
    let mut resizer = ImageProcessor::create_simd_resizer();
    resizer
        .resize(&src, &mut dst, None)
        .map_err(|e| eyre!("Export resize failed: {e:?}"))?;
    Ok((dst.into_vec(), dst_w, dst_h))
}

/// Extract raw EXIF and ICC profile bytes from a JPEG, PNG, or WebP source file.
fn extract_source_metadata(source: &Path) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    use img_parts::{ImageEXIF, ImageICC};

    if !source.is_file() {
        return (None, None);
    }
    let Ok(data) = std::fs::read(source) else {
        return (None, None);
    };
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("jpg" | "jpeg") => {
            let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(data.into()) else {
                return (None, None);
            };
            (
                jpeg.exif().map(|b| b.to_vec()),
                jpeg.icc_profile().map(|b| b.to_vec()),
            )
        }
        Some("png") => {
            let Ok(png) = img_parts::png::Png::from_bytes(data.into()) else {
                return (None, None);
            };
            (
                png.exif().map(|b| b.to_vec()),
                png.icc_profile().map(|b| b.to_vec()),
            )
        }
        Some("webp") => {
            let Ok(webp) = img_parts::webp::WebP::from_bytes(data.into()) else {
                return (None, None);
            };
            (
                webp.exif().map(|b| b.to_vec()),
                webp.icc_profile().map(|b| b.to_vec()),
            )
        }
        _ => (None, None),
    }
}

/// Reset the TIFF IFD0 Orientation tag (0x0112) to 1 (normal) in a raw EXIF
/// payload because `save_edited_with_quality` already bakes orientation into
/// the saved pixel buffer.
fn normalize_exif_orientation(exif: &mut [u8]) {
    let tiff = if exif.starts_with(b"Exif\0\0") {
        &mut exif[6..]
    } else {
        exif
    };
    if tiff.len() < 8 {
        return;
    }
    let little_endian = match &tiff[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return,
    };
    let read_u16 = |b: &[u8]| -> u16 {
        if little_endian {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        }
    };
    let read_u32 = |b: &[u8]| -> u32 {
        if little_endian {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        }
    };
    let ifd0_offset = read_u32(&tiff[4..8]) as usize;
    if ifd0_offset + 2 > tiff.len() {
        return;
    }
    let count = read_u16(&tiff[ifd0_offset..ifd0_offset + 2]) as usize;
    for i in 0..count {
        let entry = ifd0_offset + 2 + i * 12;
        if entry + 12 > tiff.len() {
            break;
        }
        let tag = read_u16(&tiff[entry..entry + 2]);
        let typ = read_u16(&tiff[entry + 2..entry + 4]);
        let cnt = read_u32(&tiff[entry + 4..entry + 8]);
        if tag == 0x0112 && typ == 3 && cnt == 1 {
            let bytes = if little_endian {
                1u16.to_le_bytes()
            } else {
                1u16.to_be_bytes()
            };
            tiff[entry + 8] = bytes[0];
            tiff[entry + 9] = bytes[1];
            tiff[entry + 10] = 0;
            tiff[entry + 11] = 0;
            break;
        }
    }
}

/// Embed EXIF and ICC chunks into an encoded JPEG, PNG, or WebP buffer.
/// Falls back to `encoded` unchanged if `img_parts` cannot parse it.
fn embed_metadata(
    format: SaveFormat,
    encoded: Vec<u8>,
    exif: Option<Vec<u8>>,
    icc: Option<Vec<u8>>,
) -> Vec<u8> {
    use img_parts::{ImageEXIF, ImageICC};

    if exif.is_none() && icc.is_none() {
        return encoded;
    }
    match format {
        SaveFormat::Jpeg => {
            if let Ok(mut jpeg) = img_parts::jpeg::Jpeg::from_bytes(encoded.clone().into()) {
                if let Some(e) = exif {
                    jpeg.set_exif(Some(e.into()));
                }
                if let Some(i) = icc {
                    jpeg.set_icc_profile(Some(i.into()));
                }
                return jpeg.encoder().bytes().to_vec();
            }
        }
        SaveFormat::Png => {
            if let Ok(mut png) = img_parts::png::Png::from_bytes(encoded.clone().into()) {
                if let Some(e) = exif {
                    png.set_exif(Some(e.into()));
                }
                if let Some(i) = icc {
                    png.set_icc_profile(Some(i.into()));
                }
                return png.encoder().bytes().to_vec();
            }
        }
        SaveFormat::WebP => {
            if let Ok(mut webp) = img_parts::webp::WebP::from_bytes(encoded.clone().into()) {
                if let Some(e) = exif {
                    webp.set_exif(Some(e.into()));
                }
                if let Some(i) = icc {
                    webp.set_icc_profile(Some(i.into()));
                }
                return webp.encoder().bytes().to_vec();
            }
        }
        SaveFormat::Qoi | SaveFormat::Bmp => {}
    }
    encoded
}

fn is_jpeg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
}

/// A save can stay in the DCT domain only when it changes geometry alone, in
/// whole quarter turns, and does not crop.
fn lossless_eligible(source: &Path, adjustments: &ImageAdjustments) -> bool {
    is_jpeg_path(source)
        && source.is_file()
        && adjustments.crop_rect_actual.is_none()
        && !ImageProcessor::has_tonal_edits(adjustments)
        && quarter_turns(adjustments.rotation).is_some()
}

/// The user rotation in whole quarter turns, or `None` for any other angle.
fn quarter_turns(rotation_rad: f32) -> Option<u8> {
    let deg = (rotation_rad.to_degrees().round() as i32).rem_euclid(360);
    (deg % 90 == 0).then_some((deg / 90) as u8)
}

/// Pixels at full resolution with EXIF orientation baked in, matching the
/// coordinate space `ImageAdjustments` are expressed in.
fn full_resolution_pixels(
    source: &Path,
    displayed: Option<&ImageData>,
) -> Result<(Arc<Vec<u8>>, u32, u32)> {
    if let Some(img) = displayed
        && img.path == source
        && !img.is_downsampled
    {
        // The preview path defers pure EXIF rotations to the GPU, so the
        // stored buffer may still be sideways. Saving it as-is wrote photos
        // in the wrong orientation.
        if img.orientation_deg == 0 {
            return Ok((Arc::clone(&img.rgba_data), img.width, img.height));
        }
        let (data, w, h) = ImageProcessor::rotate_rgba(
            &img.rgba_data,
            img.width,
            img.height,
            img.orientation_deg as i32,
        );
        return Ok((Arc::new(data), w, h));
    }

    let (frames, _) = ImageLoader::load_with(
        source,
        None,
        None,
        LoadOptions {
            frames: FrameLimit::First,
            bake_orientation: true,
        },
    )
    .map_err(|e| eyre!("Failed to open full-resolution image: {e}"))?;
    let first = frames
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("No image frames loaded"))?;
    Ok((first.rgba_data, first.width, first.height))
}

// ---------------------------------------------------------------------------
// Lossless JPEG geometry
// ---------------------------------------------------------------------------

/// An element of the dihedral group D4 (the 8 rotations/mirrors of a
/// rectangle), as an integer matrix acting on pixel coordinates with x to the
/// right and y *down*. Applying `a` then `b` is `mul(b, a)`.
type D4 = [[i8; 2]; 2];

const IDENTITY: D4 = [[1, 0], [0, 1]];
/// 90° clockwise: (x, y) -> (-y, x). Matches `ImageProcessor::rotate_rgba(90)`.
const ROT90: D4 = [[0, -1], [1, 0]];
const MIRROR_H: D4 = [[-1, 0], [0, 1]];
const MIRROR_V: D4 = [[1, 0], [0, -1]];

fn mul(a: D4, b: D4) -> D4 {
    let mut out = [[0i8; 2]; 2];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = a[r][0] * b[0][c] + a[r][1] * b[1][c];
        }
    }
    out
}

fn rotation(quarter_turns: u32) -> D4 {
    (0..quarter_turns % 4).fold(IDENTITY, |m, _| mul(ROT90, m))
}

/// The pixel transform `apply_orientation` performs for an EXIF value: mirror
/// first, then rotate.
fn exif_matrix(orientation: u32) -> D4 {
    match exif_orientation_transform(orientation) {
        Some((deg, mirror_h, mirror_v)) => {
            let mut m = IDENTITY;
            if mirror_h {
                m = mul(MIRROR_H, m);
            }
            if mirror_v {
                m = mul(MIRROR_V, m);
            }
            mul(rotation(deg / 90), m)
        }
        None => IDENTITY,
    }
}

/// The pixel transform `apply_adjustments_cpu` performs: rotate, then mirror
/// horizontally, then vertically.
fn user_matrix(quarter_turns: u8, flip_h: bool, flip_v: bool) -> D4 {
    let mut m = rotation(quarter_turns as u32);
    if flip_h {
        m = mul(MIRROR_H, m);
    }
    if flip_v {
        m = mul(MIRROR_V, m);
    }
    m
}

fn transform_op(m: D4) -> libjpeg_turbo_rs::TransformOp {
    use libjpeg_turbo_rs::TransformOp as Op;
    let table: [(Op, D4); 8] = [
        (Op::None, IDENTITY),
        (Op::HFlip, MIRROR_H),
        (Op::VFlip, MIRROR_V),
        (Op::Rot90, ROT90),
        (Op::Rot180, rotation(2)),
        (Op::Rot270, rotation(3)),
        // Main-diagonal mirror: (x, y) -> (y, x).
        (Op::Transpose, [[0, 1], [1, 0]]),
        // Anti-diagonal mirror: (x, y) -> (-y, -x).
        (Op::Transverse, [[0, -1], [-1, 0]]),
    ];
    table
        .into_iter()
        .find(|(_, t)| *t == m)
        .map(|(op, _)| op)
        // D4 is closed under `mul`, so every product is one of the 8 above.
        .unwrap_or(Op::None)
}

/// Rotate/flip a JPEG without decoding it. Returns `Ok(None)` when the edit is
/// not a whole quarter turn.
fn lossless_jpeg(source: &Path, adjustments: &ImageAdjustments) -> Result<Option<Vec<u8>>> {
    let Some(turns) = quarter_turns(adjustments.rotation) else {
        return Ok(None);
    };
    let data = std::fs::read(source).wrap_err_with(|| format!("Failed to read {source:?}"))?;

    // The stored pixels are pre-EXIF; the user's edits are relative to what
    // was displayed, i.e. post-EXIF. Fold both into a single transform and
    // then mark the result as upright.
    let orientation = libjpeg_turbo_rs::Decoder::new(&data)
        .ok()
        .and_then(|d| d.exif_orientation())
        .map(u32::from)
        .unwrap_or(1);
    let total = mul(
        user_matrix(
            turns,
            adjustments.flip_horizontal,
            adjustments.flip_vertical,
        ),
        exif_matrix(orientation),
    );

    let mut out = libjpeg_turbo_rs::transform_jpeg_with_options(
        &data,
        &libjpeg_turbo_rs::TransformOptions {
            op: transform_op(total),
            // Refuse rather than silently dropping or mangling a partial edge
            // block; the caller falls back to a full re-encode.
            perfect: true,
            copy_markers: libjpeg_turbo_rs::MarkerCopyMode::All,
            ..Default::default()
        },
    )
    .map_err(|e| eyre!("{e:?}"))?;

    // Markers are copied verbatim, including the orientation tag. Leaving it
    // would make every EXIF-aware viewer rotate the already-rotated pixels.
    reset_exif_orientation(&mut out);
    Ok(Some(out))
}

/// Set the EXIF orientation tag of a JPEG to 1 (upright) in place. Returns
/// true when a tag was found and rewritten.
pub fn reset_exif_orientation(jpeg: &mut [u8]) -> bool {
    if jpeg.len() < 4 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
        return false;
    }
    let mut i = 2;
    while i + 4 <= jpeg.len() {
        if jpeg[i] != 0xFF {
            return false;
        }
        let marker = jpeg[i + 1];
        match marker {
            // Fill byte before a marker.
            0xFF => {
                i += 1;
                continue;
            }
            // Start of scan / end of image: no metadata past this point.
            0xDA | 0xD9 => return false,
            // Standalone markers carry no length.
            0x01 | 0xD0..=0xD7 => {
                i += 2;
                continue;
            }
            _ => {}
        }
        let seg_len = u16::from_be_bytes([jpeg[i + 2], jpeg[i + 3]]) as usize;
        let start = i + 4;
        let end = i + 2 + seg_len;
        if seg_len < 2 || end > jpeg.len() {
            return false;
        }
        if marker == 0xE1
            && jpeg[start..end].starts_with(b"Exif\0\0")
            && patch_tiff_orientation(&mut jpeg[start + 6..end])
        {
            return true;
        }
        i = end;
    }
    false
}

/// Rewrite IFD0's Orientation (0x0112) entry in a TIFF-structured EXIF block.
fn patch_tiff_orientation(tiff: &mut [u8]) -> bool {
    let little = match tiff.get(0..2) {
        Some(b"II") => true,
        Some(b"MM") => false,
        _ => return false,
    };
    let read_u16 = |b: &[u8], at: usize| -> Option<u16> {
        let s: [u8; 2] = b.get(at..at + 2)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(s)
        } else {
            u16::from_be_bytes(s)
        })
    };
    let read_u32 = |b: &[u8], at: usize| -> Option<u32> {
        let s: [u8; 4] = b.get(at..at + 4)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(s)
        } else {
            u32::from_be_bytes(s)
        })
    };

    let Some(ifd0) = read_u32(tiff, 4).map(|o| o as usize) else {
        return false;
    };
    let Some(count) = read_u16(tiff, ifd0) else {
        return false;
    };
    for k in 0..count as usize {
        let entry = ifd0 + 2 + k * 12;
        let (Some(tag), Some(kind)) = (read_u16(tiff, entry), read_u16(tiff, entry + 2)) else {
            return false;
        };
        // SHORT, count 1: the value lives in the first two bytes of the
        // 4-byte value field.
        if tag == 0x0112 && kind == 3 {
            let value = if little {
                1u16.to_le_bytes()
            } else {
                1u16.to_be_bytes()
            };
            if let Some(slot) = tiff.get_mut(entry + 8..entry + 10) {
                slot.copy_from_slice(&value);
                return true;
            }
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_path_keeps_writable_extensions() {
        assert_eq!(
            edited_output_path(Path::new("dir/photo.JPG")),
            Path::new("dir/photo_edited.JPG")
        );
        assert_eq!(
            edited_output_path(Path::new("a.webp")),
            Path::new("a_edited.webp")
        );
        assert_eq!(
            edited_output_path(Path::new("a.png")),
            Path::new("a_edited.png")
        );
    }

    #[test]
    fn edited_path_falls_back_to_png_for_formats_without_an_encoder() {
        for src in ["a.heic", "a.cr2", "a.svg", "a.tiff", "a.gif", "a.jxl", "a"] {
            let out = edited_output_path(Path::new(src));
            assert_eq!(
                out.extension().and_then(|e| e.to_str()),
                Some("png"),
                "{src} -> {out:?}"
            );
        }
    }

    #[test]
    fn unwritable_output_extension_is_a_clear_error() {
        let err = save(Path::new("out.heic"), &[0; 4], 1, 1).unwrap_err();
        assert!(err.to_string().contains(".heic"), "{err}");
    }

    #[test]
    fn flatten_onto_white_composites_alpha() {
        let rgba = [0, 0, 0, 0, 10, 20, 30, 255, 0, 0, 0, 128];
        let rgb = flatten_onto_white(&rgba);
        assert_eq!(&rgb[0..3], &[255, 255, 255], "fully transparent -> white");
        assert_eq!(&rgb[3..6], &[10, 20, 30], "opaque untouched");
        assert!(
            (126..=128).contains(&rgb[6]),
            "half alpha over white ~ mid grey"
        );
    }

    #[test]
    fn d4_quarter_turns_compose() {
        assert_eq!(rotation(4), IDENTITY);
        assert_eq!(mul(ROT90, rotation(3)), IDENTITY);
        assert_eq!(mul(MIRROR_H, MIRROR_H), IDENTITY);
    }

    #[test]
    fn exif_orientations_map_to_the_documented_jpegtran_ops() {
        use libjpeg_turbo_rs::TransformOp;
        for o in 1..=8u8 {
            assert_eq!(
                transform_op(exif_matrix(o as u32)),
                TransformOp::from_exif_orientation(o).unwrap(),
                "orientation {o}"
            );
        }
    }

    #[test]
    fn quarter_turns_rejects_free_angles() {
        assert_eq!(quarter_turns(0.0), Some(0));
        assert_eq!(quarter_turns(std::f32::consts::FRAC_PI_2), Some(1));
        assert_eq!(quarter_turns(std::f32::consts::PI * 2.5), Some(1));
        assert_eq!(quarter_turns(30f32.to_radians()), None);
    }

    fn tiff_with_orientation(little: bool, value: u16) -> Vec<u8> {
        let u16b = |v: u16| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let u32b = |v: u32| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let mut t = Vec::new();
        t.extend_from_slice(if little { b"II" } else { b"MM" });
        t.extend_from_slice(&u16b(42));
        t.extend_from_slice(&u32b(8));
        t.extend_from_slice(&u16b(2));
        // An unrelated tag first, so the scan has to walk the IFD.
        t.extend_from_slice(&u16b(0x010F));
        t.extend_from_slice(&u16b(2));
        t.extend_from_slice(&u32b(4));
        t.extend_from_slice(b"Cam\0");
        t.extend_from_slice(&u16b(0x0112));
        t.extend_from_slice(&u16b(3));
        t.extend_from_slice(&u32b(1));
        t.extend_from_slice(&u16b(value));
        t.extend_from_slice(&[0, 0]);
        t.extend_from_slice(&u32b(0));
        t
    }

    #[test]
    fn reset_exif_orientation_rewrites_both_byte_orders() {
        for little in [true, false] {
            let tiff = tiff_with_orientation(little, 6);
            let mut app1 = b"Exif\0\0".to_vec();
            app1.extend_from_slice(&tiff);
            let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
            jpeg.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
            jpeg.extend_from_slice(&app1);
            jpeg.extend_from_slice(&[0xFF, 0xD9]);

            assert!(reset_exif_orientation(&mut jpeg), "little={little}");
            let meta = crate::image::parse_exif_block(&jpeg[6..6 + app1.len()]).unwrap();
            assert_eq!(meta.orientation, Some(1), "little={little}");
        }
    }

    #[test]
    fn reset_exif_orientation_ignores_garbage() {
        assert!(!reset_exif_orientation(&mut []));
        assert!(!reset_exif_orientation(&mut [0xFF, 0xD8, 0x00, 0x00]));
        assert!(!reset_exif_orientation(&mut [
            0xFF, 0xD8, 0xFF, 0xE1, 0xFF, 0xFF
        ]));
    }
}
