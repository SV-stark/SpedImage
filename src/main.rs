//! SpedImage - Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use spedimage_lib::SpedImageApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(windows)]
fn register_file_associations() {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)
        .ok();
    if let Some(classes) = classes {
        // Check if we already registered or user dismissed it
        if classes.open_subkey("SpedImage.Image").is_ok() {
            return;
        }

        // Prompt user
        let confirmed = rfd::MessageDialog::new()
            .set_title("Default Photo Viewer")
            .set_description("Would you like to register SpedImage to open image files by default?")
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes;

        if !confirmed {
            // Write a dummy key so we don't ask again
            let _ = classes.create_subkey("SpedImage.Image");
            return;
        }

        if let Ok((prog_id, _)) = classes.create_subkey("SpedImage.Image") {
            let _ = prog_id.set_value("", &"SpedImage Image File");
            if let Ok((shell, _)) = prog_id.create_subkey("shell\\open\\command") {
                let exe_path = std::env::current_exe().unwrap_or_default();
                let exe_path_lossy = exe_path.to_string_lossy();
                let cmd = format!("\"{exe_path_lossy}\" \"%1\"");
                let _ = shell.set_value("", &cmd);
            }
            if let Ok((icon, _)) = prog_id.create_subkey("DefaultIcon") {
                let exe_path = std::env::current_exe().unwrap_or_default();
                let exe_path_lossy = exe_path.to_string_lossy();
                let cmd = format!("\"{exe_path_lossy}\",0");
                let _ = icon.set_value("", &cmd);
            }
        }

        let exts = [
            ".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic", ".avif", ".bmp", ".tiff", ".tif",
            ".cr2", ".dng", ".arw", ".nef", ".raw", ".orf", ".rw2",
        ];
        for ext in exts {
            if let Ok((ext_key, _)) = classes.create_subkey(ext) {
                let _ = ext_key.set_value("", &"SpedImage.Image");
            }
        }

        // Notify Windows Explorer of the association change
        use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
        unsafe {
            SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
        }
    }
}

#[cfg(not(windows))]
fn register_file_associations() {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let filter = if cfg!(debug_assertions) {
        "spedimage=debug,winit=warn,wgpu=warn"
    } else {
        "spedimage=info,winit=warn,wgpu=warn"
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(concat!("Starting SpedImage v", env!("CARGO_PKG_VERSION")));

    // Set up panic handler for logging
    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("Application panicked: {panic_info}");
    }));

    register_file_associations();

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
