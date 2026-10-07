//! Decode one file and print what came out, so the HEIC/WebP swap can be
//! verified against a real file rather than only unit tests.
//!
//! Usage: cargo run --example verify_decode -- <path> [max_w max_h]

use spedimage_lib::image::ImageLoader;
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: verify_decode <path> [max_w max_h]");
        std::process::exit(2);
    };
    let max_w: Option<u32> = args.next().and_then(|s| s.parse().ok());
    let max_h: Option<u32> = args.next().and_then(|s| s.parse().ok());

    let path = PathBuf::from(path);
    let result = ImageLoader::load(&path, max_w, max_h);
    match result {
        Ok((frames, format)) => {
            let f = &frames[0];
            println!(
                "OK format={format:?} {}x{} rgba={} bytes={} downsampled={}",
                f.width,
                f.height,
                f.rgba_data.len(),
                f.file_size_bytes,
                f.is_downsampled
            );
            // A decoder that silently returns a black or blank frame is worse
            // than one that fails, so check the buffer is not all one value.
            let distinct = {
                let mut seen = [false; 256];
                for b in f.rgba_data.iter() {
                    seen[*b as usize] = true;
                }
                seen.iter().filter(|s| **s).count()
            };
            println!("distinct byte values: {distinct}");

            // Sample four pixels from the middle so a wrong channel order or a
            // dropped alpha channel shows up as numbers rather than as a
            // plausible-looking picture.
            let px = |x: u32, y: u32| -> String {
                let i = ((y as usize * f.width as usize) + x as usize) * 4;
                format!(
                    "({}, {}, {}, {})",
                    f.rgba_data[i],
                    f.rgba_data[i + 1],
                    f.rgba_data[i + 2],
                    f.rgba_data[i + 3]
                )
            };
            let (mx, my) = (f.width / 2, f.height / 2);
            println!(
                "center={} corners={} {} {} {}",
                px(mx, my),
                px(0, 0),
                px(f.width - 1, 0),
                px(0, f.height - 1),
                px(f.width - 1, f.height - 1)
            );
        }
        Err(e) => {
            println!("ERR {e:#}");
            std::process::exit(1);
        }
    }
}
