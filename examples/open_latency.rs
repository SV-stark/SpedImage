//! In-process decode latency: time-to-decoded-RGBA through the app's real
//! load pipeline (`load_and_downsample_with`, scale target 1920x1080).
//! Run: cargo run --release --example open_latency

use std::time::{Duration, Instant};

use spedimage_lib::image::{FrameLimit, ImageBackend, LoadOptions};

const TARGET_W: u32 = 1920;
const TARGET_H: u32 = 1080;
const WARMUP: usize = 2;
const ITERS: usize = 15;

fn opts() -> LoadOptions {
    LoadOptions {
        frames: FrameLimit::First,
        bake_orientation: false,
    }
}

fn median(v: &mut [Duration]) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn main() {
    let corpus = [
        "target/bench_corpus/bench.jpg",
        "target/bench_corpus/bench.png",
        "target/bench_corpus/bench.tiff",
        "target/bench_corpus/bench.gif",
        "target/bench_corpus/bench.heic",
    ];

    println!("| Format | File | Native dims | Decoded-to | Median | Min | Max |");
    println!("|---|---|---|---|---|---|---|");

    for path in corpus {
        let p = std::path::Path::new(path);
        if !p.exists() {
            println!("| {path} | MISSING | | | | | |");
            continue;
        }
        let kb = p.metadata().map(|m| m.len() / 1024).unwrap_or(0);

        // Probe native dims with an unrestricted decode once.
        let (native_w, native_h) =
            match ImageBackend::load_and_downsample_with(p, u32::MAX, u32::MAX, opts())
                .ok()
                .and_then(|f| f.into_iter().next())
            {
                Some(f) => (f.width, f.height),
                None => {
                    println!(
                        "| {} | {kb} KB | ERROR | | | | |",
                        p.extension().unwrap().to_string_lossy().to_uppercase()
                    );
                    continue;
                }
            };

        for _ in 0..WARMUP {
            let _ = ImageBackend::load_and_downsample_with(p, TARGET_W, TARGET_H, opts());
        }

        let mut times = Vec::with_capacity(ITERS);
        let mut final_dims = (0u32, 0u32);
        for _ in 0..ITERS {
            let t0 = Instant::now();
            if let Ok(frames) =
                ImageBackend::load_and_downsample_with(p, TARGET_W, TARGET_H, opts())
                && let Some(f) = frames.first()
            {
                final_dims = (f.width, f.height);
                // Touch every page so timing includes real memory work.
                let mut sink = 0u64;
                for chunk in f.rgba_data.chunks(4096) {
                    sink = sink.wrapping_add(chunk[0] as u64);
                }
                std::hint::black_box(sink);
            }
            times.push(t0.elapsed());
        }

        let med = median(&mut times);
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();
        println!(
            "| {} | {} KB | {}x{} | {}x{} | {:.1} ms | {:.1} ms | {:.1} ms |",
            p.extension().unwrap().to_string_lossy().to_uppercase(),
            kb,
            native_w,
            native_h,
            final_dims.0,
            final_dims.1,
            med.as_secs_f64() * 1000.0,
            min.as_secs_f64() * 1000.0,
            max.as_secs_f64() * 1000.0,
        );
    }
}
