use divan::black_box;
use fast_image_resize as fr;
use spedimage_lib::image::{ImageData, ImageLoader, ImageProcessor};

fn main() {
    divan::main();
}

/// Deterministic synthetic gradient: successive runs must measure the same
/// decode work, otherwise the numbers are not comparable.
fn gradient_rgba(w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; w as usize * h as usize * 4];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = (y * w as usize + x) * 4;
            buf[i] = (x * 255 / w as usize) as u8;
            buf[i + 1] = (y * 255 / h as usize) as u8;
            buf[i + 2] = ((x + y) % 256) as u8;
            buf[i + 3] = 255;
        }
    }
    buf
}

/// Encode a JPEG fixture into the temp dir and return its path.
fn jpeg_fixture(w: u32, h: u32) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("spedimage_bench_{w}x{h}.jpg"));
    if !path.exists() {
        let rgba = gradient_rgba(w, h);
        let rgb: Vec<u8> = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        let bytes = libjpeg_turbo_rs::compress(
            &rgb,
            w as usize,
            h as usize,
            libjpeg_turbo_rs::PixelFormat::Rgb,
            90,
            libjpeg_turbo_rs::Subsampling::S420,
        )
        .expect("encode bench fixture");
        std::fs::write(&path, bytes).expect("write bench fixture");
    }
    path
}

// --- decode path ---------------------------------------------------------
//
// `load_jpeg` picks the largest DCT scale that still leaves room for a
// downscale-only CPU pass, so these three should differ by roughly the
// square of the scale factor rather than all costing a full-size decode.

#[divan::bench]
fn bench_jpeg_full_decode_24mp(bencher: divan::Bencher) {
    let path = jpeg_fixture(6000, 4000);
    bencher.bench_local(|| black_box(ImageLoader::load(&path, None, None).unwrap()));
}

#[divan::bench]
fn bench_jpeg_preview_24mp_to_1080p(bencher: divan::Bencher) {
    let path = jpeg_fixture(6000, 4000);
    bencher.bench_local(|| black_box(ImageLoader::load(&path, Some(1920), Some(1080)).unwrap()));
}

#[divan::bench]
fn bench_jpeg_thumbnail_24mp_to_80px(bencher: divan::Bencher) {
    let path = jpeg_fixture(6000, 4000);
    bencher.bench_local(|| black_box(ImageLoader::load(&path, Some(80), Some(80)).unwrap()));
}

/// A 100-image thumbnail sweep is what opening a folder costs.
#[divan::bench]
fn bench_jpeg_thumbnail_sweep_100(bencher: divan::Bencher) {
    let path = jpeg_fixture(3000, 2000);
    bencher.bench_local(|| {
        for _ in 0..100 {
            black_box(ImageLoader::load(&path, Some(80), Some(80)).unwrap());
        }
    });
}

// --- pixel transforms ---------------------------------------------------

#[divan::bench]
fn bench_rotate_90_24mp(bencher: divan::Bencher) {
    let rgba = gradient_rgba(6000, 4000);
    bencher.bench_local(|| black_box(ImageProcessor::rotate_rgba(&rgba, 6000, 4000, 90)));
}

#[divan::bench]
fn bench_flip_24mp(bencher: divan::Bencher) {
    // `bench_local` wants an `Fn`, so the in-place mutation needs a `RefCell`.
    // Flipping twice restores the original buffer, keeping every iteration
    // identical.
    let rgba = std::cell::RefCell::new(gradient_rgba(6000, 4000));
    bencher.bench_local(|| {
        let mut buf = rgba.borrow_mut();
        ImageProcessor::flip_rgba(&mut buf, 6000, 4000, true, true);
        ImageProcessor::flip_rgba(&mut buf, 6000, 4000, true, true);
        // Sum a few bytes so the optimizer cannot drop the work, without
        // borrowing the buffer past the closure.
        black_box(buf[0])
    });
}

#[divan::bench]
fn bench_crop_24mp(bencher: divan::Bencher) {
    let rgba = gradient_rgba(6000, 4000);
    let rect = [0.1, 0.1, 0.5, 0.5];
    bencher.bench_local(|| black_box(ImageProcessor::crop_rgba(&rgba, 6000, 4000, rect)));
}

#[divan::bench]
fn bench_histogram_1080p(bencher: divan::Bencher) {
    let rgba = vec![128u8; 1920 * 1080 * 4];
    bencher.bench_local(move || {
        let mut r = [0u32; 256];
        let mut g = [0u32; 256];
        let mut b = [0u32; 256];
        ImageData::compute_rgb_histogram(black_box(&rgba), &mut r, &mut g, &mut b);
    });
}

#[divan::bench]
fn bench_histogram_4k(bencher: divan::Bencher) {
    let rgba = vec![128u8; 3840 * 2160 * 4];
    bencher.bench_local(move || {
        let mut r = [0u32; 256];
        let mut g = [0u32; 256];
        let mut b = [0u32; 256];
        ImageData::compute_rgb_histogram(black_box(&rgba), &mut r, &mut g, &mut b);
    });
}

#[divan::bench]
fn bench_simd_resize_4k_to_1080p(bencher: divan::Bencher) {
    let src_w = 3840u32;
    let src_h = 2160u32;
    let dst_w = 1920u32;
    let dst_h = 1080u32;
    let src_rgba = vec![128u8; (src_w * src_h * 4) as usize];
    let mut resizer = ImageProcessor::create_simd_resizer();

    bencher.bench_local(move || {
        let src_image =
            fr::images::ImageRef::new(src_w, src_h, black_box(&src_rgba), fr::PixelType::U8x4)
                .unwrap();
        let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
        resizer.resize(&src_image, &mut dst_image, None).unwrap();
    });
}

#[divan::bench]
fn bench_simd_resize_1080p_to_thumb(bencher: divan::Bencher) {
    let src_w = 1920u32;
    let src_h = 1080u32;
    let dst_w = 256u32;
    let dst_h = 144u32;
    let src_rgba = vec![128u8; (src_w * src_h * 4) as usize];
    let mut resizer = ImageProcessor::create_simd_resizer();

    bencher.bench_local(move || {
        let src_image =
            fr::images::ImageRef::new(src_w, src_h, black_box(&src_rgba), fr::PixelType::U8x4)
                .unwrap();
        let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
        resizer.resize(&src_image, &mut dst_image, None).unwrap();
    });
}
