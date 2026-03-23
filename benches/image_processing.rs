use divan::black_box;
use spedimage_lib::image::metadata::get_exif_orientation;

fn main() {
    divan::main();
}

#[divan::bench]
fn dummy_benchmark() {
    // A simple dummy benchmark for divan until real loads are added.
    let _ = black_box(1 + 1);
}
