//! Save-path regressions.
//!
//! `ImageBackend::save` used to hand every file to zune-image's `save`, which
//! picks an encoder by extension. Only the `png` encoder feature is enabled,
//! so Ctrl+S on a JPEG, and the JPEG/WebP entries of Save As, all failed. The
//! only save test wrote a PNG, so nothing noticed. These tests save through
//! every format the dialog offers and decode the result again.

use libjpeg_turbo_rs::{PixelFormat, Subsampling};
use spedimage_lib::image::{
    ImageData, ImageFormatType, ImageLoader, ImageProcessor, SaveFormat, edited_output_path,
    read_exif_meta, save_edited,
};
use spedimage_lib::render::ImageAdjustments;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A scratch directory removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("spedimage_save_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        Self(dir)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An asymmetric gradient: red follows x, green follows y, blue is constant.
/// Any transform that swaps or mirrors an axis changes it, so a wrong
/// rotation/flip mapping cannot pass by accident.
fn gradient_rgba(w: u32, h: u32, alpha: impl Fn(u32, u32) -> u8) -> Vec<u8> {
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            out.push((x * 255 / (w - 1).max(1)) as u8);
            out.push((y * 255 / (h - 1).max(1)) as u8);
            out.push(96);
            out.push(alpha(x, y));
        }
    }
    out
}

fn load_full(path: &Path) -> ImageData {
    let (frames, _) = ImageLoader::load(path, None, None).expect("decode saved file");
    frames.into_iter().next().expect("one frame")
}

/// Mean and max absolute difference over the RGB channels.
fn rgb_diff(a: &[u8], b: &[u8]) -> (f64, u8) {
    assert_eq!(a.len(), b.len(), "buffer sizes differ");
    let mut sum = 0u64;
    let mut max = 0u8;
    let mut n = 0u64;
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        for c in 0..3 {
            let d = pa[c].abs_diff(pb[c]);
            sum += d as u64;
            max = max.max(d);
            n += 1;
        }
    }
    (sum as f64 / n as f64, max)
}

#[test]
fn every_offered_format_saves_and_decodes_back() {
    let dir = TempDir::new("formats");
    let (w, h) = (40u32, 24u32);
    let rgba = gradient_rgba(w, h, |_, _| 255);

    for name in ["out.png", "out.jpg", "out.jpeg", "out.JPG", "out.webp"] {
        let path = dir.join(name);
        spedimage_lib::image::ImageBackend::save(&path, &rgba, w, h)
            .unwrap_or_else(|e| panic!("saving {name} failed: {e}"));
        let img = load_full(&path);
        assert_eq!((img.width, img.height), (w, h), "{name}");

        let (mean, _) = rgb_diff(&img.rgba_data, &rgba);
        if SaveFormat::from_path(&path) == Some(SaveFormat::Jpeg) {
            assert!(mean < 3.0, "{name}: JPEG drifted too far, mean diff {mean}");
        } else {
            // PNG and VP8L WebP are lossless.
            assert_eq!(img.rgba_data.as_slice(), rgba.as_slice(), "{name}");
        }
    }
}

#[test]
fn lossless_formats_keep_alpha_and_jpeg_flattens_it_onto_white() {
    let dir = TempDir::new("alpha");
    let (w, h) = (16u32, 16u32);
    // Left half fully transparent, right half opaque.
    let rgba = gradient_rgba(w, h, |x, _| if x < 8 { 0 } else { 255 });

    for name in ["a.png", "a.webp"] {
        let path = dir.join(name);
        spedimage_lib::image::ImageBackend::save(&path, &rgba, w, h).expect(name);
        let img = load_full(&path);
        assert_eq!(img.rgba_data.as_slice(), rgba.as_slice(), "{name}");
    }

    let path = dir.join("a.jpg");
    spedimage_lib::image::ImageBackend::save(&path, &rgba, w, h).expect("jpg");
    let img = load_full(&path);
    // A transparent pixel well inside the left half.
    let i = (8 * w as usize + 2) * 4;
    let px = &img.rgba_data[i..i + 3];
    assert!(
        px.iter().all(|&c| c > 240),
        "transparent area should be white, got {px:?}"
    );
}

#[test]
fn saving_to_an_unwritable_extension_explains_itself() {
    let dir = TempDir::new("badext");
    let err = spedimage_lib::image::ImageBackend::save(&dir.join("x.heic"), &[0; 16], 2, 2)
        .unwrap_err()
        .to_string();
    assert!(err.contains(".heic") && err.contains(".png"), "{err}");
}

#[test]
fn ctrl_s_on_a_format_without_an_encoder_writes_png() {
    let dir = TempDir::new("ppm");
    // Netpbm decodes but cannot be written.
    let (w, h) = (6u32, 4u32);
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in 0..h {
        for x in 0..w {
            ppm.extend_from_slice(&[(x * 40) as u8, (y * 60) as u8, 7]);
        }
    }
    let source = dir.join("scan.ppm");
    std::fs::write(&source, &ppm).expect("write ppm");

    let output = edited_output_path(&source);
    assert_eq!(output.file_name().unwrap(), "scan_edited.png");
    let outcome = save_edited(&source, &output, &ImageAdjustments::default(), None).expect("save");
    assert!(!outcome.lossless);
    let img = load_full(&output);
    assert_eq!(img.format, ImageFormatType::Png);
    assert_eq!((img.width, img.height), (w, h));
}

// ── JPEG ────────────────────────────────────────────────────────────────────

/// A TIFF/EXIF block holding only an Orientation tag.
fn exif_with_orientation(orientation: u16) -> Vec<u8> {
    let mut t = b"II".to_vec();
    t.extend_from_slice(&42u16.to_le_bytes());
    t.extend_from_slice(&8u32.to_le_bytes());
    t.extend_from_slice(&1u16.to_le_bytes());
    t.extend_from_slice(&0x0112u16.to_le_bytes());
    t.extend_from_slice(&3u16.to_le_bytes());
    t.extend_from_slice(&1u32.to_le_bytes());
    t.extend_from_slice(&orientation.to_le_bytes());
    t.extend_from_slice(&[0, 0]);
    t.extend_from_slice(&0u32.to_le_bytes());
    t
}

fn make_jpeg(w: u32, h: u32, orientation: Option<u16>) -> Vec<u8> {
    let rgba = gradient_rgba(w, h, |_, _| 255);
    let rgb: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();
    let exif = orientation.map(exif_with_orientation);
    let mut enc = libjpeg_turbo_rs::Encoder::new(&rgb, w as usize, h as usize, PixelFormat::Rgb)
        .quality(95)
        .subsampling(Subsampling::S420);
    if let Some(ref e) = exif {
        enc = enc.exif_data(e);
    }
    enc.encode().expect("encode fixture")
}

fn adjustments(quarter_turns: u32, flip_h: bool, flip_v: bool) -> ImageAdjustments {
    ImageAdjustments {
        rotation: (quarter_turns as f32) * std::f32::consts::FRAC_PI_2,
        flip_horizontal: flip_h,
        flip_vertical: flip_v,
        ..ImageAdjustments::default()
    }
}

/// What the re-encode path would produce: decode upright, apply the edits.
fn expected_pixels(source: &Path, adj: &ImageAdjustments) -> (Vec<u8>, u32, u32) {
    let img = load_full(source);
    ImageProcessor::apply_adjustments_cpu(&img.rgba_data, img.width, img.height, adj)
}

#[test]
fn ctrl_s_on_a_jpeg_produces_a_jpeg() {
    let dir = TempDir::new("ctrl_s");
    let source = dir.join("photo.jpg");
    std::fs::write(&source, make_jpeg(64, 48, None)).expect("write");

    let output = edited_output_path(&source);
    assert_eq!(output.file_name().unwrap(), "photo_edited.jpg");
    let outcome = save_edited(&source, &output, &ImageAdjustments::default(), None).expect("save");
    assert_eq!(outcome.path, output);
    let img = load_full(&output);
    assert_eq!(img.format, ImageFormatType::Jpeg);
    assert_eq!((img.width, img.height), (64, 48));
}

#[test]
fn jpeg_rotations_and_flips_are_lossless_and_match_the_pixel_pipeline() {
    let dir = TempDir::new("lossless");
    // 64x48 is a whole number of 16x16 MCUs, so every transform is "perfect".
    for orientation in [
        None,
        Some(1),
        Some(3),
        Some(6),
        Some(8),
        Some(2),
        Some(5),
        Some(7),
    ] {
        let source = dir.join("src.jpg");
        std::fs::write(&source, make_jpeg(64, 48, orientation)).expect("write");

        for turns in 0..4 {
            for (fh, fv) in [(false, false), (true, false), (false, true)] {
                let adj = adjustments(turns, fh, fv);
                let output = dir.join("out.jpg");
                let outcome = save_edited(&source, &output, &adj, None).expect("save");
                let case = format!("orientation {orientation:?}, {turns}x90, flip h={fh} v={fv}");
                assert!(outcome.lossless, "{case}: expected a lossless save");

                // The tag was rewritten, so no viewer rotates it a second time.
                let meta = read_exif_meta(&output);
                assert!(
                    matches!(meta.orientation, None | Some(1)),
                    "{case}: orientation left at {:?}",
                    meta.orientation
                );

                let (want, ww, wh) = expected_pixels(&source, &adj);
                let got = load_full(&output);
                assert_eq!((got.width, got.height), (ww, wh), "{case}");
                let (mean, _) = rgb_diff(&got.rgba_data, &want);
                // A wrong mapping mirrors a full-range gradient: mean ~ 85+.
                assert!(mean < 4.0, "{case}: mean diff {mean}");
            }
        }
    }
}

#[test]
fn lossless_save_keeps_the_original_compressed_data_for_a_no_op() {
    let dir = TempDir::new("noop");
    let source = dir.join("src.jpg");
    std::fs::write(&source, make_jpeg(64, 48, None)).expect("write");
    let output = dir.join("out.jpg");
    save_edited(&source, &output, &ImageAdjustments::default(), None).expect("save");

    // Re-encoding at quality 92 would drift; a coefficient copy decodes to
    // exactly the same pixels.
    let a = load_full(&source);
    let b = load_full(&output);
    assert_eq!(a.rgba_data, b.rgba_data);
}

#[test]
fn unaligned_jpeg_rotation_falls_back_to_a_correct_reencode() {
    let dir = TempDir::new("unaligned");
    // 50x30 has partial edge MCUs, which a DCT-domain rotation cannot move.
    let source = dir.join("src.jpg");
    std::fs::write(&source, make_jpeg(50, 30, None)).expect("write");
    let adj = adjustments(1, false, false);
    let output = dir.join("out.jpg");
    let outcome = save_edited(&source, &output, &adj, None).expect("save");
    assert!(!outcome.lossless);

    let (want, ww, wh) = expected_pixels(&source, &adj);
    let got = load_full(&output);
    assert_eq!((got.width, got.height), (ww, wh));
    assert_eq!((ww, wh), (30, 50));
    let (mean, _) = rgb_diff(&got.rgba_data, &want);
    assert!(mean < 4.0, "mean diff {mean}");
}

#[test]
fn tonal_edits_and_crops_are_reencoded() {
    let dir = TempDir::new("tonal");
    let source = dir.join("src.jpg");
    std::fs::write(&source, make_jpeg(64, 48, None)).expect("write");
    let output = dir.join("out.jpg");

    let bright = ImageAdjustments {
        brightness: 1.2,
        ..ImageAdjustments::default()
    };
    assert!(
        !save_edited(&source, &output, &bright, None)
            .unwrap()
            .lossless
    );

    let cropped = ImageAdjustments {
        crop_rect_actual: Some([0.25, 0.25, 0.5, 0.5]),
        ..ImageAdjustments::default()
    };
    assert!(
        !save_edited(&source, &output, &cropped, None)
            .unwrap()
            .lossless
    );
    let img = load_full(&output);
    assert_eq!((img.width, img.height), (32, 24));
}

#[test]
fn a_full_resolution_buffer_with_deferred_rotation_is_saved_upright() {
    // The preview path leaves pure EXIF rotations to the GPU. When that buffer
    // was full resolution, saving it used to write the sideways pixels.
    let dir = TempDir::new("deferred");
    let (w, h) = (8u32, 4u32);
    let rgba = gradient_rgba(w, h, |_, _| 255);
    let source = dir.join("src.png");
    spedimage_lib::image::ImageBackend::save(&source, &rgba, w, h).expect("png");

    let displayed = ImageData {
        width: w,
        height: h,
        format: ImageFormatType::Png,
        rgba_data: Arc::new(rgba.clone()),
        path: source.clone(),
        file_size_bytes: 0,
        frame_delay_ms: 0,
        exif_info: None,
        histogram: None,
        exif_loaded: true,
        is_downsampled: false,
        orientation_deg: 90,
        gps_coords: None,
        color_space: None,
    };
    let output = dir.join("out.png");
    save_edited(
        &source,
        &output,
        &ImageAdjustments::default(),
        Some(&displayed),
    )
    .expect("save");
    let img = load_full(&output);
    assert_eq!((img.width, img.height), (h, w));
    let (rotated, _, _) = ImageProcessor::rotate_rgba(&rgba, w, h, 90);
    assert_eq!(img.rgba_data.as_slice(), rotated.as_slice());
}
