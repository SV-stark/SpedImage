use divan::black_box;
use fast_image_resize as fr;
use spedimage_lib::image::{ImageData, ImageProcessor};

fn main() {
    divan::main();
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

