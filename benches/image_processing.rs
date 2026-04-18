use divan::black_box;

fn main() {
    divan::main();
}

#[divan::bench]
fn dummy_benchmark() {
    // A simple dummy benchmark for divan until real loads are added.
    let _ = black_box(1 + 1);
}
