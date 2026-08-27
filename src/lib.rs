//! SpedImage - Ultra-Lightweight GPU-Accelerated Image Viewer
//!
//! A high-performance, cross-platform image viewer built with Rust + WGPU.
//! Features GPU-accelerated image processing and a modern native UI.

pub mod app;
pub mod config;
pub mod image;
pub mod render;
pub mod ui;

pub use app::SpedImageApp;
pub use image::ImageBackend;
pub use render::Renderer;

pub mod startup {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    pub fn set_start(t: Instant) {
        let _ = START.set(t);
    }
    pub fn log(label: &str) {
        // Only emit when explicitly enabled (benchmarks run without it for
        // accurate cold-start numbers). Set `SPEDIMAGE_STARTUP_LOG=1` to
        // capture a detailed trace to H:\target\startup.log.
        if std::env::var_os("SPEDIMAGE_STARTUP_LOG").is_none() {
            return;
        }
        if let Some(s) = START.get() {
            let ms = s.elapsed().as_secs_f64() * 1000.0;
            eprintln!("[startup {ms:.1}ms] {label}");
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(r"H:\target\startup.log")
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(format!("[{ms:.1}ms] {label}\n").as_bytes())
                });
        }
    }
}
