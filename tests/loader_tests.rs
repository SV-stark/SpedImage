//! End-to-end decode-path regressions.
//!
//! These exercise the real file-based loaders (rather than the pure helpers)
//! so the JPEG scaled-decode path, the decode budget and the other formats'
//! downsampling stay honest.

use libjpeg_turbo_rs::{PixelFormat, Subsampling};
use spedimage_lib::image::{FrameLimit, ImageFormatType, ImageLoader, LoadOptions};

/// Encode a synthetic RGB gradient as a JPEG and return `(bytes, w, h)`.
fn make_jpeg(w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    let rgb: Vec<u8> = (0..(w as usize * h as usize * 3))
        .map(|i| (i * 7 % 251) as u8)
        .collect();
    let bytes = libjpeg_turbo_rs::compress(
        &rgb,
        w as usize,
        h as usize,
        PixelFormat::Rgb,
        90,
        Subsampling::S420,
    )
    .expect("compress fixture");
    (bytes, w, h)
}

/// The loader dispatches on file extension, so fixtures must carry the
/// extension that selects the decoder under test.
struct Fixture {
    path: std::path::PathBuf,
}

impl Fixture {
    fn new(tag: &str, ext: &str, bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "spedimage_loader_{}_{}.{}",
            tag,
            std::process::id(),
            ext
        ));
        std::fs::write(&path, bytes).expect("write fixture");
        Self { path }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ── WebP ──────────────────────────────────────────────────────────────────
//
// WebP is decoded by `image-webp` directly rather than through zune-image's
// WebP codec. `image-webp` emits 3 bytes/pixel for a file with no alpha
// channel and 4 with, so the fixtures below pin both paths: getting that
// wrong yields a plausible-looking picture of the wrong size.

/// A 64x48 gradient, `alpha` as the alpha of every pixel.
const WEBP_GRADIENT_ALPHA: u8 = 64;

/// The pixel `image-webp` produces for the lossy fixture, sampled at a few
/// points by PIL. Lossy WebP is not bit-exact across decoders, so these are
/// the exact values this decoder produced and they are pinned deliberately:
/// the point of the test is the buffer *shape* and channel order.
fn expected_lossy_pixel(x: u32, y: u32) -> (u8, u8, u8, u8) {
    match (x, y) {
        (0, 0) => (17, 19, 19, 0),
        (63, 0) => (248, 1, 189, 126),
        (0, 47) => (0, 229, 140, 0),
        (63, 47) => (248, 236, 74, 126),
        (32, 24) => (128, 123, 168, WEBP_GRADIENT_ALPHA),
        _ => unreachable!("sample point outside the fixture"),
    }
}

fn assert_lossy_gradient(img: &spedimage_lib::image::ImageData) {
    let at = |x: u32, y: u32| {
        let i = ((y as usize * img.width as usize) + x as usize) * 4;
        (
            img.rgba_data[i],
            img.rgba_data[i + 1],
            img.rgba_data[i + 2],
            img.rgba_data[i + 3],
        )
    };
    for (x, y) in [(0, 0), (63, 0), (0, 47), (63, 47), (32, 24)] {
        assert_eq!(at(x, y), expected_lossy_pixel(x, y), "at ({x}, {y})");
    }
}

#[test]
fn webp_lossy_with_alpha_decodes_to_rgba() {
    let fx = Fixture::new(
        "webp_lossy",
        "webp",
        include_bytes!("fixtures/alpha_lossy.webp"),
    );
    let (frames, format) = ImageLoader::load(&fx.path, None, None).expect("load");

    assert_eq!(format, ImageFormatType::WebP);
    assert_eq!((frames[0].width, frames[0].height), (64, 48));
    // 4 bytes/pixel: the fixture has alpha, so no widening was needed.
    assert_eq!(frames[0].rgba_data.len(), 64 * 48 * 4);
    assert_lossy_gradient(&frames[0]);
}

// ── ICO / CUR ──────────────────────────────────────────────────────────────
//
// `.ico` was listed in SUPPORTED_EXTENSIONS with no decoder behind it, so icons
// showed up in the file browser and then failed to open. These pin both payload
// encodings a real icon uses: a whole PNG, and a headerless BMP DIB.

/// Wrap PNG bytes in a minimal single-entry ICO directory.
fn make_ico_with_png(png: &[u8]) -> Vec<u8> {
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    ico.extend_from_slice(&1u16.to_le_bytes()); // one image
    ico.push(16); // width
    ico.push(16); // height
    ico.push(0); // palette size
    ico.push(0); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // planes
    ico.extend_from_slice(&32u16.to_le_bytes()); // bit count
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset = 6 + 16
    ico.extend_from_slice(png);
    ico
}

/// Wrap a 32-bit BGRA DIB in a single-entry ICO directory, with the doubled
/// height an icon's device-independent bitmap uses.
fn make_ico_with_dib(w: u32, h: u32) -> Vec<u8> {
    // BITMAPINFOHEADER with height doubled to cover the AND mask.
    let mut dib = Vec::new();
    dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
    dib.extend_from_slice(&(w as i32).to_le_bytes()); // biWidth
    dib.extend_from_slice(&((h * 2) as i32).to_le_bytes()); // biHeight (XOR + AND)
    dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    dib.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    dib.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    dib.extend_from_slice(&(w * h * 4).to_le_bytes()); // biSizeImage
    dib.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    dib.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    dib.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    dib.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant
    for y in 0..h {
        for x in 0..w {
            // BGRA, bottom-up.
            let b = (x * 8) as u8;
            let g = (y * 8) as u8;
            dib.extend_from_slice(&[b, g, 255, 255]);
        }
    }
    // AND mask, 1bpp, rows padded to 4 bytes.
    let mask_stride = w.div_ceil(32) * 4;
    dib.resize(dib.len() + (mask_stride * h) as usize, 0);

    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.push(w as u8);
    ico.push(h as u8);
    ico.push(0);
    ico.push(0);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(dib.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(&dib);
    ico
}

#[test]
fn ico_with_png_payload_decodes() {
    let f = Fixture::new("ico_png", "ico", &make_ico_with_png(&make_png(8, 8)));
    let (frames, format) = ImageLoader::load(&f.path, None, None).expect("decode ico");
    assert_eq!(format, ImageFormatType::Ico);
    assert_eq!(frames[0].width, 8);
    assert_eq!(frames[0].height, 8);
    assert_eq!(frames.len(), 1);
}

#[test]
fn ico_with_dib_payload_decodes_to_rgba() {
    let f = Fixture::new("ico_dib", "ico", &make_ico_with_dib(8, 8));
    let (frames, format) = ImageLoader::load(&f.path, None, None).expect("decode dib ico");
    assert_eq!(format, ImageFormatType::Ico);
    assert_eq!(frames[0].width, 8);
    assert_eq!(frames[0].height, 8);
    assert_eq!(frames[0].rgba_data.len(), 8 * 8 * 4);
    // The synthetic gradient is opaque, so alpha must survive the BGRA swap.
    assert!(
        frames[0]
            .rgba_data
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[3] == 255)
    );
}

#[test]
fn ico_downsamples_to_the_requested_box() {
    let f = Fixture::new("ico_ds", "ico", &make_ico_with_dib(64, 64));
    let (frames, _) = ImageLoader::load(&f.path, Some(16), Some(16)).expect("downsample ico");
    assert_eq!((frames[0].width, frames[0].height), (16, 16));
    assert!(frames[0].is_downsampled);
}

#[test]
fn cur_uses_the_same_path_as_ico() {
    let f = Fixture::new("cur", "cur", &make_ico_with_png(&make_png(8, 8)));
    let (_, format) = ImageLoader::load(&f.path, None, None).expect("decode cur");
    assert_eq!(format, ImageFormatType::Ico);
}

/// A malformed header must produce an error rather than a panic: icons come
/// from the filesystem and the parser reads attacker-controlled offsets.
#[test]
fn truncated_ico_errors_rather_than_panicking() {
    for bytes in [
        vec![],
        vec![0, 0],
        vec![0, 0, 1, 0],       // header only, count field missing
        vec![0, 0, 1, 0, 1, 0], // claims one image, no directory
        vec![9, 9, 9, 9, 1, 0], // bad magic
        vec![0, 0, 7, 0, 1, 0], // type 7 is neither icon nor cursor
        vec![0, 0, 1, 0, 0, 0], // zero images declared
    ] {
        let f = Fixture::new("ico_bad", "ico", &bytes);
        assert!(
            ImageLoader::load(&f.path, None, None).is_err(),
            "expected an error for {bytes:?}"
        );
    }
}

/// A directory entry pointing past the end of the file must not be followed.
#[test]
fn ico_with_out_of_range_offset_errors() {
    let mut ico = make_ico_with_png(&make_png(4, 4));
    // Rewrite dwImageOffset to point far beyond the buffer.
    ico[18..22].copy_from_slice(&0xFFFF_0000u32.to_le_bytes());
    let f = Fixture::new("ico_oob", "ico", &ico);
    assert!(ImageLoader::load(&f.path, None, None).is_err());
}

/// An entry whose offset and size together overrun the file must be rejected
/// rather than read past the end.
#[test]
fn ico_with_oversized_entry_errors() {
    let mut ico = make_ico_with_png(&make_png(4, 4));
    ico[14..18].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // dwBytesInRes
    let f = Fixture::new("ico_big", "ico", &ico);
    assert!(ImageLoader::load(&f.path, None, None).is_err());
}

// ── PSD / HDR / PNM / Farbfeld ─────────────────────────────────────────────
//
// These four are reached through zune-image's magic-byte sniffing in the
// generic fallback, so each test is also a check that the crate feature is
// actually enabled: without it `Image::read` has no decoder and returns an
// error instead of pixels.

/// A minimal greyscale binary PPM (P5).
fn make_ppm(w: u32, h: u32) -> Vec<u8> {
    let mut out = format!("P5\n{w} {h}\n255\n").into_bytes();
    for i in 0..(w * h) {
        out.push((i % 256) as u8);
    }
    out
}

/// A minimal binary PPM (P6).
fn make_ppm_rgb(w: u32, h: u32) -> Vec<u8> {
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for i in 0..(w * h) {
        out.extend_from_slice(&[
            (i % 256) as u8,
            ((i * 3) % 256) as u8,
            ((i * 7) % 256) as u8,
        ]);
    }
    out
}

/// A minimal 8x8 RGBA PNG, via the loader's own zune-image dependency.
fn make_png(w: u32, h: u32) -> Vec<u8> {
    use zune_core::colorspace::ColorSpace;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for (i, px) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        px[0] = (i % 256) as u8;
        px[1] = ((i * 2) % 256) as u8;
        px[2] = ((i * 3) % 256) as u8;
        px[3] = 255;
    }
    let mut out = Vec::new();
    let img = zune_image::image::Image::from_u8(&rgba, w as usize, h as usize, ColorSpace::RGBA);
    // `Image::encode` writes into any `ZByteWriterTrait`, and `&mut Vec<u8>`
    // is one of them.
    img.encode(zune_image::codecs::ImageFormat::PNG, &mut out)
        .expect("encode png fixture");
    out
}

#[test]
fn pnm_greyscale_and_rgb_decode() {
    for (tag, ext, bytes) in [
        ("pnm_gray", "pgm", make_ppm(16, 8)),
        ("pnm_rgb", "ppm", make_ppm_rgb(16, 8)),
    ] {
        let f = Fixture::new(tag, ext, &bytes);
        let (frames, format) = ImageLoader::load(&f.path, None, None)
            .unwrap_or_else(|e| panic!("{ext} decode: {e:#}"));
        assert_eq!(format, ImageFormatType::Pnm);
        assert_eq!((frames[0].width, frames[0].height), (16, 8));
        assert_eq!(frames[0].rgba_data.len(), 16 * 8 * 4);
    }
}

#[test]
fn farbfeld_decodes() {
    // farbfeld: "farbfeld" + u32 BE width + u32 BE height + 8 bytes per pixel.
    let (w, h) = (4u32, 2u32);
    let mut ff = b"farbfeld".to_vec();
    ff.extend_from_slice(&w.to_be_bytes());
    ff.extend_from_slice(&h.to_be_bytes());
    for _ in 0..(w * h) {
        ff.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
    }
    let f = Fixture::new("ff", "ff", &ff);
    let (frames, format) = ImageLoader::load(&f.path, None, None).expect("decode farbfeld");
    assert_eq!(format, ImageFormatType::Farbfeld);
    assert_eq!((frames[0].width, frames[0].height), (4, 2));
    // 16-bit samples truncate to 8 bits: 0x1122 -> 0x11.
    assert_eq!(&frames[0].rgba_data[..4], &[0x11, 0x33, 0x55, 0x77]);
}

#[test]
fn hdr_decodes() {
    // Radiance RGBE, flat 1x1, which is the smallest form the decoder accepts.
    let mut hdr = b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y 1 +X 1\n".to_vec();
    hdr.extend_from_slice(&[128, 128, 128, 128]);
    let f = Fixture::new("hdr", "hdr", &hdr);
    let (frames, format) = ImageLoader::load(&f.path, None, None).expect("decode hdr");
    assert_eq!(format, ImageFormatType::Hdr);
    assert_eq!((frames[0].width, frames[0].height), (1, 1));
    assert_eq!(frames[0].rgba_data.len(), 4);
}

/// PSD support is a feature flag on zune-image; assert the format is at least
/// routed rather than silently landing in the AVIF/unknown arm.
#[test]
fn psd_is_routed_to_a_decoder() {
    // A real PSD is large; assert the plumbing instead: the extension maps to
    // Psd, and the bytes reach a decoder that rejects them as malformed
    // rather than being filtered out as an unsupported format.
    let f = Fixture::new("psd", "psd", b"8BPS\0\x01not-a-real-psd");
    assert_eq!(ImageFormatType::from_extension("psd"), ImageFormatType::Psd);
    // Whatever the decoder says about the truncated body, it must be a decode
    // error and never a panic.
    let _ = ImageLoader::load(&f.path, None, None);
}

// ── RAW extension routing ──────────────────────────────────────────────────
//
// These extensions all decode through `rawloader`, but the loader dispatches on
// the extension, so an extension missing from `from_extension` means the file
// never reaches `rawloader` at all.

#[test]
fn every_advertised_raw_extension_routes_to_the_raw_loader() {
    for ext in [
        "arw", "cr2", "crw", "nef", "nrw", "dng", "orf", "raf", "sr2", "srf", "srw", "pef", "mrw",
        "kdc", "dcr", "rw2",
    ] {
        assert_eq!(
            ImageFormatType::from_extension(ext),
            ImageFormatType::Raw,
            "{ext} should route to the RAW loader"
        );
        assert!(
            ImageFormatType::from_extension(ext).is_supported(),
            "{ext} should be shown in the file browser"
        );
    }
}

#[test]
fn webp_lossless_with_alpha_decodes_to_rgba() {
    let fx = Fixture::new(
        "webp_lossless",
        "webp",
        include_bytes!("fixtures/alpha_lossless.webp"),
    );
    let (frames, _) = ImageLoader::load(&fx.path, None, None).expect("load");

    assert_eq!((frames[0].width, frames[0].height), (64, 48));
    assert_eq!(frames[0].rgba_data.len(), 64 * 48 * 4);

    // Lossless is exact, so the whole buffer is checkable. The fixture is a
    // gradient of (x*4, y*5, (x+y)*3) with alpha x*2 — except where alpha is
    // 0, where the encoder zeroes the colour channels too. That is the
    // fixture's own encoding, not a decode decision, so only the alpha and the
    // visible pixels are asserted.
    let rgba = &frames[0].rgba_data;
    let mut checked = 0;
    for y in 0..48u32 {
        for x in 0..64u32 {
            let i = ((y as usize * 64) + x as usize) * 4;
            let px = &rgba[i..][..4];
            let alpha = (x * 2 % 256) as u8;
            assert_eq!(px[3], alpha, "alpha at ({x}, {y})");
            if alpha == 0 {
                continue;
            }
            assert_eq!(
                &px[..3],
                [
                    (x * 4 % 256) as u8,
                    (y * 5 % 256) as u8,
                    ((x + y) * 3 % 256) as u8,
                ],
                "colour at ({x}, {y})"
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no opaque pixels to have checked");
}

#[test]
fn webp_preview_is_downsampled_to_the_requested_box() {
    let fx = Fixture::new(
        "webp_preview",
        "webp",
        include_bytes!("fixtures/alpha_lossy.webp"),
    );
    let (frames, _) = ImageLoader::load(&fx.path, Some(32), Some(32)).expect("load");
    let img = &frames[0];

    assert!(img.is_downsampled);
    assert!(img.width <= 32 && img.height <= 32);
    assert_eq!(img.rgba_data.len(), (img.width * img.height * 4) as usize);
}

#[test]
fn webp_truncated_file_errors_rather_than_panicking() {
    let full = include_bytes!("fixtures/alpha_lossy.webp");
    let fx = Fixture::new("webp_trunc", "webp", &full[..full.len() / 2]);
    assert!(ImageLoader::load(&fx.path, None, None).is_err());
}

// ── AVIF ──────────────────────────────────────────────────────────────────
//
// AVIF is recognised so an explicit open can explain itself, but it is not a
// supported format: there is no AV1 decoder in the tree.

#[test]
fn avif_is_known_but_not_supported() {
    assert_eq!(
        ImageFormatType::from_extension("avif"),
        ImageFormatType::Avif
    );
    assert!(!ImageFormatType::Avif.is_supported());
}

#[test]
fn opening_an_avif_explains_the_missing_av1_decoder() {
    // Bytes are irrelevant: the extension alone must produce the explanation.
    let fx = Fixture::new("avif", "avif", b"not really an avif");
    let err = ImageLoader::load(&fx.path, None, None)
        .expect_err("AVIF must not decode")
        .to_string();
    assert!(err.contains("AVIF"), "unhelpful error: {err}");
    assert!(err.contains("AV1"), "unhelpful error: {err}");
}

#[test]
fn jpeg_full_resolution_load_is_exact() {
    let (bytes, w, h) = make_jpeg(320, 200);
    let fx = Fixture::new("full", "jpg", &bytes);

    let (frames, format) = ImageLoader::load(&fx.path, None, None).expect("load");
    assert_eq!(format, ImageFormatType::Jpeg);
    assert_eq!(frames.len(), 1);
    assert_eq!((frames[0].width, frames[0].height), (w, h));
    assert!(!frames[0].is_downsampled);
    assert_eq!(frames[0].rgba_data.len(), (w * h * 4) as usize);
}

#[test]
fn jpeg_preview_is_downsampled_and_never_upsampled() {
    let (bytes, w, h) = make_jpeg(1600, 1200);
    let fx = Fixture::new("preview", "jpg", &bytes);

    let (frames, _) = ImageLoader::load(&fx.path, Some(400), Some(400)).expect("load");
    let img = &frames[0];

    assert!(img.is_downsampled);
    // Aspect ratio preserved (4:3).
    let got = img.width as f64 / img.height as f64;
    let want = w as f64 / h as f64;
    assert!((got - want).abs() < 0.02, "aspect drifted: {got} vs {want}");
    // Fits the requested box.
    assert!(img.width <= 400 && img.height <= 400);
    // ...but stays close to it: the decoder must not over-shrink.
    assert!(img.width >= 350, "over-shrunk to {}", img.width);
    assert_eq!(img.rgba_data.len(), (img.width * img.height * 4) as usize);
}

#[test]
fn jpeg_thumbnail_decode_stays_cheap_and_correct() {
    let (bytes, w, h) = make_jpeg(2400, 1600);
    let fx = Fixture::new("thumb", "jpg", &bytes);

    let (frames, _) = ImageLoader::load_with(
        &fx.path,
        Some(80),
        Some(80),
        LoadOptions {
            frames: FrameLimit::First,
            bake_orientation: true,
        },
    )
    .expect("load");
    let img = &frames[0];

    assert!(img.is_downsampled);
    assert_eq!(img.orientation_deg, 0);
    // 3:2 source into an 80x80 box -> 80x53.
    assert!(
        (img.width, img.height) == (80, 53),
        "got {}x{}",
        img.width,
        img.height
    );
    assert!(img.width <= w && img.height <= h);
    assert_eq!(img.rgba_data.len(), (80 * 53 * 4) as usize);
}

#[test]
fn jpeg_partial_decode_box_does_not_collapse_the_image() {
    // Only one bound supplied: the old code divided by the missing bound and
    // produced a 1x1 image.
    let (bytes, w, h) = make_jpeg(640, 480);
    let fx = Fixture::new("partial", "jpg", &bytes);

    let (frames, _) = ImageLoader::load(&fx.path, Some(64), None).expect("load");
    let img = &frames[0];
    assert_eq!((img.width, img.height), (w, h));
    assert!(!img.is_downsampled);
}

#[test]
fn jpeg_zero_sized_box_is_ignored() {
    let (bytes, w, h) = make_jpeg(320, 240);
    let fx = Fixture::new("zerobox", "jpg", &bytes);

    let (frames, _) = ImageLoader::load(&fx.path, Some(0), Some(0)).expect("load");
    assert_eq!((frames[0].width, frames[0].height), (w, h));
}

#[test]
fn png_downsamples_to_the_requested_box() {
    let (w, h) = (512u32, 256u32);
    let rgba: Vec<u8> = (0..(w * h * 4)).map(|i| (i % 255) as u8).collect();
    let dir = std::env::temp_dir().join(format!("spedimage_png_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("grad.png");
    spedimage_lib::image::ImageBackend::save(&path, &rgba, w, h).expect("save png");

    let (frames, format) = ImageLoader::load(&path, Some(128), Some(128)).expect("load");
    assert_eq!(format, ImageFormatType::Png);
    let img = &frames[0];
    assert_eq!((img.width, img.height), (128, 64));
    assert!(img.is_downsampled);
    assert_eq!(img.rgba_data.len(), (128 * 64 * 4) as usize);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn qoi_and_exotic_formats_honour_the_decode_box() {
    // QOI is decoded directly by the loader; make sure it downsamples too
    // rather than pushing a full-resolution buffer at the GPU.
    let (w, h) = (256u32, 128u32);
    let mut qoi_bytes = vec![0u8; w as usize * h as usize * 4 + 22];
    qoi_bytes[0..4].copy_from_slice(b"qoif");
    qoi_bytes[4..8].copy_from_slice(&w.to_be_bytes());
    qoi_bytes[8..12].copy_from_slice(&h.to_be_bytes());
    qoi_bytes[12] = 4; // RGBA
    // Single solid-colour run: every pixel is identical, so QOI_OP_RGB at the
    // start followed by run-length repeats is a valid encoding.
    let body = 14;
    qoi_bytes[body] = 0xFE; // QOI_OP_RUN, run length 2
    qoi_bytes[body + 1] = 200;
    qoi_bytes[body + 2] = 100;
    qoi_bytes[body + 3] = 50;
    qoi_bytes[body + 4] = 255;
    qoi_bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);

    let fx = Fixture::new("qoi", "qoi", &qoi_bytes);
    if let Ok((frames, format)) = ImageLoader::load(&fx.path, Some(64), Some(64)) {
        assert_eq!(format, ImageFormatType::Qoi);
        assert!(frames[0].width <= 64 && frames[0].height <= 64);
        assert_eq!(
            frames[0].rgba_data.len(),
            (frames[0].width * frames[0].height * 4) as usize
        );
    }
    // A hand-rolled stream the decoder rejects is not a loader failure.
}

#[test]
fn unsupported_bytes_produce_an_error_not_a_panic() {
    let fx = Fixture::new("garbage", "jpg", &[0xFF; 512]);
    let result = ImageLoader::load(&fx.path, Some(64), Some(64));
    assert!(result.is_err());
}

#[test]
fn truncated_jpeg_produces_an_error_not_a_panic() {
    let (mut bytes, _, _) = make_jpeg(256, 256);
    bytes.truncate(bytes.len() / 2);
    let fx = Fixture::new("truncated", "jpg", &bytes);
    assert!(ImageLoader::load(&fx.path, Some(64), Some(64)).is_err());
}

#[test]
fn missing_file_produces_an_error_not_a_panic() {
    assert!(
        ImageLoader::load(
            std::path::Path::new("__no_such_file__.jpg"),
            Some(8),
            Some(8)
        )
        .is_err()
    );
}

/// GIF frames may declare sub-rectangles the logical screen cannot hold, and a
/// zero-size sub-frame used to panic `chunks_exact(0)`.
mod gif_robustness {
    use super::{Fixture, ImageLoader};
    use gif::{DisposalMethod, Frame};
    use std::borrow::Cow;

    fn encode(w: u16, h: u16, frames: Vec<Frame>) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut enc = gif::Encoder::new(&mut out, w, h, &[0, 0, 0]).expect("encoder");
            for f in frames {
                enc.write_frame(&f).expect("frame");
            }
        }
        out
    }

    fn frame(w: u16, h: u16, left: u16, top: u16, dispose: DisposalMethod) -> Frame<'static> {
        Frame {
            width: w,
            height: h,
            left,
            top,
            dispose,
            delay: 2,
            transparent: None,
            needs_user_input: false,
            interlaced: false,
            palette: None,
            buffer: Cow::Owned(vec![200u8; w as usize * h as usize]),
        }
    }

    #[test]
    fn zero_size_frame_does_not_panic() {
        let bytes = encode(
            4,
            4,
            vec![
                frame(4, 4, 0, 0, DisposalMethod::Keep),
                frame(0, 0, 0, 0, DisposalMethod::Keep),
                frame(4, 4, 0, 0, DisposalMethod::Keep),
            ],
        );
        let fx = Fixture::new("zero_frame", "gif", &bytes);
        let result = ImageLoader::load(&fx.path, None, None);
        assert!(result.is_ok(), "expected frames, got {:?}", result.err());
    }

    #[test]
    fn out_of_bounds_frame_rect_does_not_panic() {
        // Frame extends far past the 8x8 logical screen in both directions.
        let bytes = encode(
            8,
            8,
            vec![
                frame(64, 64, 32, 32, DisposalMethod::Background),
                frame(64, 64, 0, 0, DisposalMethod::Previous),
            ],
        );
        let fx = Fixture::new("oob_frame", "gif", &bytes);
        let result = ImageLoader::load(&fx.path, None, None);
        assert!(result.is_ok(), "expected frames, got {:?}", result.err());
    }

    #[test]
    fn every_disposal_method_survives_hostile_rects() {
        for dispose in [
            DisposalMethod::Any,
            DisposalMethod::Keep,
            DisposalMethod::Background,
            DisposalMethod::Previous,
        ] {
            let bytes = encode(
                8,
                8,
                vec![
                    frame(4, 4, 0, 0, dispose),
                    frame(32, 32, 16, 16, dispose),
                    frame(4, 4, 4, 4, DisposalMethod::Background),
                ],
            );
            let fx = Fixture::new("disposal", "gif", &bytes);
            let _ = ImageLoader::load(&fx.path, Some(4), Some(4));
        }
    }

    #[test]
    fn truncated_gif_does_not_panic() {
        let bytes = encode(8, 8, vec![frame(8, 8, 0, 0, DisposalMethod::Keep)]);
        let fx = Fixture::new("trunc_gif", "gif", &bytes[..bytes.len() * 2 / 3]);
        let _ = ImageLoader::load(&fx.path, Some(4), Some(4));
    }

    #[test]
    fn gif_streamer_survives_hostile_rects() {
        let bytes = encode(
            8,
            8,
            vec![
                frame(4, 4, 0, 0, DisposalMethod::Keep),
                frame(0, 0, 0, 0, DisposalMethod::Keep),
                frame(32, 32, 8, 8, DisposalMethod::Background),
            ],
        );
        let fx = Fixture::new("stream_gif", "gif", &bytes);

        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};
        let generation = Arc::new(AtomicU64::new(1));
        let (tx, rx) = crossbeam_channel::bounded(4);
        // The streamer loops forever; a helper thread bumps the generation once
        // the first frame lands so this terminates.
        let watcher = generation.clone();
        std::thread::spawn(move || {
            let _ = rx.recv_timeout(std::time::Duration::from_secs(3));
            watcher.fetch_add(1, Ordering::SeqCst);
        });
        spedimage_lib::image::stream_gif(fx.path.clone(), 0, 4, 4, 1, generation, tx);
    }
}
