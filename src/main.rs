//! SpedImage - Main Entry Point
//!
//! Entry point for the GPU-accelerated image viewer application.

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

    // Run the application
    SpedImageApp::run()?;

    tracing::info!("Application exited cleanly");
    Ok(())
}
