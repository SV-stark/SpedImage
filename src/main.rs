//! SpedImage - Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::eyre::Result;
use spedimage_lib::SpedImageApp;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<()> {
    // Initialize error reporting
    color_eyre::install()?;

    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        // .with(tracing_tracy::TracyLayer::new())
        .init();

    tracing::info!("Starting SpedImage v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments for initial image path
    let args: Vec<String> = std::env::args().collect();
    let initial_path = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.exists() { Some(p) } else { None }
    } else {
        None
    };

    // OS-level single instance check via named mutex/socket
    let instance_lock = single_instance::SingleInstance::new("spedimage_app_instance_lock")
        .map_err(|e| color_eyre::eyre::eyre!("Single instance check error: {e:?}"))?;

    if !instance_lock.is_single() {
        // Secondary instance: send image path to running primary instance
        if let Some(p) = initial_path.as_ref() {
            let stream_res = TcpStream::connect("127.0.0.1:49512");
            if let Ok(mut stream) = stream_res {
                let absolute_path = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
                let path_str = absolute_path.to_string_lossy().into_owned();
                let _ = stream.write_all(path_str.as_bytes());
            }
        }
        tracing::info!("Another instance is already running. Sent path and exiting.");
        return Ok(());
    }

    // Bind local TCP listener for primary instance to receive paths from secondary instances
    let listener = match TcpListener::bind("127.0.0.1:49512") {
        Ok(l) => l,
        Err(_) => {
            tracing::info!("Failed to bind single instance port; continuing as primary.");
            TcpListener::bind("127.0.0.1:0")?
        }
    };

    if let Some(ref p) = initial_path {
        tracing::info!("Opening initial path: {:?}", p);
    }

    // Run the application
    SpedImageApp::run(initial_path, listener)?;

    tracing::info!("Application exited cleanly");
    Ok(())
}
