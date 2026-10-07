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
}

impl SaveFormat {
    pub const ALL: [SaveFormat; 3] = [SaveFormat::Png, SaveFormat::Jpeg, SaveFormat::WebP];

    pub fn from_extension(ext: &str) -> Option<Self> {
        if ext.eq_ignore_ascii_case("png") {
            Some(Self::Png)
        } else if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
            Some(Self::Jpeg)
        } else if ext.eq_ignore_ascii_case("webp") {
            Some(Self::WebP)
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
        }
    }

    /// Every extension the save dialog should accept for this format.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Png => &["png"],
            Self::Jpeg => &["jpg", "jpeg"],
            Self::WebP => &["webp"],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG Image",
            Self::Jpeg => "JPEG Image",
            Self::WebP => "WebP Image (lossless)",
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

/// Encode an RGBA8 buffer into `format`.
pub fn encode(format: SaveFormat, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
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
            .quality(JPEG_QUALITY)
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
    }
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
        eyre!("Cannot save as {ext}: use a .png, .jpg or .webp file name")
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
    let format = format_for_output(output)?;

    if format == SaveFormat::Jpeg && lossless_eligible(source, adjustments) {
        match lossless_jpeg(source, adjustments) {
            Ok(Some(bytes)) => {
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
    let (edited, w, h) = ImageProcessor::apply_adjustments_cpu(&rgba, width, height, adjustments);
    let bytes = encode(format, &edited, w, h)?;
    write_atomically(output, &bytes)?;
    Ok(SaveOutcome {
        path: output.to_path_buf(),
        lossless: false,
    })
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
