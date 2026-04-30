use std::env;

fn main() {
    println!(
        "cargo:rustc-env=BUILD_TARGET_ARCH={}",
        env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default()
    );
}
