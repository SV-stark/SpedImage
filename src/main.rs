//! SpedImage - Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::eyre::Result;
use spedimage_lib::SpedImageApp;
use std::path::PathBuf;
use std::time::Instant;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let t0 = Instant::now();
    spedimage_lib::startup::set_start(t0);
    spedimage_lib::startup::log("main enter");
    // Install panic hook only when explicitly requested — saves ~5 ms on the
    // hot startup path and avoids pulling in backtrace machinery.
    if std::env::var_os("RUST_BACKTRACE").is_some() || std::env::var_os("COLORBT").is_some() {
        let _ = color_eyre::install();
    }
    spedimage_lib::startup::log("after color_eyre");

    // Fast path: no tracing subscriber at all unless RUST_LOG is set.
    // Benchmark and normal runs stay on the `NoSubscriber` — `tracing::info!`
    // becomes a single atomic load.
    if std::env::var_os("RUST_LOG").is_some() {
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    spedimage_lib::startup::log("after tracing");

    tracing::info!("Starting SpedImage v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments for initial image path
    let args: Vec<String> = std::env::args().collect();
    spedimage_lib::startup::log("after args");
    let initial_path = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.exists() { Some(p) } else { None }
    } else {
        None
    };
    spedimage_lib::startup::log("after initial_path check");

    if let Some(ref p) = initial_path {
        tracing::info!("Opening initial path: {:?}", p);
    }

    // Defer single-instance and IPC setup until after the window is visible
    // (handled inside `SpedImageApp::run` / `resumed`). This keeps the
    // critical window-show path free of file-lock and TCP work.
    spedimage_lib::startup::log("before App::run");
    SpedImageApp::run(initial_path)?;
    spedimage_lib::startup::log("after App::run (exit)");

    tracing::info!("Application exited cleanly");
    Ok(())
}
