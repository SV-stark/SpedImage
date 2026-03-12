//! SpedImage - Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use spedimage_lib::SpedImageApp;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting SpedImage v2.0.0");

    // Parse command line arguments for initial image path
    let args: Vec<String> = std::env::args().collect();
    let initial_path = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref p) = initial_path {
        tracing::info!("Opening initial path: {:?}", p);
    }

    // Run the application
    SpedImageApp::run(initial_path)?;

    tracing::info!("Application exited cleanly");
    Ok(())
}
