use divan::black_box;
use spedimage_lib::image::ImageData;

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_histogram_1080p(bencher: divan::Bencher) {
    // Create a synthetic 1920x1080 RGBA buffer
    let rgba = vec![128u8; 1920 * 1080 * 4];
    bencher.bench_local(move |b| {
        b.iter(|| {
            let mut r = [0u32; 256];
            let mut g = [0u32; 256];
            let mut b = [0u32; 256];
            ImageData::compute_rgb_histogram(black_box(&rgba), &mut r, &mut g, &mut b);
        });
    });
}
