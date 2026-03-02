//! SpedImage - Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use spedimage_lib::SpedImageApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::new("spedimage=debug,winit=warn,wgpu=warn"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting SpedImage v{}", env!("CARGO_PKG_VERSION"));

    // Set up panic handler for logging
    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("Application panicked: {}", panic_info);
    }));

    // (5) CLI argument: accept a file path to open on startup
    // Usage: spedimage.exe [image_path]
    let initial_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    if let Some(ref p) = initial_path {
        tracing::info!("Opening from CLI: {:?}", p);
    }

    // Run the application
    SpedImageApp::run(initial_path)?;

    tracing::info!("Application exited cleanly");
    Ok(())
}
