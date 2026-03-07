# Directory Structure Report

This document contains all files from the `SpedImage` directory, optimized for LLM consumption.
Custom ignored patterns: docs,assets
Content hash: ded30f07ecb68a6f

## File Tree Structure

- 📄 CLAUDE.md
- 📄 Cargo.lock
- 📄 Cargo.toml
- 📄 LICENSE
- 📄 README.md
- 📁 assets
  - 📄 Inter-Regular.ttf
  - 📁 icons
    - 📄 icon.png
- 📄 error.log
- 📄 help.txt
- 📄 install_deps.sh
- 📄 installer.nsi
- 📄 log.txt
- 📄 scorecard.png
- 📁 src
  - 📁 app
    - 📄 actions.rs
    - 📄 events.rs
    - 📄 mod.rs
    - 📄 services.rs
    - 📄 state.rs
    - 📄 types.rs
  - 📄 gpu_renderer.rs
  - 📄 image_backend.rs
  - 📄 lib.rs
  - 📄 main.rs
  - 📄 ui.rs


### File: `CLAUDE.md`

- Size: 3841 bytes
- Modified: 2026-03-05 09:17:38 UTC

```markdown
   1 | # CLAUDE.md
   2 | 
   3 | This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
   4 | 
   5 | ---
   6 | 
   7 | ## Project: SpedImage
   8 | 
   9 | Ultra-Lightweight GPU-Accelerated Image Viewer built in Rust with WGPU for real-time image processing and rendering.
  10 | 
  11 | ## Build & Run Commands
  12 | 
  13 | ### Development Mode
  14 | ```bash
  15 | cargo build                    # Debug build
  16 | cargo test                     # Run all tests
  17 | cargo clippy                  # Lint checking
  18 | ```
  19 | 
  20 | ### Release Build (Production)
  21 | ```bash
  22 | cargo build --release
  23 | cargo run --release
  24 | ```
  25 | 
  26 | ### Run with an image on startup
  27 | ```bash
  28 | cargo run --release -- <image_path>
  29 | ```
  30 | 
  31 | ## Architecture Overview
  32 | 
  33 | ### Data Flow
  34 | 1. `main.rs` - CLI entry point, initializes logging and starts the event loop
  35 | 2. `app.rs` (`SpedImageApp`) - Main application state coordinator managing:
  36 |    - Event loop using winit
  37 |    - Image loading via `ImageBackend` (pure Rust image decoding)
  38 |    - WGPU renderer for GPU-accelerated rendering
  39 |    - UI state management
  40 | 
  41 | ### Core Components
  42 | 
  43 | | File | Responsibility |
  44 | |------|---------------|
  45 | | `src/app.rs` | Main application loop, event handling, state coordination |
  46 | | `src/gpu_renderer.rs` | WGPU pipeline, shader compilation, real-time adjustments via WGSL shaders |
  47 | | `src/image_backend.rs` | Image decoding (JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC), format detection |
  48 | | `src/ui.rs` | UI state (sidebar, crop overlay, thumbnail strip), user-facing state |
  49 | | `src/main.rs` | Entry point, CLI argument handling, logging setup |
  50 | 
  51 | ### Key Dependencies (Cargo.toml)
  52 | - **WGPU** - GPU rendering (Vulkan/Metal/DX12/OpenGL)
  53 | - **winit** - Windowing system
  54 | - **image crate** - Pure Rust image decoding
  55 | - **wgpu_glyph** - Text rendering
  56 | - **tracing** - Structured logging with environment filter
  57 | 
  58 | ### Features Flags
  59 | ```toml
  60 | # src/Cargo.toml
  61 | raw = ["rawler"]        # RAW camera format (e.g., DSLR sensors)
  62 | svg = ["resvg"]        # SVG thumbnail rendering
  63 | default = ["raw", "svg"]
  64 | heif = []              # HEIF encoding/decoding
  65 | ```
  66 | 
  67 | ## Technical Details
  68 | 
  69 | ### Logging Format
  70 | The application uses structured logging via the `tracing` crate:
  71 | - Debug builds: `spedimage=debug,winit=warn,wgpu=warn`
  72 | - Release: `spedimage=info,winit=warn,wgpu=warn`
  73 | - Can be overridden with `RUST_LOG=<level>` environment variable
  74 | 
  75 | ### GPU Rendering Pipeline
  76 | - WGPU handles the actual rendering via custom WGSL shaders
  77 | - All image adjustments (brightness, contrast, saturation) are applied on GPU - zero CPU work
  78 | - Rotation is shader-based (90° increments only)
  79 | - Thumbnail strip preview renders adjacent images for browsing
  80 | 
  81 | ### Memory Management
  82 | - Zero-copy texture loading into WGPU
  83 | - Images are decoded in-memory using the `image` crate, then transferred to GPU textures
  84 | - Background threads preload thumbnails from neighboring images in a directory
  85 | 
  86 | ## Keyboard Shortcuts (User-facing)
  87 | - `A/W` - Previous image
  88 | - `D/S` - Next image
  89 | - `R` - Rotate 90° clockwise
  90 | - `C` - Toggle crop mode
  91 | - `O` - Open file dialog
  92 | - `Ctrl+S` - Save modified image
  93 | - `F` - Toggle sidebar visibility
  94 | 
  95 | ---
  96 | 
  97 | ## File Structure
  98 | 
  99 | ```
 100 | spedimage/
 101 | ├── src/
 102 | │   ├── main.rs          # CLI entry point, logging setup
 103 | │   ├── lib.rs           # Module declarations for library crate
 104 | │   ├── app.rs           # Application event loop & state coordination (SpedImageApp)
 105 | │   ├── gpu_renderer.rs  # WGPU rendering pipeline with WGSL shaders
 106 | │   ├── image_backend.rs # Pure Rust image decoding backend
 107 | │   └── ui.rs            # UI state (sidebar, crop overlay, thumbnails)
 108 | ├── assets/               # Icon PNGs and shader files
 109 | └── Cargo.toml           # Rust manifest with dependencies and features
 110 | ```
 111 | 
 112 | ## Image Formats Supported
 113 | The pure Rust `image` crate decodes: JPEG, PNG, GIF, BMP, TIFF, WebP, HEIF (optional), RAW (optional). Shaders support AVIF as well.
```

### File: `Cargo.toml`

- Size: 1908 bytes
- Modified: 2026-03-07 05:02:59 UTC

```toml
   1 | [package]
   2 | name = "spedimage"
   3 | version = "2.0.0"
   4 | edition = "2021"
   5 | description = "Ultra-Lightweight GPU-Accelerated Image Viewer"
   6 | authors = ["SpedImage Team"]
   7 | license = "MIT"
   8 | rust-version = "1.82"
   9 | 
  10 | [dependencies]
  11 | # Windowing
  12 | winit = { version = "0.30", features = ["rwh_06"] }
  13 | 
  14 | # GPU Rendering
  15 | wgpu = "26.0"
  16 | 
  17 | # Image decoding (pure Rust)
  18 | image = { version = "0.25", default-features = false, features = [
  19 |     "jpeg",
  20 |     "png",
  21 |     "gif",
  22 |     "bmp",
  23 |     "tga",
  24 |     "tiff",
  25 |     "webp",
  26 | ] }
  27 | 
  28 | # Text Rendering
  29 | wgpu_glyph = "0.26"
  30 | 
  31 | # File dialog (native)
  32 | rfd = "0.14"
  33 | 
  34 | # Logging
  35 | tracing = "0.1"
  36 | tracing-subscriber = { version = "0.3", features = ["env-filter"] }
  37 | 
  38 | # Error handling
  39 | thiserror = "2"
  40 | anyhow = "1"
  41 | 
  42 | # Async blocking
  43 | pollster = "0.3"
  44 | 
  45 | # EXIF parsing
  46 | kamadak-exif = "0.5"
  47 | chrono = "0.4"
  48 | 
  49 | # For image manipulation
  50 | fast_image_resize = "3"
  51 | 
  52 | # For GPU data types
  53 | bytemuck = { version = "1", features = ["derive"] }
  54 | 
  55 | # LRU cache for prefetch
  56 | lru = "0.12"
  57 | 
  58 | # File watching for directory changes
  59 | notify = "7"
  60 | 
  61 | # Thread pool for async operations
  62 | rayon = "1.10"
  63 | 
  64 | # RAW camera format decoding
  65 | rawler = { version = "0.7", optional = true }
  66 | 
  67 | # SVG rendering
  68 | resvg = { version = "0.47", optional = true }
  69 | 
  70 | [features]
  71 | default = ["raw", "svg"]
  72 | heif = []
  73 | raw = ["rawler"]
  74 | svg = ["resvg"]
  75 | 
  76 | [target.'cfg(windows)'.dependencies]
  77 | winreg = "0.51"
  78 | windows = { version = "0.58", features = [
  79 |     "Win32_Foundation",
  80 |     "Win32_Graphics_Imaging",
  81 |     "Win32_System_Com",
  82 |     "Win32_Storage_FileSystem",
  83 |     "Win32_System_Memory",
  84 |     "Win32_System_DataExchange",
  85 |     "Win32_Graphics_Gdi",
  86 |     "Win32_UI_WindowsAndMessaging",
  87 |     "Win32_UI_Shell",
  88 | ] }
  89 | 
  90 | [lib]
  91 | name = "spedimage_lib"
  92 | path = "src/lib.rs"
  93 | 
  94 | [[bin]]
  95 | name = "spedimage"
  96 | path = "src/main.rs"
  97 | 
  98 | [profile.release]
  99 | opt-level = "z"
 100 | lto = true
 101 | codegen-units = 1
 102 | strip = true
 103 | panic = "abort"
```

### File: `README.md`

- Size: 5134 bytes
- Modified: 2026-03-05 17:56:30 UTC

```markdown
   1 | # 🖼️ SpedImage
   2 | **Ultra-Lightweight, GPU-Accelerated Image Viewer with Native Performance.**
   3 | 
   4 | [![Version: 2.0.0](https://img.shields.io/badge/Version-2.0.0-blue)](#)
   5 | [![Rust: 1.82+](https://img.shields.io/badge/Rust-1.82+-orange)](#)
   6 | [![Platform: Windows | Linux | macOS](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-lightgrey)](#)
   7 | [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
   8 | 
   9 | SpedImage is a high-performance, cross-platform image viewer rebuilt in **Rust** with **WGPU** for GPU-accelerated rendering. It provides memory-safe, zero-copy image processing with real-time adjustments.
  10 | 
  11 | ## 📋 Table of Contents
  12 | - [Key Features](#-key-features)
  13 | - [Format Support](#-format-support)
  14 | - [Usage & CLI](#-usage--cli)
  15 | - [Building from Source](#-building-from-source)
  16 | - [Project Architecture](#-project-architecture)
  17 | - [Keyboard Shortcuts](#-keyboard-shortcuts)
  18 | - [Contributing](#-contributing)
  19 | 
  20 | ---
  21 | 
  22 | ## 🚀 Key Features
  23 | 
  24 | ### ⚡ High-Performance Image Loading
  25 | - **Memory Efficient**: Zero-copy GPU texture loading ensures minimal RAM usage.
  26 | - **Fast Startup**: Native performance without heavy web or electron frameworks.
  27 | 
  28 | ### 🎨 GPU-Accelerated Editing
  29 | All adjustments are applied in real-time using **WGPU Shaders**—no CPU processing required.
  30 | - **Instant Adjustments**: Brightness, Contrast, and Saturation work instantly on 4K/8K images.
  31 | - **HDR Toning**: Real-time **Filmic Reinhard** tone-mapping for cinematic contrast (`H`).
  32 | - **Lossless Rotation**: Shader-based rotation (90° increments).
  33 | - **Crop**: Crop regions using smooth zoom and pan.
  34 | - **Save & Export**: Save your edits effortlessly (`Ctrl+S`).
  35 | 
  36 | ---
  37 | 
  38 | ## 🖼️ Format Support
  39 | 
  40 | | Format | Decoding Engine | OS Support |
  41 | |--------|-----------------|------------|
  42 | | JPEG, PNG, GIF, BMP, TIFF, WebP | Pure Rust (`image` crate) | All Platforms |
  43 | | RAW (CR2, NEF, ARW, DNG, etc.) | `rawler` crate + WIC fallback | All Platforms |
  44 | | SVG | `resvg` crate | All Platforms |
  45 | | HEIC / AVIF | Native OS Codecs (WIC) | Windows Only* |
  46 | 
  47 | *\* On Windows, HEIC/AVIF requires the appropriate HEVC/HEIF extensions installed from the Microsoft Store.*
  48 | 
  49 | ---
  50 | 
  51 | ## ⚡ Performance Benchmarks
  52 | 
  53 | Based on a typical consumer system (e.g., Apple M1 or Intel i7 + mid-range GPU). Times and memory usage are approximate and depend heavily on image resolution.
  54 | 
  55 | | Operation | Typical Latency | CPU Usage | Memory Impact | 
  56 | |-----------|-----------------|-----------|---------------|
  57 | | **Cold Start to Render** | < 100ms | Spike on load | Base app size (~10MB) |
  58 | | **Decoding (e.g., 24MP JPEG)** | 50-150ms | Multi-core spike | Dependent on image res |
  59 | | **GPU Upload (Zero-Copy)** | < 5ms | Near Zero | Video RAM mapped directly |
  60 | | **HDR Toning (Filmic)** | 0.0ms (0 CPU) | Zero | None |
  61 | | **Smooth Crop/Zoom Animation** | 60 FPS | Nominal (< 2%) | None |
  62 | | **Brightness/Contrast Adjust** | 0.0ms (0 CPU) | Zero | None |
  63 | 
  64 | ---
  65 | 
  66 | ## 💻 Usage & CLI
  67 | 
  68 | Launch SpedImage normally, or open a specific image directly from the command line:
  69 | 
  70 | ```bash
  71 | # Open SpedImage in the current directory
  72 | spedimage
  73 | 
  74 | # Open a specific image
  75 | spedimage /path/to/image.jpg
  76 | ```
  77 | 
  78 | ---
  79 | 
  80 | ## ⚙️ Building from Source
  81 | 
  82 | **Prerequisites:**
  83 | - **Rust** (1.82+)
  84 | - **Cargo** (comes with Rust)
  85 | 
  86 | ### 🪟 Windows / 🐧 Linux / 🍎 macOS
  87 | 
  88 | 1. **Clone**:
  89 |    ```bash
  90 |    git clone https://github.com/SV-stark/SpedImage.git
  91 |    cd spedimage
  92 |    ```
  93 | 
  94 | 2. **Build**:
  95 |    ```bash
  96 |    cargo build --release
  97 |    ```
  98 | 
  99 | 3. **Run**:
 100 |    ```bash
 101 |    cargo run --release
 102 |    ```
 103 | 
 104 | ---
 105 | 
 106 | ## 📐 Project Architecture
 107 | 
 108 | Built with a state-of-the-art native stack emphasizing **Memory Safety** and **Performance**.
 109 | 
 110 | | Component | Technology | Description |
 111 | |-----------|------------|-------------|
 112 | | **Language** | Rust 2021 | Eliminates buffer overflows and data races. |
 113 | | **Windowing** | winit | Cross-platform, reliable event loop. |
 114 | | **GPU Rendering** | WGPU | Safe access to Vulkan/Metal/DX12/OpenGL. |
 115 | | **Image Decoding**| `image` / OS codecs | Hybrid approach for maximum format compatibility. |
 116 | | **Shaders** | WGSL | Highly optimized GPU processing blocks. |
 117 | 
 118 | ---
 119 | 
 120 | ## ⌨️ Keyboard Shortcuts
 121 | 
 122 | | Key | Action |
 123 | |-----|--------|
 124 | | `A` / `W` | Previous image |
 125 | | `D` / `S` | Next image |
 126 | | `Right` / `Left` Arrow | Next / Previous image |
 127 | | `R` | Rotate 90° |
 128 | | `H` | Toggle HDR Toning |
 129 | | `C` | Toggle crop mode |
 130 | | `I` | Toggle image info (EXIF) |
 131 | | `O` | Open file dialog |
 132 | | `Ctrl+P` / `P`| Print image (Windows) |
 133 | | `Ctrl+S` | Save image |
 134 | | `F` | Toggle sidebar |
 135 | | `T` | Toggle thumbnail strip |
 136 | | `1` | Reset adjustments |
 137 | | `+` / `=` | Zoom in |
 138 | | `-` | Zoom out |
 139 | | `0` | Zoom to fit |
 140 | | `Esc` | Cancel crop / Quit |
 141 | | `?` | Toggle help overlay |
 142 | 
 143 | ---
 144 | 
 145 | ## 🤝 Contributing
 146 | Contributions, issues, and feature requests are welcome! Feel free to check out the [issues page](https://github.com/SV-stark/SpedImage/issues) if you want to contribute.
 147 | 
 148 | ---
 149 | 
 150 | ## 📜 License
 151 | SpedImage is distributed under the **[MIT License](LICENSE)**.
```

### File: `src\app\mod.rs`

- Size: 154 bytes
- Modified: 2026-03-07 12:12:27 UTC

```rust
   1 | pub mod actions;
   2 | pub mod events;
   3 | pub mod services;
   4 | pub mod state;
   5 | pub mod types;
   6 | 
   7 | pub use state::SpedImageApp;
   8 | pub use types::{AppEvent, WakeUp};
```

### File: `src\lib.rs`

- Size: 381 bytes
- Modified: 2026-03-02 13:46:35 UTC

```rust
   1 | //! SpedImage - Ultra-Lightweight GPU-Accelerated Image Viewer
   2 | //!
   3 | //! A high-performance, cross-platform image viewer built with Rust + WGPU.
   4 | //! Features GPU-accelerated image processing and a modern native UI.
   5 | 
   6 | pub mod app;
   7 | pub mod gpu_renderer;
   8 | pub mod image_backend;
   9 | pub mod ui;
  10 | 
  11 | pub use app::SpedImageApp;
  12 | pub use gpu_renderer::Renderer;
  13 | pub use image_backend::ImageBackend;
```

### File: `src\main.rs`

- Size: 3802 bytes
- Modified: 2026-03-06 12:45:32 UTC

```rust
   1 | //! SpedImage - Main Entry Point
   2 | 
   3 | #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
   4 | 
   5 | use spedimage_lib::SpedImageApp;
   6 | use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
   7 | 
   8 | #[cfg(windows)]
   9 | fn register_file_associations() {
  10 |     use winreg::enums::*;
  11 |     use winreg::RegKey;
  12 | 
  13 |     let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  14 |     let classes = hkcu
  15 |         .open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)
  16 |         .ok();
  17 |     if let Some(classes) = classes {
  18 |         // Check if we already registered or user dismissed it
  19 |         if classes.open_subkey("SpedImage.Image").is_ok() {
  20 |             return;
  21 |         }
  22 | 
  23 |         // Prompt user
  24 |         let confirmed = rfd::MessageDialog::new()
  25 |             .set_title("Default Photo Viewer")
  26 |             .set_description("Would you like to register SpedImage to open image files by default?")
  27 |             .set_buttons(rfd::MessageButtons::YesNo)
  28 |             .show()
  29 |             == rfd::MessageDialogResult::Yes;
  30 | 
  31 |         if !confirmed {
  32 |             // Write a dummy key so we don't ask again
  33 |             let _ = classes.create_subkey("SpedImage.Image");
  34 |             return;
  35 |         }
  36 | 
  37 |         if let Ok((prog_id, _)) = classes.create_subkey("SpedImage.Image") {
  38 |             let _ = prog_id.set_value("", &"SpedImage Image File");
  39 |             if let Ok((shell, _)) = prog_id.create_subkey("shell\\open\\command") {
  40 |                 let exe_path = std::env::current_exe().unwrap_or_default();
  41 |                 let exe_path_lossy = exe_path.to_string_lossy();
  42 |                 let cmd = format!("\"{exe_path_lossy}\" \"%1\"");
  43 |                 let _ = shell.set_value("", &cmd);
  44 |             }
  45 |             if let Ok((icon, _)) = prog_id.create_subkey("DefaultIcon") {
  46 |                 let exe_path = std::env::current_exe().unwrap_or_default();
  47 |                 let exe_path_lossy = exe_path.to_string_lossy();
  48 |                 let cmd = format!("\"{exe_path_lossy}\",0");
  49 |                 let _ = icon.set_value("", &cmd);
  50 |             }
  51 |         }
  52 | 
  53 |         let exts = [
  54 |             ".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic", ".avif", ".bmp", ".tiff", ".tif",
  55 |             ".cr2", ".dng", ".arw", ".nef", ".raw", ".orf", ".rw2",
  56 |         ];
  57 |         for ext in exts {
  58 |             if let Ok((ext_key, _)) = classes.create_subkey(ext) {
  59 |                 let _ = ext_key.set_value("", &"SpedImage.Image");
  60 |             }
  61 |         }
  62 | 
  63 |         // Notify Windows Explorer of the association change
  64 |         use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
  65 |         unsafe {
  66 |             SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
  67 |         }
  68 |     }
  69 | }
  70 | 
  71 | #[cfg(not(windows))]
  72 | fn register_file_associations() {}
  73 | 
  74 | fn main() -> Result<(), Box<dyn std::error::Error>> {
  75 |     // Initialize logging
  76 |     let filter = if cfg!(debug_assertions) {
  77 |         "spedimage=debug,winit=warn,wgpu=warn"
  78 |     } else {
  79 |         "spedimage=info,winit=warn,wgpu=warn"
  80 |     };
  81 | 
  82 |     tracing_subscriber::registry()
  83 |         .with(EnvFilter::new(filter))
  84 |         .with(tracing_subscriber::fmt::layer())
  85 |         .init();
  86 | 
  87 |     tracing::info!(concat!("Starting SpedImage v", env!("CARGO_PKG_VERSION")));
  88 | 
  89 |     // Set up panic handler for logging
  90 |     std::panic::set_hook(Box::new(|panic_info| {
  91 |         tracing::error!("Application panicked: {panic_info}");
  92 |     }));
  93 | 
  94 |     register_file_associations();
  95 | 
  96 |     // (5) CLI argument: accept a file path to open on startup
  97 |     // Usage: spedimage.exe [image_path]
  98 |     let initial_path = std::env::args().nth(1).map(std::path::PathBuf::from);
  99 |     if let Some(ref p) = initial_path {
 100 |         tracing::info!("Opening from CLI: {:?}", p);
 101 |     }
 102 | 
 103 |     // Run the application
 104 |     SpedImageApp::run(initial_path)?;
 105 | 
 106 |     tracing::info!("Application exited cleanly");
 107 |     Ok(())
 108 | }
```

### File: `error.log`

- Size: 2565 bytes
- Modified: 2026-03-07 12:19:04 UTC

```log
   1 |     Checking spedimage v2.0.0 (E:\SpedImage)
   2 | error: couldn't read `src\app\../assets/icons/icon.png`: The system cannot find the path specified. (os error 3)
   3 |   --> src\app\types.rs:11:29
   4 |    |
   5 | 11 | pub const APP_ICON: &[u8] = include_bytes!("../assets/icons/icon.png");
   6 |    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   7 |    |
   8 | help: there is a file with the same name in a different directory
   9 |    |
  10 | 11 | pub const APP_ICON: &[u8] = include_bytes!("../../assets/icons/icon.png");
  11 |    |                                                +++
  12 | 
  13 | warning: unused imports: `APP_ICON` and `WakeUp`
  14 |  --> src\app\actions.rs:2:47
  15 |   |
  16 | 2 | use crate::app::types::{send_event, AppEvent, WakeUp, APP_ICON};
  17 |   |                                               ^^^^^^  ^^^^^^^^
  18 |   |
  19 |   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
  20 | 
  21 | warning: unused import: `std::sync::Arc`
  22 |  --> src\app\actions.rs:7:5
  23 |   |
  24 | 7 | use std::sync::Arc;
  25 |   |     ^^^^^^^^^^^^^^
  26 | 
  27 | warning: unused import: `ElementState`
  28 |  --> src\app\actions.rs:9:20
  29 |   |
  30 | 9 | use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
  31 |   |                    ^^^^^^^^^^^^
  32 | 
  33 | warning: unused imports: `Icon` and `Window`
  34 |   --> src\app\actions.rs:13:33
  35 |    |
  36 | 13 | use winit::window::{Fullscreen, Icon, Window};
  37 |    |                                 ^^^^  ^^^^^^
  38 | 
  39 | warning: unused import: `crate::image_backend::ImageData`
  40 |  --> src\app\events.rs:4:5
  41 |   |
  42 | 4 | use crate::image_backend::ImageData;
  43 |   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  44 | 
  45 | warning: unused import: `Path`
  46 |  --> src\app\events.rs:6:17
  47 |   |
  48 | 6 | use std::path::{Path, PathBuf};
  49 |   |                 ^^^^
  50 | 
  51 | warning: unused import: `winit::dpi::PhysicalPosition`
  52 |  --> src\app\events.rs:9:5
  53 |   |
  54 | 9 | use winit::dpi::PhysicalPosition;
  55 |   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  56 | 
  57 | warning: unused import: `Fullscreen`
  58 |   --> src\app\events.rs:12:21
  59 |    |
  60 | 12 | use winit::window::{Fullscreen, Icon, Window, WindowId};
  61 |    |                     ^^^^^^^^^^
  62 | 
  63 | warning: unused import: `std::os::windows::ffi::OsStrExt`
  64 |   --> src\app\actions.rs:16:5
  65 |    |
  66 | 16 | use std::os::windows::ffi::OsStrExt;
  67 |    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  68 | 
  69 | warning: unused import: `winit::raw_window_handle::HasWindowHandle`
  70 |   --> src\app\actions.rs:12:5
  71 |    |
  72 | 12 | use winit::raw_window_handle::HasWindowHandle;
  73 |    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  74 | 
  75 | warning: `spedimage` (lib) generated 10 warnings
  76 | error: could not compile `spedimage` (lib) due to 1 previous error; 10 warnings emitted
```

### File: `install_deps.sh`

- Size: 1277 bytes
- Modified: 2026-02-07 17:27:44 UTC

```bash
   1 | #!/bin/bash
   2 | 
   3 | # SpedImage Dependency Installer
   4 | # Installs required libraries for the Linux build.
   5 | 
   6 | echo "Detecting distribution..."
   7 | 
   8 | if [ -f /etc/os-release ]; then
   9 |     . /etc/os-release
  10 |     OS=$NAME
  11 | fi
  12 | 
  13 | if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]] || [[ "$OS" == *"Pop"* ]] || [[ "$OS" == *"Mint"* ]]; then
  14 |     echo "Detected Debian/Ubuntu based system."
  15 |     echo "Installing libjpeg-dev, libheif-dev, libxqv-dev..."
  16 |     sudo apt update
  17 |     sudo apt install -y libjpeg-dev libheif-dev libpng-dev cmake build-essential libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev xorg-dev
  18 | elif [[ "$OS" == *"Fedora"* ]]; then
  19 |     echo "Detected Fedora."
  20 |     sudo dnf install -y libjpeg-turbo-devel libheif-devel libpng-devel libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel
  21 | elif [[ "$OS" == *"Arch"* ]] || [[ "$OS" == *"Manjaro"* ]]; then
  22 |     echo "Detected Arch Linux."
  23 |     sudo pacman -S --needed base-devel libjpeg-turbo libheif libpng
  24 | else
  25 |     echo "Unsupported or unknown distribution: $OS"
  26 |     echo "Please manually install: libjpeg, libpng, libheif, and X11 development headers."
  27 |     exit 1
  28 | fi
  29 | 
  30 | echo "Dependencies installed successfully!"
  31 | echo "You can now build SpedImage with:"
  32 | echo "  cmake -B build"
  33 | echo "  cmake --build build --config Release"
```

### File: `installer.nsi`

- Size: 1780 bytes
- Modified: 2026-03-05 17:56:30 UTC

```nsi
   1 | !include "MUI2.nsh"
   2 | 
   3 | Name "SpedImage"
   4 | OutFile "SpedImage_Setup.exe"
   5 | InstallDir "$PROGRAMFILES64\SpedImage"
   6 | RequestExecutionLevel admin
   7 | 
   8 | !define MUI_ICON "assets\icons\icon.png"
   9 | !define MUI_UNICON "assets\icons\icon.png"
  10 | 
  11 | !insertmacro MUI_PAGE_WELCOME
  12 | !insertmacro MUI_PAGE_LICENSE "LICENSE"
  13 | !insertmacro MUI_PAGE_DIRECTORY
  14 | !insertmacro MUI_PAGE_INSTFILES
  15 | !insertmacro MUI_PAGE_FINISH
  16 | 
  17 | !insertmacro MUI_UNPAGE_WELCOME
  18 | !insertmacro MUI_UNPAGE_CONFIRM
  19 | !insertmacro MUI_UNPAGE_INSTFILES
  20 | !insertmacro MUI_UNPAGE_FINISH
  21 | 
  22 | !insertmacro MUI_LANGUAGE "English"
  23 | 
  24 | Section "Install"
  25 |     SetOutPath "$INSTDIR"
  26 |     File "target\release\spedimage.exe"
  27 |     File "assets\icons\icon.png"
  28 |     
  29 |     WriteUninstaller "$INSTDIR\uninstall.exe"
  30 |     
  31 |     CreateShortCut "$DESKTOP\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.png"
  32 |     
  33 |     CreateDirectory "$SMPROGRAMS\SpedImage"
  34 |     CreateShortCut "$SMPROGRAMS\SpedImage\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.png"
  35 |     CreateShortCut "$SMPROGRAMS\SpedImage\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  36 | 
  37 |     ; Register file associations
  38 |     WriteRegStr HKCU "Software\Classes\SpedImage.Image" "" "SpedImage Image File"
  39 |     WriteRegStr HKCU "Software\Classes\SpedImage.Image\DefaultIcon" "" "$INSTDIR\spedimage.exe,0"
  40 |     WriteRegStr HKCU "Software\Classes\SpedImage.Image\shell\open\command" "" '"$INSTDIR\spedimage.exe" "%1"'
  41 | SectionEnd
  42 | 
  43 | Section "Uninstall"
  44 |     Delete "$DESKTOP\SpedImage.lnk"
  45 |     RMDir /r "$SMPROGRAMS\SpedImage"
  46 |     
  47 |     Delete "$INSTDIR\spedimage.exe"
  48 |     Delete "$INSTDIR\icon.png"
  49 |     Delete "$INSTDIR\uninstall.exe"
  50 |     
  51 |     ; Remove registry file associations
  52 |     DeleteRegKey HKCU "Software\Classes\SpedImage.Image"
  53 | 
  54 |     RMDir "$INSTDIR"
  55 | SectionEnd
```

### File: `scorecard.png`

- Size: 18239 bytes
- Modified: 2026-03-07 11:47:31 UTC

```text
<Binary file or unsupported encoding: 18239 bytes>
```

### File: `src\app\actions.rs`

- Size: 45073 bytes
- Modified: 2026-03-07 12:20:14 UTC

```rust
   1 | use crate::app::state::SpedImageApp;
   2 | use crate::app::types::{send_event, AppEvent};
   3 | use crate::gpu_renderer::STRIP_HEIGHT_PX;
   4 | use crate::image_backend::ImageBackend;
   5 | use anyhow::Result;
   6 | use std::path::{Path, PathBuf};
   7 | use winit::dpi::PhysicalPosition;
   8 | use winit::event::{KeyEvent, MouseScrollDelta};
   9 | use winit::event_loop::ActiveEventLoop;
  10 | use winit::keyboard::{Key, NamedKey};
  11 | use winit::raw_window_handle::HasWindowHandle;
  12 | use winit::window::Fullscreen;
  13 | 
  14 | #[cfg(windows)]
  15 | use std::os::windows::ffi::OsStrExt;
  16 | 
  17 | impl SpedImageApp {
  18 |     pub(crate) fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
  19 |         // Named keys (non-text)
  20 |         match &event.logical_key {
  21 |             Key::Named(NamedKey::Escape) => {
  22 |                 if self.ui_state.is_cropping {
  23 |                     self.cancel_crop();
  24 |                 } else if self.show_help {
  25 |                     self.show_help = false;
  26 |                     self.dirty = true;
  27 |                 } else {
  28 |                     event_loop.exit();
  29 |                 }
  30 |                 return;
  31 |             }
  32 |             Key::Named(NamedKey::ArrowLeft) => {
  33 |                 self.prev_image();
  34 |                 return;
  35 |             }
  36 |             Key::Named(NamedKey::ArrowRight) => {
  37 |                 self.next_image();
  38 |                 return;
  39 |             }
  40 |             Key::Named(NamedKey::F1) => {
  41 |                 self.show_help = !self.show_help;
  42 |                 self.dirty = true;
  43 |                 return;
  44 |             }
  45 |             Key::Named(NamedKey::F2) => {
  46 |                 self.rename_current_image();
  47 |                 return;
  48 |             }
  49 |             Key::Named(NamedKey::F11) => {
  50 |                 if let Some(ref w) = self.window {
  51 |                     let mode = if w.fullscreen().is_some() {
  52 |                         None
  53 |                     } else {
  54 |                         Some(Fullscreen::Borderless(None))
  55 |                     };
  56 |                     w.set_fullscreen(mode);
  57 |                 }
  58 |                 return;
  59 |             }
  60 |             Key::Named(NamedKey::Delete) => {
  61 |                 if self.shift_pressed && !self.ui_state.selected_indices.is_empty() {
  62 |                     // Shift+Delete: batch delete selected
  63 |                     self.batch_delete_selected();
  64 |                 } else {
  65 |                     self.delete_current_image();
  66 |                 }
  67 |                 return;
  68 |             }
  69 |             _ => {}
  70 |         }
  71 | 
  72 |         if let Some(c) = event.logical_key.to_text() {
  73 |             let ctrl = self.ctrl_pressed;
  74 |             match c {
  75 |                 "d" | "D" => {
  76 |                     if event.repeat {
  77 |                         self.next_image();
  78 |                     } else {
  79 |                         self.next_image();
  80 |                         self.held_navigation_key = Some('d');
  81 |                         self.last_advance_time = Some(std::time::Instant::now());
  82 |                     }
  83 |                 }
  84 |                 "a" | "A" => {
  85 |                     if event.repeat {
  86 |                         self.prev_image();
  87 |                     } else {
  88 |                         self.prev_image();
  89 |                         self.held_navigation_key = Some('a');
  90 |                         self.last_advance_time = Some(std::time::Instant::now());
  91 |                     }
  92 |                 }
  93 |                 "w" | "W" => {
  94 |                     if ctrl {
  95 |                         self.set_as_wallpaper();
  96 |                     } else if event.repeat {
  97 |                         self.prev_image();
  98 |                     } else {
  99 |                         self.prev_image();
 100 |                         self.held_navigation_key = Some('w');
 101 |                         self.last_advance_time = Some(std::time::Instant::now());
 102 |                     }
 103 |                 }
 104 |                 "s" | "S" => {
 105 |                     if ctrl && self.shift_pressed {
 106 |                         // Ctrl+Shift+S: batch save selected
 107 |                         self.batch_save_selected();
 108 |                     } else if ctrl {
 109 |                         self.save_image();
 110 |                     } else if event.repeat {
 111 |                         self.next_image();
 112 |                     } else {
 113 |                         self.next_image();
 114 |                         self.held_navigation_key = Some('s');
 115 |                         self.last_advance_time = Some(std::time::Instant::now());
 116 |                     }
 117 |                 }
 118 |                 "r" | "R" => self.rotate_image(),
 119 |                 "o" | "O" => self.open_file_dialog(),
 120 |                 "f" | "F" => self.toggle_sidebar(),
 121 |                 "t" | "T" => self.toggle_thumbnail_strip(),
 122 |                 "1" => self.reset_adjustments(),
 123 |                 "z" | "Z" => {
 124 |                     self.ui_state.adjustments.pixel_perfect =
 125 |                         !self.ui_state.adjustments.pixel_perfect;
 126 |                     let state = if self.ui_state.adjustments.pixel_perfect {
 127 |                         "ON"
 128 |                     } else {
 129 |                         "OFF"
 130 |                     };
 131 |                     self.ui_state
 132 |                         .set_status(format!("Pixel-Perfect Zoom: {state}"));
 133 |                     self.dirty = true;
 134 |                 }
 135 |                 "c" | "C" if ctrl => self.copy_to_clipboard(),
 136 |                 "c" | "C" => self.toggle_crop(),
 137 |                 "h" | "H" => {
 138 |                     if self.alt_pressed {
 139 |                         // do nothing; alt+h is reserved
 140 |                     } else if self.shift_pressed {
 141 |                         self.show_histogram = !self.show_histogram;
 142 |                         let state = if self.show_histogram { "ON" } else { "OFF" };
 143 |                         // Lazy compute histogram when turning on
 144 |                         if self.show_histogram {
 145 |                             if let Some(ref mut img) = self.current_image {
 146 |                                 img.compute_histogram();
 147 |                             }
 148 |                         }
 149 |                         self.ui_state.set_status(format!("Histogram: {state}"));
 150 |                         self.dirty = true;
 151 |                     } else {
 152 |                         self.toggle_hdr_toning();
 153 |                     }
 154 |                 }
 155 |                 "i" | "I" => {
 156 |                     self.ui_state.toggle_info();
 157 |                     self.dirty = true;
 158 |                 }
 159 |                 "p" | "P" if ctrl => self.print_image(),
 160 |                 "+" | "=" => self.zoom_in(None),
 161 |                 "-" => self.zoom_out(None),
 162 |                 "0" => self.zoom_fit(),
 163 |                 " " => self.toggle_slideshow(),
 164 |                 "[" => self.adjust_slideshow_interval(-1),
 165 |                 "]" => self.adjust_slideshow_interval(1),
 166 |                 "?" => {
 167 |                     self.ui_state.toggle_help();
 168 |                     self.dirty = true;
 169 |                 }
 170 |                 _ => {}
 171 |             }
 172 |         }
 173 |     }
 174 | 
 175 |     pub(crate) fn toggle_slideshow(&mut self) {
 176 |         self.slideshow_active = !self.slideshow_active;
 177 |         if self.slideshow_active {
 178 |             self.slideshow_next_time = Some(std::time::Instant::now() + self.slideshow_interval);
 179 |             self.ui_state.set_status(format!(
 180 |                 "Slideshow started ({}s per image)",
 181 |                 self.slideshow_interval.as_secs()
 182 |             ));
 183 |         } else {
 184 |             self.slideshow_next_time = None;
 185 |             self.ui_state.set_status("Slideshow paused");
 186 |         }
 187 |         self.dirty = true;
 188 |     }
 189 | 
 190 |     pub(crate) fn adjust_slideshow_interval(&mut self, change: i32) {
 191 |         let current_secs = self.slideshow_interval.as_secs() as i32;
 192 |         let new_secs = (current_secs + change).clamp(1, 120) as u64;
 193 |         self.slideshow_interval = std::time::Duration::from_secs(new_secs);
 194 |         if self.slideshow_active {
 195 |             self.slideshow_next_time = Some(std::time::Instant::now() + self.slideshow_interval);
 196 |             self.ui_state
 197 |                 .set_status(format!("Slideshow interval: {new_secs}s"));
 198 |         } else {
 199 |             self.ui_state
 200 |                 .set_status(format!("Slideshow interval configured to {new_secs}s"));
 201 |         }
 202 |         self.dirty = true;
 203 |     }
 204 | 
 205 |     pub(crate) fn handle_mouse_wheel(
 206 |         &mut self,
 207 |         delta: MouseScrollDelta,
 208 |         cursor_pos: PhysicalPosition<f64>,
 209 |     ) {
 210 |         match delta {
 211 |             MouseScrollDelta::LineDelta(_, y) => {
 212 |                 if y > 0.0 {
 213 |                     self.zoom_in(Some(cursor_pos));
 214 |                 } else if y < 0.0 {
 215 |                     self.zoom_out(Some(cursor_pos));
 216 |                 }
 217 |             }
 218 |             MouseScrollDelta::PixelDelta(pos) => {
 219 |                 if pos.y > 0.0 {
 220 |                     self.zoom_in(Some(cursor_pos));
 221 |                 } else if pos.y < 0.0 {
 222 |                     self.zoom_out(Some(cursor_pos));
 223 |                 }
 224 |             }
 225 |         }
 226 |     }
 227 | 
 228 |     pub(crate) fn load_image(&mut self, path: PathBuf) {
 229 |         // Check prefetch cache first (LRU cache handles eviction automatically)
 230 |         if let Some(cached_frames) = self.prefetch_cache.pop(&path) {
 231 |             tracing::info!("Cache hit for {:?}", path);
 232 |             if let Some(ref proxy) = self.event_proxy {
 233 |                 send_event(&self.event_tx, proxy, AppEvent::ImageLoaded(cached_frames));
 234 |             }
 235 |             self.prefetch_adjacent(&path);
 236 |             return;
 237 |         }
 238 | 
 239 |         tracing::info!("Loading image: {:?}", path);
 240 |         self.ui_state.set_status("Loading...");
 241 |         self.loading = true;
 242 |         self.dirty = true;
 243 | 
 244 |         if let Some(ref w) = self.window {
 245 |             let name = path
 246 |                 .file_name()
 247 |                 .and_then(|n| n.to_str())
 248 |                 .unwrap_or("SpedImage");
 249 |             w.set_title(&format!("SpedImage — {name}"));
 250 |         }
 251 | 
 252 |         let (max_w, max_h) = match &self.window {
 253 |             Some(w) => {
 254 |                 let size = w.inner_size();
 255 |                 (size.width, size.height)
 256 |             }
 257 |             None => (3840, 2160),
 258 |         };
 259 | 
 260 |         let tx = self.event_tx.clone();
 261 |         let proxy = self.event_proxy.clone();
 262 |         let path2 = path.clone();
 263 |         std::thread::spawn(move || {
 264 |             let event = match ImageBackend::load_and_downsample(&path2, max_w, max_h) {
 265 |                 Ok(data) => AppEvent::ImageLoaded(data),
 266 |                 Err(e) => AppEvent::ImageError(e.to_string()),
 267 |             };
 268 |             if let Some(ref proxy) = proxy {
 269 |                 send_event(&tx, proxy, event);
 270 |             }
 271 |         });
 272 | 
 273 |         self.prefetch_adjacent(&path);
 274 |     }
 275 | 
 276 |     pub(crate) fn delete_current_image(&mut self) {
 277 |         if let Some(ref image) = self.current_image {
 278 |             let path = PathBuf::from(&image.path);
 279 |             let confirmed = rfd::MessageDialog::new()
 280 |                 .set_title("Delete Image")
 281 |                 .set_description(format!(
 282 |                     "Delete {}?",
 283 |                     path.file_name().unwrap_or_default().to_string_lossy()
 284 |                 ))
 285 |                 .set_buttons(rfd::MessageButtons::YesNo)
 286 |                 .show()
 287 |                 == rfd::MessageDialogResult::Yes;
 288 |             if confirmed && std::fs::remove_file(&path).is_ok() {
 289 |                 self.ui_state.set_status(format!(
 290 |                     "Deleted: {}",
 291 |                     path.file_name().unwrap_or_default().to_string_lossy()
 292 |                 ));
 293 |                 self.current_image = None;
 294 |                 self.current_frame_delays.clear();
 295 |                 self.ui_state
 296 |                     .load_directory(path.parent().unwrap_or(&path).to_path_buf());
 297 |                 self.dirty = true;
 298 |                 self.next_image();
 299 |             }
 300 |         }
 301 |     }
 302 | 
 303 |     pub(crate) fn save_image(&mut self) {
 304 |         if let Some(ref image_data) = self.current_image {
 305 |             let path = PathBuf::from(&image_data.path);
 306 |             let mut save_path = path.clone();
 307 | 
 308 |             if let Some(stem) = path.file_stem() {
 309 |                 let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
 310 |                 let stem_lossy = stem.to_string_lossy();
 311 |                 save_path.set_file_name(format!("{stem_lossy}_edited.{ext}"));
 312 |             }
 313 | 
 314 |             self.ui_state.set_status("Saving...");
 315 |             self.dirty = true;
 316 | 
 317 |             let path_clone = path.clone();
 318 |             let save_path_clone = save_path.clone();
 319 |             let adjustments = self.ui_state.adjustments;
 320 |             let tx = self.event_tx.clone();
 321 |             let proxy = self.event_proxy.clone();
 322 | 
 323 |             std::thread::spawn(move || {
 324 |                 let result = (|| -> anyhow::Result<()> {
 325 |                     let mut img = image::open(&path_clone)?;
 326 | 
 327 |                     if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
 328 |                         let (w, h) = (img.width() as f32, img.height() as f32);
 329 |                         let crop_x = (adjustments.crop_rect[0] * w) as u32;
 330 |                         let crop_y = (adjustments.crop_rect[1] * h) as u32;
 331 |                         let crop_w = (adjustments.crop_rect[2] * w) as u32;
 332 |                         let crop_h = (adjustments.crop_rect[3] * h) as u32;
 333 |                         img = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
 334 |                     }
 335 | 
 336 |                     let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round() as i32;
 337 |                     let rot_normalized = if rot_deg < 0 { rot_deg + 360 } else { rot_deg };
 338 |                     match rot_normalized {
 339 |                         90 => img = img.rotate90(),
 340 |                         180 => img = img.rotate180(),
 341 |                         270 => img = img.rotate270(),
 342 |                         _ => {}
 343 |                     }
 344 | 
 345 |                     if (adjustments.brightness - 1.0).abs() > 0.01
 346 |                         || (adjustments.contrast - 1.0).abs() > 0.01
 347 |                     {
 348 |                         let b = (adjustments.brightness - 1.0) * 255.0;
 349 |                         let c = adjustments.contrast;
 350 |                         img = img.adjust_contrast(c);
 351 |                         if b != 0.0 {
 352 |                             img = img.brighten(b as i32);
 353 |                         }
 354 |                     }
 355 | 
 356 |                     if (adjustments.saturation - 1.0).abs() > 0.01 {
 357 |                         let sat = adjustments.saturation;
 358 |                         let mut rgba = img.to_rgba8();
 359 |                         for px in rgba.pixels_mut() {
 360 |                             let r = px[0] as f32 / 255.0;
 361 |                             let g = px[1] as f32 / 255.0;
 362 |                             let b = px[2] as f32 / 255.0;
 363 |                             let gray = 0.299 * r + 0.587 * g + 0.114 * b;
 364 |                             px[0] = ((gray + (r - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 365 |                             px[1] = ((gray + (g - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 366 |                             px[2] = ((gray + (b - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 367 |                         }
 368 |                         img = image::DynamicImage::ImageRgba8(rgba);
 369 |                     }
 370 | 
 371 |                     if adjustments.hdr_toning {
 372 |                         let mut rgba = img.to_rgba8();
 373 |                         for px in rgba.pixels_mut() {
 374 |                             for c in 0..3 {
 375 |                                 let mut color = (px[c] as f32 / 255.0) * 1.6;
 376 |                                 color = color / (1.0 + color);
 377 |                                 color = color * color * (3.0 - 2.0 * color);
 378 |                                 px[c] = (color.clamp(0.0, 1.0) * 255.0) as u8;
 379 |                             }
 380 |                         }
 381 |                         img = image::DynamicImage::ImageRgba8(rgba);
 382 |                     }
 383 | 
 384 |                     ImageBackend::save(&save_path_clone, &img, 90)?;
 385 |                     Ok(())
 386 |                 })();
 387 | 
 388 |                 let event = match result {
 389 |                     Ok(()) => AppEvent::SaveComplete(save_path_clone),
 390 |                     Err(e) => AppEvent::SaveError(e.to_string()),
 391 |                 };
 392 |                 if let Some(ref proxy) = proxy {
 393 |                     send_event(&tx, proxy, event);
 394 |                 }
 395 |             });
 396 | 
 397 |             self.ui_state.set_status(format!(
 398 |                 "Saving to {}",
 399 |                 save_path.file_name().unwrap_or_default().to_string_lossy()
 400 |             ));
 401 |         }
 402 |     }
 403 | 
 404 |     pub(crate) fn batch_save_selected(&mut self) {
 405 |         let selected: Vec<PathBuf> = self
 406 |             .ui_state
 407 |             .selected_indices
 408 |             .iter()
 409 |             .filter_map(|&i| self.ui_state.files.get(i).map(|f| f.path.clone()))
 410 |             .collect();
 411 | 
 412 |         if selected.is_empty() {
 413 |             self.ui_state
 414 |                 .set_status("No images selected for batch save");
 415 |             self.dirty = true;
 416 |             return;
 417 |         }
 418 | 
 419 |         let adjustments = self.ui_state.adjustments;
 420 |         let tx = self.event_tx.clone();
 421 |         let proxy = self.event_proxy.clone();
 422 | 
 423 |         self.ui_state
 424 |             .set_status(format!("Batch saving {} images...", selected.len()));
 425 |         self.dirty = true;
 426 | 
 427 |         std::thread::spawn(move || {
 428 |             let mut saved_count = 0;
 429 |             let mut errors = 0;
 430 | 
 431 |             for path in &selected {
 432 |                 let result = (|| -> anyhow::Result<()> {
 433 |                     let mut img = image::open(path)?;
 434 | 
 435 |                     if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
 436 |                         let (w, h) = (img.width() as f32, img.height() as f32);
 437 |                         let crop_x = (adjustments.crop_rect[0] * w) as u32;
 438 |                         let crop_y = (adjustments.crop_rect[1] * h) as u32;
 439 |                         let crop_w = (adjustments.crop_rect[2] * w) as u32;
 440 |                         let crop_h = (adjustments.crop_rect[3] * h) as u32;
 441 |                         img = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
 442 |                     }
 443 | 
 444 |                     let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round() as i32;
 445 |                     let rot_normalized = if rot_deg < 0 { rot_deg + 360 } else { rot_deg };
 446 |                     match rot_normalized {
 447 |                         90 => img = img.rotate90(),
 448 |                         180 => img = img.rotate180(),
 449 |                         270 => img = img.rotate270(),
 450 |                         _ => {}
 451 |                     }
 452 | 
 453 |                     if (adjustments.brightness - 1.0).abs() > 0.01
 454 |                         || (adjustments.contrast - 1.0).abs() > 0.01
 455 |                     {
 456 |                         let b = (adjustments.brightness - 1.0) * 255.0;
 457 |                         let c = adjustments.contrast;
 458 |                         img = img.adjust_contrast(c);
 459 |                         if b != 0.0 {
 460 |                             img = img.brighten(b as i32);
 461 |                         }
 462 |                     }
 463 | 
 464 |                     if (adjustments.saturation - 1.0).abs() > 0.01 {
 465 |                         let sat = adjustments.saturation;
 466 |                         let mut rgba = img.to_rgba8();
 467 |                         for px in rgba.pixels_mut() {
 468 |                             let r = px[0] as f32 / 255.0;
 469 |                             let g = px[1] as f32 / 255.0;
 470 |                             let b = px[2] as f32 / 255.0;
 471 |                             let gray = 0.299 * r + 0.587 * g + 0.114 * b;
 472 |                             px[0] = ((gray + (r - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 473 |                             px[1] = ((gray + (g - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 474 |                             px[2] = ((gray + (b - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
 475 |                         }
 476 |                         img = image::DynamicImage::ImageRgba8(rgba);
 477 |                     }
 478 | 
 479 |                     if adjustments.hdr_toning {
 480 |                         let mut rgba = img.to_rgba8();
 481 |                         for px in rgba.pixels_mut() {
 482 |                             for c in 0..3 {
 483 |                                 let mut color = (px[c] as f32 / 255.0) * 1.6;
 484 |                                 color = color / (1.0 + color);
 485 |                                 color = color * color * (3.0 - 2.0 * color);
 486 |                                 px[c] = (color.clamp(0.0, 1.0) * 255.0) as u8;
 487 |                             }
 488 |                         }
 489 |                         img = image::DynamicImage::ImageRgba8(rgba);
 490 |                     }
 491 | 
 492 |                     if let Some(stem) = path.file_stem() {
 493 |                         let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
 494 |                         let stem_lossy = stem.to_string_lossy();
 495 |                         let mut save_path = path.clone();
 496 |                         save_path.set_file_name(format!("{stem_lossy}_edited.{ext}"));
 497 |                         ImageBackend::save(&save_path, &img, 90)?;
 498 |                     }
 499 |                     Ok(())
 500 |                 })();
 501 | 
 502 |                 match result {
 503 |                     Ok(()) => saved_count += 1,
 504 |                     Err(e) => {
 505 |                         tracing::warn!("Failed to batch save {:?}: {}", path, e);
 506 |                         errors += 1;
 507 |                     }
 508 |                 }
 509 |             }
 510 | 
 511 |             let msg = if errors > 0 {
 512 |                 format!("Batch save: {} saved, {} errors", saved_count, errors)
 513 |             } else {
 514 |                 format!("Batch save complete: {} images saved", saved_count)
 515 |             };
 516 | 
 517 |             if let Some(ref proxy) = proxy {
 518 |                 send_event(&tx, proxy, AppEvent::SetStatus(msg));
 519 |             }
 520 |         });
 521 |     }
 522 | 
 523 |     pub(crate) fn batch_delete_selected(&mut self) {
 524 |         let selected: Vec<PathBuf> = self
 525 |             .ui_state
 526 |             .selected_indices
 527 |             .iter()
 528 |             .filter_map(|&i| self.ui_state.files.get(i).map(|f| f.path.clone()))
 529 |             .collect();
 530 | 
 531 |         if selected.is_empty() {
 532 |             self.ui_state
 533 |                 .set_status("No images selected for batch delete");
 534 |             self.dirty = true;
 535 |             return;
 536 |         }
 537 | 
 538 |         let count = selected.len();
 539 |         let confirmed = rfd::MessageDialog::new()
 540 |             .set_title("Delete Images")
 541 |             .set_description(format!("Delete {} selected images?", count))
 542 |             .set_buttons(rfd::MessageButtons::YesNo)
 543 |             .show()
 544 |             == rfd::MessageDialogResult::Yes;
 545 | 
 546 |         if !confirmed {
 547 |             return;
 548 |         }
 549 | 
 550 |         let mut deleted = 0;
 551 |         let mut errors = 0;
 552 | 
 553 |         for path in &selected {
 554 |             match std::fs::remove_file(path) {
 555 |                 Ok(()) => deleted += 1,
 556 |                 Err(e) => {
 557 |                     tracing::warn!("Failed to delete {:?}: {}", path, e);
 558 |                     errors += 1;
 559 |                 }
 560 |             }
 561 |         }
 562 | 
 563 |         let msg = if errors > 0 {
 564 |             format!("Deleted {}, {} errors", deleted, errors)
 565 |         } else {
 566 |             format!("Deleted {} images", deleted)
 567 |         };
 568 | 
 569 |         self.ui_state.set_status(msg);
 570 |         self.ui_state.selected_indices.clear();
 571 | 
 572 |         if let Some(current) = self.ui_state.current_file() {
 573 |             if let Some(parent) = current.parent() {
 574 |                 self.ui_state.load_directory(parent.to_path_buf());
 575 |             }
 576 |         }
 577 | 
 578 |         self.dirty = true;
 579 |     }
 580 | 
 581 |     pub(crate) fn next_image(&mut self) {
 582 |         self.ui_state.next_file();
 583 |         if let Some(file) = self.ui_state.current_file() {
 584 |             self.load_image(file.clone().to_path_buf());
 585 |         }
 586 |     }
 587 | 
 588 |     pub(crate) fn prev_image(&mut self) {
 589 |         self.ui_state.prev_file();
 590 |         if let Some(file) = self.ui_state.current_file() {
 591 |             self.load_image(file.clone().to_path_buf());
 592 |         }
 593 |     }
 594 | 
 595 |     pub(crate) fn rotate_image(&mut self) {
 596 |         self.ui_state.rotate_90();
 597 |         self.dirty = true;
 598 |     }
 599 | 
 600 |     pub(crate) fn toggle_crop(&mut self) {
 601 |         self.ui_state.is_cropping = !self.ui_state.is_cropping;
 602 |         if !self.ui_state.is_cropping {
 603 |             self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
 604 |             self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
 605 |         }
 606 |         self.dirty = true;
 607 |     }
 608 | 
 609 |     pub(crate) fn toggle_hdr_toning(&mut self) {
 610 |         self.ui_state.adjustments.hdr_toning = !self.ui_state.adjustments.hdr_toning;
 611 |         let label = if self.ui_state.adjustments.hdr_toning {
 612 |             "ON"
 613 |         } else {
 614 |             "OFF"
 615 |         };
 616 |         self.ui_state.set_status(format!("HDR Toning: {label}"));
 617 |         self.dirty = true;
 618 |     }
 619 | 
 620 |     pub(crate) fn cancel_crop(&mut self) {
 621 |         self.ui_state.is_cropping = false;
 622 |         self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
 623 |         self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
 624 |         self.dirty = true;
 625 |     }
 626 | 
 627 |     pub(crate) fn pick_color_at(&mut self, cursor: PhysicalPosition<f64>) {
 628 |         let (img, window) = match (&self.current_image, &self.window) {
 629 |             (Some(i), Some(w)) => (i, w),
 630 |             _ => return,
 631 |         };
 632 | 
 633 |         let size = window.inner_size();
 634 |         if size.width == 0 || size.height == 0 {
 635 |             return;
 636 |         }
 637 | 
 638 |         let available_h = if self.show_thumbnail_strip {
 639 |             (size.height as i32 - STRIP_HEIGHT_PX as i32).max(1) as f64
 640 |         } else {
 641 |             size.height as f64
 642 |         };
 643 | 
 644 |         let image_ar = img.width as f32 / img.height as f32;
 645 |         let window_ar = size.width as f32 / available_h as f32;
 646 |         let ratio = image_ar / window_ar;
 647 | 
 648 |         let (img_half_w, img_half_h) = if ratio > 1.0 {
 649 |             (1.0_f32, 1.0 / ratio)
 650 |         } else {
 651 |             (ratio, 1.0_f32)
 652 |         };
 653 | 
 654 |         let wx = (cursor.x / size.width as f64) as f32 * 2.0 - 1.0;
 655 |         let wy = (cursor.y / available_h) as f32 * 2.0 - 1.0;
 656 | 
 657 |         if wx < -img_half_w || wx > img_half_w || wy < -img_half_h || wy > img_half_h {
 658 |             self.ui_state.set_status("Color Picker: click on the image");
 659 |             self.dirty = true;
 660 |             return;
 661 |         }
 662 | 
 663 |         let u_img = (wx + img_half_w) / (2.0 * img_half_w);
 664 |         let v_img = (wy + img_half_h) / (2.0 * img_half_h);
 665 | 
 666 |         let cr = &self.ui_state.adjustments.crop_rect;
 667 |         let u = cr[0] + u_img * cr[2];
 668 |         let v = cr[1] + v_img * cr[3];
 669 |         let u = u.clamp(0.0, 1.0);
 670 |         let v = v.clamp(0.0, 1.0);
 671 | 
 672 |         let px = (u * (img.width - 1) as f32) as usize;
 673 |         let py = (v * (img.height - 1) as f32) as usize;
 674 |         let idx = (py * img.width as usize + px) * 4;
 675 | 
 676 |         if idx + 3 >= img.rgba_data.len() {
 677 |             return;
 678 |         }
 679 | 
 680 |         let r = img.rgba_data[idx];
 681 |         let g = img.rgba_data[idx + 1];
 682 |         let b = img.rgba_data[idx + 2];
 683 | 
 684 |         self.ui_state.set_status(format!(
 685 |             "Color Picker: R:{} G:{} B:{}  #{:02X}{:02X}{:02X}",
 686 |             r, g, b, r, g, b
 687 |         ));
 688 |         self.dirty = true;
 689 |     }
 690 | 
 691 |     pub(crate) fn reset_adjustments(&mut self) {
 692 |         self.ui_state.reset_adjustments();
 693 |         self.ui_state.set_status("Adjustments reset");
 694 |         self.dirty = true;
 695 |     }
 696 | 
 697 |     pub(crate) fn toggle_sidebar(&mut self) {
 698 |         self.show_sidebar = !self.show_sidebar;
 699 |         self.dirty = true;
 700 |     }
 701 | 
 702 |     pub(crate) fn toggle_thumbnail_strip(&mut self) {
 703 |         self.show_thumbnail_strip = !self.show_thumbnail_strip;
 704 |         self.dirty = true;
 705 |     }
 706 | 
 707 |     pub(crate) fn rename_current_image(&mut self) {
 708 |         if let Some(img) = &self.current_image {
 709 |             let old_path = std::path::PathBuf::from(&img.path);
 710 |             let filename = old_path
 711 |                 .file_name()
 712 |                 .unwrap_or_default()
 713 |                 .to_string_lossy()
 714 |                 .into_owned();
 715 | 
 716 |             let tx = self.event_tx.clone();
 717 |             let proxy = self.event_proxy.clone();
 718 | 
 719 |             std::thread::spawn(move || {
 720 |                 if let Some(new_path) = rfd::FileDialog::new()
 721 |                     .set_title("Rename File")
 722 |                     .set_file_name(&filename)
 723 |                     .save_file()
 724 |                 {
 725 |                     if let Err(e) = std::fs::rename(&old_path, &new_path) {
 726 |                         if let Some(ref p) = proxy {
 727 |                             send_event(&tx, p, AppEvent::SetStatus(format!("Rename failed: {e}")));
 728 |                         }
 729 |                     } else if let Some(ref p) = proxy {
 730 |                         send_event(&tx, p, AppEvent::FileRenamed(old_path, new_path));
 731 |                     }
 732 |                 }
 733 |             });
 734 |         }
 735 |     }
 736 | 
 737 |     pub(crate) fn set_as_wallpaper(&mut self) {
 738 |         #[cfg(windows)]
 739 |         {
 740 |             if let Some(img) = &self.current_image {
 741 |                 let p = &img.path;
 742 |                 tracing::info!("Setting wallpaper: {p}");
 743 |                 use std::os::windows::ffi::OsStrExt;
 744 |                 use windows::Win32::UI::WindowsAndMessaging::{
 745 |                     SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE,
 746 |                     SPI_SETDESKWALLPAPER,
 747 |                 };
 748 | 
 749 |                 let path = std::path::Path::new(&img.path);
 750 |                 let abs_path = if path.is_absolute() {
 751 |                     path.to_path_buf()
 752 |                 } else {
 753 |                     std::env::current_dir().unwrap_or_default().join(path)
 754 |                 };
 755 | 
 756 |                 let mut path_wide: Vec<u16> = abs_path.as_os_str().encode_wide().collect();
 757 |                 path_wide.push(0);
 758 | 
 759 |                 unsafe {
 760 |                     let _ = SystemParametersInfoW(
 761 |                         SPI_SETDESKWALLPAPER,
 762 |                         0,
 763 |                         Some(path_wide.as_mut_ptr() as *mut _),
 764 |                         SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
 765 |                     );
 766 |                 }
 767 |                 self.ui_state.set_status("Desktop wallpaper set!");
 768 |                 self.dirty = true;
 769 |             }
 770 |         }
 771 |         #[cfg(not(windows))]
 772 |         {
 773 |             self.ui_state
 774 |                 .set_status("Wallpaper setting not supported on this OS");
 775 |             self.dirty = true;
 776 |         }
 777 |     }
 778 | 
 779 |     pub(crate) fn show_context_menu(&mut self) {
 780 |         #[cfg(windows)]
 781 |         {
 782 |             if let Some(ref w) = self.window {
 783 |                 use std::os::windows::ffi::OsStrExt;
 784 |                 use windows::core::PCWSTR;
 785 |                 use windows::Win32::Foundation::HWND;
 786 |                 use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
 787 |                 use windows::Win32::UI::WindowsAndMessaging::{
 788 |                     AppendMenuW, CreatePopupMenu, TrackPopupMenu, MF_STRING, TPM_NONOTIFY,
 789 |                     TPM_RETURNCMD,
 790 |                 };
 791 | 
 792 |                 unsafe {
 793 |                     let hmenu = CreatePopupMenu().unwrap_or_default();
 794 |                     if hmenu.is_invalid() {
 795 |                         return;
 796 |                     }
 797 | 
 798 |                     let mut id = 1;
 799 |                     let items = [
 800 |                         "Open in Explorer",
 801 |                         "Copy (Ctrl+C)",
 802 |                         "Rename (F2)",
 803 |                         "Delete (Del)",
 804 |                         "Set as Wallpaper (Ctrl+W)",
 805 |                     ];
 806 | 
 807 |                     for item in &items {
 808 |                         let mut wide: Vec<u16> = std::ffi::OsStr::new(item).encode_wide().collect();
 809 |                         wide.push(0);
 810 |                         let _ = AppendMenuW(hmenu, MF_STRING, id, PCWSTR(wide.as_ptr()));
 811 |                         id += 1;
 812 |                     }
 813 | 
 814 |                     let mut pt = windows::Win32::Foundation::POINT::default();
 815 |                     let _ = GetCursorPos(&mut pt);
 816 | 
 817 |                     use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
 818 |                     let hwnd = if let Ok(handle) = w.window_handle() {
 819 |                         match handle.as_raw() {
 820 |                             RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut _),
 821 |                             _ => HWND::default(),
 822 |                         }
 823 |                     } else {
 824 |                         HWND::default()
 825 |                     };
 826 | 
 827 |                     let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
 828 | 
 829 |                     let cmd = TrackPopupMenu(
 830 |                         hmenu,
 831 |                         TPM_RETURNCMD | TPM_NONOTIFY,
 832 |                         pt.x,
 833 |                         pt.y,
 834 |                         0,
 835 |                         hwnd,
 836 |                         None,
 837 |                     );
 838 | 
 839 |                     let _ = windows::Win32::UI::WindowsAndMessaging::DestroyMenu(hmenu);
 840 | 
 841 |                     match cmd.0 {
 842 |                         1 => self.open_in_explorer(),
 843 |                         2 => self.copy_to_clipboard(),
 844 |                         3 => self.rename_current_image(),
 845 |                         4 => self.delete_current_image(),
 846 |                         5 => {
 847 |                             self.set_as_wallpaper();
 848 |                         }
 849 |                         _ => {}
 850 |                     }
 851 |                 }
 852 |             }
 853 |         }
 854 |     }
 855 | 
 856 |     #[cfg(windows)]
 857 |     pub(crate) fn open_in_explorer(&self) {
 858 |         #[cfg(windows)]
 859 |         {
 860 |             if let Some(img) = &self.current_image {
 861 |                 let path = std::path::Path::new(&img.path);
 862 |                 let abs_path = if path.is_absolute() {
 863 |                     path.to_path_buf()
 864 |                 } else {
 865 |                     std::env::current_dir().unwrap_or_default().join(path)
 866 |                 };
 867 | 
 868 |                 use std::os::windows::ffi::OsStrExt;
 869 |                 use windows::core::PCWSTR;
 870 |                 use windows::Win32::UI::Shell::ShellExecuteW;
 871 |                 use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
 872 |                 let abs_path_disp = abs_path.display();
 873 |                 let arg = format!("/select,\"{abs_path_disp}\"");
 874 | 
 875 |                 let verb: Vec<u16> = std::ffi::OsStr::new("open")
 876 |                     .encode_wide()
 877 |                     .chain(std::iter::once(0))
 878 |                     .collect();
 879 |                 let file: Vec<u16> = std::ffi::OsStr::new("explorer.exe")
 880 |                     .encode_wide()
 881 |                     .chain(std::iter::once(0))
 882 |                     .collect();
 883 |                 let params: Vec<u16> = std::ffi::OsStr::new(&arg)
 884 |                     .encode_wide()
 885 |                     .chain(std::iter::once(0))
 886 |                     .collect();
 887 | 
 888 |                 unsafe {
 889 |                     let _ = ShellExecuteW(
 890 |                         None,
 891 |                         PCWSTR(verb.as_ptr()),
 892 |                         PCWSTR(file.as_ptr()),
 893 |                         PCWSTR(params.as_ptr()),
 894 |                         None,
 895 |                         SW_SHOW,
 896 |                     );
 897 |                 }
 898 |             }
 899 |         }
 900 |     }
 901 | 
 902 |     pub(crate) fn print_image(&self) {
 903 |         #[cfg(windows)]
 904 |         {
 905 |             if let Some(img) = &self.current_image {
 906 |                 let p = &img.path;
 907 |                 tracing::info!("Printing image: {p}");
 908 |                 use std::os::windows::ffi::OsStrExt;
 909 |                 use windows::core::PCWSTR;
 910 |                 use windows::Win32::UI::Shell::ShellExecuteW;
 911 |                 use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
 912 | 
 913 |                 let verb: Vec<u16> = std::ffi::OsStr::new("print")
 914 |                     .encode_wide()
 915 |                     .chain(std::iter::once(0))
 916 |                     .collect();
 917 | 
 918 |                 let file: Vec<u16> = std::ffi::OsStr::new(&img.path)
 919 |                     .encode_wide()
 920 |                     .chain(std::iter::once(0))
 921 |                     .collect();
 922 | 
 923 |                 unsafe {
 924 |                     ShellExecuteW(
 925 |                         None,
 926 |                         PCWSTR(verb.as_ptr()),
 927 |                         PCWSTR(file.as_ptr()),
 928 |                         None,
 929 |                         None,
 930 |                         SW_SHOW,
 931 |                     );
 932 |                 }
 933 |             }
 934 |         }
 935 |         #[cfg(not(windows))]
 936 |         {
 937 |             tracing::warn!("Print not currently supported on this platform.");
 938 |         }
 939 |     }
 940 | 
 941 |     pub(crate) fn copy_to_clipboard(&mut self) {
 942 |         if let Some(img) = &self.current_image {
 943 |             let path = std::path::PathBuf::from(&img.path);
 944 |             self.ui_state.set_status("Copying to clipboard...");
 945 |             self.dirty = true;
 946 | 
 947 |             let tx = self.event_tx.clone();
 948 |             let proxy = self.event_proxy.clone();
 949 |             std::thread::spawn(move || {
 950 |                 let res = Self::do_copy_to_clipboard(&path);
 951 |                 if let Some(ref p) = proxy {
 952 |                     if let Err(e) = res {
 953 |                         send_event(&tx, p, AppEvent::SetStatus(format!("Copy failed: {e}")));
 954 |                     } else {
 955 |                         send_event(
 956 |                             &tx,
 957 |                             p,
 958 |                             AppEvent::SetStatus("Copied to clipboard".to_string()),
 959 |                         );
 960 |                     }
 961 |                 }
 962 |             });
 963 |         }
 964 |     }
 965 | 
 966 |     pub(crate) fn do_copy_to_clipboard(path: &Path) -> Result<()> {
 967 |         let img = image::open(path)?;
 968 | 
 969 |         #[cfg(target_os = "linux")]
 970 |         {
 971 |             let mut png_data = Vec::new();
 972 |             img.write_to(
 973 |                 &mut std::io::Cursor::new(&mut png_data),
 974 |                 image::ImageFormat::Png,
 975 |             )?;
 976 | 
 977 |             let child = std::process::Command::new("wl-copy")
 978 |                 .arg("-t")
 979 |                 .arg("image/png")
 980 |                 .stdin(std::process::Stdio::piped())
 981 |                 .spawn();
 982 | 
 983 |             if let Ok(mut c) = child {
 984 |                 if let Some(mut stdin) = c.stdin.take() {
 985 |                     use std::io::Write;
 986 |                     let _ = stdin.write_all(&png_data);
 987 |                 }
 988 |                 let _ = c.wait();
 989 |                 return Ok(());
 990 |             }
 991 | 
 992 |             let child = std::process::Command::new("xclip")
 993 |                 .args(&["-selection", "clipboard", "-t", "image/png"])
 994 |                 .stdin(std::process::Stdio::piped())
 995 |                 .spawn();
 996 | 
 997 |             if let Ok(mut c) = child {
 998 |                 if let Some(mut stdin) = c.stdin.take() {
 999 |                     use std::io::Write;
1000 |                     let _ = stdin.write_all(&png_data);
1001 |                 }
1002 |                 let _ = c.wait();
1003 |                 return Ok(());
1004 |             }
1005 | 
1006 |             anyhow::bail!("Neither wl-copy nor xclip found");
1007 |         }
1008 | 
1009 |         #[cfg(target_os = "windows")]
1010 |         {
1011 |             use windows::Win32::Foundation::HANDLE;
1012 |             use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
1013 |             use windows::Win32::System::DataExchange::{
1014 |                 CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
1015 |             };
1016 |             use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
1017 | 
1018 |             let rgba = img.into_rgba8();
1019 |             let (width, height) = rgba.dimensions();
1020 |             let mut bgra = rgba.into_raw();
1021 |             for chunk in bgra.chunks_exact_mut(4) {
1022 |                 chunk.swap(0, 2); // RGBA -> BGRA
1023 |             }
1024 | 
1025 |             let stride = (width * 4) as usize;
1026 |             let mut flipped = vec![0u8; bgra.len()];
1027 |             for (y, row) in bgra.chunks_exact(stride).enumerate() {
1028 |                 let flipped_y = (height as usize - 1) - y;
1029 |                 flipped[flipped_y * stride..(flipped_y + 1) * stride].copy_from_slice(row);
1030 |             }
1031 | 
1032 |             let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
1033 |             let size = header_size + flipped.len();
1034 | 
1035 |             unsafe {
1036 |                 let hmem = GlobalAlloc(GHND, size)?;
1037 |                 let ptr = GlobalLock(hmem) as *mut u8;
1038 | 
1039 |                 let header = BITMAPINFOHEADER {
1040 |                     biSize: header_size as u32,
1041 |                     biWidth: width as i32,
1042 |                     biHeight: height as i32,
1043 |                     biPlanes: 1,
1044 |                     biBitCount: 32,
1045 |                     biCompression: BI_RGB.0,
1046 |                     biSizeImage: flipped.len() as u32,
1047 |                     ..Default::default()
1048 |                 };
1049 | 
1050 |                 std::ptr::copy_nonoverlapping(&header as *const _ as *const u8, ptr, header_size);
1051 |                 std::ptr::copy_nonoverlapping(
1052 |                     flipped.as_ptr(),
1053 |                     ptr.add(header_size),
1054 |                     flipped.len(),
1055 |                 );
1056 |                 let _ = GlobalUnlock(hmem);
1057 | 
1058 |                 if OpenClipboard(None).is_ok() {
1059 |                     let _ = EmptyClipboard();
1060 |                     let _ = SetClipboardData(8, HANDLE(hmem.0 as *mut _)); // 8 is CF_DIB
1061 |                     let _ = CloseClipboard();
1062 |                 } else {
1063 |                     anyhow::bail!("Failed to open clipboard");
1064 |                 }
1065 |             }
1066 |             Ok(())
1067 |         }
1068 | 
1069 |         #[cfg(not(any(target_os = "linux", target_os = "windows")))]
1070 |         {
1071 |             anyhow::bail!("Native clipboard not implemented on this OS");
1072 |         }
1073 |     }
1074 | 
1075 |     pub(crate) fn open_file_dialog(&mut self) {
1076 |         let tx = self.event_tx.clone();
1077 |         let proxy = self.event_proxy.clone();
1078 |         std::thread::spawn(move || {
1079 |             if let Some(path) = rfd::FileDialog::new()
1080 |                 .add_filter("Images", &ImageBackend::supported_extensions())
1081 |                 .pick_file()
1082 |             {
1083 |                 if let Some(ref proxy) = proxy {
1084 |                     send_event(&tx, proxy, AppEvent::OpenPath(path));
1085 |                 }
1086 |             }
1087 |         });
1088 |     }
1089 | 
1090 |     pub(crate) fn zoom_in(&mut self, cursor: Option<PhysicalPosition<f64>>) {
1091 |         self.zoom_by(0.9, cursor);
1092 |     }
1093 | 
1094 |     pub(crate) fn zoom_out(&mut self, cursor: Option<PhysicalPosition<f64>>) {
1095 |         self.zoom_by(1.1, cursor);
1096 |     }
1097 | 
1098 |     pub(crate) fn zoom_by(&mut self, factor: f32, cursor: Option<PhysicalPosition<f64>>) {
1099 |         let old_w = self.ui_state.adjustments.crop_rect[2];
1100 |         let old_h = self.ui_state.adjustments.crop_rect[3];
1101 |         let new_w = (old_w * factor).clamp(0.05, 1.0);
1102 |         let new_h = (old_h * factor).clamp(0.05, 1.0);
1103 | 
1104 |         if let (Some(pos), Some(ref w)) = (cursor, &self.window) {
1105 |             let win_size = w.inner_size();
1106 |             if win_size.width > 0 && win_size.height > 0 {
1107 |                 let cx = (pos.x as f32 / win_size.width as f32)
1108 |                     .mul_add(old_w, self.ui_state.adjustments.crop_rect[0]);
1109 |                 let cy = (pos.y as f32 / win_size.height as f32)
1110 |                     .mul_add(old_h, self.ui_state.adjustments.crop_rect[1]);
1111 |                 self.ui_state.adjustments.crop_rect[0] = (cx
1112 |                     - new_w * (pos.x as f32 / win_size.width as f32))
1113 |                     .max(0.0)
1114 |                     .min(1.0 - new_w);
1115 |                 self.ui_state.adjustments.crop_rect[1] = (cy
1116 |                     - new_h * (pos.y as f32 / win_size.height as f32))
1117 |                     .max(0.0)
1118 |                     .min(1.0 - new_h);
1119 |             }
1120 |         }
1121 | 
1122 |         self.ui_state.adjustments.crop_rect[2] = new_w;
1123 |         self.ui_state.adjustments.crop_rect[3] = new_h;
1124 |         self.dirty = true;
1125 |     }
1126 | 
1127 |     pub(crate) fn zoom_fit(&mut self) {
1128 |         self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
1129 |         self.dirty = true;
1130 |     }
1131 | 
1132 |     pub(crate) fn active_thumb_index(&self) -> Option<usize> {
1133 |         let current = self.ui_state.current_file()?;
1134 |         self.thumb_paths.iter().position(|p| p == current)
1135 |     }
1136 | }
1137 | 
1138 | #[cfg(test)]
1139 | mod tests {
1140 |     use super::*;
1141 | 
1142 |     #[test]
1143 |     fn test_slideshow_toggle() {
1144 |         let mut app = SpedImageApp::new();
1145 |         assert!(!app.slideshow_active);
1146 | 
1147 |         app.toggle_slideshow();
1148 |         assert!(app.slideshow_active);
1149 |         assert!(app.slideshow_next_time.is_some());
1150 | 
1151 |         app.toggle_slideshow();
1152 |         assert!(!app.slideshow_active);
1153 |         assert!(app.slideshow_next_time.is_none());
1154 |     }
1155 | 
1156 |     #[test]
1157 |     fn test_slideshow_interval() {
1158 |         let mut app = SpedImageApp::new();
1159 |         let initial = app.slideshow_interval;
1160 | 
1161 |         app.adjust_slideshow_interval(1);
1162 |         assert_eq!(
1163 |             app.slideshow_interval,
1164 |             initial + std::time::Duration::from_secs(1)
1165 |         );
1166 | 
1167 |         app.adjust_slideshow_interval(-2);
1168 |         assert_eq!(
1169 |             app.slideshow_interval,
1170 |             initial - std::time::Duration::from_secs(1)
1171 |         );
1172 |     }
1173 | }
```

### File: `src\app\events.rs`

- Size: 25166 bytes
- Modified: 2026-03-07 12:21:05 UTC

```rust
   1 | use crate::app::state::SpedImageApp;
   2 | use crate::app::types::{AppEvent, WakeUp, APP_ICON};
   3 | use crate::gpu_renderer::{Renderer, STRIP_HEIGHT_PX};
   4 | use anyhow::Result;
   5 | use std::path::PathBuf;
   6 | use std::sync::Arc;
   7 | use winit::application::ApplicationHandler;
   8 | use winit::event::{ElementState, MouseButton, WindowEvent};
   9 | use winit::event_loop::{ActiveEventLoop, EventLoop};
  10 | use winit::window::{Icon, Window, WindowId};
  11 | 
  12 | impl SpedImageApp {
  13 |     pub fn run(initial_path: Option<PathBuf>) -> Result<()> {
  14 |         let event_loop: EventLoop<WakeUp> = EventLoop::with_user_event().build()?;
  15 |         event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
  16 |         let mut app = Self::new();
  17 |         app.event_proxy = Some(event_loop.create_proxy());
  18 |         app.initial_path = initial_path;
  19 |         event_loop.run_app(&mut app)?;
  20 |         Ok(())
  21 |     }
  22 | 
  23 |     pub(crate) fn process_events(&mut self) {
  24 |         while let Ok(event) = self.event_rx.try_recv() {
  25 |             match event {
  26 |                 AppEvent::SaveComplete(path) => {
  27 |                     self.dirty = true;
  28 |                     self.ui_state.set_status(format!(
  29 |                         "Saved: {}",
  30 |                         path.file_name().unwrap_or_default().to_string_lossy()
  31 |                     ));
  32 |                 }
  33 |                 AppEvent::SaveError(e) => {
  34 |                     self.dirty = true;
  35 |                     self.ui_state.set_status(format!("Save failed: {e}"));
  36 |                 }
  37 |                 AppEvent::SetStatus(msg) => {
  38 |                     self.dirty = true;
  39 |                     self.ui_state.set_status(msg);
  40 |                 }
  41 |                 AppEvent::Prefetched(path, frames) => {
  42 |                     self.prefetch_cache.push(path, frames);
  43 |                 }
  44 |                 AppEvent::ThumbnailLoaded(path, rgba, width, height) => {
  45 |                     if let Some(ref mut renderer) = self.renderer {
  46 |                         let already_have = renderer.thumbnails.iter().any(|t| t.path == path);
  47 |                         if !already_have {
  48 |                             if let Err(e) = renderer.upload_thumbnail(path, &rgba, width, height) {
  49 |                                 tracing::warn!("Failed to upload thumbnail: {e}");
  50 |                             } else {
  51 |                                 self.dirty = true;
  52 |                             }
  53 |                         }
  54 |                     }
  55 |                 }
  56 |                 AppEvent::ImageLoaded(mut frames) => {
  57 |                     self.ui_state.reset_adjustments();
  58 |                     self.dirty = true;
  59 |                     if frames.is_empty() {
  60 |                         continue;
  61 |                     }
  62 |                     let mut first_frame = frames.remove(0);
  63 |                     let path = PathBuf::from(&first_frame.path);
  64 | 
  65 |                     let frame_delays: Vec<u32> = frames.iter().map(|f| f.frame_delay_ms).collect();
  66 | 
  67 |                     let new_dir = path.parent().map(|p| p.to_path_buf());
  68 |                     let old_dir = self
  69 |                         .ui_state
  70 |                         .current_file()
  71 |                         .and_then(|p| p.parent().map(|p| p.to_path_buf()));
  72 | 
  73 |                     if let Some(parent) = path.parent() {
  74 |                         self.ui_state.load_directory(parent.to_path_buf());
  75 |                     }
  76 |                     for (i, f) in self.ui_state.files.iter().enumerate() {
  77 |                         if f.path == path {
  78 |                             self.ui_state.current_file_index = Some(i);
  79 |                             break;
  80 |                         }
  81 |                     }
  82 | 
  83 |                     if new_dir != old_dir || self.thumb_paths.is_empty() {
  84 |                         self.load_thumbnails_for_dir();
  85 |                         if let Some(parent) = path.parent() {
  86 |                             self.setup_file_watcher(parent);
  87 |                         }
  88 |                     }
  89 | 
  90 |                     if let Some(ref mut renderer) = self.renderer {
  91 |                         if let Err(e) = renderer.load_image(&first_frame) {
  92 |                             tracing::error!("Failed to load image to GPU: {e}");
  93 |                             self.ui_state.set_status("Failed to load image");
  94 |                             self.loading = false;
  95 |                             return;
  96 |                         }
  97 |                         if !frame_delays.is_empty() {
  98 |                             if let Err(e) = renderer.preload_gif_textures(&frames) {
  99 |                                 tracing::warn!("Failed to preload GIF textures: {e}");
 100 |                             }
 101 |                         } else {
 102 |                             for (tex, _) in renderer.gif_textures.drain(..) {
 103 |                                 tex.destroy();
 104 |                             }
 105 |                         }
 106 |                     }
 107 | 
 108 |                     first_frame.rgba_data.clear();
 109 |                     first_frame.rgba_data.shrink_to_fit();
 110 |                     drop(frames);
 111 | 
 112 |                     if !frame_delays.is_empty() && first_frame.frame_delay_ms > 0 {
 113 |                         self.next_frame_time = Some(
 114 |                             std::time::Instant::now()
 115 |                                 + std::time::Duration::from_millis(
 116 |                                     first_frame.frame_delay_ms as u64,
 117 |                                 ),
 118 |                         );
 119 |                     } else if frame_delays.is_empty() {
 120 |                         self.next_frame_time = None;
 121 |                     }
 122 | 
 123 |                     let size_mb = first_frame.file_size_bytes as f64 / 1_048_576.0;
 124 |                     let frame_info = if frame_delays.is_empty() {
 125 |                         String::new()
 126 |                     } else {
 127 |                         let len = frame_delays.len() + 1;
 128 |                         format!("  |  {len} frames")
 129 |                     };
 130 | 
 131 |                     let image_count = self.ui_state.files.iter().filter(|f| f.is_image).count();
 132 |                     let current_idx = self.ui_state.current_file_index.unwrap_or(0) + 1;
 133 | 
 134 |                     self.ui_state.set_status(format!(
 135 |                         "{}/{}  |  {}  |  {}×{}  |  {size_mb:.1} MB{}",
 136 |                         current_idx,
 137 |                         image_count,
 138 |                         path.file_name().unwrap_or_default().to_string_lossy(),
 139 |                         first_frame.width,
 140 |                         first_frame.height,
 141 |                         frame_info
 142 |                     ));
 143 | 
 144 |                     self.current_image = Some(first_frame);
 145 |                     self.current_frame_delays = frame_delays;
 146 |                     self.current_frame_idx = 0;
 147 |                     self.loading = false;
 148 |                 }
 149 |                 AppEvent::ImageError(e) => {
 150 |                     self.dirty = true;
 151 |                     tracing::error!("Failed to load image: {e}");
 152 |                     self.ui_state.set_status(format!("Error: {e}"));
 153 |                     self.loading = false;
 154 |                 }
 155 |                 AppEvent::OpenPath(path) => {
 156 |                     self.load_image(path);
 157 |                 }
 158 |                 AppEvent::FileRenamed(old_path, new_path) => {
 159 |                     if let Some(img) = &mut self.current_image {
 160 |                         if img.path == old_path {
 161 |                             img.path = new_path.to_string_lossy().into_owned();
 162 |                         }
 163 |                     }
 164 |                     for file in &mut self.ui_state.files {
 165 |                         if file.path == old_path {
 166 |                             file.path = new_path.clone();
 167 |                             file.name = new_path
 168 |                                 .file_name()
 169 |                                 .unwrap_or_default()
 170 |                                 .to_string_lossy()
 171 |                                 .into_owned();
 172 |                             break;
 173 |                         }
 174 |                     }
 175 |                     if let Some(frames) = self.prefetch_cache.pop(&old_path) {
 176 |                         self.prefetch_cache.push(new_path.clone(), frames);
 177 |                     }
 178 |                     self.ui_state.set_status(format!(
 179 |                         "Renamed to {}",
 180 |                         new_path.file_name().unwrap_or_default().to_string_lossy()
 181 |                     ));
 182 |                     self.dirty = true;
 183 |                 }
 184 |             }
 185 |         }
 186 |     }
 187 | }
 188 | 
 189 | impl ApplicationHandler<WakeUp> for SpedImageApp {
 190 |     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
 191 |         if self.window.is_none() {
 192 |             let icon = image::load_from_memory(APP_ICON).ok().and_then(|img| {
 193 |                 let rgba = img.to_rgba8();
 194 |                 let (w, h) = rgba.dimensions();
 195 |                 Icon::from_rgba(rgba.into_raw(), w, h).ok()
 196 |             });
 197 | 
 198 |             let mut attrs = Window::default_attributes()
 199 |                 .with_title("SpedImage")
 200 |                 .with_decorations(true);
 201 |             if let Some(icon) = icon {
 202 |                 attrs = attrs.with_window_icon(Some(icon));
 203 |             }
 204 |             let window = match event_loop.create_window(attrs) {
 205 |                 Ok(w) => Arc::new(w),
 206 |                 Err(e) => {
 207 |                     tracing::error!("Failed to create window: {e}");
 208 |                     event_loop.exit();
 209 |                     return;
 210 |                 }
 211 |             };
 212 |             self.window = Some(window.clone());
 213 | 
 214 |             match pollster::block_on(Renderer::new(window.clone())) {
 215 |                 Ok(renderer) => {
 216 |                     self.renderer = Some(renderer);
 217 |                 }
 218 |                 Err(e) => {
 219 |                     tracing::error!("Failed to initialize GPU renderer: {e}");
 220 |                     let _ = rfd::MessageDialog::new()
 221 |                         .set_title("GPU Error")
 222 |                         .set_description(format!(
 223 |                             "Failed to initialize GPU: {}\n\nThe app will exit.",
 224 |                             e
 225 |                         ))
 226 |                         .show();
 227 |                     event_loop.exit();
 228 |                     return;
 229 |                 }
 230 |             }
 231 |             self.dirty = true;
 232 |             if let Some(path) = self.initial_path.take() {
 233 |                 self.load_image(path);
 234 |             }
 235 |         }
 236 |     }
 237 | 
 238 |     fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: WakeUp) {
 239 |         self.dirty = true;
 240 |         if let Some(window) = &self.window {
 241 |             window.request_redraw();
 242 |         }
 243 |     }
 244 | 
 245 |     fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
 246 |         match event {
 247 |             WindowEvent::CloseRequested => {
 248 |                 event_loop.exit();
 249 |             }
 250 |             WindowEvent::Resized(size) => {
 251 |                 if let Some(ref mut renderer) = self.renderer {
 252 |                     renderer.resize(size);
 253 |                     self.dirty = true;
 254 |                 }
 255 |             }
 256 |             WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
 257 |                 if let Some(ref mut renderer) = self.renderer {
 258 |                     renderer.update_scale_factor(scale_factor);
 259 |                     self.dirty = true;
 260 |                 }
 261 |             }
 262 |             WindowEvent::DroppedFile(path) => {
 263 |                 tracing::info!("File dropped: {:?}", path);
 264 |                 self.load_image(path);
 265 |             }
 266 |             WindowEvent::KeyboardInput { event, .. } => {
 267 |                 if event.state == ElementState::Pressed {
 268 |                     self.handle_keyboard(event, event_loop);
 269 |                 } else {
 270 |                     if let Some(c) = event.logical_key.to_text() {
 271 |                         let key = c.to_lowercase().chars().next().unwrap_or(' ');
 272 |                         if self.held_navigation_key == Some(key) {
 273 |                             self.held_navigation_key = None;
 274 |                             self.last_advance_time = None;
 275 |                         }
 276 |                     }
 277 |                 }
 278 |             }
 279 |             WindowEvent::ModifiersChanged(mods) => {
 280 |                 self.ctrl_pressed = mods.state().control_key();
 281 |                 self.alt_pressed = mods.state().alt_key();
 282 |                 self.shift_pressed = mods.state().shift_key();
 283 |             }
 284 |             WindowEvent::MouseWheel { delta, .. } => {
 285 |                 self.handle_mouse_wheel(delta, self.last_cursor_pos);
 286 |             }
 287 |             WindowEvent::CursorMoved { position, .. } => {
 288 |                 if let Some(start) = self.mouse_drag_start {
 289 |                     if let Some(ref w) = self.window {
 290 |                         let size = w.inner_size();
 291 |                         if size.width > 0 && size.height > 0 {
 292 |                             let dx = (position.x - start.x) as f32 / size.width as f32;
 293 |                             let dy = (position.y - start.y) as f32 / size.height as f32;
 294 |                             let rect = &mut self.ui_state.adjustments.crop_rect;
 295 |                             rect[0] = (rect[0] - dx * rect[2]).clamp(0.0, 1.0 - rect[2]);
 296 |                             rect[1] = (rect[1] - dy * rect[3]).clamp(0.0, 1.0 - rect[3]);
 297 |                             self.dirty = true;
 298 |                         }
 299 |                     }
 300 |                     self.mouse_drag_start = Some(position);
 301 |                 }
 302 |                 self.last_cursor_pos = position;
 303 |             }
 304 |             WindowEvent::MouseInput { state, button, .. } => match (button, state) {
 305 |                 (MouseButton::Left, ElementState::Pressed) => {
 306 |                     let pos = self.last_cursor_pos;
 307 | 
 308 |                     if self.show_thumbnail_strip {
 309 |                         if let Some(ref renderer) = self.renderer {
 310 |                             if let Some(thumb_slot) = renderer.thumbnail_index_at(pos.x, pos.y) {
 311 |                                 if let Some(path) = self.thumb_paths.get(thumb_slot).cloned() {
 312 |                                     if let Some(file_idx) =
 313 |                                         self.ui_state.files.iter().position(|f| f.path == path)
 314 |                                     {
 315 |                                         if self.ctrl_pressed {
 316 |                                             if self.ui_state.selected_indices.contains(&file_idx) {
 317 |                                                 self.ui_state.selected_indices.remove(&file_idx);
 318 |                                             } else {
 319 |                                                 self.ui_state.selected_indices.insert(file_idx);
 320 |                                             }
 321 |                                             let sel_count = self.ui_state.selected_indices.len();
 322 |                                             self.ui_state.set_status(format!(
 323 |                                                 "{} item(s) selected",
 324 |                                                 sel_count
 325 |                                             ));
 326 |                                             self.dirty = true;
 327 |                                         } else {
 328 |                                             self.ui_state.selected_indices.clear();
 329 |                                             self.ui_state.current_file_index = Some(file_idx);
 330 |                                             self.load_image(path);
 331 |                                         }
 332 |                                     }
 333 |                                 }
 334 |                                 return;
 335 |                             }
 336 |                         }
 337 |                     }
 338 | 
 339 |                     if self.alt_pressed {
 340 |                         self.pick_color_at(self.last_cursor_pos);
 341 |                         return;
 342 |                     }
 343 | 
 344 |                     if let Some(ref w) = self.window {
 345 |                         let win_h = w.inner_size().height as f64;
 346 |                         if self.show_thumbnail_strip && pos.y > win_h - STRIP_HEIGHT_PX as f64 {
 347 |                             return;
 348 |                         }
 349 |                     }
 350 | 
 351 |                     if let Some(ref w) = self.window {
 352 |                         let size = w.inner_size();
 353 |                         if size.width > 0 {
 354 |                             let mouse_x_ratio = self.last_cursor_pos.x / size.width as f64;
 355 |                             if mouse_x_ratio < 0.1 {
 356 |                                 self.prev_image();
 357 |                                 return;
 358 |                             } else if mouse_x_ratio > 0.9 {
 359 |                                 self.next_image();
 360 |                                 return;
 361 |                             }
 362 |                         }
 363 |                     }
 364 | 
 365 |                     if !self.ui_state.is_cropping {
 366 |                         self.mouse_drag_start = Some(self.last_cursor_pos);
 367 |                     }
 368 |                 }
 369 |                 (MouseButton::Back, ElementState::Released) => {
 370 |                     self.prev_image();
 371 |                 }
 372 |                 (MouseButton::Forward, ElementState::Released) => {
 373 |                     self.next_image();
 374 |                 }
 375 |                 (MouseButton::Left, ElementState::Released) => {
 376 |                     self.mouse_drag_start = None;
 377 |                 }
 378 |                 (MouseButton::Right, ElementState::Released) => {
 379 |                     self.mouse_drag_start = None;
 380 |                     self.show_context_menu();
 381 |                 }
 382 |                 _ => {}
 383 |             },
 384 |             WindowEvent::RedrawRequested => {
 385 |                 self.process_events();
 386 |                 if self.dirty {
 387 |                     let status_opt: Option<String> =
 388 |                         self.ui_state.status_message.clone().map(|msg| {
 389 |                             let mut final_msg = msg;
 390 |                             let zoom_pct = (1.0 / self.ui_state.adjustments.crop_rect[2] * 100.0)
 391 |                                 .round() as u32;
 392 |                             if zoom_pct != 100 {
 393 |                                 final_msg = format!("{final_msg}  |  {zoom_pct}%");
 394 |                             }
 395 |                             if self.slideshow_active {
 396 |                                 let interval_secs = self.slideshow_interval.as_secs();
 397 |                                 final_msg = format!("▶ {interval_secs}s  |  {final_msg}");
 398 |                             }
 399 |                             final_msg
 400 |                         });
 401 | 
 402 |                     let is_cropping = self.ui_state.is_cropping;
 403 |                     let crop_rect = self.ui_state.adjustments.crop_rect;
 404 |                     let show_help = self.ui_state.show_help;
 405 |                     let show_sidebar = self.show_sidebar;
 406 |                     let show_thumbnail_strip = self.show_thumbnail_strip;
 407 |                     let show_info = self.ui_state.show_info;
 408 |                     let active_thumb_idx = self.active_thumb_index();
 409 | 
 410 |                     if show_info {
 411 |                         if let Some(ref mut img) = self.current_image {
 412 |                             img.load_exif();
 413 |                         }
 414 |                     }
 415 | 
 416 |                     let exif_text = if show_info {
 417 |                         self.current_image
 418 |                             .as_ref()
 419 |                             .and_then(|img| img.exif_info.as_deref())
 420 |                     } else {
 421 |                         None
 422 |                     };
 423 |                     let sidebar_files: Vec<String> = if show_sidebar {
 424 |                         self.ui_state.files.iter().map(|f| f.name.clone()).collect()
 425 |                     } else {
 426 |                         Vec::new()
 427 |                     };
 428 | 
 429 |                     if let Some(ref mut renderer) = self.renderer {
 430 |                         if self.loading {
 431 |                             let _ = renderer.render_loading();
 432 |                         } else {
 433 |                             let _ = renderer.render_frame(
 434 |                                 &self.ui_state.adjustments,
 435 |                                 is_cropping,
 436 |                                 crop_rect,
 437 |                                 status_opt.as_deref(),
 438 |                                 show_help,
 439 |                                 if show_sidebar {
 440 |                                     Some(&sidebar_files)
 441 |                                 } else {
 442 |                                     None
 443 |                                 },
 444 |                                 show_thumbnail_strip,
 445 |                                 active_thumb_idx,
 446 |                                 &self.ui_state.selected_indices,
 447 |                                 exif_text,
 448 |                                 self.show_histogram,
 449 |                                 self.current_image
 450 |                                     .as_ref()
 451 |                                     .and_then(|img| img.histogram.as_ref()),
 452 |                             );
 453 |                         }
 454 |                     }
 455 |                     self.dirty = false;
 456 |                 }
 457 |             }
 458 |             _ => (),
 459 |         }
 460 |     }
 461 | 
 462 |     fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
 463 |         self.process_events();
 464 | 
 465 |         let current = self.ui_state.adjustments.crop_rect;
 466 |         let target = self.ui_state.adjustments.crop_rect_target;
 467 |         let mut animating_zoom = false;
 468 | 
 469 |         let diff: f32 = current
 470 |             .iter()
 471 |             .zip(target.iter())
 472 |             .map(|(c, t)| (c - t).abs())
 473 |             .sum();
 474 |         if diff > 0.001 {
 475 |             animating_zoom = true;
 476 |             for i in 0..4 {
 477 |                 self.ui_state.adjustments.crop_rect[i] =
 478 |                     current[i] + (target[i] - current[i]) * 0.2;
 479 |             }
 480 |             self.dirty = true;
 481 |         } else if diff > 0.0 {
 482 |             self.ui_state.adjustments.crop_rect = target;
 483 |             self.dirty = true;
 484 |         }
 485 | 
 486 |         if let Some(next_time) = self.next_frame_time {
 487 |             let now = std::time::Instant::now();
 488 |             if now >= next_time && !self.current_frame_delays.is_empty() {
 489 |                 let total = self.current_frame_delays.len() + 1;
 490 |                 self.current_frame_idx = (self.current_frame_idx + 1) % total;
 491 | 
 492 |                 let delay = if self.current_frame_idx == 0 {
 493 |                     if let Some(ref mut renderer) = self.renderer {
 494 |                         renderer.swap_gif_frame(0);
 495 |                     }
 496 |                     self.current_image
 497 |                         .as_ref()
 498 |                         .map(|f| f.frame_delay_ms)
 499 |                         .unwrap_or(100)
 500 |                 } else {
 501 |                     let idx = self.current_frame_idx;
 502 |                     if let Some(ref mut renderer) = self.renderer {
 503 |                         renderer.swap_gif_frame(idx);
 504 |                     }
 505 |                     self.current_frame_delays
 506 |                         .get(self.current_frame_idx - 1)
 507 |                         .copied()
 508 |                         .unwrap_or(100)
 509 |                 };
 510 | 
 511 |                 self.next_frame_time =
 512 |                     Some(now + std::time::Duration::from_millis(delay.max(10) as u64));
 513 |                 self.dirty = true;
 514 |             }
 515 |         }
 516 | 
 517 |         let now = std::time::Instant::now();
 518 |         if self.slideshow_active {
 519 |             if let Some(st) = self.slideshow_next_time {
 520 |                 if now >= st {
 521 |                     self.next_image();
 522 |                     self.slideshow_next_time = Some(now + self.slideshow_interval);
 523 |                 }
 524 |             }
 525 |         }
 526 | 
 527 |         let mut wait_until = None;
 528 |         if animating_zoom {
 529 |             wait_until = Some(now + std::time::Duration::from_millis(16));
 530 |         }
 531 |         if let Some(ft) = self.next_frame_time {
 532 |             wait_until = Some(wait_until.map_or(ft, |w| w.min(ft)));
 533 |         }
 534 |         if self.slideshow_active {
 535 |             if let Some(st) = self.slideshow_next_time {
 536 |                 wait_until = Some(wait_until.map_or(st, |w| w.min(st)));
 537 |             }
 538 |         }
 539 | 
 540 |         const HOLD_ADVANCE_INTERVAL_MS: u64 = 150;
 541 |         if let (Some(key), Some(last_time)) = (self.held_navigation_key, self.last_advance_time) {
 542 |             let now = std::time::Instant::now();
 543 |             let elapsed = now.duration_since(last_time);
 544 |             if elapsed.as_millis() >= HOLD_ADVANCE_INTERVAL_MS as u128 {
 545 |                 match key {
 546 |                     'a' | 'w' => self.prev_image(),
 547 |                     'd' | 's' => self.next_image(),
 548 |                     _ => {}
 549 |                 }
 550 |                 self.last_advance_time = Some(now);
 551 |                 wait_until = Some(now + std::time::Duration::from_millis(HOLD_ADVANCE_INTERVAL_MS));
 552 |             }
 553 |         }
 554 | 
 555 |         if let Some(wu) = wait_until {
 556 |             event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wu));
 557 |         } else {
 558 |             event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
 559 |         }
 560 | 
 561 |         if self.dirty {
 562 |             if let Some(window) = &self.window {
 563 |                 window.request_redraw();
 564 |             }
 565 |         }
 566 |     }
 567 | }
```

### File: `src\app\services.rs`

- Size: 5641 bytes
- Modified: 2026-03-07 12:16:36 UTC

```rust
   1 | use crate::app::state::SpedImageApp;
   2 | use crate::app::types::{send_event, AppEvent, MAX_THUMBNAILS, MAX_THUMB_THREADS, THUMB_LOAD_SIZE};
   3 | use crate::image_backend::ImageBackend;
   4 | use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
   5 | use std::path::{Path, PathBuf};
   6 | use std::sync::atomic::Ordering;
   7 | 
   8 | impl SpedImageApp {
   9 |     /// (6) Prefetch the images neighboring the current one in the folder.
  10 |     pub(crate) fn prefetch_adjacent(&mut self, current: &Path) {
  11 |         let (max_w, max_h) = match &self.window {
  12 |             Some(w) => {
  13 |                 let s = w.inner_size();
  14 |                 (s.width, s.height)
  15 |             }
  16 |             None => (3840, 2160),
  17 |         };
  18 | 
  19 |         let neighbors: Vec<PathBuf> = {
  20 |             let files = &self.ui_state.files;
  21 |             let idx = files.iter().position(|f| f.path == current);
  22 |             let mut v = Vec::new();
  23 |             if let Some(i) = idx {
  24 |                 if i + 1 < files.len() {
  25 |                     v.push(files[i + 1].path.clone());
  26 |                 }
  27 |                 if i > 0 {
  28 |                     v.push(files[i - 1].path.clone());
  29 |                 }
  30 |             }
  31 |             v
  32 |         };
  33 | 
  34 |         for path in neighbors {
  35 |             if self.prefetch_cache.contains(&path) {
  36 |                 continue;
  37 |             }
  38 |             const MAX_CONCURRENT_PREFETCH: usize = 2;
  39 |             if self.prefetch_active.load(Ordering::Relaxed) >= MAX_CONCURRENT_PREFETCH {
  40 |                 continue;
  41 |             }
  42 |             self.prefetch_active.fetch_add(1, Ordering::Relaxed);
  43 |             let active = self.prefetch_active.clone();
  44 |             let tx = self.event_tx.clone();
  45 |             let proxy = self.event_proxy.clone();
  46 |             let p = path.clone();
  47 |             std::thread::spawn(move || {
  48 |                 let result = ImageBackend::load_and_downsample(&p, max_w, max_h);
  49 |                 active.fetch_sub(1, Ordering::Relaxed);
  50 |                 if let Ok(frames) = result {
  51 |                     if let Some(ref proxy) = proxy {
  52 |                         send_event(&tx, proxy, AppEvent::Prefetched(p, frames));
  53 |                     }
  54 |                 }
  55 |             });
  56 |         }
  57 |     }
  58 | 
  59 |     /// Set up a file watcher to monitor directory changes
  60 |     pub(crate) fn setup_file_watcher(&mut self, dir: &Path) {
  61 |         let tx = self.event_tx.clone();
  62 |         let proxy = self.event_proxy.clone();
  63 | 
  64 |         let watcher_result = RecommendedWatcher::new(
  65 |             move |res: Result<Event, notify::Error>| {
  66 |                 if let Ok(event) = res {
  67 |                     match event.kind {
  68 |                         notify::EventKind::Create(_)
  69 |                         | notify::EventKind::Modify(_)
  70 |                         | notify::EventKind::Remove(_) => {
  71 |                             if let Some(ref proxy) = proxy {
  72 |                                 send_event(
  73 |                                     &tx,
  74 |                                     proxy,
  75 |                                     AppEvent::SetStatus(
  76 |                                         "Directory changed - press R to refresh".to_string(),
  77 |                                     ),
  78 |                                 );
  79 |                             }
  80 |                         }
  81 |                         _ => {}
  82 |                     }
  83 |                 }
  84 |             },
  85 |             Config::default(),
  86 |         );
  87 | 
  88 |         if let Ok(mut watcher) = watcher_result {
  89 |             if watcher.watch(dir, RecursiveMode::NonRecursive).is_ok() {
  90 |                 self.file_watcher = Some(watcher);
  91 |                 tracing::debug!("File watcher started for {:?}", dir);
  92 |             }
  93 |         }
  94 |     }
  95 | 
  96 |     /// Spawn background thumbnail-loading threads for all files in the current directory.
  97 |     pub(crate) fn load_thumbnails_for_dir(&mut self) {
  98 |         let files: Vec<PathBuf> = self.ui_state.files.iter().map(|f| f.path.clone()).collect();
  99 |         self.thumb_paths = files.clone();
 100 | 
 101 |         if let Some(ref mut renderer) = self.renderer {
 102 |             renderer.clear_thumbnails();
 103 |         }
 104 | 
 105 |         let n = files.len().min(MAX_THUMBNAILS);
 106 |         let tx = self.event_tx.clone();
 107 |         let proxy = self.event_proxy.clone();
 108 |         let active = self.thumb_active.clone();
 109 | 
 110 |         for path in files.into_iter().take(n) {
 111 |             while active.load(Ordering::Relaxed) >= MAX_THUMB_THREADS {
 112 |                 std::thread::sleep(std::time::Duration::from_millis(5));
 113 |             }
 114 |             active.fetch_add(1, Ordering::Relaxed);
 115 |             let tx2 = tx.clone();
 116 |             let proxy2 = proxy.clone();
 117 |             let act2 = active.clone();
 118 |             let p = path.clone();
 119 |             std::thread::spawn(move || {
 120 |                 let result =
 121 |                     ImageBackend::load_and_downsample(&p, THUMB_LOAD_SIZE, THUMB_LOAD_SIZE);
 122 |                 act2.fetch_sub(1, Ordering::Relaxed);
 123 |                 if let Ok(frames) = result {
 124 |                     if let Some(first) = frames.into_iter().next() {
 125 |                         if let Some(ref proxy) = proxy2 {
 126 |                             send_event(
 127 |                                 &tx2,
 128 |                                 proxy,
 129 |                                 AppEvent::ThumbnailLoaded(
 130 |                                     p,
 131 |                                     first.rgba_data,
 132 |                                     first.width,
 133 |                                     first.height,
 134 |                                 ),
 135 |                             );
 136 |                         }
 137 |                     }
 138 |                 }
 139 |             });
 140 |         }
 141 |     }
 142 | }
```

### File: `src\app\state.rs`

- Size: 4231 bytes
- Modified: 2026-03-07 12:10:45 UTC

```rust
   1 | use crate::app::types::{AppEvent, WakeUp};
   2 | use crate::gpu_renderer::Renderer;
   3 | use crate::image_backend::ImageData;
   4 | use crate::ui::UiState;
   5 | use lru::LruCache;
   6 | use notify::RecommendedWatcher;
   7 | use rayon::ThreadPool;
   8 | use std::path::PathBuf;
   9 | use std::sync::atomic::AtomicUsize;
  10 | use std::sync::mpsc::{self, Receiver, Sender};
  11 | use std::sync::Arc;
  12 | use winit::dpi::PhysicalPosition;
  13 | use winit::event_loop::EventLoopProxy;
  14 | use winit::window::Window;
  15 | 
  16 | pub struct SpedImageApp {
  17 |     pub(crate) window: Option<Arc<Window>>,
  18 |     pub(crate) renderer: Option<Renderer>,
  19 |     pub(crate) ui_state: UiState,
  20 |     pub(crate) current_image: Option<ImageData>,
  21 |     pub(crate) current_frame_delays: Vec<u32>,
  22 |     pub(crate) current_frame_idx: usize,
  23 |     pub(crate) next_frame_time: Option<std::time::Instant>,
  24 |     pub(crate) loading: bool,
  25 |     pub(crate) dirty: bool,
  26 |     pub(crate) event_tx: Sender<AppEvent>,
  27 |     pub(crate) event_rx: Receiver<AppEvent>,
  28 |     pub(crate) event_proxy: Option<EventLoopProxy<WakeUp>>,
  29 |     pub(crate) mouse_drag_start: Option<PhysicalPosition<f64>>,
  30 |     pub(crate) last_cursor_pos: PhysicalPosition<f64>,
  31 |     pub(crate) show_help: bool,
  32 |     pub(crate) show_sidebar: bool,
  33 |     pub(crate) prefetch_cache: LruCache<PathBuf, Vec<ImageData>>,
  34 |     pub(crate) prefetch_active: Arc<AtomicUsize>,
  35 |     pub(crate) initial_path: Option<PathBuf>,
  36 |     pub(crate) ctrl_pressed: bool,
  37 |     pub(crate) shift_pressed: bool,
  38 |     pub(crate) held_navigation_key: Option<char>,
  39 |     pub(crate) last_advance_time: Option<std::time::Instant>,
  40 |     pub(crate) show_thumbnail_strip: bool,
  41 |     pub(crate) thumb_active: Arc<AtomicUsize>,
  42 |     pub(crate) thumb_paths: Vec<PathBuf>,
  43 |     pub(crate) slideshow_active: bool,
  44 |     pub(crate) slideshow_interval: std::time::Duration,
  45 |     pub(crate) slideshow_next_time: Option<std::time::Instant>,
  46 |     pub(crate) alt_pressed: bool,
  47 |     pub(crate) show_histogram: bool,
  48 |     #[allow(dead_code)]
  49 |     pub(crate) thread_pool: Option<Arc<ThreadPool>>,
  50 |     #[allow(dead_code)]
  51 |     pub(crate) file_watcher: Option<RecommendedWatcher>,
  52 | }
  53 | 
  54 | impl SpedImageApp {
  55 |     pub fn new() -> Self {
  56 |         let (tx, rx) = mpsc::channel();
  57 |         Self {
  58 |             window: None,
  59 |             renderer: None,
  60 |             ui_state: UiState::default(),
  61 |             current_image: None,
  62 |             current_frame_delays: Vec::new(),
  63 |             current_frame_idx: 0,
  64 |             next_frame_time: None,
  65 |             loading: false,
  66 |             dirty: true,
  67 |             event_tx: tx,
  68 |             event_rx: rx,
  69 |             event_proxy: None,
  70 |             mouse_drag_start: None,
  71 |             last_cursor_pos: PhysicalPosition::new(0.0, 0.0),
  72 |             show_help: false,
  73 |             show_sidebar: false,
  74 |             prefetch_cache: LruCache::new(std::num::NonZeroUsize::new(50).unwrap()),
  75 |             prefetch_active: Arc::new(AtomicUsize::new(0)),
  76 |             initial_path: None,
  77 |             ctrl_pressed: false,
  78 |             held_navigation_key: None,
  79 |             last_advance_time: None,
  80 |             show_thumbnail_strip: true,
  81 |             thumb_active: Arc::new(AtomicUsize::new(0)),
  82 |             thumb_paths: Vec::new(),
  83 |             slideshow_active: false,
  84 |             slideshow_interval: std::time::Duration::from_secs(5),
  85 |             slideshow_next_time: None,
  86 |             alt_pressed: false,
  87 |             shift_pressed: false,
  88 |             show_histogram: false,
  89 |             thread_pool: None,
  90 |             file_watcher: None,
  91 |         }
  92 |     }
  93 | }
  94 | 
  95 | impl Default for SpedImageApp {
  96 |     fn default() -> Self {
  97 |         Self::new()
  98 |     }
  99 | }
 100 | 
 101 | impl Drop for SpedImageApp {
 102 |     fn drop(&mut self) {
 103 |         self.prefetch_cache.clear();
 104 |         self.current_frame_delays.clear();
 105 |         self.current_image = None;
 106 |         if let Some(ref mut renderer) = self.renderer {
 107 |             renderer.clear_thumbnails();
 108 |         }
 109 |     }
 110 | }
 111 | 
 112 | #[cfg(test)]
 113 | mod tests {
 114 |     use super::*;
 115 | 
 116 |     #[test]
 117 |     fn test_app_new() {
 118 |         let app = SpedImageApp::new();
 119 |         assert!(app.window.is_none());
 120 |         assert!(app.renderer.is_none());
 121 |         assert!(!app.loading);
 122 |         assert!(app.dirty);
 123 |         assert!(!app.show_help);
 124 |     }
 125 | }
```

### File: `src\app\types.rs`

- Size: 1424 bytes
- Modified: 2026-03-07 12:19:36 UTC

```rust
   1 | use crate::image_backend::ImageData;
   2 | use std::path::PathBuf;
   3 | use std::sync::mpsc::Sender;
   4 | use winit::event_loop::EventLoopProxy;
   5 | 
   6 | /// Wakeup token sent through EventLoopProxy to wake the sleeping event loop.
   7 | /// The actual payload travels through a regular mpsc channel.
   8 | #[derive(Debug)]
   9 | pub struct WakeUp;
  10 | 
  11 | pub const APP_ICON: &[u8] = include_bytes!("../../assets/icons/icon.png");
  12 | 
  13 | /// Thumbnail size used for background loading (must match THUMB_SIZE in gpu_renderer).
  14 | pub const THUMB_LOAD_SIZE: u32 = 80;
  15 | /// Max concurrently-running thumbnail background threads.
  16 | pub const MAX_THUMB_THREADS: usize = 4;
  17 | /// Max thumbnails kept in GPU at once (older ones are not evicted but new ones stop loading).
  18 | pub const MAX_THUMBNAILS: usize = 200;
  19 | 
  20 | pub enum AppEvent {
  21 |     ImageLoaded(Vec<ImageData>),
  22 |     ImageError(String),
  23 |     OpenPath(PathBuf),
  24 |     Prefetched(PathBuf, Vec<ImageData>), // prefetch for adjacent images
  25 |     SaveComplete(PathBuf),
  26 |     SaveError(String),
  27 |     /// A thumbnail has finished loading: (path, rgba_bytes, width, height)
  28 |     ThumbnailLoaded(PathBuf, Vec<u8>, u32, u32),
  29 |     SetStatus(String),
  30 |     FileRenamed(PathBuf, PathBuf),
  31 | }
  32 | 
  33 | /// Helper: send an AppEvent through the data channel, then wake the event loop.
  34 | pub fn send_event(tx: &Sender<AppEvent>, proxy: &EventLoopProxy<WakeUp>, event: AppEvent) {
  35 |     tx.send(event).ok();
  36 |     proxy.send_event(WakeUp).ok();
  37 | }
```

### File: `src\gpu_renderer.rs`

- Size: 56588 bytes
- Modified: 2026-03-07 07:53:55 UTC

```rust
   1 | //! GPU Renderer - WGPU-based image processing pipeline
   2 | 
   3 | use anyhow::{Context, Result};
   4 | use std::sync::Arc;
   5 | use wgpu::util::DeviceExt;
   6 | use wgpu::{
   7 |     BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BindingType, BlendState,
   8 |     ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
   9 |     FragmentState, FrontFace, Instance, LoadOp, Operations, PipelineLayoutDescriptor,
  10 |     PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
  11 |     RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerDescriptor,
  12 |     ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
  13 |     TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
  14 |     TextureDimension, TextureFormat, TextureSampleType, TextureUsages, VertexBufferLayout,
  15 |     VertexFormat, VertexState, VertexStepMode,
  16 | };
  17 | use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
  18 | use winit::dpi::PhysicalSize;
  19 | use winit::window::Window;
  20 | 
  21 | use crate::image_backend::ImageData;
  22 | 
  23 | /// Height of the thumbnail strip in physical pixels.
  24 | pub const STRIP_HEIGHT_PX: u32 = 90;
  25 | /// Width of each thumbnail slot (including gap).
  26 | pub const THUMB_SLOT_W: u32 = 80;
  27 | /// Thumbnail image size (square, aspect-fit inside the slot).
  28 | pub const THUMB_SIZE: u32 = 74;
  29 | 
  30 | #[derive(Debug, Clone, Copy, PartialEq)]
  31 | pub struct ImageAdjustments {
  32 |     pub brightness: f32,
  33 |     pub contrast: f32,
  34 |     pub saturation: f32,
  35 |     pub rotation: f32,
  36 |     pub crop_rect_target: [f32; 4], // Where we want to be
  37 |     pub crop_rect: [f32; 4],        // Where we currently are (rendered)
  38 |     pub hdr_toning: bool,
  39 |     pub pixel_perfect: bool, // Nearest-neighbor sampling for pixel art
  40 | }
  41 | 
  42 | impl Default for ImageAdjustments {
  43 |     fn default() -> Self {
  44 |         Self {
  45 |             brightness: 1.0,
  46 |             contrast: 1.0,
  47 |             saturation: 1.0,
  48 |             rotation: 0.0,
  49 |             crop_rect_target: [0.0, 0.0, 1.0, 1.0],
  50 |             crop_rect: [0.0, 0.0, 1.0, 1.0],
  51 |             hdr_toning: false,
  52 |             pixel_perfect: false,
  53 |         }
  54 |     }
  55 | }
  56 | 
  57 | #[repr(C)]
  58 | #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
  59 | struct Uniforms {
  60 |     rotation: f32,
  61 |     aspect_ratio: f32,
  62 |     window_aspect_ratio: f32,
  63 |     crop_x: f32,
  64 |     crop_y: f32,
  65 |     crop_w: f32,
  66 |     crop_h: f32,
  67 |     brightness: f32,
  68 |     contrast: f32,
  69 |     saturation: f32,
  70 |     hdr_toning: f32,
  71 |     _padding: f32,
  72 | }
  73 | 
  74 | const SHADER: &str = r#"
  75 | struct VertexOutput {
  76 |     @builtin(position) position: vec4<f32>,
  77 |     @location(0) tex_coords: vec2<f32>,
  78 | };
  79 | 
  80 | struct Uniforms {
  81 |     rotation: f32,
  82 |     aspect_ratio: f32,
  83 |     window_aspect_ratio: f32,
  84 |     crop_x: f32,
  85 |     crop_y: f32,
  86 |     crop_w: f32,
  87 |     crop_h: f32,
  88 |     brightness: f32,
  89 |     contrast: f32,
  90 |     saturation: f32,
  91 |     hdr_toning: f32,
  92 |     _padding: f32,
  93 | };
  94 | 
  95 | @group(0) @binding(0)
  96 | var<uniform> uniforms: Uniforms;
  97 | 
  98 | @vertex
  99 | fn vertex_main(
 100 |     @location(0) position: vec2<f32>,
 101 |     @location(1) tex_coords: vec2<f32>
 102 | ) -> VertexOutput {
 103 |     var out: VertexOutput;
 104 |     var tex = tex_coords * vec2<f32>(uniforms.crop_w, uniforms.crop_h) 
 105 |               + vec2<f32>(uniforms.crop_x, uniforms.crop_y);
 106 |     let center = vec2<f32>(0.5, 0.5);
 107 |     let rotated_tex = rotate(tex - center, uniforms.rotation) + center;
 108 |     var pos = position;
 109 | 
 110 |     let image_ar = uniforms.aspect_ratio;
 111 |     let window_ar = uniforms.window_aspect_ratio;
 112 |     let ratio = image_ar / window_ar;
 113 | 
 114 |     if (ratio > 1.0) {
 115 |         pos.y /= ratio;
 116 |     } else {
 117 |         pos.x *= ratio;
 118 |     }
 119 | 
 120 |     out.position = vec4<f32>(pos, 0.0, 1.0);
 121 |     out.tex_coords = rotated_tex;
 122 |     return out;
 123 | }
 124 | 
 125 | fn rotate(coord: vec2<f32>, angle: f32) -> vec2<f32> {
 126 |     let s = sin(angle);
 127 |     let c = cos(angle);
 128 |     return vec2<f32>(coord.x * c - coord.y * s, coord.x * s + coord.y * c);
 129 | }
 130 | 
 131 | struct FragmentInput {
 132 |     @location(0) tex_coords: vec2<f32>,
 133 | };
 134 | 
 135 | @group(0) @binding(1)
 136 | var image_sampler: sampler;
 137 | 
 138 | @group(0) @binding(2)
 139 | var image_texture: texture_2d<f32>;
 140 | 
 141 | @fragment
 142 | fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
 143 |     let tex_color = textureSample(image_texture, image_sampler, input.tex_coords);
 144 |     var color = tex_color.rgb;
 145 |     color = color * uniforms.brightness;
 146 |     color = (color - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
 147 |     color = mix(vec3<f32>(gray), color, uniforms.saturation);
 148 | 
 149 |     if (uniforms.hdr_toning > 0.5) {
 150 |         let exposed = color * 1.6;
 151 |         color = exposed / (1.0 + exposed);
 152 |         color = color * color * (3.0 - 2.0 * color);
 153 |     }
 154 | 
 155 |     return vec4<f32>(color, tex_color.a);
 156 | }
 157 | "#;
 158 | 
 159 | const CROP_SHADER: &str = r#"
 160 | struct VertexOutput {
 161 |     @builtin(position) position: vec4<f32>,
 162 | };
 163 | 
 164 | struct CropUniforms {
 165 |     x: f32,
 166 |     y: f32,
 167 |     w: f32,
 168 |     h: f32,
 169 | };
 170 | 
 171 | @group(0) @binding(0)
 172 | var<uniform> crop: CropUniforms;
 173 | 
 174 | @vertex
 175 | fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
 176 |     // Generate a full screen quad
 177 |     var out: VertexOutput;
 178 |     let x = f32(vertex_index & 1u) * 2.0 - 1.0;
 179 |     let y = f32((vertex_index >> 1u) & 1u) * 2.0 - 1.0;
 180 |     // We invert y for Vulkan/WGPU coordinates
 181 |     out.position = vec4<f32>(x, -y, 0.0, 1.0);
 182 |     return out;
 183 | }
 184 | 
 185 | @fragment
 186 | fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
 187 |     // In viewport coordinates (0 to width/height)
 188 |     // Actually, in.position.xy is in pixel coordinates.
 189 |     // We pass normalized crop rect (0.0 to 1.0) but we don't know window bounds in shader easily.
 190 |     // Instead we can generate the crop rect in vertex shader or just draw the overlay regions.
 191 |     return vec4<f32>(0.0, 0.0, 0.0, 0.5); // Just a generic darken, we will use scissors!
 192 | }
 193 | "#;
 194 | 
 195 | /// A fully-uploaded thumbnail ready for GPU rendering.
 196 | pub struct ThumbnailEntry {
 197 |     pub path: std::path::PathBuf,
 198 |     /// Bind group pointing at the thumbnail texture (same layout as image pipeline).
 199 |     pub bind_group: Arc<BindGroup>,
 200 |     pub width: u32,
 201 |     pub height: u32,
 202 |     /// The GPU texture (kept alive so bind group stays valid).
 203 |     _texture: Texture,
 204 | }
 205 | 
 206 | pub struct Renderer {
 207 |     _window: Arc<Window>,
 208 |     device: Device,
 209 |     queue: Queue,
 210 |     surface: Surface<'static>,
 211 |     pipeline: RenderPipeline,
 212 |     crop_pipeline: RenderPipeline,
 213 |     uniform_buffer: wgpu::Buffer,
 214 |     /// Uniform buffer used with identity (no-op) uniforms for thumbnail rendering.
 215 |     thumb_uniform_buffer: wgpu::Buffer,
 216 |     vertex_buffer: wgpu::Buffer,
 217 |     sampler: Sampler,
 218 |     sampler_nearest: Sampler,
 219 |     image_texture: Option<Texture>,
 220 |     image_bind_group: Option<Arc<BindGroup>>,
 221 |     image_bind_group_nearest: Option<Arc<BindGroup>>, // For pixel-perfect mode
 222 |     pub gif_textures: Vec<(Texture, Arc<BindGroup>)>, // cached GPU textures for GIF frames
 223 |     config: SurfaceConfiguration,
 224 |     image_size: Option<(u32, u32)>,
 225 |     pub scale_factor: f64, // DPI scale (12: DPI-aware rendering)
 226 | 
 227 |     // Text rendering
 228 |     text_brush: GlyphBrush<()>,
 229 |     staging_belt: wgpu::util::StagingBelt,
 230 | 
 231 |     // Thumbnails
 232 |     pub thumbnails: Vec<ThumbnailEntry>,
 233 | }
 234 | 
 235 | impl Renderer {
 236 |     pub async fn new(window: Arc<Window>) -> Result<Self> {
 237 |         let instance = Instance::new(&wgpu::InstanceDescriptor::default());
 238 | 
 239 |         let surface = instance
 240 |             .create_surface(window.clone())
 241 |             .context("Failed to create WGPU surface")?;
 242 | 
 243 |         let adapter = instance
 244 |             .request_adapter(&RequestAdapterOptions {
 245 |                 power_preference: wgpu::PowerPreference::LowPower,
 246 |                 compatible_surface: Some(&surface),
 247 |                 ..Default::default()
 248 |             })
 249 |             .await
 250 |             .context("Failed to request WGPU adapter")?;
 251 | 
 252 |         let (device, queue) = adapter
 253 |             .request_device(&DeviceDescriptor {
 254 |                 label: Some("SpedImage Device"),
 255 |                 required_features: wgpu::Features::default(),
 256 |                 required_limits: wgpu::Limits::default(),
 257 |                 memory_hints: wgpu::MemoryHints::default(),
 258 |                 trace: wgpu::Trace::Off,
 259 |             })
 260 |             .await
 261 |             .context("Failed to request WGPU device")?;
 262 | 
 263 |         let capabilities = surface.get_capabilities(&adapter);
 264 |         let format = capabilities.formats[0];
 265 | 
 266 |         let config = SurfaceConfiguration {
 267 |             usage: TextureUsages::RENDER_ATTACHMENT,
 268 |             format,
 269 |             width: window.inner_size().width,
 270 |             height: window.inner_size().height,
 271 |             present_mode: wgpu::PresentMode::AutoNoVsync,
 272 |             alpha_mode: capabilities.alpha_modes[0],
 273 |             view_formats: vec![],
 274 |             desired_maximum_frame_latency: 2,
 275 |         };
 276 |         surface.configure(&device, &config);
 277 | 
 278 |         let shader_module = device.create_shader_module(ShaderModuleDescriptor {
 279 |             label: Some("Shader"),
 280 |             source: ShaderSource::Wgsl(SHADER.into()),
 281 |         });
 282 | 
 283 |         let vertex_data: [f32; 24] = [
 284 |             -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 1.0,
 285 |             1.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 0.0,
 286 |         ];
 287 | 
 288 |         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
 289 |             label: Some("Vertex Buffer"),
 290 |             contents: bytemuck::cast_slice(&vertex_data),
 291 |             usage: wgpu::BufferUsages::VERTEX,
 292 |         });
 293 | 
 294 |         let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
 295 |             label: Some("Uniform Buffer"),
 296 |             size: std::mem::size_of::<Uniforms>() as u64,
 297 |             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
 298 |             mapped_at_creation: false,
 299 |         });
 300 | 
 301 |         // A second uniform buffer pre-filled with "identity" values for thumbnail rendering.
 302 |         let thumb_uniforms = Uniforms {
 303 |             rotation: 0.0,
 304 |             aspect_ratio: 1.0,
 305 |             window_aspect_ratio: 1.0,
 306 |             crop_x: 0.0,
 307 |             crop_y: 0.0,
 308 |             crop_w: 1.0,
 309 |             crop_h: 1.0,
 310 |             brightness: 1.0,
 311 |             contrast: 1.0,
 312 |             saturation: 1.0,
 313 |             hdr_toning: 0.0,
 314 |             _padding: 0.0,
 315 |         };
 316 |         let thumb_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
 317 |             label: Some("Thumbnail Uniform Buffer"),
 318 |             contents: bytemuck::bytes_of(&thumb_uniforms),
 319 |             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
 320 |         });
 321 | 
 322 |         let sampler = device.create_sampler(&SamplerDescriptor {
 323 |             label: Some("Image Sampler"),
 324 |             mag_filter: wgpu::FilterMode::Linear,
 325 |             min_filter: wgpu::FilterMode::Linear,
 326 |             mipmap_filter: wgpu::FilterMode::Linear,
 327 |             address_mode_u: wgpu::AddressMode::ClampToEdge,
 328 |             address_mode_v: wgpu::AddressMode::ClampToEdge,
 329 |             ..Default::default()
 330 |         });
 331 | 
 332 |         // Nearest-neighbor sampler for pixel-perfect zoom mode
 333 |         let sampler_nearest = device.create_sampler(&SamplerDescriptor {
 334 |             label: Some("Image Sampler (Nearest)"),
 335 |             mag_filter: wgpu::FilterMode::Nearest,
 336 |             min_filter: wgpu::FilterMode::Nearest,
 337 |             mipmap_filter: wgpu::FilterMode::Nearest,
 338 |             address_mode_u: wgpu::AddressMode::ClampToEdge,
 339 |             address_mode_v: wgpu::AddressMode::ClampToEdge,
 340 |             ..Default::default()
 341 |         });
 342 | 
 343 |         let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
 344 |             label: Some("Image Bind Group Layout"),
 345 |             entries: &[
 346 |                 wgpu::BindGroupLayoutEntry {
 347 |                     binding: 0,
 348 |                     visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
 349 |                     ty: BindingType::Buffer {
 350 |                         ty: wgpu::BufferBindingType::Uniform,
 351 |                         has_dynamic_offset: false,
 352 |                         min_binding_size: None,
 353 |                     },
 354 |                     count: None,
 355 |                 },
 356 |                 wgpu::BindGroupLayoutEntry {
 357 |                     binding: 1,
 358 |                     visibility: wgpu::ShaderStages::FRAGMENT,
 359 |                     ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
 360 |                     count: None,
 361 |                 },
 362 |                 wgpu::BindGroupLayoutEntry {
 363 |                     binding: 2,
 364 |                     visibility: wgpu::ShaderStages::FRAGMENT,
 365 |                     ty: BindingType::Texture {
 366 |                         sample_type: TextureSampleType::Float { filterable: true },
 367 |                         view_dimension: wgpu::TextureViewDimension::D2,
 368 |                         multisampled: false,
 369 |                     },
 370 |                     count: None,
 371 |                 },
 372 |             ],
 373 |         });
 374 | 
 375 |         let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
 376 |             label: Some("Pipeline Layout"),
 377 |             bind_group_layouts: &[&bind_group_layout],
 378 |             push_constant_ranges: &[],
 379 |         });
 380 | 
 381 |         let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
 382 |             label: Some("Image Render Pipeline"),
 383 |             layout: Some(&pipeline_layout),
 384 |             vertex: VertexState {
 385 |                 module: &shader_module,
 386 |                 entry_point: Some("vertex_main"),
 387 |                 compilation_options: wgpu::PipelineCompilationOptions::default(),
 388 |                 buffers: &[VertexBufferLayout {
 389 |                     array_stride: 16,
 390 |                     step_mode: VertexStepMode::Vertex,
 391 |                     attributes: &[
 392 |                         wgpu::VertexAttribute {
 393 |                             format: VertexFormat::Float32x2,
 394 |                             offset: 0,
 395 |                             shader_location: 0,
 396 |                         },
 397 |                         wgpu::VertexAttribute {
 398 |                             format: VertexFormat::Float32x2,
 399 |                             offset: 8,
 400 |                             shader_location: 1,
 401 |                         },
 402 |                     ],
 403 |                 }],
 404 |             },
 405 |             fragment: Some(FragmentState {
 406 |                 module: &shader_module,
 407 |                 entry_point: Some("fragment_main"),
 408 |                 compilation_options: wgpu::PipelineCompilationOptions::default(),
 409 |                 targets: &[Some(ColorTargetState {
 410 |                     format,
 411 |                     blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
 412 |                     write_mask: ColorWrites::ALL,
 413 |                 })],
 414 |             }),
 415 |             primitive: PrimitiveState {
 416 |                 topology: PrimitiveTopology::TriangleList,
 417 |                 front_face: FrontFace::Cw,
 418 |                 ..Default::default()
 419 |             },
 420 |             depth_stencil: None,
 421 |             multisample: wgpu::MultisampleState::default(),
 422 |             multiview: None,
 423 |             cache: None,
 424 |         });
 425 | 
 426 |         let crop_shader_module = device.create_shader_module(ShaderModuleDescriptor {
 427 |             label: Some("Crop Shader"),
 428 |             source: ShaderSource::Wgsl(CROP_SHADER.into()),
 429 |         });
 430 | 
 431 |         let crop_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
 432 |             label: Some("Crop Pipeline Layout"),
 433 |             bind_group_layouts: &[],
 434 |             push_constant_ranges: &[],
 435 |         });
 436 | 
 437 |         let crop_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
 438 |             label: Some("Crop Overlay Pipeline"),
 439 |             layout: Some(&crop_pipeline_layout),
 440 |             vertex: VertexState {
 441 |                 module: &crop_shader_module,
 442 |                 entry_point: Some("vertex_main"),
 443 |                 compilation_options: wgpu::PipelineCompilationOptions::default(),
 444 |                 buffers: &[], // generating vertices directly
 445 |             },
 446 |             fragment: Some(FragmentState {
 447 |                 module: &crop_shader_module,
 448 |                 entry_point: Some("fragment_main"),
 449 |                 compilation_options: wgpu::PipelineCompilationOptions::default(),
 450 |                 targets: &[Some(ColorTargetState {
 451 |                     format,
 452 |                     blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
 453 |                     write_mask: ColorWrites::ALL,
 454 |                 })],
 455 |             }),
 456 |             primitive: PrimitiveState {
 457 |                 topology: PrimitiveTopology::TriangleStrip,
 458 |                 strip_index_format: None,
 459 |                 front_face: FrontFace::Cw,
 460 |                 cull_mode: None,
 461 |                 unclipped_depth: false,
 462 |                 polygon_mode: wgpu::PolygonMode::Fill,
 463 |                 conservative: false,
 464 |             },
 465 |             depth_stencil: None,
 466 |             multisample: wgpu::MultisampleState::default(),
 467 |             multiview: None,
 468 |             cache: None,
 469 |         });
 470 | 
 471 |         // Embed Inter-Regular as the guaranteed font fallback; try Segoe UI first on Windows.
 472 |         const EMBEDDED_FONT: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");
 473 |         let font_bytes = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf")
 474 |             .unwrap_or_else(|_| EMBEDDED_FONT.to_vec());
 475 | 
 476 |         let font = ab_glyph::FontArc::try_from_vec(font_bytes).unwrap_or_else(|_| {
 477 |             ab_glyph::FontArc::try_from_slice(EMBEDDED_FONT)
 478 |                 .expect("Embedded Inter-Regular.ttf failed to parse — check asset integrity")
 479 |         });
 480 | 
 481 |         let text_brush = GlyphBrushBuilder::using_font(font).build(&device, format);
 482 | 
 483 |         let staging_belt = wgpu::util::StagingBelt::new(1024);
 484 | 
 485 |         Ok(Self {
 486 |             _window: window.clone(),
 487 |             device,
 488 |             queue,
 489 |             surface,
 490 |             pipeline,
 491 |             crop_pipeline,
 492 |             uniform_buffer,
 493 |             thumb_uniform_buffer,
 494 |             vertex_buffer,
 495 |             sampler,
 496 |             sampler_nearest,
 497 |             image_texture: None,
 498 |             image_bind_group: None,
 499 |             image_bind_group_nearest: None,
 500 |             gif_textures: Vec::new(),
 501 |             config,
 502 |             image_size: None,
 503 |             scale_factor: window.scale_factor(),
 504 |             text_brush,
 505 |             staging_belt,
 506 |             thumbnails: Vec::new(),
 507 |         })
 508 |     }
 509 | 
 510 |     pub fn resize(&mut self, size: PhysicalSize<u32>) {
 511 |         if size.width == 0 || size.height == 0 {
 512 |             return;
 513 |         }
 514 |         self.config.width = size.width;
 515 |         self.config.height = size.height;
 516 |         self.surface.configure(&self.device, &self.config);
 517 |         // wgpu_glyph text positions update automatically with new viewport dimensions
 518 |     }
 519 | 
 520 |     pub fn load_image(&mut self, image_data: &ImageData) -> Result<()> {
 521 |         // Explicitly destroy old GPU texture to free VRAM immediately
 522 |         if let Some(old_tex) = self.image_texture.take() {
 523 |             old_tex.destroy();
 524 |         }
 525 |         self.image_bind_group = None;
 526 |         self.image_bind_group_nearest = None;
 527 | 
 528 |         let width = image_data.width;
 529 |         let height = image_data.height;
 530 | 
 531 |         let texture = self.device.create_texture(&TextureDescriptor {
 532 |             label: Some("Image Texture"),
 533 |             size: Extent3d {
 534 |                 width,
 535 |                 height,
 536 |                 depth_or_array_layers: 1,
 537 |             },
 538 |             mip_level_count: 1,
 539 |             sample_count: 1,
 540 |             dimension: TextureDimension::D2,
 541 |             format: TextureFormat::Rgba8Unorm,
 542 |             usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
 543 |             view_formats: &[],
 544 |         });
 545 | 
 546 |         self.queue.write_texture(
 547 |             TexelCopyTextureInfo {
 548 |                 texture: &texture,
 549 |                 mip_level: 0,
 550 |                 origin: wgpu::Origin3d::ZERO,
 551 |                 aspect: TextureAspect::All,
 552 |             },
 553 |             image_data.as_rgba(),
 554 |             TexelCopyBufferLayout {
 555 |                 offset: 0,
 556 |                 bytes_per_row: Some(width * 4),
 557 |                 rows_per_image: Some(height),
 558 |             },
 559 |             Extent3d {
 560 |                 width,
 561 |                 height,
 562 |                 depth_or_array_layers: 1,
 563 |             },
 564 |         );
 565 | 
 566 |         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
 567 |         let bind_group_layout = self.pipeline.get_bind_group_layout(0);
 568 | 
 569 |         let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
 570 |             label: Some("Image Bind Group"),
 571 |             layout: &bind_group_layout,
 572 |             entries: &[
 573 |                 BindGroupEntry {
 574 |                     binding: 0,
 575 |                     resource: BindingResource::Buffer(
 576 |                         self.uniform_buffer.as_entire_buffer_binding(),
 577 |                     ),
 578 |                 },
 579 |                 BindGroupEntry {
 580 |                     binding: 1,
 581 |                     resource: BindingResource::Sampler(&self.sampler),
 582 |                 },
 583 |                 BindGroupEntry {
 584 |                     binding: 2,
 585 |                     resource: BindingResource::TextureView(&view),
 586 |                 },
 587 |             ],
 588 |         }));
 589 | 
 590 |         // Create pixel-perfect (nearest-neighbor) bind group
 591 |         let bind_group_nearest = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
 592 |             label: Some("Image Bind Group (Nearest)"),
 593 |             layout: &bind_group_layout,
 594 |             entries: &[
 595 |                 BindGroupEntry {
 596 |                     binding: 0,
 597 |                     resource: BindingResource::Buffer(
 598 |                         self.uniform_buffer.as_entire_buffer_binding(),
 599 |                     ),
 600 |                 },
 601 |                 BindGroupEntry {
 602 |                     binding: 1,
 603 |                     resource: BindingResource::Sampler(&self.sampler_nearest),
 604 |                 },
 605 |                 BindGroupEntry {
 606 |                     binding: 2,
 607 |                     resource: BindingResource::TextureView(&view),
 608 |                 },
 609 |             ],
 610 |         }));
 611 | 
 612 |         self.image_texture = Some(texture);
 613 |         self.image_bind_group = Some(bind_group);
 614 |         self.image_bind_group_nearest = Some(bind_group_nearest);
 615 |         self.image_size = Some((width, height));
 616 | 
 617 |         tracing::debug!("Loaded image into GPU: {width}x{height}");
 618 |         Ok(())
 619 |     }
 620 | 
 621 |     /// Upload a single thumbnail RGBA buffer to the GPU and return its bind group.
 622 |     /// The bind group uses `thumb_uniform_buffer` so thumbnails render with identity settings.
 623 |     pub fn upload_thumbnail(
 624 |         &mut self,
 625 |         path: std::path::PathBuf,
 626 |         rgba: &[u8],
 627 |         width: u32,
 628 |         height: u32,
 629 |     ) -> Result<()> {
 630 |         let texture = self.device.create_texture(&TextureDescriptor {
 631 |             label: Some("Thumbnail Texture"),
 632 |             size: Extent3d {
 633 |                 width,
 634 |                 height,
 635 |                 depth_or_array_layers: 1,
 636 |             },
 637 |             mip_level_count: 1,
 638 |             sample_count: 1,
 639 |             dimension: TextureDimension::D2,
 640 |             format: TextureFormat::Rgba8Unorm,
 641 |             usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
 642 |             view_formats: &[],
 643 |         });
 644 | 
 645 |         self.queue.write_texture(
 646 |             TexelCopyTextureInfo {
 647 |                 texture: &texture,
 648 |                 mip_level: 0,
 649 |                 origin: wgpu::Origin3d::ZERO,
 650 |                 aspect: TextureAspect::All,
 651 |             },
 652 |             rgba,
 653 |             TexelCopyBufferLayout {
 654 |                 offset: 0,
 655 |                 bytes_per_row: Some(width * 4),
 656 |                 rows_per_image: Some(height),
 657 |             },
 658 |             Extent3d {
 659 |                 width,
 660 |                 height,
 661 |                 depth_or_array_layers: 1,
 662 |             },
 663 |         );
 664 | 
 665 |         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
 666 |         let bind_group_layout = self.pipeline.get_bind_group_layout(0);
 667 | 
 668 |         let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
 669 |             label: Some("Thumbnail Bind Group"),
 670 |             layout: &bind_group_layout,
 671 |             entries: &[
 672 |                 BindGroupEntry {
 673 |                     binding: 0,
 674 |                     resource: BindingResource::Buffer(
 675 |                         self.thumb_uniform_buffer.as_entire_buffer_binding(),
 676 |                     ),
 677 |                 },
 678 |                 BindGroupEntry {
 679 |                     binding: 1,
 680 |                     resource: BindingResource::Sampler(&self.sampler),
 681 |                 },
 682 |                 BindGroupEntry {
 683 |                     binding: 2,
 684 |                     resource: BindingResource::TextureView(&view),
 685 |                 },
 686 |             ],
 687 |         }));
 688 | 
 689 |         self.thumbnails.push(ThumbnailEntry {
 690 |             path,
 691 |             bind_group,
 692 |             width,
 693 |             height,
 694 |             _texture: texture,
 695 |         });
 696 | 
 697 |         Ok(())
 698 |     }
 699 | 
 700 |     /// Remove all thumbnails and free their GPU textures.
 701 |     pub fn clear_thumbnails(&mut self) {
 702 |         // ThumbnailEntry holds the texture so dropping the Vec frees VRAM.
 703 |         self.thumbnails.clear();
 704 |     }
 705 | 
 706 |     /// Encode image draw commands into `encoder` targeting `view`.
 707 |     /// Does NOT submit or present — caller owns the frame lifetime.
 708 |     fn encode_image(
 709 |         &self,
 710 |         adjustments: &ImageAdjustments,
 711 |         view: &wgpu::TextureView,
 712 |         encoder: &mut wgpu::CommandEncoder,
 713 |     ) {
 714 |         let window_aspect_ratio = if self.config.height > 0 {
 715 |             self.config.width as f32 / self.config.height as f32
 716 |         } else {
 717 |             1.0
 718 |         };
 719 | 
 720 |         let uniforms = Uniforms {
 721 |             rotation: adjustments.rotation,
 722 |             aspect_ratio: self
 723 |                 .image_size
 724 |                 .map(|(w, h)| w as f32 / h as f32)
 725 |                 .unwrap_or(1.0),
 726 |             window_aspect_ratio,
 727 |             crop_x: adjustments.crop_rect[0],
 728 |             crop_y: adjustments.crop_rect[1],
 729 |             crop_w: adjustments.crop_rect[2],
 730 |             crop_h: adjustments.crop_rect[3],
 731 |             brightness: adjustments.brightness,
 732 |             contrast: adjustments.contrast,
 733 |             saturation: adjustments.saturation,
 734 |             hdr_toning: if adjustments.hdr_toning { 1.0 } else { 0.0 },
 735 |             _padding: 0.0,
 736 |         };
 737 | 
 738 |         self.queue
 739 |             .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
 740 | 
 741 |         let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
 742 |             label: Some("Render Pass"),
 743 |             color_attachments: &[Some(RenderPassColorAttachment {
 744 |                 view,
 745 |                 resolve_target: None,
 746 |                 ops: Operations {
 747 |                     load: LoadOp::Clear(wgpu::Color::BLACK),
 748 |                     store: StoreOp::Store,
 749 |                 },
 750 |                 depth_slice: None,
 751 |             })],
 752 |             depth_stencil_attachment: None,
 753 |             timestamp_writes: None,
 754 |             occlusion_query_set: None,
 755 |         });
 756 | 
 757 |         if let Some(bind_group) = &self.image_bind_group {
 758 |             render_pass.set_pipeline(&self.pipeline);
 759 |             render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
 760 | 
 761 |             // Use nearest-neighbor sampler for pixel-perfect mode
 762 |             if adjustments.pixel_perfect {
 763 |                 if let Some(bg) = &self.image_bind_group_nearest {
 764 |                     render_pass.set_bind_group(0, bg.as_ref(), &[]);
 765 |                 }
 766 |             } else {
 767 |                 render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
 768 |             }
 769 |             render_pass.draw(0..6, 0..1);
 770 |         }
 771 |     }
 772 | 
 773 |     /// Encode the thumbnail strip at the bottom of the screen.
 774 |     /// Draws a dark background, then each uploaded thumbnail in its slot.
 775 |     /// `active_idx` is the index into `self.thumbnails` that is currently displayed.
 776 |     fn encode_thumbnail_strip(
 777 |         &mut self,
 778 |         active_idx: Option<usize>,
 779 |         selected_indices: &std::collections::HashSet<usize>,
 780 |         view: &wgpu::TextureView,
 781 |         encoder: &mut wgpu::CommandEncoder,
 782 |     ) {
 783 |         let win_w = self.config.width;
 784 |         let win_h = self.config.height;
 785 | 
 786 |         if win_h < STRIP_HEIGHT_PX || win_w < THUMB_SLOT_W {
 787 |             return;
 788 |         }
 789 | 
 790 |         let strip_y = win_h - STRIP_HEIGHT_PX;
 791 | 
 792 |         // --- Pass 1: darken the strip background using the crop pipeline --------
 793 |         {
 794 |             let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
 795 |                 label: Some("Thumbnail Strip Background"),
 796 |                 color_attachments: &[Some(RenderPassColorAttachment {
 797 |                     view,
 798 |                     resolve_target: None,
 799 |                     ops: Operations {
 800 |                         load: LoadOp::Load,
 801 |                         store: StoreOp::Store,
 802 |                     },
 803 |                     depth_slice: None,
 804 |                 })],
 805 |                 depth_stencil_attachment: None,
 806 |                 timestamp_writes: None,
 807 |                 occlusion_query_set: None,
 808 |             });
 809 |             pass.set_pipeline(&self.crop_pipeline);
 810 |             pass.set_scissor_rect(0, strip_y, win_w, STRIP_HEIGHT_PX);
 811 |             pass.draw(0..4, 0..1);
 812 |         }
 813 | 
 814 |         // --- Pass 2: draw each thumbnail using set_viewport --------------------
 815 |         let n = self.thumbnails.len();
 816 |         if n == 0 {
 817 |             return;
 818 |         }
 819 | 
 820 |         // Centre the strip horizontally; if too many thumbnails, they scroll left
 821 |         let total_w = n as u32 * THUMB_SLOT_W;
 822 |         let start_x: i64 = if total_w <= win_w {
 823 |             ((win_w - total_w) / 2) as i64
 824 |         } else if let Some(ai) = active_idx {
 825 |             // Keep active thumbnail in view
 826 |             let active_cx = ai as i64 * THUMB_SLOT_W as i64 + THUMB_SLOT_W as i64 / 2;
 827 |             let half_win = win_w as i64 / 2;
 828 |             let raw = half_win - active_cx;
 829 |             raw.clamp(win_w as i64 - total_w as i64, 0)
 830 |         } else {
 831 |             0
 832 |         };
 833 | 
 834 |         // Padding inside a slot so the thumbnail image is centred
 835 |         let pad = (THUMB_SLOT_W - THUMB_SIZE) / 2;
 836 | 
 837 |         // We need to iterate by index to capture bind groups - collect them first
 838 |         let bind_groups: Vec<(Arc<BindGroup>, u32, u32)> = self
 839 |             .thumbnails
 840 |             .iter()
 841 |             .map(|t| (Arc::clone(&t.bind_group), t.width, t.height))
 842 |             .collect();
 843 | 
 844 |         for (i, (bg, tw, th)) in bind_groups.iter().enumerate() {
 845 |             let slot_x = start_x + (i as i64) * THUMB_SLOT_W as i64;
 846 |             // Skip thumbnails fully outside the window
 847 |             if slot_x + THUMB_SLOT_W as i64 <= 0 || slot_x >= win_w as i64 {
 848 |                 continue;
 849 |             }
 850 | 
 851 |             // Compute aspect-fit rect inside the slot's THUMB_SIZE square
 852 |             let thumb_ar = *tw as f32 / (*th).max(1) as f32;
 853 |             let (fit_w, fit_h) = if thumb_ar >= 1.0 {
 854 |                 (THUMB_SIZE, (THUMB_SIZE as f32 / thumb_ar).round() as u32)
 855 |             } else {
 856 |                 ((THUMB_SIZE as f32 * thumb_ar).round() as u32, THUMB_SIZE)
 857 |             };
 858 | 
 859 |             // Centre fit rect inside the THUMB_SIZE square
 860 |             let offset_x = (THUMB_SIZE - fit_w) / 2;
 861 |             let offset_y = (THUMB_SIZE - fit_h) / 2;
 862 | 
 863 |             let vp_x = (slot_x + pad as i64 + offset_x as i64).max(0) as u32;
 864 |             let vp_y = strip_y + pad + offset_y;
 865 |             let vp_w = fit_w.min(win_w.saturating_sub(vp_x));
 866 |             let vp_h = fit_h;
 867 | 
 868 |             if vp_w == 0 || vp_h == 0 {
 869 |                 continue;
 870 |             }
 871 | 
 872 |             // Update thumb uniforms for this thumbnail's aspect ratio
 873 |             let ar = *tw as f32 / (*th).max(1) as f32;
 874 |             let thumb_uniforms = Uniforms {
 875 |                 rotation: 0.0,
 876 |                 aspect_ratio: ar,
 877 |                 window_aspect_ratio: ar, // square viewport ≈ identity
 878 |                 crop_x: 0.0,
 879 |                 crop_y: 0.0,
 880 |                 crop_w: 1.0,
 881 |                 crop_h: 1.0,
 882 |                 brightness: 1.0,
 883 |                 contrast: 1.0,
 884 |                 saturation: 1.0,
 885 |                 hdr_toning: 0.0,
 886 |                 _padding: 0.0,
 887 |             };
 888 |             self.queue.write_buffer(
 889 |                 &self.thumb_uniform_buffer,
 890 |                 0,
 891 |                 bytemuck::bytes_of(&thumb_uniforms),
 892 |             );
 893 | 
 894 |             let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
 895 |                 label: Some("Thumbnail Draw"),
 896 |                 color_attachments: &[Some(RenderPassColorAttachment {
 897 |                     view,
 898 |                     resolve_target: None,
 899 |                     ops: Operations {
 900 |                         load: LoadOp::Load,
 901 |                         store: StoreOp::Store,
 902 |                     },
 903 |                     depth_slice: None,
 904 |                 })],
 905 |                 depth_stencil_attachment: None,
 906 |                 timestamp_writes: None,
 907 |                 occlusion_query_set: None,
 908 |             });
 909 | 
 910 |             pass.set_pipeline(&self.pipeline);
 911 |             pass.set_viewport(vp_x as f32, vp_y as f32, vp_w as f32, vp_h as f32, 0.0, 1.0);
 912 |             // Scissor to strip area so thumbnails cannot bleed outside
 913 |             pass.set_scissor_rect(0, strip_y, win_w, STRIP_HEIGHT_PX);
 914 |             pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
 915 |             pass.set_bind_group(0, bg.as_ref(), &[]);
 916 |             pass.draw(0..6, 0..1);
 917 |         }
 918 | 
 919 |         // --- Pass 3: highlight borders (active and selected) ---------------------
 920 |         for (i, _) in bind_groups.iter().enumerate() {
 921 |             let is_active = active_idx == Some(i);
 922 |             let is_selected = selected_indices.contains(&i);
 923 | 
 924 |             if is_active || is_selected {
 925 |                 let slot_x = start_x + i as i64 * THUMB_SLOT_W as i64;
 926 |                 if slot_x + THUMB_SLOT_W as i64 > 0 && slot_x < win_w as i64 {
 927 |                     let bx = slot_x as i32 + 2;
 928 |                     let by = strip_y as i32 + 2;
 929 |                     let bw = THUMB_SLOT_W as i32 - 4;
 930 |                     let bh = STRIP_HEIGHT_PX as i32 - 4;
 931 |                     // Draw a subtle border for selected, prominent for active
 932 |                     let bsize = if is_active { 2 } else { 1 };
 933 | 
 934 |                     let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
 935 |                         label: Some("Thumbnail Border Pass"),
 936 |                         color_attachments: &[Some(RenderPassColorAttachment {
 937 |                             view,
 938 |                             resolve_target: None,
 939 |                             ops: Operations {
 940 |                                 load: LoadOp::Load,
 941 |                                 store: StoreOp::Store,
 942 |                             },
 943 |                             depth_slice: None,
 944 |                         })],
 945 |                         depth_stencil_attachment: None,
 946 |                         timestamp_writes: None,
 947 |                         occlusion_query_set: None,
 948 |                     });
 949 |                     pass.set_pipeline(&self.crop_pipeline);
 950 | 
 951 |                     // Top border
 952 |                     let _b_win_w = win_w;
 953 |                     let _b_win_h = win_h;
 954 | 
 955 |                     if by >= 0 {
 956 |                         pass.set_scissor_rect(
 957 |                             bx.max(0) as u32,
 958 |                             by.max(0) as u32,
 959 |                             bw.max(0) as u32,
 960 |                             bsize,
 961 |                         );
 962 |                         pass.draw(0..4, 0..1);
 963 |                     }
 964 |                     // Bottom border
 965 |                     let bot = by + bh - bsize as i32;
 966 |                     if bot < win_h as i32 {
 967 |                         pass.set_scissor_rect(
 968 |                             bx.max(0) as u32,
 969 |                             bot.max(0) as u32,
 970 |                             bw.max(0) as u32,
 971 |                             bsize,
 972 |                         );
 973 |                         pass.draw(0..4, 0..1);
 974 |                     }
 975 |                     // Left border
 976 |                     if bx >= 0 {
 977 |                         pass.set_scissor_rect(
 978 |                             bx.max(0) as u32,
 979 |                             by.max(0) as u32,
 980 |                             bsize,
 981 |                             bh.max(0) as u32,
 982 |                         );
 983 |                         pass.draw(0..4, 0..1);
 984 |                     }
 985 |                     // Right border
 986 |                     let rx = bx + bw - bsize as i32;
 987 |                     if rx < win_w as i32 {
 988 |                         pass.set_scissor_rect(
 989 |                             rx.max(0) as u32,
 990 |                             by.max(0) as u32,
 991 |                             bsize,
 992 |                             bh.max(0) as u32,
 993 |                         );
 994 |                         pass.draw(0..4, 0..1);
 995 |                     }
 996 |                 }
 997 |             }
 998 |         }
 999 |     }
1000 | 
1001 |     /// Encode UI overlay commands into `encoder` targeting `view`.
1002 |     /// Does NOT submit or present — caller owns the frame lifetime.
1003 |     #[allow(clippy::too_many_arguments)]
1004 |     fn encode_ui_overlay(
1005 |         &mut self,
1006 |         is_cropping: bool,
1007 |         crop_rect: [f32; 4],
1008 |         status_text: Option<&str>,
1009 |         show_help: bool,
1010 |         sidebar_files: Option<&[String]>,
1011 |         show_thumbnail_strip: bool,
1012 |         exif_text: Option<&str>,
1013 |         show_histogram: bool,
1014 |         histogram_data: Option<&([u32; 256], [u32; 256], [u32; 256])>,
1015 |         view: &wgpu::TextureView,
1016 |         encoder: &mut wgpu::CommandEncoder,
1017 |     ) {
1018 |         // --- this is the body of the old render_ui_overlay, minus frame acquire/present ---
1019 | 
1020 |         // 1. Text rendering — queue all sections
1021 |         let scale = self.scale_factor as f32;
1022 |         let has_text = true; // We queue navigation arrows at minimum
1023 | 
1024 |         #[allow(unused_variables)]
1025 |         let help_text = "Shortcuts:\nA/W: Prev Image\nD/S: Next Image\nR: Rotate\nC: Toggle Crop\nH: Toggle HDR\nCtrl+S: Save\nF: Toggle Sidebar\nT: Toggle Thumbnails\nEsc: Quit";
1026 |         let sidebar_list_text: String = sidebar_files
1027 |             .map(|files| {
1028 |                 files
1029 |                     .iter()
1030 |                     .enumerate()
1031 |                     .map(|(i, name)| format!("{}. {name}", i + 1))
1032 |                     .collect::<Vec<_>>()
1033 |                     .join("\n")
1034 |             })
1035 |             .unwrap_or_default();
1036 | 
1037 |         // Navigation elements
1038 |         let nav_y = if show_thumbnail_strip && !self.thumbnails.is_empty() {
1039 |             // Push nav arrows up above the strip
1040 |             (self.config.height as f32 - STRIP_HEIGHT_PX as f32) / 2.0
1041 |         } else {
1042 |             self.config.height as f32 / 2.0
1043 |         };
1044 | 
1045 |         self.text_brush.queue(
1046 |             Section::default()
1047 |                 .add_text(
1048 |                     Text::new("◀")
1049 |                         .with_scale(48.0 * scale)
1050 |                         .with_color([0.8f32, 0.8, 0.8, 0.6]),
1051 |                 )
1052 |                 .with_screen_position((20.0 * scale, nav_y)),
1053 |         );
1054 |         self.text_brush.queue(
1055 |             Section::default()
1056 |                 .add_text(
1057 |                     Text::new("▶")
1058 |                         .with_scale(48.0 * scale)
1059 |                         .with_color([0.8f32, 0.8, 0.8, 0.6]),
1060 |                 )
1061 |                 .with_screen_position((self.config.width as f32 - 60.0 * scale, nav_y)),
1062 |         );
1063 | 
1064 |         // Status text — move it above the strip when strip is visible
1065 |         let status_y = if show_thumbnail_strip && !self.thumbnails.is_empty() {
1066 |             self.config.height as f32 - STRIP_HEIGHT_PX as f32 - 28.0 * scale
1067 |         } else {
1068 |             self.config.height as f32 - 30.0 * scale
1069 |         };
1070 | 
1071 |         if let Some(status) = status_text {
1072 |             self.text_brush.queue(
1073 |                 Section::default()
1074 |                     .add_text(
1075 |                         Text::new(status)
1076 |                             .with_scale(18.0 * scale)
1077 |                             .with_color([1.0f32, 1.0, 1.0, 1.0]),
1078 |                     )
1079 |                     .with_screen_position((10.0 * scale, status_y)),
1080 |             );
1081 |         }
1082 | 
1083 |         // --- Histogram Rendering ---
1084 |         if show_histogram {
1085 |             if let Some((r_hist, g_hist, b_hist)) = histogram_data {
1086 |                 let h_w = 256.0 * scale;
1087 |                 let h_h = 100.0 * scale;
1088 |                 let h_x = self.config.width as f32 - h_w - 10.0 * scale;
1089 |                 let h_y = 10.0 * scale; // top right
1090 | 
1091 |                 // Background
1092 |                 self.text_brush.queue(
1093 |                     Section::default()
1094 |                         .add_text(
1095 |                             Text::new("▇")
1096 |                                 .with_scale(h_h)
1097 |                                 .with_color([0.0, 0.0, 0.0, 0.4]),
1098 |                         )
1099 |                         .with_screen_position((h_x, h_y))
1100 |                         .with_bounds((h_w, h_h)),
1101 |                 );
1102 | 
1103 |                 let max_val = r_hist
1104 |                     .iter()
1105 |                     .chain(g_hist.iter())
1106 |                     .chain(b_hist.iter())
1107 |                     .max()
1108 |                     .copied()
1109 |                     .unwrap_or(1)
1110 |                     .max(1);
1111 | 
1112 |                 // Draw R, G, B bars
1113 |                 for (chan_idx, (hist, color)) in [
1114 |                     (r_hist, [1.0f32, 0.3, 0.3, 0.6]),
1115 |                     (g_hist, [0.3f32, 1.0, 0.3, 0.6]),
1116 |                     (b_hist, [0.3f32, 0.3, 1.0, 0.6]),
1117 |                 ]
1118 |                 .into_iter()
1119 |                 .enumerate()
1120 |                 {
1121 |                     let mut bars = String::new();
1122 |                     // Subsample to 64 bins for performance and readability in text
1123 |                     for i in (0..256).step_by(4) {
1124 |                         let val = hist[i..i + 4].iter().sum::<u32>() / 4;
1125 |                         let bar_h = (val as f32 / max_val as f32 * 8.0).round() as u32;
1126 |                         let char = match bar_h {
1127 |                             0 => " ",
1128 |                             1 => " ",
1129 |                             2 => "▂",
1130 |                             3 => "▃",
1131 |                             4 => "▄",
1132 |                             5 => "▅",
1133 |                             6 => "▆",
1134 |                             7 => "▇",
1135 |                             _ => "█",
1136 |                         };
1137 |                         bars.push_str(char);
1138 |                     }
1139 | 
1140 |                     self.text_brush.queue(
1141 |                         Section::default()
1142 |                             .add_text(Text::new(&bars).with_scale(h_h / 4.0).with_color(color))
1143 |                             .with_screen_position((h_x, h_y + (chan_idx as f32 * h_h / 4.0))),
1144 |                     );
1145 |                 }
1146 |             }
1147 |         }
1148 | 
1149 |         if show_help {
1150 |             self.text_brush.queue(
1151 |                 Section::default()
1152 |                     .add_text(
1153 |                         Text::new(help_text)
1154 |                             .with_scale(16.0 * scale)
1155 |                             .with_color([0.9f32, 0.9, 0.9, 1.0]),
1156 |                     )
1157 |                     .with_screen_position((10.0 * scale, 10.0 * scale)),
1158 |             );
1159 |         }
1160 | 
1161 |         if let Some(exif) = exif_text {
1162 |             self.text_brush.queue(
1163 |                 Section::default()
1164 |                     .add_text(
1165 |                         Text::new(exif)
1166 |                             .with_scale(15.0 * scale)
1167 |                             .with_color([0.85f32, 0.95, 1.0, 1.0]),
1168 |                     )
1169 |                     .with_screen_position((10.0 * scale, 10.0 * scale)),
1170 |             );
1171 |         }
1172 | 
1173 |         if sidebar_files.map(|f| !f.is_empty()).unwrap_or(false) {
1174 |             self.text_brush.queue(
1175 |                 Section::default()
1176 |                     .add_text(
1177 |                         Text::new(&sidebar_list_text)
1178 |                             .with_scale(14.0 * scale)
1179 |                             .with_color([0.85f32, 0.95, 1.0, 1.0]),
1180 |                     )
1181 |                     .with_screen_position((self.config.width as f32 - 280.0 * scale, 10.0 * scale))
1182 |                     .with_bounds((270.0 * scale, self.config.height as f32 - 20.0 * scale)),
1183 |             );
1184 |         }
1185 | 
1186 |         {
1187 |             let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
1188 |                 label: Some("UI Overlay Pass"),
1189 |                 color_attachments: &[Some(RenderPassColorAttachment {
1190 |                     view,
1191 |                     resolve_target: None,
1192 |                     ops: Operations {
1193 |                         load: LoadOp::Load,
1194 |                         store: StoreOp::Store,
1195 |                     },
1196 |                     depth_slice: None,
1197 |                 })],
1198 |                 depth_stencil_attachment: None,
1199 |                 timestamp_writes: None,
1200 |                 occlusion_query_set: None,
1201 |             });
1202 | 
1203 |             if is_cropping {
1204 |                 render_pass.set_pipeline(&self.crop_pipeline);
1205 | 
1206 |                 let win_w = self.config.width;
1207 |                 let win_h = self.config.height;
1208 | 
1209 |                 let cx = (crop_rect[0] * win_w as f32) as u32;
1210 |                 let cy = (crop_rect[1] * win_h as f32) as u32;
1211 |                 let cw = (crop_rect[2] * win_w as f32) as u32;
1212 |                 let ch = (crop_rect[3] * win_h as f32) as u32;
1213 | 
1214 |                 let cx = cx.min(win_w.saturating_sub(1));
1215 |                 let cy = cy.min(win_h.saturating_sub(1));
1216 |                 let cw = cw.min(win_w - cx);
1217 |                 let ch = ch.min(win_h - cy);
1218 | 
1219 |                 if cy > 0 {
1220 |                     render_pass.set_scissor_rect(0, 0, win_w, cy);
1221 |                     render_pass.draw(0..4, 0..1);
1222 |                 }
1223 |                 if cy + ch < win_h {
1224 |                     render_pass.set_scissor_rect(0, cy + ch, win_w, win_h - (cy + ch));
1225 |                     render_pass.draw(0..4, 0..1);
1226 |                 }
1227 |                 if cx > 0 {
1228 |                     render_pass.set_scissor_rect(0, cy, cx, ch);
1229 |                     render_pass.draw(0..4, 0..1);
1230 |                 }
1231 |                 if cx + cw < win_w {
1232 |                     render_pass.set_scissor_rect(cx + cw, cy, win_w - (cx + cw), ch);
1233 |                     render_pass.draw(0..4, 0..1);
1234 |                 }
1235 |             }
1236 |         } // drop render_pass
1237 | 
1238 |         // Draw queued text
1239 |         if has_text {
1240 |             if let Err(e) = self.text_brush.draw_queued(
1241 |                 &self.device,
1242 |                 &mut self.staging_belt,
1243 |                 encoder,
1244 |                 view,
1245 |                 self.config.width,
1246 |                 self.config.height,
1247 |             ) {
1248 |                 tracing::warn!("Text draw error: {e}");
1249 |             }
1250 |             self.staging_belt.finish();
1251 |         }
1252 |     }
1253 | 
1254 |     /// Combined render: acquires surface texture once, draws image then UI, presents once.
1255 |     /// This is the correct path for still images — avoids the double-present black screen bug.
1256 |     #[allow(clippy::too_many_arguments)]
1257 |     pub fn render_frame(
1258 |         &mut self,
1259 |         adjustments: &ImageAdjustments,
1260 |         is_cropping: bool,
1261 |         crop_rect: [f32; 4],
1262 |         status_text: Option<&str>,
1263 |         show_help: bool,
1264 |         sidebar_files: Option<&[String]>,
1265 |         show_thumbnail_strip: bool,
1266 |         active_thumb_idx: Option<usize>,
1267 |         selected_indices: &std::collections::HashSet<usize>,
1268 |         exif_text: Option<&str>,
1269 |         show_histogram: bool,
1270 |         histogram_data: Option<&([u32; 256], [u32; 256], [u32; 256])>,
1271 |     ) -> Result<()> {
1272 |         let frame = self
1273 |             .surface
1274 |             .get_current_texture()
1275 |             .context("Failed to get current surface texture")?;
1276 |         let view = frame
1277 |             .texture
1278 |             .create_view(&wgpu::TextureViewDescriptor::default());
1279 |         let mut encoder = self
1280 |             .device
1281 |             .create_command_encoder(&CommandEncoderDescriptor {
1282 |                 label: Some("Frame Encoder"),
1283 |             });
1284 | 
1285 |         // 1. Draw image (clears to black first)
1286 |         self.encode_image(adjustments, &view, &mut encoder);
1287 | 
1288 |         // 2. Draw thumbnail strip (before text so text overlays on top)
1289 |         if show_thumbnail_strip && !self.thumbnails.is_empty() {
1290 |             self.encode_thumbnail_strip(active_thumb_idx, selected_indices, &view, &mut encoder);
1291 |         }
1292 | 
1293 |         // 3. Draw UI overlay on top (LoadOp::Load to preserve image pixels)
1294 |         self.encode_ui_overlay(
1295 |             is_cropping,
1296 |             crop_rect,
1297 |             status_text,
1298 |             show_help,
1299 |             sidebar_files,
1300 |             show_thumbnail_strip,
1301 |             exif_text,
1302 |             show_histogram,
1303 |             histogram_data,
1304 |             &view,
1305 |             &mut encoder,
1306 |         );
1307 | 
1308 |         // 4. Single submit + present
1309 |         self.queue.submit([encoder.finish()]);
1310 |         self.staging_belt.recall();
1311 |         frame.present();
1312 |         Ok(())
1313 |     }
1314 | 
1315 |     pub fn render_ui_overlay(
1316 |         &mut self,
1317 |         is_cropping: bool,
1318 |         crop_rect: [f32; 4],
1319 |         status_text: Option<&str>,
1320 |         show_help: bool,
1321 |         sidebar_files: Option<&[String]>,
1322 |     ) -> Result<()> {
1323 |         let frame = self
1324 |             .surface
1325 |             .get_current_texture()
1326 |             .context("Failed to get current surface texture")?;
1327 |         let view = frame
1328 |             .texture
1329 |             .create_view(&wgpu::TextureViewDescriptor::default());
1330 |         let mut encoder = self
1331 |             .device
1332 |             .create_command_encoder(&CommandEncoderDescriptor {
1333 |                 label: Some("UI Render Encoder"),
1334 |             });
1335 |         self.encode_ui_overlay(
1336 |             is_cropping,
1337 |             crop_rect,
1338 |             status_text,
1339 |             show_help,
1340 |             sidebar_files,
1341 |             false,
1342 |             None,
1343 |             false,
1344 |             None,
1345 |             &view,
1346 |             &mut encoder,
1347 |         );
1348 |         self.queue.submit([encoder.finish()]);
1349 |         self.staging_belt.recall();
1350 |         frame.present();
1351 |         Ok(())
1352 |     }
1353 | 
1354 |     pub fn render_loading(&self) -> Result<()> {
1355 |         let frame = self
1356 |             .surface
1357 |             .get_current_texture()
1358 |             .context("Failed to get current surface texture")?;
1359 | 
1360 |         let view = frame
1361 |             .texture
1362 |             .create_view(&wgpu::TextureViewDescriptor::default());
1363 | 
1364 |         let mut encoder = self
1365 |             .device
1366 |             .create_command_encoder(&CommandEncoderDescriptor {
1367 |                 label: Some("Loading Encoder"),
1368 |             });
1369 | 
1370 |         {
1371 |             let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
1372 |                 label: Some("Loading Pass"),
1373 |                 color_attachments: &[Some(RenderPassColorAttachment {
1374 |                     view: &view,
1375 |                     resolve_target: None,
1376 |                     ops: Operations {
1377 |                         // Dark gray color for loading screen
1378 |                         load: LoadOp::Clear(wgpu::Color {
1379 |                             r: 0.1,
1380 |                             g: 0.1,
1381 |                             b: 0.1,
1382 |                             a: 1.0,
1383 |                         }),
1384 |                         store: StoreOp::Store,
1385 |                     },
1386 |                     depth_slice: None,
1387 |                 })],
1388 |                 depth_stencil_attachment: None,
1389 |                 timestamp_writes: None,
1390 |                 occlusion_query_set: None,
1391 |             });
1392 |         }
1393 | 
1394 |         self.queue.submit([encoder.finish()]);
1395 |         frame.present();
1396 |         Ok(())
1397 |     }
1398 | 
1399 |     pub fn has_image(&self) -> bool {
1400 |         self.image_texture.is_some()
1401 |     }
1402 | 
1403 |     pub fn gif_frame_count(&self) -> usize {
1404 |         self.gif_textures.len()
1405 |     }
1406 | 
1407 |     /// Upload all GIF frames to GPU once. During playback, swap_gif_frame is
1408 |     /// called with an index — zero CPU→GPU copies per frame after initial upload.
1409 |     pub fn preload_gif_textures(&mut self, frames: &[ImageData]) -> Result<()> {
1410 |         // Explicitly destroy old GIF textures to free VRAM
1411 |         for (tex, _) in self.gif_textures.drain(..) {
1412 |             tex.destroy();
1413 |         }
1414 | 
1415 |         // Cap VRAM usage for GIF frames (256 MB max)
1416 |         const MAX_GIF_VRAM_BYTES: u64 = 256 * 1024 * 1024;
1417 |         let total_vram: u64 = frames
1418 |             .iter()
1419 |             .map(|f| (f.width as u64) * (f.height as u64) * 4)
1420 |             .sum();
1421 |         let frames_to_load = if total_vram > MAX_GIF_VRAM_BYTES && !frames.is_empty() {
1422 |             let per_frame = total_vram / frames.len() as u64;
1423 |             let max_frames = (MAX_GIF_VRAM_BYTES / per_frame).max(1) as usize;
1424 |             tracing::warn!(
1425 |                 "GIF VRAM budget exceeded ({:.1} MB), limiting to {} of {} frames",
1426 |                 total_vram as f64 / 1_048_576.0,
1427 |                 max_frames,
1428 |                 frames.len()
1429 |             );
1430 |             &frames[..max_frames]
1431 |         } else {
1432 |             frames
1433 |         };
1434 | 
1435 |         let bind_group_layout = self.pipeline.get_bind_group_layout(0);
1436 | 
1437 |         for frame in frames_to_load {
1438 |             let (width, height) = (frame.width, frame.height);
1439 |             let texture = self.device.create_texture(&TextureDescriptor {
1440 |                 label: Some("GIF Frame Texture"),
1441 |                 size: Extent3d {
1442 |                     width,
1443 |                     height,
1444 |                     depth_or_array_layers: 1,
1445 |                 },
1446 |                 mip_level_count: 1,
1447 |                 sample_count: 1,
1448 |                 dimension: TextureDimension::D2,
1449 |                 format: TextureFormat::Rgba8Unorm,
1450 |                 usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
1451 |                 view_formats: &[],
1452 |             });
1453 |             self.queue.write_texture(
1454 |                 TexelCopyTextureInfo {
1455 |                     texture: &texture,
1456 |                     mip_level: 0,
1457 |                     origin: wgpu::Origin3d::ZERO,
1458 |                     aspect: TextureAspect::All,
1459 |                 },
1460 |                 frame.as_rgba(),
1461 |                 TexelCopyBufferLayout {
1462 |                     offset: 0,
1463 |                     bytes_per_row: Some(width * 4),
1464 |                     rows_per_image: Some(height),
1465 |                 },
1466 |                 Extent3d {
1467 |                     width,
1468 |                     height,
1469 |                     depth_or_array_layers: 1,
1470 |                 },
1471 |             );
1472 |             let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
1473 |             let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
1474 |                 label: Some("GIF Frame Bind Group"),
1475 |                 layout: &bind_group_layout,
1476 |                 entries: &[
1477 |                     BindGroupEntry {
1478 |                         binding: 0,
1479 |                         resource: BindingResource::Buffer(
1480 |                             self.uniform_buffer.as_entire_buffer_binding(),
1481 |                         ),
1482 |                     },
1483 |                     BindGroupEntry {
1484 |                         binding: 1,
1485 |                         resource: BindingResource::Sampler(&self.sampler),
1486 |                     },
1487 |                     BindGroupEntry {
1488 |                         binding: 2,
1489 |                         resource: BindingResource::TextureView(&view),
1490 |                     },
1491 |                 ],
1492 |             }));
1493 |             self.gif_textures.push((texture, bind_group));
1494 |         }
1495 |         let len = self.gif_textures.len();
1496 |         tracing::debug!("Preloaded {len} GIF frames to GPU");
1497 |         Ok(())
1498 |     }
1499 | 
1500 |     /// Swap the active bind group to a cached GIF frame (no GPU transfer).
1501 |     pub fn swap_gif_frame(&mut self, idx: usize) {
1502 |         if let Some((tex, bg)) = self.gif_textures.get(idx) {
1503 |             self.image_size = Some((tex.width(), tex.height()));
1504 |             self.image_bind_group = Some(Arc::clone(bg));
1505 |         }
1506 |     }
1507 | 
1508 |     pub fn update_scale_factor(&mut self, scale: f64) {
1509 |         self.scale_factor = scale;
1510 |     }
1511 | 
1512 |     /// Return the index into `self.thumbnails` for a given pixel click coordinate
1513 |     /// within the thumbnail strip. Returns None if the click is not in the strip.
1514 |     pub fn thumbnail_index_at(&self, x: f64, y: f64) -> Option<usize> {
1515 |         let win_h = self.config.height as f64;
1516 |         let win_w = self.config.width as f64;
1517 |         let strip_y = win_h - STRIP_HEIGHT_PX as f64;
1518 | 
1519 |         if y < strip_y || self.thumbnails.is_empty() {
1520 |             return None;
1521 |         }
1522 | 
1523 |         let n = self.thumbnails.len();
1524 |         let total_w = n as f64 * THUMB_SLOT_W as f64;
1525 |         let start_x: f64 = if total_w <= win_w {
1526 |             (win_w - total_w) / 2.0
1527 |         } else {
1528 |             0.0
1529 |         };
1530 | 
1531 |         if x < start_x || x >= start_x + total_w {
1532 |             return None;
1533 |         }
1534 | 
1535 |         let slot = ((x - start_x) / THUMB_SLOT_W as f64) as usize;
1536 |         if slot < n {
1537 |             Some(slot)
1538 |         } else {
1539 |             None
1540 |         }
1541 |     }
1542 | }
1543 | 
1544 | #[cfg(test)]
1545 | mod tests {
1546 |     use super::*;
1547 | 
1548 |     #[test]
1549 |     fn test_image_adjustments_default() {
1550 |         let adj = ImageAdjustments::default();
1551 | 
1552 |         assert_eq!(adj.brightness, 1.0);
1553 |         assert_eq!(adj.contrast, 1.0);
1554 |         assert_eq!(adj.saturation, 1.0);
1555 |         assert_eq!(adj.rotation, 0.0);
1556 |         assert_eq!(adj.crop_rect, [0.0, 0.0, 1.0, 1.0]);
1557 |         assert_eq!(adj.crop_rect_target, [0.0, 0.0, 1.0, 1.0]);
1558 |         assert!(!adj.hdr_toning);
1559 |     }
1560 | }
```

### File: `src\image_backend.rs`

- Size: 30651 bytes
- Modified: 2026-03-07 11:44:15 UTC

```rust
   1 | //! Image Backend - Cross-platform image loading
   2 | //!
   3 | //! Handles image decoding using the `image` crate with support for
   4 | //! multiple formats including HEIC via libheif-rs.
   5 | 
   6 | use anyhow::{Context, Result};
   7 | use image::{DynamicImage, GenericImageView, ImageFormat};
   8 | use std::path::Path;
   9 | use thiserror::Error;
  10 | 
  11 | #[derive(Error, Debug)]
  12 | pub enum ImageError {
  13 |     #[error("Failed to load image: {0}")]
  14 |     LoadError(String),
  15 |     #[error("Unsupported format: {0}")]
  16 |     UnsupportedFormat(String),
  17 |     #[error("IO error: {0}")]
  18 |     IoError(#[from] std::io::Error),
  19 |     #[error("Image decoding error: {0}")]
  20 |     DecodeError(String),
  21 | }
  22 | 
  23 | /// Supported image formats
  24 | #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  25 | pub enum ImageFormatType {
  26 |     Jpeg,
  27 |     Png,
  28 |     Gif,
  29 |     Bmp,
  30 |     Tiff,
  31 |     WebP,
  32 |     Heic,
  33 |     Avif,
  34 |     /// RAW camera formats (Canon, Nikon, Sony, Fuji, Adobe DNG, etc.)
  35 |     Raw,
  36 |     /// SVG vector graphics
  37 |     Svg,
  38 |     Unknown,
  39 | }
  40 | 
  41 | impl ImageFormatType {
  42 |     pub fn from_extension(ext: &str) -> Self {
  43 |         match ext.to_lowercase().as_str() {
  44 |             "jpg" | "jpeg" => Self::Jpeg,
  45 |             "png" => Self::Png,
  46 |             "gif" => Self::Gif,
  47 |             "bmp" => Self::Bmp,
  48 |             "tiff" | "tif" => Self::Tiff,
  49 |             "webp" => Self::WebP,
  50 |             "heic" | "heif" => Self::Heic,
  51 |             "avif" => Self::Avif,
  52 |             // RAW formats: Canon, Nikon, Sony, Fuji, Olympus, Panasonic, Leica, Adobe
  53 |             "cr2" | "cr3" | "crw"  // Canon
  54 |             | "nef" | "nrw"        // Nikon
  55 |             | "arw" | "srf" | "sr2" // Sony
  56 |             | "raf"                // Fujifilm
  57 |             | "orf"                // Olympus
  58 |             | "rw2"                // Panasonic
  59 |             | "dng"                // Adobe DNG (universal)
  60 |             | "mrw"                // Minolta
  61 |             | "pef"                // Pentax
  62 |             | "3fr"                // Hasselblad
  63 |             | "rwl"                // Leica
  64 |             | "raw" | "rw1"        // Generic
  65 |             => Self::Raw,
  66 |             "svg" => Self::Svg,
  67 |             _ => Self::Unknown,
  68 |         }
  69 |     }
  70 | 
  71 |     pub fn is_supported(&self) -> bool {
  72 |         match self {
  73 |             Self::Unknown => false,
  74 |             // RAW requires the `raw` feature, SVG requires `svg`
  75 |             #[cfg(not(feature = "raw"))]
  76 |             Self::Raw => false,
  77 |             #[cfg(not(feature = "svg"))]
  78 |             Self::Svg => false,
  79 |             _ => true,
  80 |         }
  81 |     }
  82 | }
  83 | 
  84 | /// Image data container with metadata
  85 | #[derive(Debug, Clone)]
  86 | pub struct ImageData {
  87 |     pub width: u32,
  88 |     pub height: u32,
  89 |     pub format: ImageFormatType,
  90 |     pub rgba_data: Vec<u8>,
  91 |     pub path: String,
  92 |     pub file_size_bytes: u64,
  93 |     pub frame_delay_ms: u32,
  94 |     pub exif_info: Option<String>,
  95 |     pub histogram: Option<([u32; 256], [u32; 256], [u32; 256])>,
  96 |     pub exif_loaded: bool,
  97 | }
  98 | 
  99 | impl ImageData {
 100 |     /// Get the raw RGBA bytes for GPU upload
 101 |     pub fn as_rgba(&self) -> &[u8] {
 102 |         &self.rgba_data
 103 |     }
 104 | 
 105 |     /// Load EXIF data lazily on demand
 106 |     pub fn load_exif(&mut self) {
 107 |         if self.exif_loaded {
 108 |             return;
 109 |         }
 110 |         self.exif_info = ImageBackend::extract_exif_lazy(&self.path);
 111 |         self.exif_loaded = true;
 112 |     }
 113 | 
 114 |     pub fn compute_histogram(&mut self) {
 115 |         if self.histogram.is_some() {
 116 |             return;
 117 |         }
 118 |         if self.rgba_data.is_empty() {
 119 |             return;
 120 |         }
 121 |         let mut r_hist = [0u32; 256];
 122 |         let mut g_hist = [0u32; 256];
 123 |         let mut b_hist = [0u32; 256];
 124 | 
 125 |         for chunk in self.rgba_data.chunks_exact(4) {
 126 |             r_hist[chunk[0] as usize] += 1;
 127 |             g_hist[chunk[1] as usize] += 1;
 128 |             b_hist[chunk[2] as usize] += 1;
 129 |         }
 130 | 
 131 |         self.histogram = Some((r_hist, g_hist, b_hist));
 132 |     }
 133 | 
 134 |     /// Get aspect ratio
 135 |     pub fn aspect_ratio(&self) -> f32 {
 136 |         self.width as f32 / self.height as f32
 137 |     }
 138 | 
 139 |     /// Get total pixel count
 140 |     pub fn pixel_count(&self) -> u32 {
 141 |         self.width * self.height
 142 |     }
 143 | }
 144 | 
 145 | /// Image backend for loading and decoding images
 146 | pub struct ImageBackend;
 147 | 
 148 | impl ImageBackend {
 149 |     /// Load an image from a file path
 150 |     pub fn load(path: &Path) -> Result<Vec<ImageData>> {
 151 |         let ext = path
 152 |             .extension()
 153 |             .and_then(|e| e.to_str())
 154 |             .unwrap_or("")
 155 |             .to_lowercase();
 156 | 
 157 |         let format = ImageFormatType::from_extension(&ext);
 158 | 
 159 |         tracing::debug!("Loading image: {:?}, format: {:?}", path, format);
 160 | 
 161 |         let metadata = std::fs::metadata(path).context("Failed to get file metadata")?;
 162 |         let file_size_bytes = metadata.len();
 163 | 
 164 |         if format == ImageFormatType::Gif {
 165 |             if let Ok(file) = std::fs::File::open(path) {
 166 |                 let reader = std::io::BufReader::new(file);
 167 |                 if let Ok(decoder) = image::codecs::gif::GifDecoder::new(reader) {
 168 |                     use image::AnimationDecoder;
 169 |                     let frames: Vec<image::Frame> =
 170 |                         decoder.into_frames().filter_map(|f| f.ok()).collect();
 171 |                     if !frames.is_empty() {
 172 |                         let mut results = Vec::new();
 173 |                         for frame in frames {
 174 |                             let delay = frame.delay().numer_denom_ms().0;
 175 |                             let img = frame.into_buffer();
 176 |                             let (width, height) = img.dimensions();
 177 |                             results.push(ImageData {
 178 |                                 width,
 179 |                                 height,
 180 |                                 format,
 181 |                                 rgba_data: img.into_raw(),
 182 |                                 path: path.to_string_lossy().to_string(),
 183 |                                 file_size_bytes,
 184 |                                 frame_delay_ms: delay,
 185 |                                 exif_info: None,
 186 |                                 histogram: None,
 187 |                                 exif_loaded: false,
 188 |                             });
 189 |                         }
 190 |                         return Ok(results);
 191 |                     }
 192 |                 }
 193 |             }
 194 |         }
 195 | 
 196 |         // Note: EXIF is now lazy-loaded on demand via load_exif()
 197 | 
 198 |         let (rgba_data, width, height) = match format {
 199 |             ImageFormatType::Heic | ImageFormatType::Avif => Self::load_heif(path)?,
 200 |             #[cfg(feature = "raw")]
 201 |             ImageFormatType::Raw => Self::load_raw(path)?,
 202 |             #[cfg(feature = "svg")]
 203 |             ImageFormatType::Svg => Self::load_svg(path)?,
 204 |             _ => Self::load_standard(path)?,
 205 |         };
 206 | 
 207 |         // Note: EXIF is lazy-loaded on demand via load_exif()
 208 |         Ok(vec![ImageData {
 209 |             width,
 210 |             height,
 211 |             format,
 212 |             rgba_data,
 213 |             path: path.to_string_lossy().to_string(),
 214 |             file_size_bytes,
 215 |             frame_delay_ms: 0,
 216 |             exif_info: None,
 217 |             histogram: None,
 218 |             exif_loaded: false,
 219 |         }])
 220 |     }
 221 | 
 222 |     fn extract_exif_lazy(path: &str) -> Option<String> {
 223 |         let path = std::path::Path::new(path);
 224 |         let file = std::fs::File::open(path).ok()?;
 225 |         let mut bufreader = std::io::BufReader::new(&file);
 226 |         let exifreader = exif::Reader::new();
 227 |         let exif_data = exifreader.read_from_container(&mut bufreader).ok()?;
 228 | 
 229 |         let mut out = String::new();
 230 | 
 231 |         // Helper macro to append EXIF fields concisely
 232 |         let mut add_field = |tag: exif::Tag, label: &str| {
 233 |             if let Some(field) = exif_data.get_field(tag, exif::In::PRIMARY) {
 234 |                 out.push_str(label);
 235 |                 out.push_str(&field.display_value().with_unit(&exif_data).to_string());
 236 |                 out.push('\n');
 237 |             }
 238 |         };
 239 | 
 240 |         add_field(exif::Tag::Make, "Make: ");
 241 |         add_field(exif::Tag::Model, "Model: ");
 242 |         add_field(exif::Tag::LensModel, "Lens: ");
 243 | 
 244 |         let mut exposure_line = String::new();
 245 |         if let Some(f) = exif_data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
 246 |             exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
 247 |             exposure_line.push_str("  ");
 248 |         }
 249 |         if let Some(f) = exif_data.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
 250 |             exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
 251 |             exposure_line.push_str("  ");
 252 |         }
 253 |         if let Some(f) = exif_data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
 254 |             exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
 255 |             exposure_line.push_str("s  ");
 256 |         }
 257 |         if let Some(f) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY)
 258 |         {
 259 |             // ISO
 260 |             exposure_line.push_str("ISO ");
 261 |             exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
 262 |         }
 263 | 
 264 |         add_field(exif::Tag::DateTimeOriginal, "Date: ");
 265 | 
 266 |         if !exposure_line.is_empty() {
 267 |             out.push_str("Exposure: ");
 268 |             out.push_str(&exposure_line);
 269 |             out.push('\n');
 270 |         }
 271 | 
 272 |         if out.is_empty() {
 273 |             None
 274 |         } else {
 275 |             Some(out.trim_end().to_string())
 276 |         }
 277 |     }
 278 | 
 279 |     /// Load an image and optionally downsample it if it exceeds maximum dimensions
 280 |     pub fn load_and_downsample(path: &Path, max_w: u32, max_h: u32) -> Result<Vec<ImageData>> {
 281 |         let mut frames = Self::load(path)?;
 282 | 
 283 |         // Fast path: if the first frame is already small enough, just return
 284 |         if let Some(first) = frames.first() {
 285 |             if first.width <= max_w && first.height <= max_h {
 286 |                 return Ok(frames);
 287 |             }
 288 |         } else {
 289 |             return Ok(frames);
 290 |         }
 291 | 
 292 |         use fast_image_resize as fir;
 293 |         let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
 294 | 
 295 |         for data in frames.iter_mut() {
 296 |             let src = fir::Image::from_vec_u8(
 297 |                 std::num::NonZeroU32::new(data.width).unwrap(),
 298 |                 std::num::NonZeroU32::new(data.height).unwrap(),
 299 |                 std::mem::take(&mut data.rgba_data),
 300 |                 fir::PixelType::U8x4,
 301 |             )?;
 302 | 
 303 |             let scale_w = max_w as f32 / data.width as f32;
 304 |             let scale_h = max_h as f32 / data.height as f32;
 305 |             let scale = scale_w.min(scale_h).min(1.0);
 306 | 
 307 |             let dst_w = (data.width as f32 * scale).round() as u32;
 308 |             let dst_h = (data.height as f32 * scale).round() as u32;
 309 | 
 310 |             let mut dst = fir::Image::new(
 311 |                 std::num::NonZeroU32::new(dst_w).unwrap(),
 312 |                 std::num::NonZeroU32::new(dst_h).unwrap(),
 313 |                 fir::PixelType::U8x4,
 314 |             );
 315 |             resizer.resize(&src.view(), &mut dst.view_mut())?;
 316 | 
 317 |             data.rgba_data = dst.into_vec();
 318 |             data.width = dst_w;
 319 |             data.height = dst_h;
 320 |         }
 321 | 
 322 |         Ok(frames)
 323 |     }
 324 | 
 325 |     /// Load standard image formats using the `image` crate
 326 |     fn load_standard(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
 327 |         let img = image::open(path).with_context(|| {
 328 |             let p = path.display();
 329 |             format!("Failed to open image: {p}")
 330 |         })?;
 331 | 
 332 |         let (width, height) = img.dimensions();
 333 |         let rgba = img.to_rgba8();
 334 |         let rgba_data = rgba.into_raw();
 335 | 
 336 |         tracing::debug!("Loaded standard image: {width}x{height}");
 337 |         Ok((rgba_data, width, height))
 338 |     }
 339 | 
 340 |     /// Load HEIC/AVIF images using built-in OS codecs (WIC on Windows)
 341 |     #[cfg(windows)]
 342 |     fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
 343 |         // First try the image crate in case it handles it
 344 |         if let Ok(img) = image::open(path) {
 345 |             let (width, height) = img.dimensions();
 346 |             let rgba = img.to_rgba8();
 347 |             return Ok((rgba.into_raw(), width, height));
 348 |         }
 349 | 
 350 |         // Fallback to Windows Imaging Component (WIC)
 351 |         use windows::core::HSTRING;
 352 |         use windows::Win32::Foundation::GENERIC_READ;
 353 |         use windows::Win32::Graphics::Imaging::*;
 354 |         use windows::Win32::System::Com::{
 355 |             CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
 356 |         };
 357 | 
 358 |         // Initialize COM (it might already be initialized by winit, but safe to re-call)
 359 |         let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
 360 | 
 361 |         let result = (|| -> windows::core::Result<(Vec<u8>, u32, u32)> {
 362 |             unsafe {
 363 |                 let factory: IWICImagingFactory =
 364 |                     CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
 365 | 
 366 |                 let path_hstring = HSTRING::from(path.as_os_str());
 367 |                 let decoder = factory.CreateDecoderFromFilename(
 368 |                     &path_hstring,
 369 |                     None,
 370 |                     GENERIC_READ,
 371 |                     WICDecodeMetadataCacheOnDemand,
 372 |                 )?;
 373 | 
 374 |                 let frame = decoder.GetFrame(0)?;
 375 | 
 376 |                 let mut width = 0;
 377 |                 let mut height = 0;
 378 |                 frame.GetSize(&mut width, &mut height)?;
 379 | 
 380 |                 let converter = factory.CreateFormatConverter()?;
 381 |                 converter.Initialize(
 382 |                     &frame,
 383 |                     &GUID_WICPixelFormat32bppRGBA,
 384 |                     WICBitmapDitherTypeNone,
 385 |                     None,
 386 |                     0.0,
 387 |                     WICBitmapPaletteTypeCustom,
 388 |                 )?;
 389 | 
 390 |                 let stride = width * 4;
 391 |                 let size = stride * height;
 392 |                 let mut buffer: Vec<u8> = vec![0; size as usize];
 393 | 
 394 |                 converter.CopyPixels(std::ptr::null(), stride, &mut buffer)?;
 395 | 
 396 |                 Ok((buffer, width, height))
 397 |             }
 398 |         })();
 399 | 
 400 |         match result {
 401 |             Ok(data) => {
 402 |                 let (w, h) = (data.1, data.2);
 403 |                 tracing::debug!("Loaded HEIC via WIC: {w}x{h}");
 404 |                 Ok(data)
 405 |             }
 406 |             Err(e) => {
 407 |                 let msg = format!(
 408 |                     "Failed to decode HEIC using Windows WIC: {e}. You may need to install the 'HEVC Video Extensions' and 'HEIF Image Extensions' from the Microsoft Store.", 
 409 |                 );
 410 |                 tracing::error!("{msg}");
 411 |                 Err(anyhow::anyhow!(msg))
 412 |             }
 413 |         }
 414 |     }
 415 | 
 416 |     #[cfg(not(windows))]
 417 |     fn load_heif(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
 418 |         // Fallback: try image crate, but provide custom error if it fails
 419 |         let res = image::open(path);
 420 |         match res {
 421 |             Ok(img) => {
 422 |                 let (width, height) = img.dimensions();
 423 |                 let rgba = img.to_rgba8();
 424 |                 Ok((rgba.into_raw(), width, height))
 425 |             }
 426 |             Err(_) => Err(anyhow::anyhow!(
 427 |                 "HEIC/AVIF support is currently only available natively on Windows. \
 428 |                      Please use JPEG or PNG on this platform, or install an HEIF decoder."
 429 |             )),
 430 |         }
 431 |     }
 432 | 
 433 |     // -------------------------------------------------------------------------
 434 |     // RAW camera format loading
 435 |     // -------------------------------------------------------------------------
 436 | 
 437 |     /// Load RAW camera files using the `rawler` crate.
 438 |     /// Falls back to Windows WIC on Windows if rawler cannot decode the format.
 439 |     #[cfg(feature = "raw")]
 440 |     fn load_raw(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
 441 |         tracing::debug!("Loading RAW file: {:?}", path);
 442 | 
 443 |         let result: anyhow::Result<(Vec<u8>, u32, u32)> = (|| {
 444 |             let raw_image =
 445 |                 rawler::decode_file(path).map_err(|e| anyhow::anyhow!("rawler decode: {e}"))?;
 446 | 
 447 |             let width = raw_image.width;
 448 |             let height = raw_image.height;
 449 | 
 450 |             // rawler gives us 16-bit CFA mosaic data (Bayer pattern).
 451 |             // We do a minimal debayer by treating R/G/B channels via the CFA pattern,
 452 |             // then scale 16-bit → 8-bit via a simple linear mapping.
 453 |             let rgba: Vec<u8> = match raw_image.data {
 454 |                 rawler::RawImageData::Integer(ref pixels) => {
 455 |                     // Find actual max to normalise instead of assuming 65535
 456 |                     let max_val = pixels.iter().copied().max().unwrap_or(1).max(1) as f32;
 457 |                     let cfa = &raw_image.camera.cfa;
 458 |                     let mut out = vec![0u8; (width * height * 4) as usize];
 459 |                     for y in 0..height {
 460 |                         for x in 0..width {
 461 |                             let idx = (y * width + x) as usize;
 462 |                             let raw = pixels[idx];
 463 |                             let channel = cfa.color_at(x, y);
 464 |                             let v = (raw as f32 / max_val * 255.0).clamp(0.0, 255.0) as u8;
 465 |                             let dest = idx * 4;
 466 |                             // Assign to the correct RGB channel; others stay 0 until
 467 |                             // averaged by neighbouring pixels in a full debayer.
 468 |                             // For this fast-path we just display each channel visually.
 469 |                             match channel {
 470 |                                 0 => {
 471 |                                     out[dest] = v;
 472 |                                     out[dest + 3] = 255;
 473 |                                 } // R
 474 |                                 1 => {
 475 |                                     out[dest + 1] = v;
 476 |                                     out[dest + 3] = 255;
 477 |                                 } // G
 478 |                                 2 => {
 479 |                                     out[dest + 2] = v;
 480 |                                     out[dest + 3] = 255;
 481 |                                 } // B
 482 |                                 _ => {
 483 |                                     out[dest] = v;
 484 |                                     out[dest + 1] = v;
 485 |                                     out[dest + 2] = v;
 486 |                                     out[dest + 3] = 255;
 487 |                                 }
 488 |                             }
 489 |                         }
 490 |                     }
 491 |                     out
 492 |                 }
 493 |                 rawler::RawImageData::Float(ref pixels) => pixels
 494 |                     .iter()
 495 |                     .flat_map(|&v| {
 496 |                         let b = (v.clamp(0.0, 1.0) * 255.0) as u8;
 497 |                         [b, b, b, 255u8]
 498 |                     })
 499 |                     .collect(),
 500 |             };
 501 | 
 502 |             tracing::debug!("RAW decoded: {width}x{height}");
 503 |             Ok((rgba, width as u32, height as u32))
 504 |         })();
 505 | 
 506 |         match result {
 507 |             Ok(data) => Ok(data),
 508 |             Err(rawler_err) => {
 509 |                 tracing::warn!("rawler failed ({rawler_err}), trying WIC fallback");
 510 |                 // Windows fallback: WIC can handle DNG/ARW/NEF with proper codec installed
 511 |                 #[cfg(windows)]
 512 |                 {
 513 |                     if let Ok(data) = Self::load_heif(path) {
 514 |                         return Ok(data);
 515 |                     }
 516 |                 }
 517 |                 Err(anyhow::anyhow!(
 518 |                     "Failed to load RAW file. \
 519 |                     On Windows, install the \"Microsoft Raw Image Extension\" from the Store. \
 520 |                     Error: {}",
 521 |                     rawler_err
 522 |                 ))
 523 |             }
 524 |         }
 525 |     }
 526 | 
 527 |     // -------------------------------------------------------------------------
 528 |     // SVG vector graphics loading
 529 |     // -------------------------------------------------------------------------
 530 | 
 531 |     /// Rasterize an SVG file to RGBA using `resvg`.
 532 |     /// SVGs are rendered at their native viewBox size, capped at 2000px on the
 533 |     /// longest axis so they look sharp on 4K screens without wasting VRAM.
 534 |     #[cfg(feature = "svg")]
 535 |     fn load_svg(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
 536 |         tracing::debug!("Loading SVG file: {:?}", path);
 537 | 
 538 |         let svg_data = std::fs::read(path)?;
 539 | 
 540 |         // resvg re-exports usvg and tiny_skia — use those to avoid version mismatches.
 541 |         use resvg::tiny_skia;
 542 |         use resvg::usvg;
 543 | 
 544 |         // Parse SVG
 545 |         let opts = usvg::Options::default();
 546 |         let tree = usvg::Tree::from_data(&svg_data, &opts)
 547 |             .map_err(|e| anyhow::anyhow!("SVG parse error: {e}"))?;
 548 | 
 549 |         // Determine output size, cap at 2000px on longest edge
 550 |         const MAX_SIDE: f32 = 2000.0;
 551 |         let svg_size = tree.size();
 552 |         let (svg_w, svg_h) = (svg_size.width(), svg_size.height());
 553 |         let scale = (MAX_SIDE / svg_w.max(svg_h)).min(1.0);
 554 |         let out_w = (svg_w * scale).round() as u32;
 555 |         let out_h = (svg_h * scale).round() as u32;
 556 | 
 557 |         if out_w == 0 || out_h == 0 {
 558 |             return Err(anyhow::anyhow!("SVG has zero-size viewBox"));
 559 |         }
 560 | 
 561 |         // Create a transparent pixmap and render into it
 562 |         let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h)
 563 |             .ok_or_else(|| anyhow::anyhow!("Failed to allocate SVG pixmap ({out_w}x{out_h})"))?;
 564 | 
 565 |         let transform = tiny_skia::Transform::from_scale(scale, scale);
 566 |         resvg::render(&tree, transform, &mut pixmap.as_mut());
 567 | 
 568 |         tracing::debug!("SVG rasterized at {out_w}x{out_h} (scale {scale:.2})");
 569 | 
 570 |         // tiny-skia produces premultiplied RGBA (BGRA on some platforms).
 571 |         // Un-premultiply alpha for correct GPU texture sampling.
 572 |         let mut rgba = pixmap.take();
 573 |         for px in rgba.chunks_exact_mut(4) {
 574 |             let a = px[3];
 575 |             if a > 0 && a < 255 {
 576 |                 let inv = 255.0 / a as f32;
 577 |                 px[0] = (px[0] as f32 * inv).min(255.0) as u8;
 578 |                 px[1] = (px[1] as f32 * inv).min(255.0) as u8;
 579 |                 px[2] = (px[2] as f32 * inv).min(255.0) as u8;
 580 |             }
 581 |         }
 582 | 
 583 |         Ok((rgba, out_w, out_h))
 584 |     }
 585 | 
 586 |     /// Save image to file
 587 |     pub fn save(path: &Path, image: &DynamicImage, quality: u8) -> Result<()> {
 588 |         let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
 589 | 
 590 |         match ext.to_lowercase().as_str() {
 591 |             "jpg" | "jpeg" => {
 592 |                 let rgb = image.to_rgb8();
 593 |                 let mut file = std::fs::File::create(path)?;
 594 |                 let encoder =
 595 |                     image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
 596 |                 rgb.write_with_encoder(encoder)
 597 |                     .context("Failed to write JPEG")?;
 598 |             }
 599 |             "png" => {
 600 |                 image.save_with_format(path, ImageFormat::Png)?;
 601 |             }
 602 |             _ => {
 603 |                 image.save(path)?;
 604 |             }
 605 |         }
 606 | 
 607 |         let p = path.display();
 608 |         tracing::info!("Saved image to: {p}");
 609 |         Ok(())
 610 |     }
 611 | 
 612 |     /// Check if a file is a supported image
 613 |     pub fn is_supported(path: &Path) -> bool {
 614 |         let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
 615 | 
 616 |         ImageFormatType::from_extension(ext).is_supported()
 617 |     }
 618 | 
 619 |     /// Get list of supported extensions
 620 |     pub fn supported_extensions() -> Vec<&'static str> {
 621 |         let mut exts = vec![
 622 |             "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "heic", "heif", "avif",
 623 |         ];
 624 |         #[cfg(feature = "raw")]
 625 |         exts.extend_from_slice(&[
 626 |             "cr2", "cr3", "crw", "nef", "nrw", "arw", "srf", "sr2", "raf", "orf", "rw2", "dng",
 627 |             "mrw", "pef", "3fr", "rwl", "raw", "rw1",
 628 |         ]);
 629 |         #[cfg(feature = "svg")]
 630 |         exts.push("svg");
 631 |         exts
 632 |     }
 633 | }
 634 | 
 635 | #[cfg(test)]
 636 | mod tests {
 637 |     use super::*;
 638 | 
 639 |     #[test]
 640 |     fn test_format_detection() {
 641 |         assert_eq!(
 642 |             ImageFormatType::from_extension("jpg"),
 643 |             ImageFormatType::Jpeg
 644 |         );
 645 |         assert_eq!(ImageFormatType::from_extension("PNG"), ImageFormatType::Png);
 646 |         assert_eq!(
 647 |             ImageFormatType::from_extension("heic"),
 648 |             ImageFormatType::Heic
 649 |         );
 650 |         #[cfg(feature = "svg")]
 651 |         assert_eq!(ImageFormatType::from_extension("svg"), ImageFormatType::Svg);
 652 |         #[cfg(feature = "raw")]
 653 |         {
 654 |             assert_eq!(ImageFormatType::from_extension("dng"), ImageFormatType::Raw);
 655 |             assert_eq!(ImageFormatType::from_extension("arw"), ImageFormatType::Raw);
 656 |             assert_eq!(ImageFormatType::from_extension("cr2"), ImageFormatType::Raw);
 657 |             assert_eq!(ImageFormatType::from_extension("nef"), ImageFormatType::Raw);
 658 |         }
 659 |         assert_eq!(
 660 |             ImageFormatType::from_extension("unknown"),
 661 |             ImageFormatType::Unknown
 662 |         );
 663 |     }
 664 | 
 665 |     #[test]
 666 |     fn test_load_and_downsample() {
 667 |         let temp_dir = std::env::temp_dir();
 668 |         let test_img_path = temp_dir.join("test_spedimage_downsample.png");
 669 | 
 670 |         // Create a 1000x1000 test image
 671 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 1000));
 672 |         img.save(&test_img_path).unwrap();
 673 | 
 674 |         // Downsample to 500x500
 675 |         let result = ImageBackend::load_and_downsample(&test_img_path, 500, 500).unwrap();
 676 |         assert_eq!(result.len(), 1);
 677 |         let data = &result[0];
 678 | 
 679 |         assert_eq!(data.width, 500);
 680 |         assert_eq!(data.height, 500);
 681 |         assert_eq!(data.rgba_data.len(), 500 * 500 * 4);
 682 | 
 683 |         // Downsample to 400x300 (should keep aspect ratio -> 300x300)
 684 |         let result = ImageBackend::load_and_downsample(&test_img_path, 400, 300).unwrap();
 685 |         let data = &result[0];
 686 | 
 687 |         // 1000x1000 mapped to fit within 400x300 -> 300x300
 688 |         assert_eq!(data.width, 300);
 689 |         assert_eq!(data.height, 300);
 690 | 
 691 |         std::fs::remove_file(&test_img_path).unwrap();
 692 |     }
 693 | 
 694 |     #[test]
 695 |     fn test_format_is_supported() {
 696 |         assert!(ImageFormatType::Jpeg.is_supported());
 697 |         assert!(ImageFormatType::Png.is_supported());
 698 |         assert!(ImageFormatType::Gif.is_supported());
 699 |         assert!(ImageFormatType::Bmp.is_supported());
 700 |         assert!(ImageFormatType::Tiff.is_supported());
 701 |         assert!(ImageFormatType::WebP.is_supported());
 702 |         assert!(ImageFormatType::Heic.is_supported());
 703 |         assert!(ImageFormatType::Avif.is_supported());
 704 |         assert!(!ImageFormatType::Unknown.is_supported());
 705 | 
 706 |         #[cfg(feature = "raw")]
 707 |         assert!(ImageFormatType::Raw.is_supported());
 708 |         #[cfg(not(feature = "raw"))]
 709 |         assert!(!ImageFormatType::Raw.is_supported());
 710 | 
 711 |         #[cfg(feature = "svg")]
 712 |         assert!(ImageFormatType::Svg.is_supported());
 713 |         #[cfg(not(feature = "svg"))]
 714 |         assert!(!ImageFormatType::Svg.is_supported());
 715 |     }
 716 | 
 717 |     #[test]
 718 |     fn test_image_data_methods() {
 719 |         let data = ImageData {
 720 |             width: 800,
 721 |             height: 600,
 722 |             format: ImageFormatType::Png,
 723 |             rgba_data: vec![0u8; 800 * 600 * 4],
 724 |             path: "/test/image.png".to_string(),
 725 |             file_size_bytes: 1024,
 726 |             frame_delay_ms: 0,
 727 |             exif_info: None,
 728 |             histogram: None,
 729 |             exif_loaded: false,
 730 |         };
 731 | 
 732 |         assert_eq!(data.as_rgba().len(), 800 * 600 * 4);
 733 |         assert!((data.aspect_ratio() - 800.0 / 600.0).abs() < 0.001);
 734 |         assert_eq!(data.pixel_count(), 800 * 600);
 735 |     }
 736 | 
 737 |     #[test]
 738 |     fn test_save_jpeg() {
 739 |         let temp_dir = std::env::temp_dir();
 740 |         let test_path = temp_dir.join("test_save_jpeg.jpg");
 741 | 
 742 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
 743 |         ImageBackend::save(&test_path, &img, 90).unwrap();
 744 | 
 745 |         assert!(test_path.exists());
 746 |         std::fs::remove_file(&test_path).unwrap();
 747 |     }
 748 | 
 749 |     #[test]
 750 |     fn test_save_png() {
 751 |         let temp_dir = std::env::temp_dir();
 752 |         let test_path = temp_dir.join("test_save_png.png");
 753 | 
 754 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
 755 |         ImageBackend::save(&test_path, &img, 90).unwrap();
 756 | 
 757 |         assert!(test_path.exists());
 758 |         std::fs::remove_file(&test_path).unwrap();
 759 |     }
 760 | 
 761 |     #[test]
 762 |     fn test_save_bmp() {
 763 |         let temp_dir = std::env::temp_dir();
 764 |         let test_path = temp_dir.join("test_save_bmp.bmp");
 765 | 
 766 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
 767 |         ImageBackend::save(&test_path, &img, 90).unwrap();
 768 | 
 769 |         assert!(test_path.exists());
 770 |         std::fs::remove_file(&test_path).unwrap();
 771 |     }
 772 | 
 773 |     #[test]
 774 |     fn test_is_supported() {
 775 |         let temp_dir = std::env::temp_dir();
 776 |         let jpg_path = temp_dir.join("test_is_supported_1.jpg");
 777 |         let png_path = temp_dir.join("test_is_supported_2.png");
 778 |         let unknown_path = temp_dir.join("test_is_supported_3.xyz");
 779 | 
 780 |         std::fs::File::create(&jpg_path).unwrap();
 781 |         std::fs::File::create(&png_path).unwrap();
 782 |         std::fs::File::create(&unknown_path).unwrap();
 783 | 
 784 |         assert!(ImageBackend::is_supported(&jpg_path));
 785 |         assert!(ImageBackend::is_supported(&png_path));
 786 |         assert!(!ImageBackend::is_supported(&unknown_path));
 787 | 
 788 |         std::fs::remove_file(&jpg_path).unwrap();
 789 |         std::fs::remove_file(&png_path).unwrap();
 790 |         std::fs::remove_file(&unknown_path).unwrap();
 791 |     }
 792 | 
 793 |     #[test]
 794 |     fn test_supported_extensions() {
 795 |         let exts = ImageBackend::supported_extensions();
 796 |         assert!(exts.contains(&"jpg"));
 797 |         assert!(exts.contains(&"jpeg"));
 798 |         assert!(exts.contains(&"png"));
 799 |         assert!(exts.contains(&"gif"));
 800 |         assert!(exts.contains(&"bmp"));
 801 |         assert!(exts.contains(&"tiff"));
 802 |         assert!(exts.contains(&"webp"));
 803 |         assert!(exts.contains(&"heic"));
 804 |         assert!(exts.contains(&"avif"));
 805 | 
 806 |         #[cfg(feature = "raw")]
 807 |         {
 808 |             assert!(exts.contains(&"dng"));
 809 |             assert!(exts.contains(&"arw"));
 810 |             assert!(exts.contains(&"cr2"));
 811 |         }
 812 | 
 813 |         #[cfg(feature = "svg")]
 814 |         assert!(exts.contains(&"svg"));
 815 |     }
 816 | 
 817 |     #[test]
 818 |     fn test_load_standard_image() {
 819 |         let temp_dir = std::env::temp_dir();
 820 |         let test_img_path = temp_dir.join("test_load.png");
 821 | 
 822 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
 823 |         img.save(&test_img_path).unwrap();
 824 | 
 825 |         let result = ImageBackend::load(&test_img_path).unwrap();
 826 |         assert_eq!(result.len(), 1);
 827 |         assert_eq!(result[0].width, 100);
 828 |         assert_eq!(result[0].height, 100);
 829 | 
 830 |         std::fs::remove_file(&test_img_path).unwrap();
 831 |     }
 832 | 
 833 |     #[test]
 834 |     fn test_gif_loading() {
 835 |         let temp_dir = std::env::temp_dir();
 836 |         let test_gif_path = temp_dir.join("test_animated.gif");
 837 | 
 838 |         let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10));
 839 |         img.save(&test_gif_path).unwrap();
 840 | 
 841 |         let result = ImageBackend::load(&test_gif_path).unwrap();
 842 |         assert!(!result.is_empty());
 843 | 
 844 |         std::fs::remove_file(&test_gif_path).unwrap();
 845 |     }
 846 | }
```

### File: `src\ui.rs`

- Size: 10775 bytes
- Modified: 2026-03-05 18:04:05 UTC

```rust
   1 | //! UI Layer - User interface components
   2 | //!
   3 | //! Provides UI elements for the image viewer including file browser,
   4 | //! adjustment controls, and toolbar.
   5 | 
   6 | use crate::gpu_renderer::ImageAdjustments;
   7 | use std::path::PathBuf;
   8 | 
   9 | /// File entry for the sidebar
  10 | #[derive(Debug, Clone)]
  11 | pub struct FileEntry {
  12 |     pub path: PathBuf,
  13 |     pub name: String,
  14 |     pub is_image: bool,
  15 | }
  16 | 
  17 | impl FileEntry {
  18 |     pub fn new(path: PathBuf) -> Self {
  19 |         let name = path
  20 |             .file_name()
  21 |             .and_then(|n| n.to_str())
  22 |             .unwrap_or("Unknown")
  23 |             .to_string();
  24 | 
  25 |         let is_image = ImageBackend::is_supported(&path);
  26 | 
  27 |         Self {
  28 |             path,
  29 |             name,
  30 |             is_image,
  31 |         }
  32 |     }
  33 | }
  34 | 
  35 | /// Application state for UI
  36 | #[derive(Debug, Clone, Default)]
  37 | pub struct UiState {
  38 |     pub files: Vec<FileEntry>,
  39 |     pub current_file_index: Option<usize>,
  40 |     pub adjustments: ImageAdjustments,
  41 |     pub is_cropping: bool,
  42 |     pub show_file_dialog: bool,
  43 |     pub show_help: bool,
  44 |     pub show_info: bool,
  45 |     pub status_message: Option<String>,
  46 |     /// Set of file indices that are currently selected in the thumbnail strip.
  47 |     pub selected_indices: std::collections::HashSet<usize>,
  48 | }
  49 | 
  50 | impl UiState {
  51 |     /// Get the current file path if any
  52 |     pub fn current_file(&self) -> Option<&PathBuf> {
  53 |         self.current_file_index
  54 |             .and_then(|idx| self.files.get(idx))
  55 |             .map(|f| &f.path)
  56 |     }
  57 | 
  58 |     /// Navigate to next image
  59 |     pub fn next_file(&mut self) {
  60 |         if let Some(current) = self.current_file_index {
  61 |             let image_count = self.files.iter().filter(|f| f.is_image).count();
  62 |             if image_count > 0 {
  63 |                 // Find next image file
  64 |                 let mut search_idx = (current + 1) % self.files.len();
  65 |                 for _ in 0..self.files.len() {
  66 |                     if self.files[search_idx].is_image {
  67 |                         self.current_file_index = Some(search_idx);
  68 |                         return;
  69 |                     }
  70 |                     search_idx = (search_idx + 1) % self.files.len();
  71 |                 }
  72 |             }
  73 |         } else if !self.files.is_empty() {
  74 |             // Select first image
  75 |             for (i, f) in self.files.iter().enumerate() {
  76 |                 if f.is_image {
  77 |                     self.current_file_index = Some(i);
  78 |                     return;
  79 |                 }
  80 |             }
  81 |         }
  82 |     }
  83 | 
  84 |     /// Navigate to previous image
  85 |     pub fn prev_file(&mut self) {
  86 |         if let Some(current) = self.current_file_index {
  87 |             if !self.files.is_empty() {
  88 |                 // Find previous image file
  89 |                 let mut search_idx = if current == 0 {
  90 |                     self.files.len() - 1
  91 |                 } else {
  92 |                     current - 1
  93 |                 };
  94 |                 for _ in 0..self.files.len() {
  95 |                     if self.files[search_idx].is_image {
  96 |                         self.current_file_index = Some(search_idx);
  97 |                         return;
  98 |                     }
  99 |                     search_idx = if search_idx == 0 {
 100 |                         self.files.len() - 1
 101 |                     } else {
 102 |                         search_idx - 1
 103 |                     };
 104 |                 }
 105 |             }
 106 |         }
 107 |     }
 108 | 
 109 |     /// Load files from a directory
 110 |     pub fn load_directory(&mut self, dir: PathBuf) {
 111 |         self.files.clear();
 112 |         self.current_file_index = None;
 113 | 
 114 |         if let Ok(entries) = std::fs::read_dir(&dir) {
 115 |             let mut files: Vec<_> = entries
 116 |                 .filter_map(|e| e.ok())
 117 |                 .map(|e| FileEntry::new(e.path()))
 118 |                 .collect();
 119 | 
 120 |             // Sort files by name
 121 |             files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
 122 | 
 123 |             // Filter to only images and sort
 124 |             files.retain(|f| f.is_image);
 125 |             self.files = files;
 126 | 
 127 |             // Select first image
 128 |             if !self.files.is_empty() {
 129 |                 self.current_file_index = Some(0);
 130 |             }
 131 |         }
 132 |     }
 133 | 
 134 |     /// Reset all adjustments to default
 135 |     pub fn reset_adjustments(&mut self) {
 136 |         self.adjustments = ImageAdjustments::default();
 137 |     }
 138 | 
 139 |     /// Rotate by 90 degrees
 140 |     pub fn rotate_90(&mut self) {
 141 |         self.adjustments.rotation += std::f32::consts::FRAC_PI_2;
 142 |     }
 143 | 
 144 |     /// Set status message
 145 |     pub fn set_status(&mut self, message: impl Into<String>) {
 146 |         self.status_message = Some(message.into());
 147 |     }
 148 | 
 149 |     /// Clear status message
 150 |     pub fn clear_status(&mut self) {
 151 |         self.status_message = None;
 152 |     }
 153 | 
 154 |     /// Get current status message as str (empty if none)
 155 |     pub fn get_status(&self) -> &str {
 156 |         self.status_message.as_deref().unwrap_or("")
 157 |     }
 158 | 
 159 |     /// Toggle help overlay
 160 |     pub fn toggle_help(&mut self) {
 161 |         self.show_help = !self.show_help;
 162 |     }
 163 | 
 164 |     /// Toggle info panel
 165 |     pub fn toggle_info(&mut self) {
 166 |         self.show_info = !self.show_info;
 167 |     }
 168 | }
 169 | 
 170 | // Re-export ImageBackend for file filtering
 171 | use crate::image_backend::ImageBackend;
 172 | 
 173 | #[cfg(test)]
 174 | mod tests {
 175 |     use super::*;
 176 |     use std::env;
 177 | 
 178 |     #[test]
 179 |     fn test_file_entry_new() {
 180 |         let temp_dir = env::temp_dir();
 181 |         let test_path = temp_dir.join("test_image.png");
 182 | 
 183 |         std::fs::File::create(&test_path).unwrap();
 184 | 
 185 |         let entry = FileEntry::new(test_path.clone());
 186 |         assert_eq!(entry.name, "test_image.png");
 187 |         assert!(entry.is_image);
 188 | 
 189 |         std::fs::remove_file(&test_path).unwrap();
 190 |     }
 191 | 
 192 |     #[test]
 193 |     fn test_file_entry_unknown_extension() {
 194 |         let temp_dir = env::temp_dir();
 195 |         let test_path = temp_dir.join("test.xyz");
 196 | 
 197 |         if std::fs::File::create(&test_path).is_err() {
 198 |             return;
 199 |         }
 200 | 
 201 |         let entry = FileEntry::new(test_path.clone());
 202 |         assert_eq!(entry.name, "test.xyz");
 203 |         assert!(!entry.is_image);
 204 | 
 205 |         let _ = std::fs::remove_file(&test_path);
 206 |     }
 207 | 
 208 |     #[test]
 209 |     fn test_ui_state_current_file() {
 210 |         let mut state = UiState::default();
 211 |         assert!(state.current_file().is_none());
 212 | 
 213 |         let temp_dir = env::temp_dir();
 214 |         let path = temp_dir.join("test.png");
 215 |         std::fs::File::create(&path).unwrap();
 216 | 
 217 |         state.files.push(FileEntry::new(path.clone()));
 218 |         state.current_file_index = Some(0);
 219 | 
 220 |         assert_eq!(state.current_file(), Some(&path));
 221 | 
 222 |         std::fs::remove_file(&path).unwrap();
 223 |     }
 224 | 
 225 |     #[test]
 226 |     fn test_ui_state_next_file() {
 227 |         let mut state = UiState::default();
 228 | 
 229 |         let temp_dir = env::temp_dir();
 230 |         let path1 = temp_dir.join("a.png");
 231 |         let path2 = temp_dir.join("b.png");
 232 |         let path3 = temp_dir.join("c.png");
 233 | 
 234 |         std::fs::File::create(&path1).unwrap();
 235 |         std::fs::File::create(&path2).unwrap();
 236 |         std::fs::File::create(&path3).unwrap();
 237 | 
 238 |         state.files.push(FileEntry::new(path1));
 239 |         state.files.push(FileEntry::new(path2.clone()));
 240 |         state.files.push(FileEntry::new(path3));
 241 |         state.current_file_index = Some(0);
 242 | 
 243 |         state.next_file();
 244 |         assert_eq!(state.current_file_index, Some(1));
 245 | 
 246 |         state.next_file();
 247 |         assert_eq!(state.current_file_index, Some(2));
 248 | 
 249 |         state.next_file();
 250 |         assert_eq!(state.current_file_index, Some(0));
 251 | 
 252 |         std::fs::remove_file(&path2).unwrap();
 253 |     }
 254 | 
 255 |     #[test]
 256 |     fn test_ui_state_prev_file() {
 257 |         let mut state = UiState::default();
 258 | 
 259 |         let temp_dir = env::temp_dir();
 260 |         let path1 = temp_dir.join("a.png");
 261 |         let path2 = temp_dir.join("b.png");
 262 | 
 263 |         std::fs::File::create(&path1).unwrap();
 264 |         std::fs::File::create(path2.clone()).unwrap();
 265 | 
 266 |         state.files.push(FileEntry::new(path1.clone()));
 267 |         state.files.push(FileEntry::new(path2));
 268 |         state.current_file_index = Some(1);
 269 | 
 270 |         state.prev_file();
 271 |         assert_eq!(state.current_file_index, Some(0));
 272 | 
 273 |         state.prev_file();
 274 |         assert_eq!(state.current_file_index, Some(1));
 275 | 
 276 |         std::fs::remove_file(&path1).unwrap();
 277 |     }
 278 | 
 279 |     #[test]
 280 |     fn test_ui_state_load_directory() {
 281 |         let mut state = UiState::default();
 282 | 
 283 |         let temp_dir = env::temp_dir();
 284 |         let test_dir = temp_dir.join("spedimage_test_dir");
 285 |         std::fs::create_dir(&test_dir).unwrap();
 286 | 
 287 |         let path1 = test_dir.join("aaa.png");
 288 |         let path2 = test_dir.join("bbb.jpg");
 289 | 
 290 |         std::fs::File::create(&path1).unwrap();
 291 |         std::fs::File::create(&path2).unwrap();
 292 | 
 293 |         state.load_directory(test_dir.clone());
 294 | 
 295 |         assert!(!state.files.is_empty());
 296 |         assert!(state.current_file_index.is_some());
 297 | 
 298 |         std::fs::remove_file(&path1).unwrap();
 299 |         std::fs::remove_file(&path2).unwrap();
 300 |         std::fs::remove_dir(&test_dir).unwrap();
 301 |     }
 302 | 
 303 |     #[test]
 304 |     fn test_ui_state_reset_adjustments() {
 305 |         let mut state = UiState::default();
 306 | 
 307 |         state.adjustments.brightness = 2.0;
 308 |         state.adjustments.contrast = 1.5;
 309 |         state.adjustments.rotation = std::f32::consts::FRAC_PI_4;
 310 |         state.adjustments.hdr_toning = true;
 311 | 
 312 |         state.reset_adjustments();
 313 | 
 314 |         assert_eq!(state.adjustments.brightness, 1.0);
 315 |         assert_eq!(state.adjustments.contrast, 1.0);
 316 |         assert_eq!(state.adjustments.rotation, 0.0);
 317 |         assert!(!state.adjustments.hdr_toning);
 318 |     }
 319 | 
 320 |     #[test]
 321 |     fn test_ui_state_rotate_90() {
 322 |         let mut state = UiState::default();
 323 | 
 324 |         let initial_rotation = state.adjustments.rotation;
 325 |         state.rotate_90();
 326 | 
 327 |         assert_eq!(
 328 |             state.adjustments.rotation,
 329 |             initial_rotation + std::f32::consts::FRAC_PI_2
 330 |         );
 331 |     }
 332 | 
 333 |     #[test]
 334 |     fn test_ui_state_set_status() {
 335 |         let mut state = UiState::default();
 336 | 
 337 |         state.set_status("Test message");
 338 |         assert_eq!(state.get_status(), "Test message");
 339 | 
 340 |         state.set_status("Another message");
 341 |         assert_eq!(state.get_status(), "Another message");
 342 |     }
 343 | 
 344 |     #[test]
 345 |     fn test_ui_state_clear_status() {
 346 |         let mut state = UiState::default();
 347 | 
 348 |         state.set_status("Test message");
 349 |         state.clear_status();
 350 | 
 351 |         assert_eq!(state.get_status(), "");
 352 |     }
 353 | 
 354 |     #[test]
 355 |     fn test_ui_state_toggle_help() {
 356 |         let mut state = UiState::default();
 357 | 
 358 |         assert!(!state.show_help);
 359 | 
 360 |         state.toggle_help();
 361 |         assert!(state.show_help);
 362 | 
 363 |         state.toggle_help();
 364 |         assert!(!state.show_help);
 365 |     }
 366 | }
```

### File: `LICENSE`

- Size: 1086 bytes
- Modified: 2026-02-07 11:12:59 UTC

```text
   1 | MIT License
   2 | 
   3 | Copyright (c) 2026 Sv-stark
   4 | 
   5 | Permission is hereby granted, free of charge, to any person obtaining a copy
   6 | of this software and associated documentation files (the "Software"), to deal
   7 | in the Software without restriction, including without limitation the rights
   8 | to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
   9 | copies of the Software, and to permit persons to whom the Software is
  10 | furnished to do so, subject to the following conditions:
  11 | 
  12 | The above copyright notice and this permission notice shall be included in all
  13 | copies or substantial portions of the Software.
  14 | 
  15 | THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  16 | IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  17 | FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  18 | AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  19 | LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  20 | OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  21 | SOFTWARE.
```

### File: `assets\Inter-Regular.ttf`

- Size: 299187 bytes
- Modified: 2026-03-02 17:52:15 UTC

```ttf
   1 | 
   2 | 
   3 | 
   4 | 
   5 | 
   6 | 
   7 | 
   8 | 
   9 | <!DOCTYPE html>
  10 | <html
  11 |   lang="en"
  12 |   
  13 |   data-color-mode="auto" data-light-theme="light" data-dark-theme="dark"
  14 |   data-a11y-animated-images="system" data-a11y-link-underlines="true"
  15 |   
  16 |   >
  17 | 
  18 | 
  19 | 
  20 | 
  21 |   <head>
  22 |     <meta charset="utf-8">
  23 |   <link rel="dns-prefetch" href="https://github.githubassets.com">
  24 |   <link rel="dns-prefetch" href="https://avatars.githubusercontent.com">
  25 |   <link rel="dns-prefetch" href="https://github-cloud.s3.amazonaws.com">
  26 |   <link rel="dns-prefetch" href="https://user-images.githubusercontent.com/">
  27 |   <link rel="preconnect" href="https://github.githubassets.com" crossorigin>
  28 |   <link rel="preconnect" href="https://avatars.githubusercontent.com">
  29 | 
  30 |       <link crossorigin="anonymous" rel="preload" as="script" href="https://github.githubassets.com/assets/global-banner-disable-8a300af6d815087d.js" />
  31 | 
  32 |   <link rel="preload" href="https://github.githubassets.com/assets/mona-sans-14595085164a.woff2" as="font" type="font/woff2" crossorigin>
  33 | 
  34 | 
  35 |   <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/light-8f714f203754d3e3.css" /><link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/light_high_contrast-852c22d357937740.css" /><link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/dark-b5a0f9dbeed37e9c.css" /><link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/dark_high_contrast-b1b5000c4cba6bc9.css" /><link data-color-theme="light" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light-8f714f203754d3e3.css" /><link data-color-theme="light_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light_high_contrast-852c22d357937740.css" /><link data-color-theme="light_colorblind" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light_colorblind-cd9e31fed96ffe09.css" /><link data-color-theme="light_colorblind_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light_colorblind_high_contrast-d8387aac10e96d7c.css" /><link data-color-theme="light_tritanopia" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light_tritanopia-de5a42a9379cb835.css" /><link data-color-theme="light_tritanopia_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/light_tritanopia_high_contrast-b80844282a9e6620.css" /><link data-color-theme="dark" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark-b5a0f9dbeed37e9c.css" /><link data-color-theme="dark_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_high_contrast-b1b5000c4cba6bc9.css" /><link data-color-theme="dark_colorblind" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_colorblind-85d7bc4072a238f0.css" /><link data-color-theme="dark_colorblind_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_colorblind_high_contrast-f4838f9524519a33.css" /><link data-color-theme="dark_tritanopia" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_tritanopia-23bce8ff4d024f04.css" /><link data-color-theme="dark_tritanopia_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_tritanopia_high_contrast-e088d4bc2d56f7d6.css" /><link data-color-theme="dark_dimmed" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_dimmed-6b102382a27a656f.css" /><link data-color-theme="dark_dimmed_high_contrast" crossorigin="anonymous" media="all" rel="stylesheet" data-href="https://github.githubassets.com/assets/dark_dimmed_high_contrast-1d3e6d98532ab032.css" />
  36 | 
  37 |   <style type="text/css">
  38 |     :root {
  39 |       --tab-size-preference: 4;
  40 |     }
  41 | 
  42 |     pre, code {
  43 |       tab-size: var(--tab-size-preference);
  44 |     }
  45 |   </style>
  46 | 
  47 |     <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-primitives-26e89bb5a0c37ae9.css" />
  48 |     <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-bcf0a93b19072a86.css" />
  49 |     <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/global-95af332f763e4560.css" />
  50 |     <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/github-8ecc5798f0584cac.css" />
  51 |   <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/repository-dd7b55261c401703.css" />
  52 | 
  53 |   
  54 | 
  55 |   <script type="application/json" id="client-env">{"locale":"en","featureFlags":["a11y_status_checks_ruleset","actions_custom_images_public_preview_visibility","actions_custom_images_storage_billing_ui_visibility","actions_image_version_event","alternate_user_config_repo","arianotify_comprehensive_migration","batch_suggested_changes","codespaces_prebuild_region_target_update","coding_agent_model_selection","coding_agent_model_selection_all_skus","copilot_3p_agent_hovercards","copilot_agent_sessions_alive_updates","copilot_agent_snippy","copilot_agent_task_list_v2","copilot_agent_tasks_btn_code_nav","copilot_agent_tasks_btn_code_view","copilot_agent_tasks_btn_code_view_lines","copilot_agent_tasks_btn_repo","copilot_api_agentic_issue_marshal_yaml","copilot_ask_mode_dropdown","copilot_chat_attach_multiple_images","copilot_chat_clear_model_selection_for_default_change","copilot_chat_enable_tool_call_logs","copilot_chat_file_redirect","copilot_chat_input_commands","copilot_chat_opening_thread_switch","copilot_chat_reduce_quota_checks","copilot_chat_repository_picker","copilot_chat_search_bar_redirect","copilot_chat_selection_attachments","copilot_chat_vision_in_claude","copilot_chat_vision_preview_gate","copilot_cli_install_cta","copilot_coding_agent_task_response","copilot_custom_copilots","copilot_custom_copilots_feature_preview","copilot_duplicate_thread","copilot_extensions_hide_in_dotcom_chat","copilot_extensions_removal_on_marketplace","copilot_features_sql_server_logo","copilot_features_zed_logo","copilot_file_block_ref_matching","copilot_ftp_hyperspace_upgrade_prompt","copilot_icebreakers_experiment_dashboard","copilot_icebreakers_experiment_hyperspace","copilot_immersive_embedded","copilot_immersive_job_result_preview","copilot_immersive_layout_routes","copilot_immersive_structured_model_picker","copilot_immersive_task_hyperlinking","copilot_immersive_task_within_chat_thread","copilot_mc_cli_resume_any_users_task","copilot_mission_control_always_send_integration_id","copilot_mission_control_use_task_name","copilot_org_policy_page_focus_mode","copilot_redirect_header_button_to_agents","copilot_share_active_subthread","copilot_spaces_ga","copilot_spaces_individual_policies_ga","copilot_spaces_pagination","copilot_spark_empty_state","copilot_spark_handle_nil_friendly_name","copilot_stable_conversation_view","copilot_swe_agent_hide_model_picker_if_only_auto","copilot_swe_agent_pr_comment_model_picker","copilot_swe_agent_use_subagents","copilot_unconfigured_is_inherited","copilot_usage_metrics_ga","custom_instructions_file_references","custom_properties_consolidate_default_value_input","dashboard_lists_max_age_filter","dashboard_universe_2025_feedback_dialog","flex_cta_groups_mvp","global_nav_menu_lazy_load","global_nav_react","global_user_menu_lazy_load","hyperspace_2025_logged_out_batch_1","hyperspace_2025_logged_out_batch_2","initial_per_page_pagination_updates","issue_fields_global_search","issue_fields_report_usage","issue_fields_timeline_events","issues_cca_assign_actor_with_agent","issues_dashboard_inp_optimization","issues_dashboard_semantic_search","issues_diff_based_label_updates","issues_expanded_file_types","issues_index_semantic_search","issues_lazy_load_comment_box_suggestions","issues_react_auto_retry_on_error","issues_react_bots_timeline_pagination","issues_react_chrome_container_query_fix","issues_react_hot_cache","issues_react_low_quality_comment_warning","issues_react_prohibit_title_fallback","issues_react_safari_scroll_preservation","issues_react_use_turbo_for_cross_repo_navigation","landing_pages_ninetailed","landing_pages_web_vitals_tracking","lifecycle_label_name_updates","marketing_pages_search_explore_provider","memex_default_issue_create_repository","memex_grouped_by_edit_route","memex_live_update_hovercard","memex_mwl_filter_field_delimiter","mission_control_retry_on_401","mission_control_use_body_html","notifications_menu_defer_labels","oauth_authorize_clickjacking_protection","open_agent_session_in_vscode_insiders","open_agent_session_in_vscode_stable","primer_react_css_has_selector_perf","primer_react_spinner_synchronize_animations","prs_conversations_react","react_quality_profiling","ruleset_deletion_confirmation","sample_network_conn_type","session_logs_ungroup_reasoning_text","site_calculator_actions_2025","site_features_copilot_universe","site_homepage_collaborate_video","spark_prompt_secret_scanning","spark_server_connection_status","suppress_automated_browser_vitals","suppress_non_representative_vitals","viewscreen_sandbox","webp_support","workbench_store_readonly"],"copilotApiOverrideUrl":"https://api.githubcopilot.com"}</script>
  56 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/high-contrast-cookie-9c894bc29dbadce2.js"></script>
  57 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/wp-runtime-7da5381da4b30ce2.js" defer="defer"></script>
  58 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/28839-632d00a964e8dbd5.js" defer="defer"></script>
  59 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/49863-8861e351482cb073.js" defer="defer"></script>
  60 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/64220-1215bd360f02816c.js" defer="defer"></script>
  61 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/environment-39e5b412c63ea4f0.js" defer="defer"></script>
  62 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/runtime-helpers-3cd71e27e349021d.js" defer="defer"></script>
  63 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/2966-25cb8e34b31306a4.js" defer="defer"></script>
  64 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/96232-fb82336d69225835.js" defer="defer"></script>
  65 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/41013-ac21ea90ed8590af.js" defer="defer"></script>
  66 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/51210-185739338ae8119b.js" defer="defer"></script>
  67 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/24387-6b7f5e596897eded.js" defer="defer"></script>
  68 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/81683-740d112caee5baa9.js" defer="defer"></script>
  69 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/46740-84bdd4782a486b80.js" defer="defer"></script>
  70 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/81751-d3fe9e061a21f8d3.js" defer="defer"></script>
  71 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/github-elements-837d26c249ef0f1d.js" defer="defer"></script>
  72 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/element-registry-d7c9798e94b4eb05.js" defer="defer"></script>
  73 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/react-core-dcd512f43fdba85e.js" defer="defer"></script>
  74 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/react-lib-e74a1db7c21f7e74.js" defer="defer"></script>
  75 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/7053-20b4a6914bbde21f.js" defer="defer"></script>
  76 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/79039-9ce5da88e09eef89.js" defer="defer"></script>
  77 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/61110-212553c409076913.js" defer="defer"></script>
  78 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/2887-0e9a84f5dc250853.js" defer="defer"></script>
  79 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/26533-6b8040883d16f6ae.js" defer="defer"></script>
  80 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/62249-2895213a788d973c.js" defer="defer"></script>
  81 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/2694-372ce035e93800e0.js" defer="defer"></script>
  82 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/56093-886facf7a5ba3dc9.js" defer="defer"></script>
  83 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/28360-f4444f13a0dc6af9.js" defer="defer"></script>
  84 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/12142-b56dc06030d9574f.js" defer="defer"></script>
  85 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/22960-12a7c572a55f9c87.js" defer="defer"></script>
  86 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/51683-f6844a8acd415e3b.js" defer="defer"></script>
  87 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/76545-cf3621a1f2f36d8d.js" defer="defer"></script>
  88 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/behaviors-e50dd1f2cdd55827.js" defer="defer"></script>
  89 | <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/react-core.35e5d09cdbc69350.module.css" />
  90 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/38302-277be92deeed3c63.js" defer="defer"></script>
  91 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/notifications-global-fc52e59df1c89647.js" defer="defer"></script>
  92 |   
  93 | 
  94 |   <title>Page not found · GitHub · GitHub</title>
  95 | 
  96 | 
  97 | 
  98 |   <meta name="route-pattern" content="/:user_id/:repository/raw/*name(/*path)" data-turbo-transient>
  99 |   <meta name="route-controller" content="blob" data-turbo-transient>
 100 |   <meta name="route-action" content="raw" data-turbo-transient>
 101 |   <meta name="fetch-nonce" content="v2:afd07c08-7607-8b66-f672-53a8cb7f132a">
 102 | 
 103 |     
 104 |   <meta name="current-catalog-service-hash" content="f3abb0cc802f3d7b95fc8762b94bdcb13bf39634c40c357301c4aa1d67a256fb">
 105 | 
 106 | 
 107 |   <meta name="request-id" content="2BB5:33ECD8:155653:1825F3:69A5CE4E" data-turbo-transient="true" /><meta name="html-safe-nonce" content="efd01ea49977f81812a98fbdc7eb3f53b8d12104329d2583b7db8aba789c7eec" data-turbo-transient="true" /><meta name="visitor-payload" content="eyJyZWZlcnJlciI6bnVsbCwicmVxdWVzdF9pZCI6IjJCQjU6MzNFQ0Q4OjE1NTY1MzoxODI1RjM6NjlBNUNFNEUiLCJ2aXNpdG9yX2lkIjoiMzUyMzIwMTM1MTU4OTE1NDM4MiIsInJlZ2lvbl9lZGdlIjoiY2VudHJhbGluZGlhIiwicmVnaW9uX3JlbmRlciI6ImlhZCJ9" data-turbo-transient="true" /><meta name="visitor-hmac" content="f3121f345516369ffdda6bed02edca2a3de1be644e9da86fb721314bb2430fe8" data-turbo-transient="true" />
 108 | 
 109 | 
 110 |     <meta name="hovercard-subject-tag" content="repository:101033179" data-turbo-transient>
 111 | 
 112 | 
 113 |   <meta name="github-keyboard-shortcuts" content="copilot" data-turbo-transient="true" />
 114 |   
 115 | 
 116 |   <meta name="selected-link" value="/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf" data-turbo-transient>
 117 |   <link rel="assets" href="https://github.githubassets.com/">
 118 | 
 119 |     <meta name="google-site-verification" content="Apib7-x98H0j5cPqHWwSMm6dNU4GmODRoqxLiDzdx9I">
 120 | 
 121 | <meta name="octolytics-url" content="https://collector.github.com/github/collect" />
 122 | 
 123 | <meta name="octolytics-page-type" content="marketing" />
 124 | 
 125 | 
 126 | 
 127 |   
 128 | 
 129 |   
 130 | 
 131 | 
 132 | 
 133 | 
 134 |     <meta name="user-login" content="">
 135 | 
 136 |   
 137 | 
 138 |     <meta name="viewport" content="width=device-width">
 139 | 
 140 |     
 141 | 
 142 |       <meta name="description" content="GitHub is where people build software. More than 150 million people use GitHub to discover, fork, and contribute to over 420 million projects.">
 143 | 
 144 |       <link rel="search" type="application/opensearchdescription+xml" href="/opensearch.xml" title="GitHub">
 145 | 
 146 |     <link rel="fluid-icon" href="https://github.com/fluidicon.png" title="GitHub">
 147 |     <meta property="fb:app_id" content="1401488693436528">
 148 |     <meta name="apple-itunes-app" content="app-id=1477376905, app-argument=https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf" />
 149 | 
 150 |       <meta property="og:url" content="https://github.com">
 151 |   <meta property="og:site_name" content="GitHub">
 152 |   <meta property="og:title" content="Build software better, together">
 153 |   <meta property="og:description" content="GitHub is where people build software. More than 150 million people use GitHub to discover, fork, and contribute to over 420 million projects.">
 154 |   <meta property="og:image" content="https://github.githubassets.com/assets/github-logo-55c5b9a1fe52.png">
 155 |   <meta property="og:image:type" content="image/png">
 156 |   <meta property="og:image:width" content="1200">
 157 |   <meta property="og:image:height" content="1200">
 158 |   <meta property="og:image" content="https://github.githubassets.com/assets/github-mark-57519b92ca4e.png">
 159 |   <meta property="og:image:type" content="image/png">
 160 |   <meta property="og:image:width" content="1200">
 161 |   <meta property="og:image:height" content="620">
 162 |   <meta property="og:image" content="https://github.githubassets.com/assets/github-octocat-13c86b8b336d.png">
 163 |   <meta property="og:image:type" content="image/png">
 164 |   <meta property="og:image:width" content="1200">
 165 |   <meta property="og:image:height" content="620">
 166 | 
 167 |   <meta property="twitter:site" content="github">
 168 |   <meta property="twitter:site:id" content="13334762">
 169 |   <meta property="twitter:creator" content="github">
 170 |   <meta property="twitter:creator:id" content="13334762">
 171 |   <meta property="twitter:card" content="summary_large_image">
 172 |   <meta property="twitter:title" content="GitHub">
 173 |   <meta property="twitter:description" content="GitHub is where people build software. More than 150 million people use GitHub to discover, fork, and contribute to over 420 million projects.">
 174 |   <meta property="twitter:image" content="https://github.githubassets.com/assets/github-logo-55c5b9a1fe52.png">
 175 |   <meta property="twitter:image:width" content="1200">
 176 |   <meta property="twitter:image:height" content="1200">
 177 | 
 178 | 
 179 | 
 180 | 
 181 |       <meta name="hostname" content="github.com">
 182 | 
 183 | 
 184 | 
 185 |         <meta name="expected-hostname" content="github.com">
 186 | 
 187 | 
 188 |   <meta http-equiv="x-pjax-version" content="0f5e627b3222bed5614516e365974b68a10f20d4d255d6b7fdf51e49f99775bb" data-turbo-track="reload">
 189 |   <meta http-equiv="x-pjax-csp-version" content="568c098497d98702bac1642a2a853732a047a6ced28eabd3e15d50041a890235" data-turbo-track="reload">
 190 |   <meta http-equiv="x-pjax-css-version" content="3a421c5f1445b7e5ad5460dd263b7954b153fa40b048a6fde2f9e012764d6d50" data-turbo-track="reload">
 191 |   <meta http-equiv="x-pjax-js-version" content="6b3e851adbd799069ed1ede836cf767262b80792b82de130dfaf9517a1c6fa6d" data-turbo-track="reload">
 192 | 
 193 |   <meta name="turbo-cache-control" content="no-preview" data-turbo-transient="">
 194 | 
 195 |       <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/site-f0f3cf01d3270d68.css" />
 196 |   <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/error-5fcb903c2170e4d4.css" />
 197 |   <meta name="is_logged_out_page" content="true">
 198 | 
 199 |   
 200 | 
 201 | 
 202 | 
 203 |       <link rel="canonical" href="https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf" data-turbo-transient>
 204 | 
 205 | 
 206 |     <meta name="turbo-body-classes" content="logged-out env-production page-responsive min-height-full d-flex flex-column">
 207 |   <meta name="disable-turbo" content="false">
 208 | 
 209 | 
 210 |   <meta name="browser-stats-url" content="https://api.github.com/_private/browser/stats">
 211 | 
 212 |   <meta name="browser-errors-url" content="https://api.github.com/_private/browser/errors">
 213 | 
 214 |   <meta name="release" content="d177956c91815b0258f43ff58f3f2f1c74e4e990">
 215 |   <meta name="ui-target" content="full">
 216 | 
 217 |   <link rel="mask-icon" href="https://github.githubassets.com/assets/pinned-octocat-093da3e6fa40.svg" color="#000000">
 218 |   <link rel="alternate icon" class="js-site-favicon" type="image/png" href="https://github.githubassets.com/favicons/favicon.png">
 219 |   <link rel="icon" class="js-site-favicon" type="image/svg+xml" href="https://github.githubassets.com/favicons/favicon.svg" data-base-href="https://github.githubassets.com/favicons/favicon">
 220 | 
 221 | <meta name="theme-color" content="#1e2327">
 222 | <meta name="color-scheme" content="light dark" />
 223 | 
 224 | 
 225 |   <link rel="manifest" href="/manifest.json" crossOrigin="use-credentials">
 226 | 
 227 |   </head>
 228 | 
 229 |   <body class="logged-out env-production page-responsive min-height-full d-flex flex-column" style="word-wrap: break-word;" >
 230 |     <div data-turbo-body class="logged-out env-production page-responsive min-height-full d-flex flex-column" style="word-wrap: break-word;" >
 231 |       <div id="__primerPortalRoot__" role="region" style="z-index: 1000; position: absolute; width: 100%;" data-turbo-permanent></div>
 232 |       
 233 | 
 234 |     <div class="position-relative header-wrapper js-header-wrapper ">
 235 |       <a href="#start-of-content" data-skip-target-assigned="false" class="px-2 tmp-py-4 color-bg-accent-emphasis color-fg-on-emphasis show-on-focus js-skip-to-content">Skip to content</a>
 236 | 
 237 |       <span data-view-component="true" class="progress-pjax-loader Progress position-fixed width-full">
 238 |     <span style="width: 0%;" data-view-component="true" class="Progress-item progress-pjax-loader-bar left-0 top-0 color-bg-accent-emphasis"></span>
 239 | </span>      
 240 |       
 241 |       <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-react-css.472b5991857bf128.module.css" />
 242 | 
 243 | <react-partial
 244 |   partial-name="keyboard-shortcuts-dialog"
 245 |   data-ssr="false"
 246 |   data-attempted-ssr="false"
 247 |   data-react-profiling="true"
 248 | >
 249 |   
 250 |   <script type="application/json" data-target="react-partial.embeddedData">{"props":{"docsUrl":"https://docs.github.com/get-started/accessibility/keyboard-shortcuts"}}</script>
 251 |   <div data-target="react-partial.reactRoot"></div>
 252 | </react-partial>
 253 | 
 254 | 
 255 | 
 256 | 
 257 | 
 258 |       
 259 | 
 260 |           
 261 | 
 262 |               
 263 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/46752-4c55523fe83d3457.js" defer="defer"></script>
 264 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/93308-bf887482583069d7.js" defer="defer"></script>
 265 | <script crossorigin="anonymous" type="application/javascript" src="https://github.githubassets.com/assets/sessions-ef46f0030f2d06d6.js" defer="defer"></script>
 266 | 
 267 | <style>
 268 |   /* Override primer focus outline color for marketing header dropdown links for better contrast */
 269 |   [data-color-mode="light"] .HeaderMenu-dropdown-link:focus-visible,
 270 |   [data-color-mode="light"] .HeaderMenu-trailing-link a:focus-visible {
 271 |     outline-color: var(--color-accent-fg);
 272 |   }
 273 | </style>
 274 | 
 275 | <header class="HeaderMktg header-logged-out js-details-container js-header Details f4 tmp-py-3" role="banner" data-is-top="true" data-color-mode=auto data-light-theme=light data-dark-theme=dark>
 276 |   <h2 class="sr-only">Navigation Menu</h2>
 277 | 
 278 |   <button type="button" class="HeaderMktg-backdrop d-lg-none border-0 position-fixed top-0 left-0 width-full height-full js-details-target" aria-label="Toggle navigation">
 279 |     <span class="d-none">Toggle navigation</span>
 280 |   </button>
 281 | 
 282 |   <div class="d-flex flex-column flex-lg-row flex-items-center tmp-px-3 tmp-px-md-4 tmp-px-lg-5 height-full position-relative z-1">
 283 |     <div class="d-flex flex-justify-between flex-items-center width-full width-lg-auto">
 284 |       <div class="flex-1">
 285 |         <button aria-label="Toggle navigation" aria-expanded="false" type="button" data-view-component="true" class="js-details-target js-nav-padding-recalculate js-header-menu-toggle Button--link Button--medium Button d-lg-none color-fg-inherit p-1">  <span class="Button-content">
 286 |     <span class="Button-label"><div class="HeaderMenu-toggle-bar rounded my-1"></div>
 287 |             <div class="HeaderMenu-toggle-bar rounded my-1"></div>
 288 |             <div class="HeaderMenu-toggle-bar rounded my-1"></div></span>
 289 |   </span>
 290 | </button>
 291 |       </div>
 292 | 
 293 |       <a class="tmp-mr-lg-3 color-fg-inherit flex-order-2 js-prevent-focus-on-mobile-nav"
 294 |         href="/"
 295 |         aria-label="Homepage"
 296 |         data-analytics-event="{&quot;category&quot;:&quot;Marketing nav&quot;,&quot;action&quot;:&quot;click to go to homepage&quot;,&quot;label&quot;:&quot;ref_page:Marketing;ref_cta:Logomark;ref_loc:Header&quot;}">
 297 |         <svg height="32" aria-hidden="true" viewBox="0 0 24 24" version="1.1" width="32" data-view-component="true" class="octicon octicon-mark-github">
 298 |     <path d="M12 1C5.923 1 1 5.923 1 12c0 4.867 3.149 8.979 7.521 10.436.55.096.756-.233.756-.522 0-.262-.013-1.128-.013-2.049-2.764.509-3.479-.674-3.699-1.292-.124-.317-.66-1.293-1.127-1.554-.385-.207-.936-.715-.014-.729.866-.014 1.485.797 1.691 1.128.99 1.663 2.571 1.196 3.204.907.096-.715.385-1.196.701-1.471-2.448-.275-5.005-1.224-5.005-5.432 0-1.196.426-2.186 1.128-2.956-.111-.275-.496-1.402.11-2.915 0 0 .921-.288 3.024 1.128a10.193 10.193 0 0 1 2.75-.371c.936 0 1.871.123 2.75.371 2.104-1.43 3.025-1.128 3.025-1.128.605 1.513.221 2.64.111 2.915.701.77 1.127 1.747 1.127 2.956 0 4.222-2.571 5.157-5.019 5.432.399.344.743 1.004.743 2.035 0 1.471-.014 2.654-.014 3.025 0 .289.206.632.756.522C19.851 20.979 23 16.854 23 12c0-6.077-4.922-11-11-11Z"></path>
 299 | </svg>
 300 |       </a>
 301 | 
 302 |       <div class="d-flex flex-1 flex-order-2 text-right d-lg-none gap-2 flex-justify-end">
 303 |           <a
 304 |             href="/login?return_to=https%3A%2F%2Fgithub.com%2Frsms%2Finter%2Fraw%2Fmaster%2Ffonts%2Fdesktop%2FInter-Regular.otf"
 305 |             class="HeaderMenu-link HeaderMenu-button d-inline-flex f5 no-underline border color-border-default rounded-2 px-2 py-1 color-fg-inherit js-prevent-focus-on-mobile-nav"
 306 |             data-hydro-click="{&quot;event_type&quot;:&quot;authentication.click&quot;,&quot;payload&quot;:{&quot;location_in_page&quot;:&quot;site header menu&quot;,&quot;repository_id&quot;:null,&quot;auth_type&quot;:&quot;SIGN_UP&quot;,&quot;originating_url&quot;:&quot;https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf&quot;,&quot;user_id&quot;:null}}" data-hydro-click-hmac="1251f23b33512ea2eb7663f363273438c7e2126b3cf24d994d4ff66cfc53b217"
 307 |             data-analytics-event="{&quot;category&quot;:&quot;Marketing nav&quot;,&quot;action&quot;:&quot;click to Sign in&quot;,&quot;label&quot;:&quot;ref_page:Marketing;ref_cta:Sign in;ref_loc:Header&quot;}"
 308 |           >
 309 |             Sign in
 310 |           </a>
 311 |               <div class="AppHeader-appearanceSettings">
 312 |     <react-partial-anchor>
 313 |       <button data-target="react-partial-anchor.anchor" id="icon-button-69970cf7-a581-42c9-8d22-490324a0210a" aria-labelledby="tooltip-b86f6db3-a918-4cac-b8ae-d5efe140e58f" type="button" disabled="disabled" data-view-component="true" class="Button Button--iconOnly Button--invisible Button--medium AppHeader-button HeaderMenu-link border cursor-wait">  <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-sliders Button-visual">
 314 |     <path d="M15 2.75a.75.75 0 0 1-.75.75h-4a.75.75 0 0 1 0-1.5h4a.75.75 0 0 1 .75.75Zm-8.5.75v1.25a.75.75 0 0 0 1.5 0v-4a.75.75 0 0 0-1.5 0V2H1.75a.75.75 0 0 0 0 1.5H6.5Zm1.25 5.25a.75.75 0 0 0 0-1.5h-6a.75.75 0 0 0 0 1.5h6ZM15 8a.75.75 0 0 1-.75.75H11.5V10a.75.75 0 1 1-1.5 0V6a.75.75 0 0 1 1.5 0v1.25h2.75A.75.75 0 0 1 15 8Zm-9 5.25v-2a.75.75 0 0 0-1.5 0v1.25H1.75a.75.75 0 0 0 0 1.5H4.5v1.25a.75.75 0 0 0 1.5 0v-2Zm9 0a.75.75 0 0 1-.75.75h-6a.75.75 0 0 1 0-1.5h6a.75.75 0 0 1 .75.75Z"></path>
 315 | </svg>
 316 | </button><tool-tip id="tooltip-b86f6db3-a918-4cac-b8ae-d5efe140e58f" for="icon-button-69970cf7-a581-42c9-8d22-490324a0210a" popover="manual" data-direction="s" data-type="label" data-view-component="true" class="sr-only position-absolute">Appearance settings</tool-tip>
 317 | 
 318 |       <template data-target="react-partial-anchor.template">
 319 |         <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-react-css.472b5991857bf128.module.css" />
 320 | <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/appearance-settings.4810edf2ebf35234.module.css" />
 321 | 
 322 | <react-partial
 323 |   partial-name="appearance-settings"
 324 |   data-ssr="false"
 325 |   data-attempted-ssr="false"
 326 |   data-react-profiling="true"
 327 | >
 328 |   
 329 |   <script type="application/json" data-target="react-partial.embeddedData">{"props":{}}</script>
 330 |   <div data-target="react-partial.reactRoot"></div>
 331 | </react-partial>
 332 | 
 333 | 
 334 |       </template>
 335 |     </react-partial-anchor>
 336 |   </div>
 337 | 
 338 |       </div>
 339 |     </div>
 340 | 
 341 | 
 342 |     <div class="HeaderMenu js-header-menu height-fit position-lg-relative d-lg-flex flex-column flex-auto top-0">
 343 |       <div class="HeaderMenu-wrapper d-flex flex-column flex-self-start flex-lg-row flex-auto rounded rounded-lg-0">
 344 |             <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-react-css.472b5991857bf128.module.css" />
 345 | <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/marketing-navigation.10d24ff3507f03d8.module.css" />
 346 | 
 347 | <react-partial
 348 |   partial-name="marketing-navigation"
 349 |   data-ssr="true"
 350 |   data-attempted-ssr="true"
 351 |   data-react-profiling="true"
 352 | >
 353 |   
 354 |   <script type="application/json" data-target="react-partial.embeddedData">{"props":{"should_use_dotcom_links":true}}</script>
 355 |   <div data-target="react-partial.reactRoot"><nav class="MarketingNavigation-module__nav__W0KYY" aria-label="Global"><ul class="MarketingNavigation-module__list__tFbMb"><li><div class="NavDropdown-module__container__l2YeI js-details-container js-header-menu-item"><button type="button" class="NavDropdown-module__button__PEHWX js-details-target" aria-expanded="false">Platform<svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavDropdown-module__buttonIcon__Tkl8_" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></button><div class="NavDropdown-module__dropdown__xm1jd"><ul class="NavDropdown-module__list__zuCgG"><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">AI CODE CREATION</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/features/copilot" data-analytics-event="{&quot;action&quot;:&quot;github_copilot&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_copilot_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-copilot NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M23.922 16.992c-.861 1.495-5.859 5.023-11.922 5.023-6.063 0-11.061-3.528-11.922-5.023A.641.641 0 0 1 0 16.736v-2.869a.841.841 0 0 1 .053-.22c.372-.935 1.347-2.292 2.605-2.656.167-.429.414-1.055.644-1.517a10.195 10.195 0 0 1-.052-1.086c0-1.331.282-2.499 1.132-3.368.397-.406.89-.717 1.474-.952 1.399-1.136 3.392-2.093 6.122-2.093 2.731 0 4.767.957 6.166 2.093.584.235 1.077.546 1.474.952.85.869 1.132 2.037 1.132 3.368 0 .368-.014.733-.052 1.086.23.462.477 1.088.644 1.517 1.258.364 2.233 1.721 2.605 2.656a.832.832 0 0 1 .053.22v2.869a.641.641 0 0 1-.078.256ZM12.172 11h-.344a4.323 4.323 0 0 1-.355.508C10.703 12.455 9.555 13 7.965 13c-1.725 0-2.989-.359-3.782-1.259a2.005 2.005 0 0 1-.085-.104L4 11.741v6.585c1.435.779 4.514 2.179 8 2.179 3.486 0 6.565-1.4 8-2.179v-6.585l-.098-.104s-.033.045-.085.104c-.793.9-2.057 1.259-3.782 1.259-1.59 0-2.738-.545-3.508-1.492a4.323 4.323 0 0 1-.355-.508h-.016.016Zm.641-2.935c.136 1.057.403 1.913.878 2.497.442.544 1.134.938 2.344.938 1.573 0 2.292-.337 2.657-.751.384-.435.558-1.15.558-2.361 0-1.14-.243-1.847-.705-2.319-.477-.488-1.319-.862-2.824-1.025-1.487-.161-2.192.138-2.533.529-.269.307-.437.808-.438 1.578v.021c0 .265.021.562.063.893Zm-1.626 0c.042-.331.063-.628.063-.894v-.02c-.001-.77-.169-1.271-.438-1.578-.341-.391-1.046-.69-2.533-.529-1.505.163-2.347.537-2.824 1.025-.462.472-.705 1.179-.705 2.319 0 1.211.175 1.926.558 2.361.365.414 1.084.751 2.657.751 1.21 0 1.902-.394 2.344-.938.475-.584.742-1.44.878-2.497Z"></path><path d="M14.5 14.25a1 1 0 0 1 1 1v2a1 1 0 0 1-2 0v-2a1 1 0 0 1 1-1Zm-5 0a1 1 0 0 1 1 1v2a1 1 0 0 1-2 0v-2a1 1 0 0 1 1-1Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Copilot</span><span class="NavLink-module__subtitle__X4gkW">Write better code with AI</span></div></a></li><li><a href="https://github.com/features/spark" data-analytics-event="{&quot;action&quot;:&quot;github_spark&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_spark_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-sparkle-fill NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M11.296 1.924c.24-.656 1.168-.656 1.408 0l.717 1.958a11.25 11.25 0 0 0 6.697 6.697l1.958.717c.657.24.657 1.168 0 1.408l-1.958.717a11.25 11.25 0 0 0-6.697 6.697l-.717 1.958c-.24.657-1.168.657-1.408 0l-.717-1.958a11.25 11.25 0 0 0-6.697-6.697l-1.958-.717c-.656-.24-.656-1.168 0-1.408l1.958-.717a11.25 11.25 0 0 0 6.697-6.697l.717-1.958Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Spark</span><span class="NavLink-module__subtitle__X4gkW">Build and deploy intelligent apps</span></div></a></li><li><a href="https://github.com/features/models" data-analytics-event="{&quot;action&quot;:&quot;github_models&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_models_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-ai-model NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M19.375 8.5a3.25 3.25 0 1 1-3.163 4h-3a3.252 3.252 0 0 1-4.443 2.509L7.214 17.76a3.25 3.25 0 1 1-1.342-.674l1.672-2.957A3.238 3.238 0 0 1 6.75 12c0-.907.371-1.727.97-2.316L6.117 6.846A3.253 3.253 0 0 1 1.875 3.75a3.25 3.25 0 1 1 5.526 2.32l1.603 2.836A3.25 3.25 0 0 1 13.093 11h3.119a3.252 3.252 0 0 1 3.163-2.5ZM10 10.25a1.75 1.75 0 1 0-.001 3.499A1.75 1.75 0 0 0 10 10.25ZM5.125 2a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5Zm12.5 9.75a1.75 1.75 0 1 0 3.5 0 1.75 1.75 0 0 0-3.5 0Zm-14.25 8.5a1.75 1.75 0 1 0 3.501-.001 1.75 1.75 0 0 0-3.501.001Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Models</span><span class="NavLink-module__subtitle__X4gkW">Manage and compare prompts</span></div></a></li><li><a href="https://github.com/mcp" data-analytics-event="{&quot;action&quot;:&quot;mcp_registry&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;mcp_registry_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-mcp NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M9.795 1.694a4.287 4.287 0 0 1 6.061 0 4.28 4.28 0 0 1 1.181 3.819 4.282 4.282 0 0 1 3.819 1.181 4.287 4.287 0 0 1 0 6.061l-6.793 6.793a.249.249 0 0 0 0 .353l2.617 2.618a.75.75 0 1 1-1.061 1.061l-2.617-2.618a1.75 1.75 0 0 1 0-2.475l6.793-6.793a2.785 2.785 0 1 0-3.939-3.939l-5.9 5.9a.734.734 0 0 1-.249.165.749.749 0 0 1-.812-1.225l5.9-5.901a2.785 2.785 0 1 0-3.939-3.939L2.931 10.68A.75.75 0 1 1 1.87 9.619l7.925-7.925Z"></path><path d="M12.42 4.069a.752.752 0 0 1 1.061 0 .752.752 0 0 1 0 1.061L7.33 11.28a2.788 2.788 0 0 0 0 3.94 2.788 2.788 0 0 0 3.94 0l6.15-6.151a.752.752 0 0 1 1.061 0 .752.752 0 0 1 0 1.061l-6.151 6.15a4.285 4.285 0 1 1-6.06-6.06l6.15-6.151Z"></path></svg><span class="NavLink-module__title__Q7t0p">MCP Registry<sup class="NavLink-module__label__bil7n">New</sup></span><span class="NavLink-module__subtitle__X4gkW">Integrate external tools</span></div></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">DEVELOPER WORKFLOWS</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/features/actions" data-analytics-event="{&quot;action&quot;:&quot;actions&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;actions_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-workflow NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M1 3a2 2 0 0 1 2-2h6.5a2 2 0 0 1 2 2v6.5a2 2 0 0 1-2 2H7v4.063C7 16.355 7.644 17 8.438 17H12.5v-2.5a2 2 0 0 1 2-2H21a2 2 0 0 1 2 2V21a2 2 0 0 1-2 2h-6.5a2 2 0 0 1-2-2v-2.5H8.437A2.939 2.939 0 0 1 5.5 15.562V11.5H3a2 2 0 0 1-2-2Zm2-.5a.5.5 0 0 0-.5.5v6.5a.5.5 0 0 0 .5.5h6.5a.5.5 0 0 0 .5-.5V3a.5.5 0 0 0-.5-.5ZM14.5 14a.5.5 0 0 0-.5.5V21a.5.5 0 0 0 .5.5H21a.5.5 0 0 0 .5-.5v-6.5a.5.5 0 0 0-.5-.5Z"></path></svg><span class="NavLink-module__title__Q7t0p">Actions</span><span class="NavLink-module__subtitle__X4gkW">Automate any workflow</span></div></a></li><li><a href="https://github.com/features/codespaces" data-analytics-event="{&quot;action&quot;:&quot;codespaces&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;codespaces_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-codespaces NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.5 3.75C3.5 2.784 4.284 2 5.25 2h13.5c.966 0 1.75.784 1.75 1.75v7.5A1.75 1.75 0 0 1 18.75 13H5.25a1.75 1.75 0 0 1-1.75-1.75Zm-2 12c0-.966.784-1.75 1.75-1.75h17.5c.966 0 1.75.784 1.75 1.75v4a1.75 1.75 0 0 1-1.75 1.75H3.25a1.75 1.75 0 0 1-1.75-1.75ZM5.25 3.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h13.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Zm-2 12a.25.25 0 0 0-.25.25v4c0 .138.112.25.25.25h17.5a.25.25 0 0 0 .25-.25v-4a.25.25 0 0 0-.25-.25Z"></path><path d="M10 17.75a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5a.75.75 0 0 1-.75-.75Zm-4 0a.75.75 0 0 1 .75-.75h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1-.75-.75Z"></path></svg><span class="NavLink-module__title__Q7t0p">Codespaces</span><span class="NavLink-module__subtitle__X4gkW">Instant dev environments</span></div></a></li><li><a href="https://github.com/features/issues" data-analytics-event="{&quot;action&quot;:&quot;issues&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;issues_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-issue-opened NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M12 1c6.075 0 11 4.925 11 11s-4.925 11-11 11S1 18.075 1 12 5.925 1 12 1ZM2.5 12a9.5 9.5 0 0 0 9.5 9.5 9.5 9.5 0 0 0 9.5-9.5A9.5 9.5 0 0 0 12 2.5 9.5 9.5 0 0 0 2.5 12Zm9.5 2a2 2 0 1 1-.001-3.999A2 2 0 0 1 12 14Z"></path></svg><span class="NavLink-module__title__Q7t0p">Issues</span><span class="NavLink-module__subtitle__X4gkW">Plan and track work</span></div></a></li><li><a href="https://github.com/features/code-review" data-analytics-event="{&quot;action&quot;:&quot;code_review&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;code_review_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-code NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M15.22 4.97a.75.75 0 0 1 1.06 0l6.5 6.5a.75.75 0 0 1 0 1.06l-6.5 6.5a.749.749 0 0 1-1.275-.326.749.749 0 0 1 .215-.734L21.19 12l-5.97-5.97a.75.75 0 0 1 0-1.06Zm-6.44 0a.75.75 0 0 1 0 1.06L2.81 12l5.97 5.97a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215l-6.5-6.5a.75.75 0 0 1 0-1.06l6.5-6.5a.75.75 0 0 1 1.06 0Z"></path></svg><span class="NavLink-module__title__Q7t0p">Code Review</span><span class="NavLink-module__subtitle__X4gkW">Manage code changes</span></div></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">APPLICATION SECURITY</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/security/advanced-security" data-analytics-event="{&quot;action&quot;:&quot;github_advanced_security&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_advanced_security_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-shield-check NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M16.53 9.78a.75.75 0 0 0-1.06-1.06L11 13.19l-1.97-1.97a.75.75 0 0 0-1.06 1.06l2.5 2.5a.75.75 0 0 0 1.06 0l5-5Z"></path><path d="m12.54.637 8.25 2.675A1.75 1.75 0 0 1 22 4.976V10c0 6.19-3.771 10.704-9.401 12.83a1.704 1.704 0 0 1-1.198 0C5.77 20.705 2 16.19 2 10V4.976c0-.758.489-1.43 1.21-1.664L11.46.637a1.748 1.748 0 0 1 1.08 0Zm-.617 1.426-8.25 2.676a.249.249 0 0 0-.173.237V10c0 5.46 3.28 9.483 8.43 11.426a.199.199 0 0 0 .14 0C17.22 19.483 20.5 15.461 20.5 10V4.976a.25.25 0 0 0-.173-.237l-8.25-2.676a.253.253 0 0 0-.154 0Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Advanced Security</span><span class="NavLink-module__subtitle__X4gkW">Find and fix vulnerabilities</span></div></a></li><li><a href="https://github.com/security/advanced-security/code-security" data-analytics-event="{&quot;action&quot;:&quot;code_security&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;code_security_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-code-square NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M10.3 8.24a.75.75 0 0 1-.04 1.06L7.352 12l2.908 2.7a.75.75 0 1 1-1.02 1.1l-3.5-3.25a.75.75 0 0 1 0-1.1l3.5-3.25a.75.75 0 0 1 1.06.04Zm3.44 1.06a.75.75 0 1 1 1.02-1.1l3.5 3.25a.75.75 0 0 1 0 1.1l-3.5 3.25a.75.75 0 1 1-1.02-1.1l2.908-2.7-2.908-2.7Z"></path><path d="M2 3.75C2 2.784 2.784 2 3.75 2h16.5c.966 0 1.75.784 1.75 1.75v16.5A1.75 1.75 0 0 1 20.25 22H3.75A1.75 1.75 0 0 1 2 20.25Zm1.75-.25a.25.25 0 0 0-.25.25v16.5c0 .138.112.25.25.25h16.5a.25.25 0 0 0 .25-.25V3.75a.25.25 0 0 0-.25-.25Z"></path></svg><span class="NavLink-module__title__Q7t0p">Code security</span><span class="NavLink-module__subtitle__X4gkW">Secure your code as you build</span></div></a></li><li><a href="https://github.com/security/advanced-security/secret-protection" data-analytics-event="{&quot;action&quot;:&quot;secret_protection&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;secret_protection_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-lock NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6 9V7.25C6 3.845 8.503 1 12 1s6 2.845 6 6.25V9h.5a2.5 2.5 0 0 1 2.5 2.5v8a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 19.5v-8A2.5 2.5 0 0 1 5.5 9Zm-1.5 2.5v8a1 1 0 0 0 1 1h13a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1h-13a1 1 0 0 0-1 1Zm3-4.25V9h9V7.25c0-2.67-1.922-4.75-4.5-4.75-2.578 0-4.5 2.08-4.5 4.75Z"></path></svg><span class="NavLink-module__title__Q7t0p">Secret protection</span><span class="NavLink-module__subtitle__X4gkW">Stop leaks before they start</span></div></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ NavGroup-module__hasSeparator__FnMrN"><span class="NavGroup-module__title__Wzxz2">EXPLORE</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/why-github" data-analytics-event="{&quot;action&quot;:&quot;why_github&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;why_github_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Why GitHub</span></a></li><li><a href="https://docs.github.com" data-analytics-event="{&quot;action&quot;:&quot;documentation&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;documentation_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Documentation</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://github.blog" data-analytics-event="{&quot;action&quot;:&quot;blog&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;blog_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Blog</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://github.blog/changelog" data-analytics-event="{&quot;action&quot;:&quot;changelog&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;changelog_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Changelog</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://github.com/marketplace" data-analytics-event="{&quot;action&quot;:&quot;marketplace&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;marketplace_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Marketplace</span></a></li></ul></div></li></ul><div class="NavDropdown-module__trailingLinkContainer__VgJGL"><a href="https://github.com/features" data-analytics-event="{&quot;action&quot;:&quot;view_all_features&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_features_link_platform_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all features</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></div></div></div></li><li><div class="NavDropdown-module__container__l2YeI js-details-container js-header-menu-item"><button type="button" class="NavDropdown-module__button__PEHWX js-details-target" aria-expanded="false">Solutions<svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavDropdown-module__buttonIcon__Tkl8_" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></button><div class="NavDropdown-module__dropdown__xm1jd"><ul class="NavDropdown-module__list__zuCgG"><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">BY COMPANY SIZE</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/enterprise" data-analytics-event="{&quot;action&quot;:&quot;enterprises&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;enterprises_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Enterprises</span></a></li><li><a href="https://github.com/team" data-analytics-event="{&quot;action&quot;:&quot;small_and_medium_teams&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;small_and_medium_teams_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Small and medium teams</span></a></li><li><a href="https://github.com/enterprise/startups" data-analytics-event="{&quot;action&quot;:&quot;startups&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;startups_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Startups</span></a></li><li><a href="https://github.com/solutions/industry/nonprofits" data-analytics-event="{&quot;action&quot;:&quot;nonprofits&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;nonprofits_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Nonprofits</span></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">BY USE CASE</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/solutions/use-case/app-modernization" data-analytics-event="{&quot;action&quot;:&quot;app_modernization&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;app_modernization_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">App Modernization</span></a></li><li><a href="https://github.com/solutions/use-case/devsecops" data-analytics-event="{&quot;action&quot;:&quot;devsecops&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;devsecops_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">DevSecOps</span></a></li><li><a href="https://github.com/solutions/use-case/devops" data-analytics-event="{&quot;action&quot;:&quot;devops&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;devops_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">DevOps</span></a></li><li><a href="https://github.com/solutions/use-case/ci-cd" data-analytics-event="{&quot;action&quot;:&quot;ci/cd&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;ci/cd_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">CI/CD</span></a></li><li><a href="https://github.com/solutions/use-case" data-analytics-event="{&quot;action&quot;:&quot;view_all_use_cases&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_use_cases_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all use cases</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">BY INDUSTRY</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/solutions/industry/healthcare" data-analytics-event="{&quot;action&quot;:&quot;healthcare&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;healthcare_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Healthcare</span></a></li><li><a href="https://github.com/solutions/industry/financial-services" data-analytics-event="{&quot;action&quot;:&quot;financial_services&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;financial_services_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Financial services</span></a></li><li><a href="https://github.com/solutions/industry/manufacturing" data-analytics-event="{&quot;action&quot;:&quot;manufacturing&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;manufacturing_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Manufacturing</span></a></li><li><a href="https://github.com/solutions/industry/government" data-analytics-event="{&quot;action&quot;:&quot;government&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;government_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Government</span></a></li><li><a href="https://github.com/solutions/industry" data-analytics-event="{&quot;action&quot;:&quot;view_all_industries&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_industries_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all industries</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></li></ul></div></li></ul><div class="NavDropdown-module__trailingLinkContainer__VgJGL"><a href="https://github.com/solutions" data-analytics-event="{&quot;action&quot;:&quot;view_all_solutions&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;solutions&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_solutions_link_solutions_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all solutions</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></div></div></div></li><li><div class="NavDropdown-module__container__l2YeI js-details-container js-header-menu-item"><button type="button" class="NavDropdown-module__button__PEHWX js-details-target" aria-expanded="false">Resources<svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavDropdown-module__buttonIcon__Tkl8_" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></button><div class="NavDropdown-module__dropdown__xm1jd"><ul class="NavDropdown-module__list__zuCgG"><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">EXPLORE BY TOPIC</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/resources/articles?topic=ai" data-analytics-event="{&quot;action&quot;:&quot;ai&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;ai_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">AI</span></a></li><li><a href="https://github.com/resources/articles?topic=software-development" data-analytics-event="{&quot;action&quot;:&quot;software_development&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;software_development_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Software Development</span></a></li><li><a href="https://github.com/resources/articles?topic=devops" data-analytics-event="{&quot;action&quot;:&quot;devops&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;devops_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">DevOps</span></a></li><li><a href="https://github.com/resources/articles?topic=security" data-analytics-event="{&quot;action&quot;:&quot;security&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;security_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Security</span></a></li><li><a href="https://github.com/resources/articles" data-analytics-event="{&quot;action&quot;:&quot;view_all_topics&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_topics_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all topics</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">EXPLORE BY TYPE</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/customer-stories" data-analytics-event="{&quot;action&quot;:&quot;customer_stories&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;customer_stories_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Customer stories</span></a></li><li><a href="https://github.com/resources/events" data-analytics-event="{&quot;action&quot;:&quot;events__webinars&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;events__webinars_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Events &amp; webinars</span></a></li><li><a href="https://github.com/resources/whitepapers" data-analytics-event="{&quot;action&quot;:&quot;ebooks__reports&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;ebooks__reports_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Ebooks &amp; reports</span></a></li><li><a href="https://github.com/solutions/executive-insights" data-analytics-event="{&quot;action&quot;:&quot;business_insights&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;business_insights_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Business insights</span></a></li><li><a href="https://skills.github.com" data-analytics-event="{&quot;action&quot;:&quot;github_skills&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_skills_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">GitHub Skills</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">SUPPORT &amp; SERVICES</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://docs.github.com" data-analytics-event="{&quot;action&quot;:&quot;documentation&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;documentation_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Documentation</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://support.github.com" data-analytics-event="{&quot;action&quot;:&quot;customer_support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;customer_support_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Customer support</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://github.com/orgs/community/discussions" data-analytics-event="{&quot;action&quot;:&quot;community_forum&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;community_forum_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Community forum</span></a></li><li><a href="https://github.com/trust-center" data-analytics-event="{&quot;action&quot;:&quot;trust_center&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;trust_center_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Trust center</span></a></li><li><a href="https://github.com/partners" data-analytics-event="{&quot;action&quot;:&quot;partners&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;partners_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Partners</span></a></li></ul></div></li></ul><div class="NavDropdown-module__trailingLinkContainer__VgJGL"><a href="https://github.com/resources" data-analytics-event="{&quot;action&quot;:&quot;view_all_resources&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;resources&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;view_all_resources_link_resources_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">View all resources</span><svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavLink-module__arrowIcon__amekg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></a></div></div></div></li><li><div class="NavDropdown-module__container__l2YeI js-details-container js-header-menu-item"><button type="button" class="NavDropdown-module__button__PEHWX js-details-target" aria-expanded="false">Open Source<svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavDropdown-module__buttonIcon__Tkl8_" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></button><div class="NavDropdown-module__dropdown__xm1jd"><ul class="NavDropdown-module__list__zuCgG"><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">COMMUNITY</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/sponsors" data-analytics-event="{&quot;action&quot;:&quot;github_sponsors&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_sponsors_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-sponsor-tiers NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M16.004 1.25C18.311 1.25 20 3.128 20 5.75c0 2.292-1.23 4.464-3.295 6.485-.481.47-.98.909-1.482 1.31l.265 1.32 1.375 7.5a.75.75 0 0 1-.982.844l-3.512-1.207a.75.75 0 0 0-.488 0L8.37 23.209a.75.75 0 0 1-.982-.844l1.378-7.512.261-1.309c-.5-.4-1-.838-1.481-1.31C5.479 10.215 4.25 8.043 4.25 5.75c0-2.622 1.689-4.5 3.996-4.5 1.55 0 2.947.752 3.832 1.967l.047.067.047-.067a4.726 4.726 0 0 1 3.612-1.962l.22-.005ZM13.89 14.531c-.418.285-.828.542-1.218.77l-.18.103a.75.75 0 0 1-.734 0l-.071-.04-.46-.272c-.282-.173-.573-.36-.868-.562l-.121.605-1.145 6.239 2.3-.79a2.248 2.248 0 0 1 1.284-.054l.18.053 2.299.79-1.141-6.226-.125-.616ZM16.004 2.75c-1.464 0-2.731.983-3.159 2.459-.209.721-1.231.721-1.44 0-.428-1.476-1.695-2.459-3.16-2.459-1.44 0-2.495 1.173-2.495 3 0 1.811 1.039 3.647 2.844 5.412a19.624 19.624 0 0 0 3.734 2.84l-.019-.011-.184-.111.147-.088a19.81 19.81 0 0 0 3.015-2.278l.37-.352C17.46 9.397 18.5 7.561 18.5 5.75c0-1.827-1.055-3-2.496-3Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Sponsors</span><span class="NavLink-module__subtitle__X4gkW">Fund open source developers</span></div></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">PROGRAMS</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://securitylab.github.com" data-analytics-event="{&quot;action&quot;:&quot;security_lab&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;security_lab_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Security Lab</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://maintainers.github.com" data-analytics-event="{&quot;action&quot;:&quot;maintainer_community&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;maintainer_community_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Maintainer Community</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li><li><a href="https://github.com/accelerator" data-analytics-event="{&quot;action&quot;:&quot;accelerator&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;accelerator_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Accelerator</span></a></li><li><a href="https://archiveprogram.github.com" data-analytics-event="{&quot;action&quot;:&quot;archive_program&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;archive_program_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4" target="_blank" rel="noreferrer"><span class="NavLink-module__title__Q7t0p">Archive Program</span><svg aria-hidden="true" focusable="false" class="octicon octicon-link-external NavLink-module__externalIcon__eWIry" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-3.5a.75.75 0 0 1 1.5 0v3.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5C2 2.784 2.784 2 3.75 2Zm6.854-1h4.146a.25.25 0 0 1 .25.25v4.146a.25.25 0 0 1-.427.177L13.03 4.03 9.28 7.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.75-3.75-1.543-1.543A.25.25 0 0 1 10.604 1Z"></path></svg></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">REPOSITORIES</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/topics" data-analytics-event="{&quot;action&quot;:&quot;topics&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;topics_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Topics</span></a></li><li><a href="https://github.com/trending" data-analytics-event="{&quot;action&quot;:&quot;trending&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;trending_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Trending</span></a></li><li><a href="https://github.com/collections" data-analytics-event="{&quot;action&quot;:&quot;collections&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;open_source&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;collections_link_open_source_navbar&quot;}" class="NavLink-module__link__EG3d4"><span class="NavLink-module__title__Q7t0p">Collections</span></a></li></ul></div></li></ul></div></div></li><li><div class="NavDropdown-module__container__l2YeI js-details-container js-header-menu-item"><button type="button" class="NavDropdown-module__button__PEHWX js-details-target" aria-expanded="false">Enterprise<svg aria-hidden="true" focusable="false" class="octicon octicon-chevron-right NavDropdown-module__buttonIcon__Tkl8_" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"></path></svg></button><div class="NavDropdown-module__dropdown__xm1jd"><ul class="NavDropdown-module__list__zuCgG"><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">ENTERPRISE SOLUTIONS</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/enterprise" data-analytics-event="{&quot;action&quot;:&quot;enterprise_platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;enterprise&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;enterprise_platform_link_enterprise_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-stack NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M11.063 1.456a1.749 1.749 0 0 1 1.874 0l8.383 5.316a1.751 1.751 0 0 1 0 2.956l-8.383 5.316a1.749 1.749 0 0 1-1.874 0L2.68 9.728a1.751 1.751 0 0 1 0-2.956Zm1.071 1.267a.25.25 0 0 0-.268 0L3.483 8.039a.25.25 0 0 0 0 .422l8.383 5.316a.25.25 0 0 0 .268 0l8.383-5.316a.25.25 0 0 0 0-.422Z"></path><path d="M1.867 12.324a.75.75 0 0 1 1.035-.232l8.964 5.685a.25.25 0 0 0 .268 0l8.964-5.685a.75.75 0 0 1 .804 1.267l-8.965 5.685a1.749 1.749 0 0 1-1.874 0l-8.965-5.685a.75.75 0 0 1-.231-1.035Z"></path><path d="M1.867 16.324a.75.75 0 0 1 1.035-.232l8.964 5.685a.25.25 0 0 0 .268 0l8.964-5.685a.75.75 0 0 1 .804 1.267l-8.965 5.685a1.749 1.749 0 0 1-1.874 0l-8.965-5.685a.75.75 0 0 1-.231-1.035Z"></path></svg><span class="NavLink-module__title__Q7t0p">Enterprise platform</span><span class="NavLink-module__subtitle__X4gkW">AI-powered developer platform</span></div></a></li></ul></div></li><li><div class="NavGroup-module__group__W8SqJ"><span class="NavGroup-module__title__Wzxz2">AVAILABLE ADD-ONS</span><ul class="NavGroup-module__list__UCOFy"><li><a href="https://github.com/security/advanced-security" data-analytics-event="{&quot;action&quot;:&quot;github_advanced_security&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;enterprise&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;github_advanced_security_link_enterprise_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-shield-check NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M16.53 9.78a.75.75 0 0 0-1.06-1.06L11 13.19l-1.97-1.97a.75.75 0 0 0-1.06 1.06l2.5 2.5a.75.75 0 0 0 1.06 0l5-5Z"></path><path d="m12.54.637 8.25 2.675A1.75 1.75 0 0 1 22 4.976V10c0 6.19-3.771 10.704-9.401 12.83a1.704 1.704 0 0 1-1.198 0C5.77 20.705 2 16.19 2 10V4.976c0-.758.489-1.43 1.21-1.664L11.46.637a1.748 1.748 0 0 1 1.08 0Zm-.617 1.426-8.25 2.676a.249.249 0 0 0-.173.237V10c0 5.46 3.28 9.483 8.43 11.426a.199.199 0 0 0 .14 0C17.22 19.483 20.5 15.461 20.5 10V4.976a.25.25 0 0 0-.173-.237l-8.25-2.676a.253.253 0 0 0-.154 0Z"></path></svg><span class="NavLink-module__title__Q7t0p">GitHub Advanced Security</span><span class="NavLink-module__subtitle__X4gkW">Enterprise-grade security features</span></div></a></li><li><a href="https://github.com/features/copilot/copilot-business" data-analytics-event="{&quot;action&quot;:&quot;copilot_for_business&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;enterprise&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;copilot_for_business_link_enterprise_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-copilot NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M23.922 16.992c-.861 1.495-5.859 5.023-11.922 5.023-6.063 0-11.061-3.528-11.922-5.023A.641.641 0 0 1 0 16.736v-2.869a.841.841 0 0 1 .053-.22c.372-.935 1.347-2.292 2.605-2.656.167-.429.414-1.055.644-1.517a10.195 10.195 0 0 1-.052-1.086c0-1.331.282-2.499 1.132-3.368.397-.406.89-.717 1.474-.952 1.399-1.136 3.392-2.093 6.122-2.093 2.731 0 4.767.957 6.166 2.093.584.235 1.077.546 1.474.952.85.869 1.132 2.037 1.132 3.368 0 .368-.014.733-.052 1.086.23.462.477 1.088.644 1.517 1.258.364 2.233 1.721 2.605 2.656a.832.832 0 0 1 .053.22v2.869a.641.641 0 0 1-.078.256ZM12.172 11h-.344a4.323 4.323 0 0 1-.355.508C10.703 12.455 9.555 13 7.965 13c-1.725 0-2.989-.359-3.782-1.259a2.005 2.005 0 0 1-.085-.104L4 11.741v6.585c1.435.779 4.514 2.179 8 2.179 3.486 0 6.565-1.4 8-2.179v-6.585l-.098-.104s-.033.045-.085.104c-.793.9-2.057 1.259-3.782 1.259-1.59 0-2.738-.545-3.508-1.492a4.323 4.323 0 0 1-.355-.508h-.016.016Zm.641-2.935c.136 1.057.403 1.913.878 2.497.442.544 1.134.938 2.344.938 1.573 0 2.292-.337 2.657-.751.384-.435.558-1.15.558-2.361 0-1.14-.243-1.847-.705-2.319-.477-.488-1.319-.862-2.824-1.025-1.487-.161-2.192.138-2.533.529-.269.307-.437.808-.438 1.578v.021c0 .265.021.562.063.893Zm-1.626 0c.042-.331.063-.628.063-.894v-.02c-.001-.77-.169-1.271-.438-1.578-.341-.391-1.046-.69-2.533-.529-1.505.163-2.347.537-2.824 1.025-.462.472-.705 1.179-.705 2.319 0 1.211.175 1.926.558 2.361.365.414 1.084.751 2.657.751 1.21 0 1.902-.394 2.344-.938.475-.584.742-1.44.878-2.497Z"></path><path d="M14.5 14.25a1 1 0 0 1 1 1v2a1 1 0 0 1-2 0v-2a1 1 0 0 1 1-1Zm-5 0a1 1 0 0 1 1 1v2a1 1 0 0 1-2 0v-2a1 1 0 0 1 1-1Z"></path></svg><span class="NavLink-module__title__Q7t0p">Copilot for Business</span><span class="NavLink-module__subtitle__X4gkW">Enterprise-grade AI features</span></div></a></li><li><a href="https://github.com/premium-support" data-analytics-event="{&quot;action&quot;:&quot;premium_support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;enterprise&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;premium_support_link_enterprise_navbar&quot;}" class="NavLink-module__link__EG3d4"><div class="NavLink-module__text__XvpLQ"><svg aria-hidden="true" focusable="false" class="octicon octicon-comment-discussion NavLink-module__icon__ltGNM" viewBox="0 0 24 24" width="24" height="24" fill="currentColor" display="inline-block" overflow="visible" style="vertical-align:text-bottom"><path d="M1.75 1h12.5c.966 0 1.75.784 1.75 1.75v9.5A1.75 1.75 0 0 1 14.25 14H8.061l-2.574 2.573A1.458 1.458 0 0 1 3 15.543V14H1.75A1.75 1.75 0 0 1 0 12.25v-9.5C0 1.784.784 1 1.75 1ZM1.5 2.75v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25Z"></path><path d="M22.5 8.75a.25.25 0 0 0-.25-.25h-3.5a.75.75 0 0 1 0-1.5h3.5c.966 0 1.75.784 1.75 1.75v9.5A1.75 1.75 0 0 1 22.25 20H21v1.543a1.457 1.457 0 0 1-2.487 1.03L15.939 20H10.75A1.75 1.75 0 0 1 9 18.25v-1.465a.75.75 0 0 1 1.5 0v1.465c0 .138.112.25.25.25h5.5a.75.75 0 0 1 .53.22l2.72 2.72v-2.19a.75.75 0 0 1 .75-.75h2a.25.25 0 0 0 .25-.25v-9.5Z"></path></svg><span class="NavLink-module__title__Q7t0p">Premium Support</span><span class="NavLink-module__subtitle__X4gkW">Enterprise-grade 24/7 support</span></div></a></li></ul></div></li></ul></div></div></li><li><a href="https://github.com/pricing" data-analytics-event="{&quot;action&quot;:&quot;pricing&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;context&quot;:&quot;pricing&quot;,&quot;location&quot;:&quot;navbar&quot;,&quot;label&quot;:&quot;pricing_link_pricing_navbar&quot;}" class="NavLink-module__link__EG3d4 MarketingNavigation-module__navLink__hUomM"><span class="NavLink-module__title__Q7t0p">Pricing</span></a></li></ul></nav><script type="application/json" id="__PRIMER_DATA__R_0___">{"resolvedServerColorMode":"day"}</script></div>
 356 | </react-partial>
 357 | 
 358 | 
 359 | 
 360 |         <div class="d-flex flex-column flex-lg-row width-full flex-justify-end flex-lg-items-center text-center tmp-mt-3 tmp-mt-lg-0 text-lg-left tmp-ml-lg-3">
 361 |                 
 362 | 
 363 | 
 364 | <qbsearch-input class="search-input" data-scope="owner:rsms" data-custom-scopes-path="/search/custom_scopes" data-delete-custom-scopes-csrf="i3h7E27akuxY538iXFU-x7evIR_QubkqZJDwfHGF_-m4lSgYGkYKibP5JCs3F-tlWD0QqFchabh0Fd4K5m_ApA" data-max-custom-scopes="10" data-header-redesign-enabled="false" data-initial-value="" data-blackbird-suggestions-path="/search/suggestions" data-jump-to-suggestions-path="/_graphql/GetSuggestedNavigationDestinations" data-current-repository="" data-current-org="" data-current-owner="" data-logged-in="false" data-copilot-chat-enabled="false" data-nl-search-enabled="false" data-retain-scroll-position="true">
 365 |   <div
 366 |     class="search-input-container search-with-dialog position-relative d-flex flex-row flex-items-center tmp-mr-4 rounded"
 367 |     data-action="click:qbsearch-input#searchInputContainerClicked"
 368 |   >
 369 |       <button
 370 |         type="button"
 371 |         class="header-search-button placeholder  input-button form-control d-flex flex-1 flex-self-stretch flex-items-center no-wrap width-full py-0 pl-2 pr-0 text-left border-0 box-shadow-none"
 372 |         data-target="qbsearch-input.inputButton"
 373 |         aria-label="Search or jump to…"
 374 |         aria-haspopup="dialog"
 375 |         placeholder="Search or jump to..."
 376 |         data-hotkey=s,/
 377 |         autocapitalize="off"
 378 |         data-analytics-event="{&quot;location&quot;:&quot;navbar&quot;,&quot;action&quot;:&quot;searchbar&quot;,&quot;context&quot;:&quot;global&quot;,&quot;tag&quot;:&quot;input&quot;,&quot;label&quot;:&quot;searchbar_input_global_navbar&quot;}"
 379 |         data-action="click:qbsearch-input#handleExpand"
 380 |       >
 381 |         <div class="mr-2 color-fg-muted">
 382 |           <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-search">
 383 |     <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"></path>
 384 | </svg>
 385 |         </div>
 386 |         <span class="flex-1" data-target="qbsearch-input.inputButtonText">Search or jump to...</span>
 387 |           <div class="d-flex" data-target="qbsearch-input.hotkeyIndicator">
 388 |             <svg xmlns="http://www.w3.org/2000/svg" width="22" height="20" aria-hidden="true" class="mr-1"><path fill="none" stroke="#979A9C" opacity=".4" d="M3.5.5h12c1.7 0 3 1.3 3 3v13c0 1.7-1.3 3-3 3h-12c-1.7 0-3-1.3-3-3v-13c0-1.7 1.3-3 3-3z"></path><path fill="#979A9C" d="M11.8 6L8 15.1h-.9L10.8 6h1z"></path></svg>
 389 |           </div>
 390 |       </button>
 391 | 
 392 |     <input type="hidden" name="type" class="js-site-search-type-field">
 393 | 
 394 |     
 395 | <div class="Overlay--hidden " data-modal-dialog-overlay>
 396 |   <modal-dialog data-action="close:qbsearch-input#handleClose cancel:qbsearch-input#handleClose" data-target="qbsearch-input.searchSuggestionsDialog" role="dialog" id="search-suggestions-dialog" aria-modal="true" aria-labelledby="search-suggestions-dialog-header" data-view-component="true" class="Overlay Overlay--width-large Overlay--height-auto">
 397 |       <h1 id="search-suggestions-dialog-header" class="sr-only">Search code, repositories, users, issues, pull requests...</h1>
 398 |     <div class="Overlay-body Overlay-body--paddingNone">
 399 |       
 400 |           <div data-view-component="true">        <div class="search-suggestions position-fixed width-full color-shadow-large border color-fg-default color-bg-default overflow-hidden d-flex flex-column query-builder-container"
 401 |           style="border-radius: 12px;"
 402 |           data-target="qbsearch-input.queryBuilderContainer"
 403 |           hidden
 404 |         >
 405 |           <!-- '"` --><!-- </textarea></xmp> --></option></form><form id="query-builder-test-form" action="" accept-charset="UTF-8" method="get">
 406 |   <query-builder data-target="qbsearch-input.queryBuilder" id="query-builder-query-builder-test" data-filter-key=":" data-view-component="true" class="QueryBuilder search-query-builder">
 407 |     <div class="FormControl FormControl--fullWidth">
 408 |       <label id="query-builder-test-label" for="query-builder-test" class="FormControl-label sr-only">
 409 |         Search
 410 |       </label>
 411 |       <div
 412 |         class="QueryBuilder-StyledInput width-fit "
 413 |         data-target="query-builder.styledInput"
 414 |       >
 415 |           <span id="query-builder-test-leadingvisual-wrap" class="FormControl-input-leadingVisualWrap QueryBuilder-leadingVisualWrap">
 416 |             <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-search FormControl-input-leadingVisual">
 417 |     <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"></path>
 418 | </svg>
 419 |           </span>
 420 |         <div data-target="query-builder.styledInputContainer" class="QueryBuilder-StyledInputContainer">
 421 |           <div
 422 |             aria-hidden="true"
 423 |             class="QueryBuilder-StyledInputContent"
 424 |             data-target="query-builder.styledInputContent"
 425 |           ></div>
 426 |           <div class="QueryBuilder-InputWrapper">
 427 |             <div aria-hidden="true" class="QueryBuilder-Sizer" data-target="query-builder.sizer"></div>
 428 |             <input id="query-builder-test" name="query-builder-test" value="" autocomplete="off" type="text" role="combobox" spellcheck="false" aria-expanded="false" aria-describedby="validation-0061a286-fb40-4432-b4e5-aa727ae4b012" data-target="query-builder.input" data-action="
 429 |           input:query-builder#inputChange
 430 |           blur:query-builder#inputBlur
 431 |           keydown:query-builder#inputKeydown
 432 |           focus:query-builder#inputFocus
 433 |         " data-view-component="true" class="FormControl-input QueryBuilder-Input FormControl-medium" />
 434 |           </div>
 435 |         </div>
 436 |           <span data-target="query-builder.clearButton" hidden>
 437 |             <span class="sr-only" id="query-builder-test-clear">Clear</span>
 438 |             <button role="button" id="query-builder-test-clear-button" aria-labelledby="query-builder-test-clear query-builder-test-label" data-action="
 439 |                   click:query-builder#clear
 440 |                   focus:query-builder#clearButtonFocus
 441 |                   blur:query-builder#clearButtonBlur
 442 |                 " variant="small" type="button" data-view-component="true" class="Button Button--iconOnly Button--invisible Button--medium mr-1 px-2 py-0 d-flex flex-items-center rounded-1 color-fg-muted">  <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x-circle-fill Button-visual">
 443 |     <path d="M2.343 13.657A8 8 0 1 1 13.658 2.343 8 8 0 0 1 2.343 13.657ZM6.03 4.97a.751.751 0 0 0-1.042.018.751.751 0 0 0-.018 1.042L6.94 8 4.97 9.97a.749.749 0 0 0 .326 1.275.749.749 0 0 0 .734-.215L8 9.06l1.97 1.97a.749.749 0 0 0 1.275-.326.749.749 0 0 0-.215-.734L9.06 8l1.97-1.97a.749.749 0 0 0-.326-1.275.749.749 0 0 0-.734.215L8 6.94Z"></path>
 444 | </svg>
 445 | </button>
 446 | 
 447 |           </span>
 448 |       </div>
 449 |       <template id="search-icon">
 450 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-search">
 451 |     <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"></path>
 452 | </svg>
 453 | </template>
 454 | 
 455 | <template id="code-icon">
 456 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-code">
 457 |     <path d="m11.28 3.22 4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.749.749 0 0 1-1.275-.326.749.749 0 0 1 .215-.734L13.94 8l-3.72-3.72a.749.749 0 0 1 .326-1.275.749.749 0 0 1 .734.215Zm-6.56 0a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042L2.06 8l3.72 3.72a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L.47 8.53a.75.75 0 0 1 0-1.06Z"></path>
 458 | </svg>
 459 | </template>
 460 | 
 461 | <template id="file-code-icon">
 462 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-file-code">
 463 |     <path d="M4 1.75C4 .784 4.784 0 5.75 0h5.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v8.586A1.75 1.75 0 0 1 14.25 15h-9a.75.75 0 0 1 0-1.5h9a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 10 4.25V1.5H5.75a.25.25 0 0 0-.25.25v2.5a.75.75 0 0 1-1.5 0Zm1.72 4.97a.75.75 0 0 1 1.06 0l2 2a.75.75 0 0 1 0 1.06l-2 2a.749.749 0 0 1-1.275-.326.749.749 0 0 1 .215-.734l1.47-1.47-1.47-1.47a.75.75 0 0 1 0-1.06ZM3.28 7.78 1.81 9.25l1.47 1.47a.751.751 0 0 1-.018 1.042.751.751 0 0 1-1.042.018l-2-2a.75.75 0 0 1 0-1.06l2-2a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042Zm8.22-6.218V4.25c0 .138.112.25.25.25h2.688l-.011-.013-2.914-2.914-.013-.011Z"></path>
 464 | </svg>
 465 | </template>
 466 | 
 467 | <template id="history-icon">
 468 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-history">
 469 |     <path d="m.427 1.927 1.215 1.215a8.002 8.002 0 1 1-1.6 5.685.75.75 0 1 1 1.493-.154 6.5 6.5 0 1 0 1.18-4.458l1.358 1.358A.25.25 0 0 1 3.896 6H.25A.25.25 0 0 1 0 5.75V2.104a.25.25 0 0 1 .427-.177ZM7.75 4a.75.75 0 0 1 .75.75v2.992l2.028.812a.75.75 0 0 1-.557 1.392l-2.5-1A.751.751 0 0 1 7 8.25v-3.5A.75.75 0 0 1 7.75 4Z"></path>
 470 | </svg>
 471 | </template>
 472 | 
 473 | <template id="repo-icon">
 474 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-repo">
 475 |     <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.249.249 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z"></path>
 476 | </svg>
 477 | </template>
 478 | 
 479 | <template id="bookmark-icon">
 480 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-bookmark">
 481 |     <path d="M3 2.75C3 1.784 3.784 1 4.75 1h6.5c.966 0 1.75.784 1.75 1.75v11.5a.75.75 0 0 1-1.227.579L8 11.722l-3.773 3.107A.751.751 0 0 1 3 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.91l3.023-2.489a.75.75 0 0 1 .954 0l3.023 2.49V2.75a.25.25 0 0 0-.25-.25Z"></path>
 482 | </svg>
 483 | </template>
 484 | 
 485 | <template id="plus-circle-icon">
 486 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-plus-circle">
 487 |     <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm7.25-3.25v2.5h2.5a.75.75 0 0 1 0 1.5h-2.5v2.5a.75.75 0 0 1-1.5 0v-2.5h-2.5a.75.75 0 0 1 0-1.5h2.5v-2.5a.75.75 0 0 1 1.5 0Z"></path>
 488 | </svg>
 489 | </template>
 490 | 
 491 | <template id="circle-icon">
 492 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-dot-fill">
 493 |     <path d="M8 4a4 4 0 1 1 0 8 4 4 0 0 1 0-8Z"></path>
 494 | </svg>
 495 | </template>
 496 | 
 497 | <template id="trash-icon">
 498 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-trash">
 499 |     <path d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75ZM4.496 6.675l.66 6.6a.25.25 0 0 0 .249.225h5.19a.25.25 0 0 0 .249-.225l.66-6.6a.75.75 0 0 1 1.492.149l-.66 6.6A1.748 1.748 0 0 1 10.595 15h-5.19a1.75 1.75 0 0 1-1.741-1.575l-.66-6.6a.75.75 0 1 1 1.492-.15ZM6.5 1.75V3h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25Z"></path>
 500 | </svg>
 501 | </template>
 502 | 
 503 | <template id="team-icon">
 504 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-people">
 505 |     <path d="M2 5.5a3.5 3.5 0 1 1 5.898 2.549 5.508 5.508 0 0 1 3.034 4.084.75.75 0 1 1-1.482.235 4 4 0 0 0-7.9 0 .75.75 0 0 1-1.482-.236A5.507 5.507 0 0 1 3.102 8.05 3.493 3.493 0 0 1 2 5.5ZM11 4a3.001 3.001 0 0 1 2.22 5.018 5.01 5.01 0 0 1 2.56 3.012.749.749 0 0 1-.885.954.752.752 0 0 1-.549-.514 3.507 3.507 0 0 0-2.522-2.372.75.75 0 0 1-.574-.73v-.352a.75.75 0 0 1 .416-.672A1.5 1.5 0 0 0 11 5.5.75.75 0 0 1 11 4Zm-5.5-.5a2 2 0 1 0-.001 3.999A2 2 0 0 0 5.5 3.5Z"></path>
 506 | </svg>
 507 | </template>
 508 | 
 509 | <template id="project-icon">
 510 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-project">
 511 |     <path d="M1.75 0h12.5C15.216 0 16 .784 16 1.75v12.5A1.75 1.75 0 0 1 14.25 16H1.75A1.75 1.75 0 0 1 0 14.25V1.75C0 .784.784 0 1.75 0ZM1.5 1.75v12.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25ZM11.75 3a.75.75 0 0 1 .75.75v7.5a.75.75 0 0 1-1.5 0v-7.5a.75.75 0 0 1 .75-.75Zm-8.25.75a.75.75 0 0 1 1.5 0v5.5a.75.75 0 0 1-1.5 0ZM8 3a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 3Z"></path>
 512 | </svg>
 513 | </template>
 514 | 
 515 | <template id="pencil-icon">
 516 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-pencil">
 517 |     <path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm.176 4.823L9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064Zm1.238-3.763a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354Z"></path>
 518 | </svg>
 519 | </template>
 520 | 
 521 | <template id="copilot-icon">
 522 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-copilot">
 523 |     <path d="M7.998 15.035c-4.562 0-7.873-2.914-7.998-3.749V9.338c.085-.628.677-1.686 1.588-2.065.013-.07.024-.143.036-.218.029-.183.06-.384.126-.612-.201-.508-.254-1.084-.254-1.656 0-.87.128-1.769.693-2.484.579-.733 1.494-1.124 2.724-1.261 1.206-.134 2.262.034 2.944.765.05.053.096.108.139.165.044-.057.094-.112.143-.165.682-.731 1.738-.899 2.944-.765 1.23.137 2.145.528 2.724 1.261.566.715.693 1.614.693 2.484 0 .572-.053 1.148-.254 1.656.066.228.098.429.126.612.012.076.024.148.037.218.924.385 1.522 1.471 1.591 2.095v1.872c0 .766-3.351 3.795-8.002 3.795Zm0-1.485c2.28 0 4.584-1.11 5.002-1.433V7.862l-.023-.116c-.49.21-1.075.291-1.727.291-1.146 0-2.059-.327-2.71-.991A3.222 3.222 0 0 1 8 6.303a3.24 3.24 0 0 1-.544.743c-.65.664-1.563.991-2.71.991-.652 0-1.236-.081-1.727-.291l-.023.116v4.255c.419.323 2.722 1.433 5.002 1.433ZM6.762 2.83c-.193-.206-.637-.413-1.682-.297-1.019.113-1.479.404-1.713.7-.247.312-.369.789-.369 1.554 0 .793.129 1.171.308 1.371.162.181.519.379 1.442.379.853 0 1.339-.235 1.638-.54.315-.322.527-.827.617-1.553.117-.935-.037-1.395-.241-1.614Zm4.155-.297c-1.044-.116-1.488.091-1.681.297-.204.219-.359.679-.242 1.614.091.726.303 1.231.618 1.553.299.305.784.54 1.638.54.922 0 1.28-.198 1.442-.379.179-.2.308-.578.308-1.371 0-.765-.123-1.242-.37-1.554-.233-.296-.693-.587-1.713-.7Z"></path><path d="M6.25 9.037a.75.75 0 0 1 .75.75v1.501a.75.75 0 0 1-1.5 0V9.787a.75.75 0 0 1 .75-.75Zm4.25.75v1.501a.75.75 0 0 1-1.5 0V9.787a.75.75 0 0 1 1.5 0Z"></path>
 524 | </svg>
 525 | </template>
 526 | 
 527 | <template id="copilot-error-icon">
 528 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-copilot-error">
 529 |     <path d="M16 11.24c0 .112-.072.274-.21.467L13 9.688V7.862l-.023-.116c-.49.21-1.075.291-1.727.291-.198 0-.388-.009-.571-.029L6.833 5.226a4.01 4.01 0 0 0 .17-.782c.117-.935-.037-1.395-.241-1.614-.193-.206-.637-.413-1.682-.297-.683.076-1.115.231-1.395.415l-1.257-.91c.579-.564 1.413-.877 2.485-.996 1.206-.134 2.262.034 2.944.765.05.053.096.108.139.165.044-.057.094-.112.143-.165.682-.731 1.738-.899 2.944-.765 1.23.137 2.145.528 2.724 1.261.566.715.693 1.614.693 2.484 0 .572-.053 1.148-.254 1.656.066.228.098.429.126.612.012.076.024.148.037.218.924.385 1.522 1.471 1.591 2.095Zm-5.083-8.707c-1.044-.116-1.488.091-1.681.297-.204.219-.359.679-.242 1.614.091.726.303 1.231.618 1.553.299.305.784.54 1.638.54.922 0 1.28-.198 1.442-.379.179-.2.308-.578.308-1.371 0-.765-.123-1.242-.37-1.554-.233-.296-.693-.587-1.713-.7Zm2.511 11.074c-1.393.776-3.272 1.428-5.43 1.428-4.562 0-7.873-2.914-7.998-3.749V9.338c.085-.628.677-1.686 1.588-2.065.013-.07.024-.143.036-.218.029-.183.06-.384.126-.612-.18-.455-.241-.963-.252-1.475L.31 4.107A.747.747 0 0 1 0 3.509V3.49a.748.748 0 0 1 .625-.73c.156-.026.306.047.435.139l14.667 10.578a.592.592 0 0 1 .227.264.752.752 0 0 1 .046.249v.022a.75.75 0 0 1-1.19.596Zm-1.367-.991L5.635 7.964a5.128 5.128 0 0 1-.889.073c-.652 0-1.236-.081-1.727-.291l-.023.116v4.255c.419.323 2.722 1.433 5.002 1.433 1.539 0 3.089-.505 4.063-.934Z"></path>
 530 | </svg>
 531 | </template>
 532 | 
 533 | <template id="workflow-icon">
 534 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-workflow">
 535 |     <path d="M0 1.75C0 .784.784 0 1.75 0h3.5C6.216 0 7 .784 7 1.75v3.5A1.75 1.75 0 0 1 5.25 7H4v4a1 1 0 0 0 1 1h4v-1.25C9 9.784 9.784 9 10.75 9h3.5c.966 0 1.75.784 1.75 1.75v3.5A1.75 1.75 0 0 1 14.25 16h-3.5A1.75 1.75 0 0 1 9 14.25v-.75H5A2.5 2.5 0 0 1 2.5 11V7h-.75A1.75 1.75 0 0 1 0 5.25Zm1.75-.25a.25.25 0 0 0-.25.25v3.5c0 .138.112.25.25.25h3.5a.25.25 0 0 0 .25-.25v-3.5a.25.25 0 0 0-.25-.25Zm9 9a.25.25 0 0 0-.25.25v3.5c0 .138.112.25.25.25h3.5a.25.25 0 0 0 .25-.25v-3.5a.25.25 0 0 0-.25-.25Z"></path>
 536 | </svg>
 537 | </template>
 538 | 
 539 | <template id="book-icon">
 540 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-book">
 541 |     <path d="M0 1.75A.75.75 0 0 1 .75 1h4.253c1.227 0 2.317.59 3 1.501A3.743 3.743 0 0 1 11.006 1h4.245a.75.75 0 0 1 .75.75v10.5a.75.75 0 0 1-.75.75h-4.507a2.25 2.25 0 0 0-1.591.659l-.622.621a.75.75 0 0 1-1.06 0l-.622-.621A2.25 2.25 0 0 0 5.258 13H.75a.75.75 0 0 1-.75-.75Zm7.251 10.324.004-5.073-.002-2.253A2.25 2.25 0 0 0 5.003 2.5H1.5v9h3.757a3.75 3.75 0 0 1 1.994.574ZM8.755 4.75l-.004 7.322a3.752 3.752 0 0 1 1.992-.572H14.5v-9h-3.495a2.25 2.25 0 0 0-2.25 2.25Z"></path>
 542 | </svg>
 543 | </template>
 544 | 
 545 | <template id="code-review-icon">
 546 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-code-review">
 547 |     <path d="M1.75 1h12.5c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0 1 14.25 13H8.061l-2.574 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25v-8.5C0 1.784.784 1 1.75 1ZM1.5 2.75v8.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-8.5a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25Zm5.28 1.72a.75.75 0 0 1 0 1.06L5.31 7l1.47 1.47a.751.751 0 0 1-.018 1.042.751.751 0 0 1-1.042.018l-2-2a.75.75 0 0 1 0-1.06l2-2a.75.75 0 0 1 1.06 0Zm2.44 0a.75.75 0 0 1 1.06 0l2 2a.75.75 0 0 1 0 1.06l-2 2a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L10.69 7 9.22 5.53a.75.75 0 0 1 0-1.06Z"></path>
 548 | </svg>
 549 | </template>
 550 | 
 551 | <template id="codespaces-icon">
 552 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-codespaces">
 553 |     <path d="M0 11.25c0-.966.784-1.75 1.75-1.75h12.5c.966 0 1.75.784 1.75 1.75v3A1.75 1.75 0 0 1 14.25 16H1.75A1.75 1.75 0 0 1 0 14.25Zm2-9.5C2 .784 2.784 0 3.75 0h8.5C13.216 0 14 .784 14 1.75v5a1.75 1.75 0 0 1-1.75 1.75h-8.5A1.75 1.75 0 0 1 2 6.75Zm1.75-.25a.25.25 0 0 0-.25.25v5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-5a.25.25 0 0 0-.25-.25Zm-2 9.5a.25.25 0 0 0-.25.25v3c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-3a.25.25 0 0 0-.25-.25Z"></path><path d="M7 12.75a.75.75 0 0 1 .75-.75h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1-.75-.75Zm-4 0a.75.75 0 0 1 .75-.75h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1-.75-.75Z"></path>
 554 | </svg>
 555 | </template>
 556 | 
 557 | <template id="comment-icon">
 558 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-comment">
 559 |     <path d="M1 2.75C1 1.784 1.784 1 2.75 1h10.5c.966 0 1.75.784 1.75 1.75v7.5A1.75 1.75 0 0 1 13.25 12H9.06l-2.573 2.573A1.458 1.458 0 0 1 4 13.543V12H2.75A1.75 1.75 0 0 1 1 10.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h4.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"></path>
 560 | </svg>
 561 | </template>
 562 | 
 563 | <template id="comment-discussion-icon">
 564 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-comment-discussion">
 565 |     <path d="M1.75 1h8.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 10.25 10H7.061l-2.574 2.573A1.458 1.458 0 0 1 2 11.543V10h-.25A1.75 1.75 0 0 1 0 8.25v-5.5C0 1.784.784 1 1.75 1ZM1.5 2.75v5.5c0 .138.112.25.25.25h1a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h3.5a.25.25 0 0 0 .25-.25v-5.5a.25.25 0 0 0-.25-.25h-8.5a.25.25 0 0 0-.25.25Zm13 2a.25.25 0 0 0-.25-.25h-.5a.75.75 0 0 1 0-1.5h.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 14.25 12H14v1.543a1.458 1.458 0 0 1-2.487 1.03L9.22 12.28a.749.749 0 0 1 .326-1.275.749.749 0 0 1 .734.215l2.22 2.22v-2.19a.75.75 0 0 1 .75-.75h1a.25.25 0 0 0 .25-.25Z"></path>
 566 | </svg>
 567 | </template>
 568 | 
 569 | <template id="organization-icon">
 570 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-organization">
 571 |     <path d="M1.75 16A1.75 1.75 0 0 1 0 14.25V1.75C0 .784.784 0 1.75 0h8.5C11.216 0 12 .784 12 1.75v12.5c0 .085-.006.168-.018.25h2.268a.25.25 0 0 0 .25-.25V8.285a.25.25 0 0 0-.111-.208l-1.055-.703a.749.749 0 1 1 .832-1.248l1.055.703c.487.325.779.871.779 1.456v5.965A1.75 1.75 0 0 1 14.25 16h-3.5a.766.766 0 0 1-.197-.026c-.099.017-.2.026-.303.026h-3a.75.75 0 0 1-.75-.75V14h-1v1.25a.75.75 0 0 1-.75.75Zm-.25-1.75c0 .138.112.25.25.25H4v-1.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 .75.75v1.25h2.25a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25h-8.5a.25.25 0 0 0-.25.25ZM3.75 6h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1 0-1.5ZM3 3.75A.75.75 0 0 1 3.75 3h.5a.75.75 0 0 1 0 1.5h-.5A.75.75 0 0 1 3 3.75Zm4 3A.75.75 0 0 1 7.75 6h.5a.75.75 0 0 1 0 1.5h-.5A.75.75 0 0 1 7 6.75ZM7.75 3h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1 0-1.5ZM3 9.75A.75.75 0 0 1 3.75 9h.5a.75.75 0 0 1 0 1.5h-.5A.75.75 0 0 1 3 9.75ZM7.75 9h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1 0-1.5Z"></path>
 572 | </svg>
 573 | </template>
 574 | 
 575 | <template id="rocket-icon">
 576 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-rocket">
 577 |     <path d="M14.064 0h.186C15.216 0 16 .784 16 1.75v.186a8.752 8.752 0 0 1-2.564 6.186l-.458.459c-.314.314-.641.616-.979.904v3.207c0 .608-.315 1.172-.833 1.49l-2.774 1.707a.749.749 0 0 1-1.11-.418l-.954-3.102a1.214 1.214 0 0 1-.145-.125L3.754 9.816a1.218 1.218 0 0 1-.124-.145L.528 8.717a.749.749 0 0 1-.418-1.11l1.71-2.774A1.748 1.748 0 0 1 3.31 4h3.204c.288-.338.59-.665.904-.979l.459-.458A8.749 8.749 0 0 1 14.064 0ZM8.938 3.623h-.002l-.458.458c-.76.76-1.437 1.598-2.02 2.5l-1.5 2.317 2.143 2.143 2.317-1.5c.902-.583 1.74-1.26 2.499-2.02l.459-.458a7.25 7.25 0 0 0 2.123-5.127V1.75a.25.25 0 0 0-.25-.25h-.186a7.249 7.249 0 0 0-5.125 2.123ZM3.56 14.56c-.732.732-2.334 1.045-3.005 1.148a.234.234 0 0 1-.201-.064.234.234 0 0 1-.064-.201c.103-.671.416-2.273 1.15-3.003a1.502 1.502 0 1 1 2.12 2.12Zm6.94-3.935c-.088.06-.177.118-.266.175l-2.35 1.521.548 1.783 1.949-1.2a.25.25 0 0 0 .119-.213ZM3.678 8.116 5.2 5.766c.058-.09.117-.178.176-.266H3.309a.25.25 0 0 0-.213.119l-1.2 1.95ZM12 5a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path>
 578 | </svg>
 579 | </template>
 580 | 
 581 | <template id="shield-check-icon">
 582 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-shield-check">
 583 |     <path d="m8.533.133 5.25 1.68A1.75 1.75 0 0 1 15 3.48V7c0 1.566-.32 3.182-1.303 4.682-.983 1.498-2.585 2.813-5.032 3.855a1.697 1.697 0 0 1-1.33 0c-2.447-1.042-4.049-2.357-5.032-3.855C1.32 10.182 1 8.566 1 7V3.48a1.75 1.75 0 0 1 1.217-1.667l5.25-1.68a1.748 1.748 0 0 1 1.066 0Zm-.61 1.429.001.001-5.25 1.68a.251.251 0 0 0-.174.237V7c0 1.36.275 2.666 1.057 3.859.784 1.194 2.121 2.342 4.366 3.298a.196.196 0 0 0 .154 0c2.245-.957 3.582-2.103 4.366-3.297C13.225 9.666 13.5 8.358 13.5 7V3.48a.25.25 0 0 0-.174-.238l-5.25-1.68a.25.25 0 0 0-.153 0ZM11.28 6.28l-3.5 3.5a.75.75 0 0 1-1.06 0l-1.5-1.5a.749.749 0 0 1 .326-1.275.749.749 0 0 1 .734.215l.97.97 2.97-2.97a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042Z"></path>
 584 | </svg>
 585 | </template>
 586 | 
 587 | <template id="heart-icon">
 588 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-heart">
 589 |     <path d="m8 14.25.345.666a.75.75 0 0 1-.69 0l-.008-.004-.018-.01a7.152 7.152 0 0 1-.31-.17 22.055 22.055 0 0 1-3.434-2.414C2.045 10.731 0 8.35 0 5.5 0 2.836 2.086 1 4.25 1 5.797 1 7.153 1.802 8 3.02 8.847 1.802 10.203 1 11.75 1 13.914 1 16 2.836 16 5.5c0 2.85-2.045 5.231-3.885 6.818a22.066 22.066 0 0 1-3.744 2.584l-.018.01-.006.003h-.002ZM4.25 2.5c-1.336 0-2.75 1.164-2.75 3 0 2.15 1.58 4.144 3.365 5.682A20.58 20.58 0 0 0 8 13.393a20.58 20.58 0 0 0 3.135-2.211C12.92 9.644 14.5 7.65 14.5 5.5c0-1.836-1.414-3-2.75-3-1.373 0-2.609.986-3.029 2.456a.749.749 0 0 1-1.442 0C6.859 3.486 5.623 2.5 4.25 2.5Z"></path>
 590 | </svg>
 591 | </template>
 592 | 
 593 | <template id="server-icon">
 594 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-server">
 595 |     <path d="M1.75 1h12.5c.966 0 1.75.784 1.75 1.75v4c0 .372-.116.717-.314 1 .198.283.314.628.314 1v4a1.75 1.75 0 0 1-1.75 1.75H1.75A1.75 1.75 0 0 1 0 12.75v-4c0-.358.109-.707.314-1a1.739 1.739 0 0 1-.314-1v-4C0 1.784.784 1 1.75 1ZM1.5 2.75v4c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-4a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25Zm.25 5.75a.25.25 0 0 0-.25.25v4c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-4a.25.25 0 0 0-.25-.25ZM7 4.75A.75.75 0 0 1 7.75 4h4.5a.75.75 0 0 1 0 1.5h-4.5A.75.75 0 0 1 7 4.75ZM7.75 10h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM3 4.75A.75.75 0 0 1 3.75 4h.5a.75.75 0 0 1 0 1.5h-.5A.75.75 0 0 1 3 4.75ZM3.75 10h.5a.75.75 0 0 1 0 1.5h-.5a.75.75 0 0 1 0-1.5Z"></path>
 596 | </svg>
 597 | </template>
 598 | 
 599 | <template id="globe-icon">
 600 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-globe">
 601 |     <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM5.78 8.75a9.64 9.64 0 0 0 1.363 4.177c.255.426.542.832.857 1.215.245-.296.551-.705.857-1.215A9.64 9.64 0 0 0 10.22 8.75Zm4.44-1.5a9.64 9.64 0 0 0-1.363-4.177c-.307-.51-.612-.919-.857-1.215a9.927 9.927 0 0 0-.857 1.215A9.64 9.64 0 0 0 5.78 7.25Zm-5.944 1.5H1.543a6.507 6.507 0 0 0 4.666 5.5c-.123-.181-.24-.365-.352-.552-.715-1.192-1.437-2.874-1.581-4.948Zm-2.733-1.5h2.733c.144-2.074.866-3.756 1.58-4.948.12-.197.237-.381.353-.552a6.507 6.507 0 0 0-4.666 5.5Zm10.181 1.5c-.144 2.074-.866 3.756-1.58 4.948-.12.197-.237.381-.353.552a6.507 6.507 0 0 0 4.666-5.5Zm2.733-1.5a6.507 6.507 0 0 0-4.666-5.5c.123.181.24.365.353.552.714 1.192 1.436 2.874 1.58 4.948Z"></path>
 602 | </svg>
 603 | </template>
 604 | 
 605 | <template id="issue-opened-icon">
 606 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-issue-opened">
 607 |     <path d="M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z"></path><path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z"></path>
 608 | </svg>
 609 | </template>
 610 | 
 611 | <template id="device-mobile-icon">
 612 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-device-mobile">
 613 |     <path d="M3.75 0h8.5C13.216 0 14 .784 14 1.75v12.5A1.75 1.75 0 0 1 12.25 16h-8.5A1.75 1.75 0 0 1 2 14.25V1.75C2 .784 2.784 0 3.75 0ZM3.5 1.75v12.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25h-8.5a.25.25 0 0 0-.25.25ZM8 13a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path>
 614 | </svg>
 615 | </template>
 616 | 
 617 | <template id="package-icon">
 618 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-package">
 619 |     <path d="m8.878.392 5.25 3.045c.54.314.872.89.872 1.514v6.098a1.75 1.75 0 0 1-.872 1.514l-5.25 3.045a1.75 1.75 0 0 1-1.756 0l-5.25-3.045A1.75 1.75 0 0 1 1 11.049V4.951c0-.624.332-1.201.872-1.514L7.122.392a1.75 1.75 0 0 1 1.756 0ZM7.875 1.69l-4.63 2.685L8 7.133l4.755-2.758-4.63-2.685a.248.248 0 0 0-.25 0ZM2.5 5.677v5.372c0 .09.047.171.125.216l4.625 2.683V8.432Zm6.25 8.271 4.625-2.683a.25.25 0 0 0 .125-.216V5.677L8.75 8.432Z"></path>
 620 | </svg>
 621 | </template>
 622 | 
 623 | <template id="credit-card-icon">
 624 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-credit-card">
 625 |     <path d="M10.75 9a.75.75 0 0 0 0 1.5h1.5a.75.75 0 0 0 0-1.5h-1.5Z"></path><path d="M0 3.75C0 2.784.784 2 1.75 2h12.5c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0 1 14.25 14H1.75A1.75 1.75 0 0 1 0 12.25ZM14.5 6.5h-13v5.75c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25Zm0-2.75a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25V5h13Z"></path>
 626 | </svg>
 627 | </template>
 628 | 
 629 | <template id="play-icon">
 630 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-play">
 631 |     <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm4.879-2.773 4.264 2.559a.25.25 0 0 1 0 .428l-4.264 2.559A.25.25 0 0 1 6 10.559V5.442a.25.25 0 0 1 .379-.215Z"></path>
 632 | </svg>
 633 | </template>
 634 | 
 635 | <template id="gift-icon">
 636 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-gift">
 637 |     <path d="M2 2.75A2.75 2.75 0 0 1 4.75 0c.983 0 1.873.42 2.57 1.232.268.318.497.668.68 1.042.183-.375.411-.725.68-1.044C9.376.42 10.266 0 11.25 0a2.75 2.75 0 0 1 2.45 4h.55c.966 0 1.75.784 1.75 1.75v2c0 .698-.409 1.301-1 1.582v4.918A1.75 1.75 0 0 1 13.25 16H2.75A1.75 1.75 0 0 1 1 14.25V9.332C.409 9.05 0 8.448 0 7.75v-2C0 4.784.784 4 1.75 4h.55c-.192-.375-.3-.8-.3-1.25ZM7.25 9.5H2.5v4.75c0 .138.112.25.25.25h4.5Zm1.5 0v5h4.5a.25.25 0 0 0 .25-.25V9.5Zm0-4V8h5.5a.25.25 0 0 0 .25-.25v-2a.25.25 0 0 0-.25-.25Zm-7 0a.25.25 0 0 0-.25.25v2c0 .138.112.25.25.25h5.5V5.5h-5.5Zm3-4a1.25 1.25 0 0 0 0 2.5h2.309c-.233-.818-.542-1.401-.878-1.793-.43-.502-.915-.707-1.431-.707ZM8.941 4h2.309a1.25 1.25 0 0 0 0-2.5c-.516 0-1 .205-1.43.707-.337.392-.646.975-.879 1.793Z"></path>
 638 | </svg>
 639 | </template>
 640 | 
 641 | <template id="code-square-icon">
 642 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-code-square">
 643 |     <path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v12.5A1.75 1.75 0 0 1 14.25 16H1.75A1.75 1.75 0 0 1 0 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25Zm7.47 3.97a.75.75 0 0 1 1.06 0l2 2a.75.75 0 0 1 0 1.06l-2 2a.749.749 0 0 1-1.275-.326.749.749 0 0 1 .215-.734L10.69 8 9.22 6.53a.75.75 0 0 1 0-1.06ZM6.78 6.53 5.31 8l1.47 1.47a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215l-2-2a.75.75 0 0 1 0-1.06l2-2a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042Z"></path>
 644 | </svg>
 645 | </template>
 646 | 
 647 | <template id="device-desktop-icon">
 648 |   <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-device-desktop">
 649 |     <path d="M14.25 1c.966 0 1.75.784 1.75 1.75v7.5A1.75 1.75 0 0 1 14.25 12h-3.727c.099 1.041.52 1.872 1.292 2.757A.752.752 0 0 1 11.25 16h-6.5a.75.75 0 0 1-.565-1.243c.772-.885 1.192-1.716 1.292-2.757H1.75A1.75 1.75 0 0 1 0 10.25v-7.5C0 1.784.784 1 1.75 1ZM1.75 2.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25ZM9.018 12H6.982a5.72 5.72 0 0 1-.765 2.5h3.566a5.72 5.72 0 0 1-.765-2.5Z"></path>
 650 | </svg>
 651 | </template>
 652 | 
 653 |         <div class="position-relative">
 654 |                         <ul
 655 |               role="listbox"
 656 |               class="ActionListWrap QueryBuilder-ListWrap"
 657 |               aria-label="Suggestions"
 658 |               data-action="
 659 |                 combobox-commit:query-builder#comboboxCommit
 660 |                 mousedown:query-builder#resultsMousedown
 661 |               "
 662 |               data-target="query-builder.resultsList"
 663 |               data-persist-list=false
 664 |               id="query-builder-test-results"
 665 |               tabindex="-1"
 666 |             ></ul>
 667 | 
 668 |         </div>
 669 |       <div class="FormControl-inlineValidation" id="validation-0061a286-fb40-4432-b4e5-aa727ae4b012" hidden="hidden">
 670 |         <span class="FormControl-inlineValidation--visual">
 671 |           <svg aria-hidden="true" height="12" viewBox="0 0 12 12" version="1.1" width="12" data-view-component="true" class="octicon octicon-alert-fill">
 672 |     <path d="M4.855.708c.5-.896 1.79-.896 2.29 0l4.675 8.351a1.312 1.312 0 0 1-1.146 1.954H1.33A1.313 1.313 0 0 1 .183 9.058ZM7 7V3H5v4Zm-1 3a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"></path>
 673 | </svg>
 674 |         </span>
 675 |         <span></span>
 676 | </div>    </div>
 677 |     <div data-target="query-builder.screenReaderFeedback" aria-live="polite" aria-atomic="true" class="sr-only"></div>
 678 | </query-builder></form>
 679 |           <div class="d-flex flex-row color-fg-muted tmp-px-3 text-small color-bg-default search-feedback-prompt">
 680 |             <a target="_blank" href="https://docs.github.com/search-github/github-code-search/understanding-github-code-search-syntax" data-view-component="true" class="Link color-fg-accent text-normal ml-2">Search syntax tips</a>            <div class="d-flex flex-1"></div>
 681 |           </div>
 682 |         </div>
 683 | </div>
 684 | 
 685 |     </div>
 686 | </modal-dialog></div>
 687 |   </div>
 688 |   <div data-action="click:qbsearch-input#retract" class="dark-backdrop position-fixed" hidden data-target="qbsearch-input.darkBackdrop"></div>
 689 |   <div class="color-fg-default">
 690 |     
 691 | <dialog-helper>
 692 |   <dialog data-target="qbsearch-input.feedbackDialog" data-action="close:qbsearch-input#handleDialogClose cancel:qbsearch-input#handleDialogClose" id="feedback-dialog" aria-modal="true" aria-labelledby="feedback-dialog-title" aria-describedby="feedback-dialog-description" data-view-component="true" class="Overlay Overlay-whenNarrow Overlay--size-medium Overlay--motion-scaleFade Overlay--disableScroll">
 693 |     <div data-view-component="true" class="Overlay-header">
 694 |   <div class="Overlay-headerContentWrap">
 695 |     <div class="Overlay-titleWrap">
 696 |       <h1 class="Overlay-title " id="feedback-dialog-title">
 697 |         Provide feedback
 698 |       </h1>
 699 |         
 700 |     </div>
 701 |     <div class="Overlay-actionWrap">
 702 |       <button data-close-dialog-id="feedback-dialog" aria-label="Close" aria-label="Close" type="button" data-view-component="true" class="close-button Overlay-closeButton"><svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x">
 703 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
 704 | </svg></button>
 705 |     </div>
 706 |   </div>
 707 |   
 708 | </div>
 709 |       <scrollable-region data-labelled-by="feedback-dialog-title">
 710 |         <div data-view-component="true" class="Overlay-body">        <!-- '"` --><!-- </textarea></xmp> --></option></form><form id="code-search-feedback-form" data-turbo="false" action="/search/feedback" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="7-IRsXUmaV4bEKaja47ZB9s4hxAj1OkrzAAoG3ZhXZEi6IP1Tn3v7F798wDIo22OFzYvuoh12PJriMX_GPxxhg" />
 711 |           <p>We read every piece of feedback, and take your input very seriously.</p>
 712 |           <textarea name="feedback" class="form-control width-full mb-2" style="height: 120px" id="feedback"></textarea>
 713 |           <input name="include_email" id="include_email" aria-label="Include my email address so I can be contacted" class="form-control mr-2" type="checkbox">
 714 |           <label for="include_email" style="font-weight: normal">Include my email address so I can be contacted</label>
 715 | </form></div>
 716 |       </scrollable-region>
 717 |       <div data-view-component="true" class="Overlay-footer Overlay-footer--alignEnd">          <button data-close-dialog-id="feedback-dialog" type="button" data-view-component="true" class="btn">    Cancel
 718 | </button>
 719 |           <button form="code-search-feedback-form" data-action="click:qbsearch-input#submitFeedback" type="submit" data-view-component="true" class="btn-primary btn">    Submit feedback
 720 | </button>
 721 | </div>
 722 | </dialog></dialog-helper>
 723 | 
 724 |     <custom-scopes data-target="qbsearch-input.customScopesManager">
 725 |     
 726 | <dialog-helper>
 727 |   <dialog data-target="custom-scopes.customScopesModalDialog" data-action="close:qbsearch-input#handleDialogClose cancel:qbsearch-input#handleDialogClose" id="custom-scopes-dialog" aria-modal="true" aria-labelledby="custom-scopes-dialog-title" aria-describedby="custom-scopes-dialog-description" data-view-component="true" class="Overlay Overlay-whenNarrow Overlay--size-medium Overlay--motion-scaleFade Overlay--disableScroll">
 728 |     <div data-view-component="true" class="Overlay-header Overlay-header--divided">
 729 |   <div class="Overlay-headerContentWrap">
 730 |     <div class="Overlay-titleWrap">
 731 |       <h1 class="Overlay-title " id="custom-scopes-dialog-title">
 732 |         Saved searches
 733 |       </h1>
 734 |         <h2 id="custom-scopes-dialog-description" class="Overlay-description">Use saved searches to filter your results more quickly</h2>
 735 |     </div>
 736 |     <div class="Overlay-actionWrap">
 737 |       <button data-close-dialog-id="custom-scopes-dialog" aria-label="Close" aria-label="Close" type="button" data-view-component="true" class="close-button Overlay-closeButton"><svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x">
 738 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
 739 | </svg></button>
 740 |     </div>
 741 |   </div>
 742 |   
 743 | </div>
 744 |       <scrollable-region data-labelled-by="custom-scopes-dialog-title">
 745 |         <div data-view-component="true" class="Overlay-body">        <div data-target="custom-scopes.customScopesModalDialogFlash"></div>
 746 | 
 747 |         <div hidden class="create-custom-scope-form" data-target="custom-scopes.createCustomScopeForm">
 748 |         <!-- '"` --><!-- </textarea></xmp> --></option></form><form id="custom-scopes-dialog-form" data-turbo="false" action="/search/custom_scopes" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="MuJf3oY5hMHqYXx3Yi1hiIVcj3c8rGvp4TnhmKpiThIkSWsioC_wuL4bo098cP9ST721ZYEkjcKfkfAAZKta4A" />
 749 |           <div data-target="custom-scopes.customScopesModalDialogFlash"></div>
 750 | 
 751 |           <input type="hidden" id="custom_scope_id" name="custom_scope_id" data-target="custom-scopes.customScopesIdField">
 752 | 
 753 |           <div class="form-group">
 754 |             <label for="custom_scope_name">Name</label>
 755 |             <auto-check src="/search/custom_scopes/check_name" required>
 756 |               <input
 757 |                 type="text"
 758 |                 name="custom_scope_name"
 759 |                 id="custom_scope_name"
 760 |                 data-target="custom-scopes.customScopesNameField"
 761 |                 class="form-control"
 762 |                 autocomplete="off"
 763 |                 placeholder="github-ruby"
 764 |                 required
 765 |                 maxlength="50">
 766 |               <input type="hidden" value="fFHyJfBtaf3xlVSKvIkJfWmV9VhCtOPPMBr4d1IwLPWrSi15b_YecnaVv0A3mJgwaACEeVroo1Bbi64Irqde5g" data-csrf="true" />
 767 |             </auto-check>
 768 |           </div>
 769 | 
 770 |           <div class="form-group">
 771 |             <label for="custom_scope_query">Query</label>
 772 |             <input
 773 |               type="text"
 774 |               name="custom_scope_query"
 775 |               id="custom_scope_query"
 776 |               data-target="custom-scopes.customScopesQueryField"
 777 |               class="form-control"
 778 |               autocomplete="off"
 779 |               placeholder="(repo:mona/a OR repo:mona/b) AND lang:python"
 780 |               required
 781 |               maxlength="500">
 782 |           </div>
 783 | 
 784 |           <p class="text-small color-fg-muted">
 785 |             To see all available qualifiers, see our <a class="Link--inTextBlock" href="https://docs.github.com/search-github/github-code-search/understanding-github-code-search-syntax">documentation</a>.
 786 |           </p>
 787 | </form>        </div>
 788 | 
 789 |         <div data-target="custom-scopes.manageCustomScopesForm">
 790 |           <div data-target="custom-scopes.list"></div>
 791 |         </div>
 792 | 
 793 | </div>
 794 |       </scrollable-region>
 795 |       <div data-view-component="true" class="Overlay-footer Overlay-footer--alignEnd Overlay-footer--divided">          <button data-action="click:custom-scopes#customScopesCancel" type="button" data-view-component="true" class="btn">    Cancel
 796 | </button>
 797 |           <button form="custom-scopes-dialog-form" data-action="click:custom-scopes#customScopesSubmit" data-target="custom-scopes.customScopesSubmitButton" type="submit" data-view-component="true" class="btn-primary btn">    Create saved search
 798 | </button>
 799 | </div>
 800 | </dialog></dialog-helper>
 801 |     </custom-scopes>
 802 |   </div>
 803 | </qbsearch-input>
 804 | 
 805 | 
 806 |             <div class="position-relative HeaderMenu-link-wrap d-lg-inline-block">
 807 |               <a
 808 |                 href="/login?return_to=https%3A%2F%2Fgithub.com%2Frsms%2Finter%2Fraw%2Fmaster%2Ffonts%2Fdesktop%2FInter-Regular.otf"
 809 |                 class="HeaderMenu-link HeaderMenu-link--sign-in HeaderMenu-button flex-shrink-0 no-underline d-none d-lg-inline-flex border border-lg-0 rounded px-2 py-1"
 810 |                 style="margin-left: 12px;"
 811 |                 data-hydro-click="{&quot;event_type&quot;:&quot;authentication.click&quot;,&quot;payload&quot;:{&quot;location_in_page&quot;:&quot;site header menu&quot;,&quot;repository_id&quot;:null,&quot;auth_type&quot;:&quot;SIGN_UP&quot;,&quot;originating_url&quot;:&quot;https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf&quot;,&quot;user_id&quot;:null}}" data-hydro-click-hmac="1251f23b33512ea2eb7663f363273438c7e2126b3cf24d994d4ff66cfc53b217"
 812 |                 data-analytics-event="{&quot;category&quot;:&quot;Marketing nav&quot;,&quot;action&quot;:&quot;click to go to homepage&quot;,&quot;label&quot;:&quot;ref_page:Marketing;ref_cta:Sign in;ref_loc:Header&quot;}"
 813 |               >
 814 |                 Sign in
 815 |               </a>
 816 |                 <div style="right: -30%; background-color: transparent; border: none" data-view-component="true" class="auth-form-body Popover position-absolute d-none d-sm-none d-md-none d-lg-block">
 817 |   <div style="width: 300px" data-view-component="true" class="Popover-message Box Popover-message--top-right color-fg-default p-4 mt-2 mx-auto text-left">
 818 |     <h4 data-view-component="true" class="color-fg-default mb-2">                    Sign in to GitHub
 819 | </h4>
 820 |                         
 821 | <!-- '"` --><!-- </textarea></xmp> --></option></form><form data-turbo="false" action="/session" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="YrJaoaj4Ype0Khe51pUnvnB30b1WWxhXogzgjwVEAqLSunmpcmpn1EdVVH-_tUyNlad51hQZKszL7387CTxSPg" />  <input type="hidden" name="add_account" id="add_account" autocomplete="off" class="form-control" />
 822 | 
 823 |     <label for="login_field">
 824 |       Username or email address
 825 |     </label>
 826 |     <input type="text" name="login" id="login_field" class="form-control input-block js-login-field" autocapitalize="off" autocorrect="off" autocomplete="username" autofocus="autofocus" required="required" />
 827 | 
 828 |   <div class="position-relative">
 829 |     <label for="password">
 830 |       Password
 831 |     </label>
 832 |     <input type="password" name="password" id="password" class="form-control form-control input-block js-password-field" autocomplete="current-password" required="required" />
 833 |     <a class="label-link position-absolute top-0 right-0" id="forgot-password" href="/password_reset">Forgot password?</a>
 834 |     
 835 | <input type="hidden" name="webauthn-conditional" value="undefined">
 836 | <input type="hidden" class="js-support" name="javascript-support" value="unknown">
 837 | <input type="hidden" class="js-webauthn-support" name="webauthn-support" value="unknown">
 838 | <input type="hidden" class="js-webauthn-iuvpaa-support" name="webauthn-iuvpaa-support" value="unknown">
 839 | <input type="hidden" name="return_to" id="return_to" value="https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf" autocomplete="off" class="form-control" />
 840 | <input type="hidden" name="allow_signup" id="allow_signup" autocomplete="off" class="form-control" />
 841 | <input type="hidden" name="client_id" id="client_id" autocomplete="off" class="form-control" />
 842 | <input type="hidden" name="integration" id="integration" autocomplete="off" class="form-control" />
 843 | <input type="text" name="required_field_3b20" hidden="hidden" class="form-control" /><input type="hidden" name="timestamp" value="1772473934767" autocomplete="off" class="form-control" /><input type="hidden" name="timestamp_secret" value="8e0c09fd94c867c94e41edf97bad04d243eb00874cb3d24c9dd7342455a6ac85" autocomplete="off" class="form-control" />
 844 | 
 845 |     <input type="submit" name="commit" value="Sign in" class="btn btn-primary btn-block js-sign-in-button" data-disable-with="Signing in…" data-signin-label="Sign in" data-sso-label="Sign in with your identity provider" development="false" disable-emu-sso="false" />
 846 |   </div>
 847 | </form>  <webauthn-status class="js-webauthn-login-emu-control">
 848 |         <div data-target="webauthn-status.partial" class="d-flex flex-justify-between flex-column tmp-mt-3 mb-0" hidden>
 849 |           <a href="/login?return_to=https%3A%2F%2Fgithub.com%2Frsms%2Finter%2Fraw%2Fmaster%2Ffonts%2Fdesktop%2FInter-Regular.otf" data-analytics-event="{&quot;category&quot;:&quot;passkey_404_login&quot;,&quot;action&quot;:&quot;clicked&quot;,&quot;label&quot;:null}" data-view-component="true" class="Button--link Button--medium Button">  <span class="Button-content">
 850 |     <span class="Button-label">or continue with other methods</span>
 851 |   </span>
 852 | </a>
 853 |         </div>
 854 |   </webauthn-status>
 855 | 
 856 | 
 857 | </div></div>            </div>
 858 | 
 859 |               <a href="/signup?ref_cta=Sign+up&amp;ref_loc=header+logged+out&amp;ref_page=%2Frsms%2Finter%2Fraw%2Fmaster%2Ffonts%2Fdesktop%2FInter-Regular.otf&amp;source=header"
 860 |                 class="HeaderMenu-link HeaderMenu-link--sign-up HeaderMenu-button flex-shrink-0 d-flex d-lg-inline-flex no-underline border color-border-default rounded px-2 py-1"
 861 |                 data-hydro-click="{&quot;event_type&quot;:&quot;authentication.click&quot;,&quot;payload&quot;:{&quot;location_in_page&quot;:&quot;site header menu&quot;,&quot;repository_id&quot;:null,&quot;auth_type&quot;:&quot;SIGN_UP&quot;,&quot;originating_url&quot;:&quot;https://github.com/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf&quot;,&quot;user_id&quot;:null}}" data-hydro-click-hmac="1251f23b33512ea2eb7663f363273438c7e2126b3cf24d994d4ff66cfc53b217"
 862 |                 data-analytics-event="{&quot;category&quot;:&quot;Sign up&quot;,&quot;action&quot;:&quot;click to sign up for account&quot;,&quot;label&quot;:&quot;ref_page:/rsms/inter/raw/master/fonts/desktop/Inter-Regular.otf;ref_cta:Sign up;ref_loc:header logged out&quot;}"
 863 |               >
 864 |                 Sign up
 865 |               </a>
 866 | 
 867 |                 <div class="AppHeader-appearanceSettings">
 868 |     <react-partial-anchor>
 869 |       <button data-target="react-partial-anchor.anchor" id="icon-button-60102b14-f1e8-4391-8847-b640f9bf8383" aria-labelledby="tooltip-1d7562a8-5f46-4978-b4ba-9d9012296770" type="button" disabled="disabled" data-view-component="true" class="Button Button--iconOnly Button--invisible Button--medium AppHeader-button HeaderMenu-link border cursor-wait">  <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-sliders Button-visual">
 870 |     <path d="M15 2.75a.75.75 0 0 1-.75.75h-4a.75.75 0 0 1 0-1.5h4a.75.75 0 0 1 .75.75Zm-8.5.75v1.25a.75.75 0 0 0 1.5 0v-4a.75.75 0 0 0-1.5 0V2H1.75a.75.75 0 0 0 0 1.5H6.5Zm1.25 5.25a.75.75 0 0 0 0-1.5h-6a.75.75 0 0 0 0 1.5h6ZM15 8a.75.75 0 0 1-.75.75H11.5V10a.75.75 0 1 1-1.5 0V6a.75.75 0 0 1 1.5 0v1.25h2.75A.75.75 0 0 1 15 8Zm-9 5.25v-2a.75.75 0 0 0-1.5 0v1.25H1.75a.75.75 0 0 0 0 1.5H4.5v1.25a.75.75 0 0 0 1.5 0v-2Zm9 0a.75.75 0 0 1-.75.75h-6a.75.75 0 0 1 0-1.5h6a.75.75 0 0 1 .75.75Z"></path>
 871 | </svg>
 872 | </button><tool-tip id="tooltip-1d7562a8-5f46-4978-b4ba-9d9012296770" for="icon-button-60102b14-f1e8-4391-8847-b640f9bf8383" popover="manual" data-direction="s" data-type="label" data-view-component="true" class="sr-only position-absolute">Appearance settings</tool-tip>
 873 | 
 874 |       <template data-target="react-partial-anchor.template">
 875 |         <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/primer-react-css.472b5991857bf128.module.css" />
 876 | <link crossorigin="anonymous" media="all" rel="stylesheet" href="https://github.githubassets.com/assets/appearance-settings.4810edf2ebf35234.module.css" />
 877 | 
 878 | <react-partial
 879 |   partial-name="appearance-settings"
 880 |   data-ssr="false"
 881 |   data-attempted-ssr="false"
 882 |   data-react-profiling="true"
 883 | >
 884 |   
 885 |   <script type="application/json" data-target="react-partial.embeddedData">{"props":{}}</script>
 886 |   <div data-target="react-partial.reactRoot"></div>
 887 | </react-partial>
 888 | 
 889 | 
 890 |       </template>
 891 |     </react-partial-anchor>
 892 |   </div>
 893 | 
 894 |           <button type="button" class="sr-only js-header-menu-focus-trap d-block d-lg-none">Resetting focus</button>
 895 |         </div>
 896 |       </div>
 897 |     </div>
 898 |   </div>
 899 | </header>
 900 | 
 901 |       <div hidden="hidden" data-view-component="true" class="js-stale-session-flash stale-session-flash flash flash-warn flash-full">
 902 |   
 903 |         <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-alert">
 904 |     <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path>
 905 | </svg>
 906 |         <span class="js-stale-session-flash-signed-in" hidden>You signed in with another tab or window. <a class="Link--inTextBlock" href="">Reload</a> to refresh your session.</span>
 907 |         <span class="js-stale-session-flash-signed-out" hidden>You signed out in another tab or window. <a class="Link--inTextBlock" href="">Reload</a> to refresh your session.</span>
 908 |         <span class="js-stale-session-flash-switched" hidden>You switched accounts on another tab or window. <a class="Link--inTextBlock" href="">Reload</a> to refresh your session.</span>
 909 | 
 910 |     <button id="icon-button-fc0741fd-f0cc-4e02-9d8a-49515ababd47" aria-labelledby="tooltip-b103d2c8-2ec0-459b-bfa7-0eddfaf785a3" type="button" data-view-component="true" class="Button Button--iconOnly Button--invisible Button--medium flash-close js-flash-close">  <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x Button-visual">
 911 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
 912 | </svg>
 913 | </button><tool-tip id="tooltip-b103d2c8-2ec0-459b-bfa7-0eddfaf785a3" for="icon-button-fc0741fd-f0cc-4e02-9d8a-49515ababd47" popover="manual" data-direction="s" data-type="label" data-view-component="true" class="sr-only position-absolute">Dismiss alert</tool-tip>
 914 | 
 915 | 
 916 |   
 917 | </div>
 918 |     </div>
 919 | 
 920 |   <div id="start-of-content" class="show-on-focus"></div>
 921 | 
 922 | 
 923 | 
 924 | 
 925 | 
 926 | 
 927 | 
 928 | 
 929 |     <div id="js-flash-container" class="flash-container" data-turbo-replace>
 930 | 
 931 | 
 932 | 
 933 | 
 934 |   <template class="js-flash-template">
 935 |     
 936 | <div class="flash flash-full   {{ className }}">
 937 |   <div >
 938 |     <button autofocus class="flash-close js-flash-close" type="button" aria-label="Dismiss this message">
 939 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x">
 940 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
 941 | </svg>
 942 |     </button>
 943 |     <div aria-atomic="true" role="alert" class="js-flash-alert">
 944 |       
 945 |       <div>{{ message }}</div>
 946 | 
 947 |     </div>
 948 |   </div>
 949 | </div>
 950 |   </template>
 951 | </div>
 952 | 
 953 | 
 954 |     
 955 | 
 956 | 
 957 | 
 958 | 
 959 | 
 960 | 
 961 |   <div
 962 |     class="application-main d-flex flex-auto flex-column"
 963 |     data-commit-hovercards-enabled
 964 |     data-discussion-hovercards-enabled
 965 |     data-issue-and-pr-hovercards-enabled
 966 |     data-project-hovercards-enabled
 967 |   >
 968 |         <main class="font-mktg " >
 969 |     
 970 | 
 971 | 
 972 |   <div class="position-relative" style="z-index: 0; transition: all 0.25s ease-in">
 973 |     <div class="position-absolute overflow-hidden width-full top-0 left-0" style="height: 370px" data-hpc>
 974 |       <img alt="" class="position-absolute" height="415" width="940" style="top: -20px; left: -20px; z-index: 1; width: 110%; height: 425px"
 975 |       src="data:image/jpeg;base64,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">
 976 |     </div>
 977 |     <div class="position-relative d-block my-0 mx-auto overflow-hidden" style="width: 940px; height: 370px; clear: both">
 978 |       <img alt="404 &ldquo;This is not the web page you are looking for&rdquo;" class="position-absolute" height="249" width="271" style="z-index: 10; left: 72px; top: 72px"
 979 |       src="data:image/png;base64,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">
 980 | 
 981 |       <img alt="" class="position-absolute" height="230" width="188" style="top: 94px; left: 356px; z-index: 9;"
 982 |       src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAALwAAADmCAMAAABYgh8IAAAAA3NCSVQICAjb4U/gAAABgFBMVEX///9SOCxSOjH/wp8AAAD+wJ4ICAhWPjL/7tDMQjj////66834vZuZmZmVcl+bdmN7KCIxIRr39/dUQjpRS0nFQjhKMihptaVQRUEzJyAyIx46KSF8LSdAKyJSOCxUQjqcincQEBBSOjFSOjHzp4tSOCz/xqZSOjFkTEBUQjpSOjG7qJP/+PQpHhpSOjFSOjH87+jz4cRqUURDMSlTSURQRUFUQjr/1r//0bAaEg9UQjqHZFL/59nOTENTSURSOCyNfGojGhddV0wpKSnGl3yWlJKUh3d4YlM5OTkQEBDez7fLvKWdj4hzW0xIQj/05+bbp4nWZlZTSURTSURSOjFSOjHo17yMgn5+bFwZLSlWPjL358v/4sL5w6XWxKyJcmF3VkYhFxIzMzP86MynnIi1jHRXmYwhISEYGBhSOCwICAj50rvehn+SblqRMClWPjIICAgAAACLZWJAa2I2XVRRS0kQEBBUQjpLOC/xt5blqqWvhnBNh3spRD4ICAhWPjKmNNozAAAAgHRSTlMA////////////////////////RBH///8i///////uZv//d4j/3f+q/1WZ////Zrv/////IjN3////M////zPM//8RM/////8RiP////8R////EUTM7v////+7/////////yL/////RGa73f////+q7u7///8id5mq//////+q7kFCNkwAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAAcdEVYdFNvZnR3YXJlAEFkb2JlIEZpcmV3b3JrcyBDUzQGstOgAAAgAElEQVR4nMVdh0PUyBo32U2C7gLC0vvCozcBQREbFlTUs/feFfXOe/q84on/+pv2lZkkm6x3nCNuSzL5zTe/r8zMl2TXrsTSNrny2/z8+PBgZX/yDv9e6Tp6rXKongMGPV0i+TI+eXWngGWUrsmpGxrGt2u5DxrxfIPel3++d2ywbQcxJpa2ym8DHit5hV9B6JGvP/q+P145saNgrdJWmcKeV/LzvBs5xXfds4togUDvfRv5l/hf+c3Xvc763/NWch3b5bFjtOhNB3jjR3cY965dhzTN7aJ0L5fo57CxrPHw4Vh+zfke5ILnvoNZ/+BH3mSeGqboCHV8RE2R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">
 983 | 
 984 |       <img alt="" class="position-absolute" height="156" width="440" style="top: 150px; left: 432px; z-index: 8;"
 985 |       src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAbgAAACcCAMAAAA6Xk4VAAAAA3NCSVQICAjb4U/gAAADAFBMVEX///9NmcCTfmuUe2OQd2KMdWGGcFqEbVqEa1JNmcCbhGucf2ibhGucf2iUe2NHhaiQd2KfinFJm8ajhWubhGucf2iDeW2Qd2KBdmuMclqKbllzYExJm8ajhWubhGucf2iMclqKbllJnctEnMubhGucf2icfWKQd2JChaxJnctEnMujhWubhGuegWWcf2iQd2KjhWubhGuegWWcf2iUe2OMclqKbllIodGjhWubhGuegWWUe2OMclpEpNdBoNOnimujhWuegWWUe2M8iriMclpsWkhDp92ljXOnimujhWulhGSegWWcfWKUe2M4i76VeF2Uc1mMclqOb1NCq+FDp92tjXCnimujhWulhGQyi8WMclpPrdxLrN1Cq+FAquM9quM/qOOvkG87peCtjXCtjGunimuqh2o2n9ujhWurhGSlhGQ2ltKcfWIvktAvjs0yi8Uqi8sticWUc1mTcVRpUkJkUUFardVTrdhPqNSvkG+yj3CtjGutiWenimuqh2qrhGSegWWcfWIyi8UxiL+Uc1m9poq9pIa1nYJgrdNirNBardVqqsezmn2wmX5aqtCVnZWtlXq0k3NapMxTps+sk3a0kW6yj3CvkG+zjmymkXZTositjGuljXNSncOtiWdQm7+nimuqh2qfinFNmcCrhGSchnOjhWtQlrx5jpGlhGSbhGuUhHWegWWmfmGcf2iMgniVgW2ifF2cfWKTfmtIjrVCjLSceluUe2ODfnhAiLeVeF2PemR6enqQd2I7h7qZdFlChayUc1mMdWE6hbZ0eXw6g6+TcVSMclqOb1M6gKaGcFpqdX2KblmMa1OEbVphc4GEa1JecX+EaE4yeaKDZk98aFSDZEwxdJ5RbYF5ZFJ7YkswcJZ1YU9DaoN5XklzYEwubJN0XEkpapM5ZYFzWUNsWkhrV0MzYX8tX35pUkJoUj5kUUEpXH1jTzxgTj9hTDpbSjpRQjZSQjNMPzNLPDFHOS1CODBENyxANCs9NC86MCo3LSgwKSktJycvJyUrJCR/7i4wAAABAHRSTlMAEREREREREREiIiIzMzMzM0REREREREREREREVVVVVVVVZmZmZmZmZnd3d3d3d3eIiIiIiIiImZmZmZmZqqqqqqqqqqqqu7u7u7u7u7u7u7u7u8zMzMzMzMzM3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d7u7u7u7u7u7u7u7u7u7u////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////WBVVlgAAAAlwSFlzAAALEgAACxIB0t1+/AAAABx0RVh0U29mdHdhcmUAQWRvYmUgRmlyZXdvcmtzIENTNAay06AAACAASURBVHic7Z0LYFtl2ce7DS8MxsdgE0VwoOxTUWFsMHCICKKuXJSbOoG0xSGKjG0OAbmo3NwYKDbYnKWkBjsGmtoOEyXIGNRUvrVOOktpCy2jI5hmTbuypDljY03p97z39z3nJM2lJIPmaZqce855f+f/XN5zkpSUHBB20OzZcxacsuCURaWlpYtgYPbsaYXepaKlsumfWHBO6Q9+5LKyH5R+fkah969oZptx0jnf/YUlMcl+fsHcQu9n0bhNnbOg9Ae14zEjtsFz57kfKPQOFw10tmB8mQmr9SC788wiuoLa1LnnZAANDGPzBNpfWHV4ofd98tqMLy7JCBrj9kI0FtP1gYWF3v9JaVPnlv48U2rUTwZjcV2P6/Ho8YU+iMlns865PXNqLpcbc9P1WDyO4A1/vNDHMcns89ZF2viGuLWD2PS9+/fpMdBcoY9kUtmcLFykAOcHXvrIWCIxsgcGvlbog5lEdkrW2LDgQroO3MbGRscSe8BZFvpoJo/NzZ4binB+SEv2AbWxxNjY28CwKLk82fTMqjbFUErZAwEOmCFLjAG43YU+oMlipdlzQ56yfliPv4X8JLb9QK6YWObFDsuBGwLXoseGEwkc4ZC7LPrKfNlXcuCGQtx2Pb4HgCVGsbsc3avrNxX6kCaH3ZEjuKGYvo85SnjZH48VS7l82Cdy4IZzk1h8+O3RUUINnOXIcFz/cKEPajLYolzAbYBiIK7H9icgn0wkcIwbgRq8mJ3kwc7PBRy6mhOP6YgbqQbAYvF4MTvJg2XbR8nAtUH5jZJKym5sbE8xrcyLZd9LScC1AzhwkaOjOMSNjo3u0eM/LfRBTQbLodsEg+vS43Ge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">
 986 | 
 987 |       <img alt="" class="position-absolute" height="49" width="166" style="  top: 297px; left: 371px; z-index: 7;"
 988 |       src="data:image/png;base64,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">
 989 | 
 990 |       <img alt="" class="position-absolute" height="75" width="430" style="top: 263px; left: 442px; z-index: 6;"
 991 |       src="data:image/png;base64,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">
 992 | 
 993 |       <img alt="" class="position-absolute" height="123" width="304" style="top: 73px; left: 467px; z-index: 5;"
 994 |       src="data:image/png;base64,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">
 995 | 
 996 |       <img alt="" class="position-absolute" height="50" width="116" style="top: 113px; left: 762px; z-index: 4;"
 997 |       src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAHQAAAAyCAMAAAC6RQ9kAAAAA3NCSVQICAjb4U/gAAABgFBMVEX////kyZm1pIDexpr73qqnmnallnGnmnn33aqjlXn12anszpi7p4PUvZPu1aXPuo6tm3zr0aLfxZbNt4xxbWOUim6yn3l7c2NsaF2+rIStm3yom4C2o3v22KSznW+rnIGejGuqlm3ZxJZxbWN0bFvUvZPv0JvnzZ2MfmPArYq7p4PXvo7Qu5LFsoq+rIRsaWDx057ErYKznXCsmnKFe2qom4CPgWSMhG3Qu5Kck3OllnGjlXl7c2NzcGeJf217c2OJf23FsoqUim6+rISck3PArYp/eWl/eGWfknrGsIzArYrGsIyck3PNt4zXvo67p4Pr0aLQu5K7p4Omk2zfxZbZxJbErYL12anu1aWwm2753KfmzaHny5aMg3N/eWmfknqajnmUiXSMhG2Mg3OrnIGjlXmUim6UiXSMhG2EeGN/eGW1pICajHOFe2rGsIy7p4OVhme+rIStm3ynmnmejGuUim6VhmfUvZPQu5LNt4zexprPuo7GsIy7p4Oyn3mmk2w+/MV0AAAAgHRSTlMA////////////////////RP///yJ3/yIR7v9E////RHe7/xEiu///RHfu/////xH/////IjNEVaqqzMwRESIzVXeIu8zdIiIzZmZ3d4iZqru7u7vM3d3u7u7///8RESIiIiIiMzMzMzMzM0RERFVVVWZmZmZmZnd3d4iIiIiIiOmar1cAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAAcdEVYdFNvZnR3YXJlAEFkb2JlIEZpcmV3b3JrcyBDUzQGstOgAAAFzUlEQVRYheWXh3saRxDFvSzHHQsczUEKCFVkiINKFEW25RZFcVwjJFvNTu+99+p/PfNmCwfcxYosf8n3ZcB3gjvv797sm9nl1KlHxNyjbngCMf7U/wU6929Al/5L6X3nq43f5ud/uf3ZpScARXp/GB/68q1fD4IHYXP5TC73em731psnDYXS9dMDX729nVZBmkIpVc4hXr188tCLA9CfAiIqftM/1WwTda90otC5IeilbahkoYAG9Ocyiz1J6JDSNw41Kg0wvRTOTVC3HgvzwcbG7ZJrfoNKLx8qTms6IBxT+cPyY1JvHqgmxrhlPpPSq54rm21WyFyXZDzGGVC/OSby8jYP1KRBdq9Y6Dnvgrn8cwACZRc+4rklU8FXIZv4xvGg20qLeEDUPS576kjnfAN9D5cUpxZYfeJsa6m7x2J+CSIkBCFV/ZaFWqVbSrtI6dwq7SjODc9q7liFc2Dmyzw6EtxP71IjDKkdNFUzRDT5tkDBUUobOLd3VND46urqjQZi6mYYlsvl18oIQbE3T1ERUkiEwEngjIOsdrvdgrQx+yzHt6c5xuil3wlbgBJGEGY0Gk84hP4k+idh+PxnPnWkGIuH2tG0AIvQWAsXRqFRiy+OxkyE2oGlyd+wPP1IfeE6wSeh9B/nmKB5sZjJZjNeDGuh6lPUk6BTFiVdIqWFWqR0avsZZ2g1m6FXDLRYoQuZzEQSVESsaU/mi7JyEepTwE0+VRBaaYFGzmYnYqCyhku11PUE9zrvGJ3WREKG6HFK2TVFVyadU1QrZwGVkENDx0AXAa0nKG1EjCP7J6GhKtrfbc8LSKnQ0BryW4+DVmm2s36yewct5GZXitB2HTR2dEDu8RGoWMTQMU4qigIm1fu7khGuTI179JGgQdgut8t4604VmPQyVEp2UiYOWskiUq8kuFdGKlVEfCpIaaAqvud7nufjQEdu9TCSTm8BRopxUlGIOl3K1BJLpl9+ItJ96HOZUlohlIei8/lsoNLMqcwgRieVoHlUUz1Jqe0xwprWaQ9p/maZCJUsNTBGkk9zekUd/vVjoLKKx/ET5nTAvJFlRDA0XdEwpBYH3hxFoIsYetRJRYnM05Vk90a7jssul0w6PWs1cnY9Lhq4l6GCe1J21EmktMKN49MEaH9FkW4N0U8CaMVmVx8HoHQXehLa3YhS1DDFF/Fz6tqsNGDhHMVQdq5v0F6gTMmY9FbYSSM9qYglAdAXE+bUrcvWSq4Xc3p1Xi03MEp5TnFXHe1hxL7kXrmAJLyUlN74zoD0kqxZrdTTZcolo1Jdo1Swk2JWNygt4HFeToQKl1jhVlL8UQ44vWwi0yK4H0bcK6qZOCeRUvQkkjr8M9Ok15VLZLXWSTbu1Rq1XOx3AzunuJF70oiTihisjiXoagK0vzFxD6C1a/faKuUU87LjoLg51kmYU3EWUmOdVLIrdv9g9i5Up0rXqe/Anu1ITinpyY42wiJG7QEa66TSYOeLFKwQ5XSkI+l5xZIe9N1Lt+f1GjYYVYxyHnMa66RGZF9m59WuqrTKaCMZmb5Lr1OKHouxR5XSZh3GjnXSFEtsi3a7uLCwsNi1WnkRV1hlTHK1YGxbVBTKq/WIk4qcrIvwWJyTplgdpqrKeYn2f1ZqSlRn1+efiYBeM0YSet835KQip60HaJyTuGQkVkkNjW7RsHOoGBsZrdit8Mbsmq5TGe+kIruil9STSvwf006pbUnOSEaibQ7YKFml2m55bB6GnFTkx95BocY5SdcpulvV2s71ClunFurbOe0KuW96r+mx2WEoHmcWNZOdmZ5e61xvRaFLq+/Sr8TzFH/82bt7t/cMx+87O/Qzcffw8HBPNyTdgP2+e/eNUikK3O5qE4iaifxziOrziJWVyZVJxPrMw+mvO5+0YjtjUlz56MOPX6B4//NSo/HdnTvf9zY3N3u9/fX19Xv37k9i6JXJ+zMcF6Y5fux01tY6a51WqzXWcrS/AGShj5noGrZMAAAAAElFTkSuQmCC">
 998 |     </div>
 999 |   </div>
1000 | 
1001 | <div class="container-lg tmp-mt-5 tmp-px-3">
1002 |   <!-- '"` --><!-- </textarea></xmp> --></option></form><form role="search" data-turbo="false" action="/search" accept-charset="UTF-8" method="get">
1003 |     <label for="not-found-search" class="d-block text-normal color-fg-muted mb-1 f4">Find code, projects, and people on GitHub:</label>
1004 |     <div class="d-flex flex-items-center">
1005 |       <input type="text" name="q" id="not-found-search" class="flex-auto input-lg form-control mr-2">
1006 |         <button type="submit" data-view-component="true" class="btn">    Search
1007 | </button>
1008 |     </div>
1009 | </form>
1010 |     <div class="tmp-mt-5 color-fg-muted text-center">
1011 |       <a href="https://support.github.com?tags=dotcom-404" class="Link--secondary">Contact Support</a> &mdash;
1012 |       <a href="https://githubstatus.com" class="Link--secondary">GitHub Status</a> &mdash;
1013 |       <a href="https://twitter.com/githubstatus" class="Link--secondary">@githubstatus</a>
1014 |     </div>
1015 | </div>
1016 | 
1017 |   </main>
1018 | 
1019 |   </div>
1020 | 
1021 |             <footer role="contentinfo" class="footer tmp-pt-6 position-relative" data-analytics-visible="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;visible&quot;,&quot;label&quot;:&quot;text: Marketing footer&quot;}" >
1022 |   <h2 class="sr-only">Site-wide Links</h2>
1023 |   <div class="container-xl p-responsive">
1024 |     <div class="d-flex flex-wrap tmp-py-5 tmp-mb-5">
1025 |       <section class="col-12 col-lg-4 tmp-mb-5 tmp-pr-lg-4">
1026 |         <a href="/" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to home&quot;,&quot;label&quot;:&quot;text:home&quot;}" class="color-fg-default d-inline-block" aria-label="Go to GitHub homepage">
1027 |           <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 367.4 90" class="footer-logo-mktg d-block" height="30"><g fill="currentColor"><path d="m46.1 0c-25.5 0-46.1 20.6-46.1 46.1 0 20.4 13.2 37.7 31.5 43.8 2.3.4 3.2-1 3.2-2.2 0-1.1-.1-4.7-.1-8.6-11.6 2.1-14.6-2.8-15.5-5.4-.5-1.3-2.8-5.4-4.7-6.5-1.6-.9-3.9-3-.1-3.1 3.6-.1 6.2 3.3 7.1 4.7 4.2 7 10.8 5 13.4 3.8.4-3 1.6-5 2.9-6.2-10.3-1.2-21-5.1-21-22.8 0-5 1.8-9.2 4.7-12.4-.5-1.2-2.1-5.9.5-12.2 0 0 3.9-1.2 12.7 4.7 3.7-1 7.6-1.6 11.5-1.6s7.8.5 11.5 1.6c8.8-6 12.7-4.7 12.7-4.7 2.5 6.3.9 11.1.5 12.2 2.9 3.2 4.7 7.3 4.7 12.4 0 17.7-10.8 21.6-21.1 22.8 1.7 1.4 3.1 4.2 3.1 8.5 0 6.2-.1 11.1-.1 12.7 0 1.2.9 2.7 3.2 2.2 18.2-6.1 31.4-23.4 31.4-43.8.3-25.4-20.4-46-45.9-46z"></path><path d="m221.6 67.1h-.1zm0 0c-.5 0-1.8.3-3.2.3-4.4 0-5.9-2-5.9-4.6v-17.5h8.9c.5 0 .9-.4.9-1.1v-9.5c0-.5-.4-.9-.9-.9h-8.9v-11.7c0-.4-.3-.7-.8-.7h-12c-.5 0-.8.3-.8.7v12.1s-6.1 1.5-6.5 1.6-.7.5-.7.9v7.6c0 .6.4 1.1.9 1.1h6.2v18.3c0 13.6 9.5 15 16 15 3 0 6.5-.9 7.1-1.2.3-.1.5-.5.5-.9v-8.4c.1-.6-.3-1-.8-1.1zm132.2-12.2c0-10.1-4.1-11.4-8.4-11-3.3.2-6 1.9-6 1.9v19.6s2.7 1.9 6.8 2c5.8.2 7.6-1.9 7.6-12.5zm13.6-.9c0 19.1-6.2 24.6-17 24.6-9.1 0-14.1-4.6-14.1-4.6s-.2 2.6-.5 2.9c-.2.3-.4.4-.8.4h-8.3c-.6 0-1.1-.4-1.1-.9l.1-62c0-.5.4-.9.9-.9h11.9c.5 0 .9.4.9.9l-.1 20.9s4.6-3 11.3-3h.1c6.8-0 16.7 2.5 16.7 21.7zm-48.7-20.2h-11.7c-.6 0-.9.4-.9 1.1v30.3s-3.1 2.2-7.3 2.2-5.4-1.9-5.4-6.1v-26.5c0-.5-.4-.9-.9-.9h-11.9c-.5 0-.9.4-.9.9v28.5c0 12.3 6.9 15.3 16.3 15.3 7.8 0 14.1-4.3 14.1-4.3s.3 2.2.4 2.5.5.5.9.5h7.5c.6 0 .9-.4.9-.9l.1-41.7c-.1-.4-.6-.9-1.2-.9zm-132.2 0h-11.9c-.5 0-.9.5-.9 1.1v40.9c0 1.1.7 1.5 1.7 1.5h10.7c1.1 0 1.4-.5 1.4-1.5v-41.1c0-.5-.5-.9-1-.9zm-5.8-18.9c-4.3 0-7.7 3.4-7.7 7.7s3.4 7.7 7.7 7.7c4.2 0 7.6-3.4 7.6-7.7s-3.4-7.7-7.6-7.7zm92-1.4h-11.8c-.5 0-.9.4-.9.9v22.8h-18.5v-22.7c0-.5-.4-.9-.9-.9h-11.9c-.5 0-.9.4-.9.9v62c0 .5.5.9.9.9h11.9c.5 0 .9-.4.9-.9v-26.6h18.5l-.1 26.5c0 .5.4.9.9.9h11.9c.5 0 .9-.4.9-.9v-62c0-.4-.4-.9-.9-.9zm-105.3 27.5v32c0 .2-.1.6-.3.7 0 0-7 5-18.5 5-13.9 0-30.3-4.4-30.3-33 0-28.7 14.4-34.6 28.4-34.5 12.2 0 17.1 2.7 17.8 3.2.2.3.3.5.3.8l-2.3 9.9c0 .5-.5 1.1-1.1.9-2-.6-5-1.8-12.1-1.8-8.2 0-17 2.3-17 20.8s8.4 20.6 14.4 20.6c5.1 0 7-.6 7-.6v-12.8h-8.2c-.6 0-1.1-.4-1.1-.9v-10.3c0-.5.4-.9 1.1-.9h20.9c.6-.1 1 .4 1 .9z"></path></g></svg>
1028 |         </a>
1029 | 
1030 |         <h3 class="h5 tmp-mt-4 mb-0" id="subscribe-to-newsletter">Subscribe to our developer newsletter</h3>
1031 |         <p class="f5 color-fg-muted tmp-mb-3">Get tips, technical guides, and best practices. Twice a month.</p>
1032 |         <a class="btn-mktg tmp-mb-4 btn-muted-mktg" data-analytics-event="{&quot;category&quot;:&quot;Subscribe&quot;,&quot;action&quot;:&quot;click to Subscribe&quot;,&quot;label&quot;:&quot;ref_cta:Subscribe;&quot;}" href="https://github.com/newsletter">
1033 |   Subscribe
1034 |   
1035 |   
1036 | </a>
1037 | 
1038 |       </section>
1039 | 
1040 |       <nav class="col-6 col-sm-3 col-lg-2 tmp-mb-6 tmp-mb-md-2 tmp-pr-3 tmp-pr-lg-0 tmp-pl-lg-4" aria-labelledby="footer-title-product">
1041 |         <h3 class="h5 tmp-mb-3 text-mono color-fg-muted text-normal" id="footer-title-product">
1042 |           Platform
1043 |         </h3>
1044 | 
1045 |         <ul class="list-style-none color-fg-muted f5">
1046 |           <li class="lh-condensed tmp-mb-3">
1047 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;features&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;features_link_platform_footer&quot;}" href="/features">Features</a>
1048 |           </li>
1049 |           <li class="lh-condensed tmp-mb-3">
1050 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;enterprise&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;enterprise_link_platform_footer&quot;}" href="/enterprise">Enterprise</a>
1051 |           </li>
1052 |           <li class="lh-condensed tmp-mb-3">
1053 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;copilot&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;copilot_link_platform_footer&quot;}" href="/features/copilot">Copilot</a>
1054 |           </li>
1055 |           <li class="lh-condensed tmp-mb-3">
1056 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;ai&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;ai_link_platform_footer&quot;}" href="/features/ai">AI</a>
1057 |           </li>
1058 |           <li class="lh-condensed tmp-mb-3">
1059 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;security&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;security_link_platform_footer&quot;}" href="/security">Security</a>
1060 |           </li>
1061 |           <li class="lh-condensed tmp-mb-3">
1062 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;pricing&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;pricing_link_platform_footer&quot;}" href="/pricing">Pricing</a>
1063 |           </li>
1064 |           <li class="lh-condensed tmp-mb-3">
1065 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;team&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;team_link_platform_footer&quot;}" href="/team">Team</a>
1066 |           </li>
1067 |           <li class="lh-condensed tmp-mb-3">
1068 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;resources&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;resources_link_platform_footer&quot;}" href="https://resources.github.com">Resources</a>
1069 |           </li>
1070 |           <li class="lh-condensed tmp-mb-3">
1071 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;roadmap&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;roadmap_link_platform_footer&quot;}" href="https://github.com/github/roadmap">Roadmap</a>
1072 |           </li>
1073 |           <li class="lh-condensed tmp-mb-3">
1074 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;compare&quot;,&quot;context&quot;:&quot;platform&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;compare_link_platform_footer&quot;}" href="/resources/articles/devops-tools-comparison">Compare GitHub</a>
1075 |           </li>
1076 |         </ul>
1077 |       </nav>
1078 | 
1079 |       <nav class="col-6 col-sm-3 col-lg-2 tmp-mb-6 tmp-mb-md-2 tmp-pr-3 tmp-pr-md-0 tmp-pl-md-4" aria-labelledby="footer-title-platform">
1080 |         <h3 class="h5 tmp-mb-3 text-mono color-fg-muted text-normal" id="footer-title-platform">
1081 |           Ecosystem
1082 |         </h3>
1083 | 
1084 |         <ul class="list-style-none f5">
1085 |           <li class="lh-condensed tmp-mb-3">
1086 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;dev-api&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;dev-api_link_ecosystem_footer&quot;}" href="https://docs.github.com/get-started/exploring-integrations/about-building-integrations">Developer API</a>
1087 |           </li>
1088 |           <li class="lh-condensed tmp-mb-3">
1089 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;partners&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;partners_link_ecosystem_footer&quot;}" href="https://partner.github.com">Partners</a>
1090 |           </li>
1091 |           <li class="lh-condensed tmp-mb-3">
1092 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;edu&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;edu_link_ecosystem_footer&quot;}" href="https://github.com/edu">Education</a>
1093 |           </li>
1094 |           <li class="lh-condensed tmp-mb-3">
1095 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;cli&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;cli_link_ecosystem_footer&quot;}" href="https://cli.github.com">GitHub CLI</a>
1096 |           </li>
1097 |           <li class="lh-condensed tmp-mb-3">
1098 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;desktop&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;desktop_link_ecosystem_footer&quot;}" href="https://desktop.github.com">GitHub Desktop</a>
1099 |           </li>
1100 |           <li class="lh-condensed tmp-mb-3">
1101 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;mobile&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;mobile_link_ecosystem_footer&quot;}" href="https://github.com/mobile">GitHub Mobile</a>
1102 |           </li>
1103 |           <li class="lh-condensed tmp-mb-3">
1104 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;marketplace&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;marketplace_link_ecosystem_footer&quot;}" href="https://github.com/marketplace">GitHub Marketplace</a>
1105 |           </li>
1106 |           <li class="lh-condensed tmp-mb-3">
1107 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;mcp_registry&quot;,&quot;context&quot;:&quot;ecosystem&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;mcp_registry_link_ecosystem_footer&quot;}" href="https://github.com/mcp">MCP Registry</a>
1108 |           </li>
1109 |         </ul>
1110 |       </nav>
1111 | 
1112 |       <nav class="col-6 col-sm-3 col-lg-2 tmp-mb-6 tmp-mb-md-2 tmp-pr-3 tmp-pr-md-0 tmp-pl-md-4" aria-labelledby="footer-title-support">
1113 |         <h3 class="h5 tmp-mb-3 text-mono color-fg-muted text-normal" id="footer-title-support">
1114 |           Support
1115 |         </h3>
1116 | 
1117 |         <ul class="list-style-none f5">
1118 |           <li class="lh-condensed tmp-mb-3">
1119 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;docs&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;docs_link_support_footer&quot;}" href="https://docs.github.com">Docs</a>
1120 |           </li>
1121 | 
1122 |           <li class="lh-condensed tmp-mb-3">
1123 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;community&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;community_link_support_footer&quot;}" href="https://github.community">Community Forum</a>
1124 |           </li>
1125 | 
1126 |           <li class="lh-condensed tmp-mb-3">
1127 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;services&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;services_link_support_footer&quot;}" href="https://services.github.com">Professional Services</a>
1128 |           </li>
1129 | 
1130 |           <li class="lh-condensed tmp-mb-3">
1131 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;premium_support&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;premium_support_link_support_footer&quot;}" href="/enterprise/premium-support">Premium Support</a>
1132 |           </li>
1133 | 
1134 |           <li class="lh-condensed tmp-mb-3">
1135 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;skills&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;skills_link_support_footer&quot;}" href="https://skills.github.com">Skills</a>
1136 |           </li>
1137 | 
1138 |           <li class="lh-condensed tmp-mb-3">
1139 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;status&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;status_link_support_footer&quot;}" href="https://www.githubstatus.com">Status</a>
1140 |           </li>
1141 | 
1142 |           <li class="lh-condensed tmp-mb-3">
1143 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;contact_github&quot;,&quot;context&quot;:&quot;support&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;contact_github_link_support_footer&quot;}" href="https://support.github.com?tags=dotcom-footer">Contact GitHub</a>
1144 |           </li>
1145 |         </ul>
1146 |       </nav>
1147 | 
1148 |       <nav class="col-6 col-sm-3 col-lg-2 tmp-mb-6 tmp-mb-md-2 tmp-pr-3 tmp-pr-md-0 tmp-pl-md-4" aria-labelledby="footer-title-company">
1149 |         <h3 class="h5 tmp-mb-3 text-mono color-fg-muted text-normal" id="footer-title-company">
1150 |           Company
1151 |         </h3>
1152 | 
1153 |         <ul class="list-style-none f5">
1154 |           <li class="lh-condensed tmp-mb-3">
1155 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;about&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;about_link_company_footer&quot;}" href="https://github.com/about">About</a>
1156 |           </li>
1157 |           <li class="lh-condensed tmp-mb-3">
1158 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;why_github&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;why_github_link_company_footer&quot;}" href="https://github.com/why-github">Why GitHub</a>
1159 |           </li>
1160 |           <li class="lh-condensed tmp-mb-3">
1161 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;customer_stories&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;customer_stories_link_company_footer&quot;}" href="/customer-stories?type=enterprise">Customer stories</a>
1162 |           </li>
1163 |           <li class="lh-condensed tmp-mb-3">
1164 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;blog&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;blog_link_company_footer&quot;}" href="https://github.blog">Blog</a>
1165 |           </li>
1166 |           <li class="lh-condensed tmp-mb-3">
1167 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;readme&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;readme_link_company_footer&quot;}" href="/readme">The ReadME Project</a>
1168 |           </li>
1169 |           <li class="lh-condensed tmp-mb-3">
1170 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;careers&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;careers_link_company_footer&quot;}" href="https://github.careers">Careers</a>
1171 |           </li>
1172 |           <li class="lh-condensed tmp-mb-3">
1173 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;newsroom&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;newsroom_link_company_footer&quot;}" href="/newsroom">Newsroom</a>
1174 |           </li>
1175 |           <li class="lh-condensed tmp-mb-3">
1176 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;inclusion&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;inclusion_link_company_footer&quot;}" href="/about/diversity">Inclusion</a>
1177 |           </li>
1178 |           <li class="lh-condensed tmp-mb-3">
1179 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;social_impact&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;social_impact_link_company_footer&quot;}" href="https://socialimpact.github.com">Social Impact</a>
1180 |           </li>
1181 |           <li class="lh-condensed tmp-mb-3">
1182 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;shop&quot;,&quot;context&quot;:&quot;company&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;shop_link_company_footer&quot;}" href="https://shop.github.com">Shop</a>
1183 |           </li>
1184 |         </ul>
1185 |       </nav>
1186 |     </div>
1187 |   </div>
1188 | 
1189 |   <div class="color-bg-subtle">
1190 |     <div class="container-xl p-responsive f6 tmp-py-4 d-md-flex flex-justify-between flex-items-center gap-3">
1191 |       <nav aria-label="Legal and Resource Links">
1192 |         <ul class="list-style-none d-flex flex-wrap color-fg-muted gapx-3">
1193 |           <li>
1194 |             &copy; <time datetime="2026">2026</time> GitHub, Inc.
1195 |           </li>
1196 | 
1197 |           <li>
1198 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;terms&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;terms_link_subfooter_footer&quot;}" href="https://docs.github.com/site-policy/github-terms/github-terms-of-service">Terms</a>
1199 |           </li>
1200 | 
1201 |           <li>
1202 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;privacy&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;privacy_link_subfooter_footer&quot;}" href="https://docs.github.com/site-policy/privacy-policies/github-privacy-statement">Privacy</a>
1203 |             <a href="https://github.com/github/site-policy/pull/582" class="Link--secondary">(Updated 02/2024)<time datetime="2024-02" class="sr-only">02/2024</time></a>
1204 |           </li>
1205 | 
1206 |           <li>
1207 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;sitemap&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;sitemap_link_subfooter_footer&quot;}" href="/sitemap">Sitemap</a>
1208 |           </li>
1209 | 
1210 |           <li>
1211 |             <a class="Link--secondary" data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;what_is_git&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;what_is_git_link_subfooter_footer&quot;}" href="/git-guides">What is Git?</a>
1212 |           </li>
1213 | 
1214 |             <li >
1215 |   <cookie-consent-link>
1216 |     <button
1217 |       type="button"
1218 |       class="Link--secondary underline-on-hover border-0 p-0 color-bg-transparent"
1219 |       data-action="click:cookie-consent-link#showConsentManagement"
1220 |       data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;cookies&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;cookies_link_subfooter_footer&quot;}"
1221 |     >
1222 |        Manage cookies
1223 |     </button>
1224 |   </cookie-consent-link>
1225 | </li>
1226 | 
1227 | <li>
1228 |   <cookie-consent-link>
1229 |     <button
1230 |       type="button"
1231 |       class="Link--secondary underline-on-hover border-0 p-0 color-bg-transparent text-left"
1232 |       data-action="click:cookie-consent-link#showConsentManagement"
1233 |       data-analytics-event="{&quot;location&quot;:&quot;footer&quot;,&quot;action&quot;:&quot;dont_share_info&quot;,&quot;context&quot;:&quot;subfooter&quot;,&quot;tag&quot;:&quot;link&quot;,&quot;label&quot;:&quot;dont_share_info_link_subfooter_footer&quot;}"
1234 |     >
1235 |       Do not share my personal information
1236 |     </button>
1237 |   </cookie-consent-link>
1238 | </li>
1239 | 
1240 |         </ul>
1241 |       </nav>
1242 | 
1243 |       <nav aria-label="GitHub&#39;s Social Media Links" class="footer-social tmp-mt-3 tmp-mt-md-0 d-flex gapx-6 gapy-1 flex-wrap flex-items-center flex-lg-justify-end">
1244 |         
1245 | <ul class="list-style-none d-flex flex-items-center lh-condensed-ultra gap-3">
1246 |     <li>
1247 |       <a href="https://www.linkedin.com/company/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to Linkedin&quot;,&quot;label&quot;:&quot;text:linkedin&quot;}">
1248 |         <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 19 18" aria-hidden="true" class="d-block" width="19" height="18"><path d="M3.94 2A2 2 0 1 1 2 0a2 2 0 0 1 1.94 2zM4 5.48H0V18h4zm6.32 0H6.34V18h3.94v-6.57c0-3.66 4.77-4 4.77 0V18H19v-7.93c0-6.17-7.06-5.94-8.72-2.91z" fill="currentColor"></path></svg>
1249 |         <span class="sr-only">GitHub on LinkedIn</span>
1250 |       </a>
1251 |     </li>
1252 |     <li>
1253 |       <a href="https://www.instagram.com/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to Instagram&quot;,&quot;label&quot;:&quot;text:instagram&quot;}">
1254 |         <svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 24 24" aria-hidden="true" class="d-block" width="18" height="18"><title>Instagram</title><path d="M12 0C8.74 0 8.333.015 7.053.072 5.775.132 4.905.333 4.14.63c-.789.306-1.459.717-2.126 1.384S.935 3.35.63 4.14C.333 4.905.131 5.775.072 7.053.012 8.333 0 8.74 0 12s.015 3.667.072 4.947c.06 1.277.261 2.148.558 2.913.306.788.717 1.459 1.384 2.126.667.666 1.336 1.079 2.126 1.384.766.296 1.636.499 2.913.558C8.333 23.988 8.74 24 12 24s3.667-.015 4.947-.072c1.277-.06 2.148-.262 2.913-.558.788-.306 1.459-.718 2.126-1.384.666-.667 1.079-1.335 1.384-2.126.296-.765.499-1.636.558-2.913.06-1.28.072-1.687.072-4.947s-.015-3.667-.072-4.947c-.06-1.277-.262-2.149-.558-2.913-.306-.789-.718-1.459-1.384-2.126C21.319 1.347 20.651.935 19.86.63c-.765-.297-1.636-.499-2.913-.558C15.667.012 15.26 0 12 0zm0 2.16c3.203 0 3.585.016 4.85.071 1.17.055 1.805.249 2.227.415.562.217.96.477 1.382.896.419.42.679.819.896 1.381.164.422.36 1.057.413 2.227.057 1.266.07 1.646.07 4.85s-.015 3.585-.074 4.85c-.061 1.17-.256 1.805-.421 2.227-.224.562-.479.96-.899 1.382-.419.419-.824.679-1.38.896-.42.164-1.065.36-2.235.413-1.274.057-1.649.07-4.859.07-3.211 0-3.586-.015-4.859-.074-1.171-.061-1.816-.256-2.236-.421-.569-.224-.96-.479-1.379-.899-.421-.419-.69-.824-.9-1.38-.165-.42-.359-1.065-.42-2.235-.045-1.26-.061-1.649-.061-4.844 0-3.196.016-3.586.061-4.861.061-1.17.255-1.814.42-2.234.21-.57.479-.96.9-1.381.419-.419.81-.689 1.379-.898.42-.166 1.051-.361 2.221-.421 1.275-.045 1.65-.06 4.859-.06l.045.03zm0 3.678c-3.405 0-6.162 2.76-6.162 6.162 0 3.405 2.76 6.162 6.162 6.162 3.405 0 6.162-2.76 6.162-6.162 0-3.405-2.76-6.162-6.162-6.162zM12 16c-2.21 0-4-1.79-4-4s1.79-4 4-4 4 1.79 4 4-1.79 4-4 4zm7.846-10.405c0 .795-.646 1.44-1.44 1.44-.795 0-1.44-.646-1.44-1.44 0-.794.646-1.439 1.44-1.439.793-.001 1.44.645 1.44 1.439z" fill="currentColor"></path></svg>
1255 |         <span class="sr-only">GitHub on Instagram</span>
1256 |       </a>
1257 |     </li>
1258 |     <li>
1259 |       <a href="https://www.youtube.com/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to YouTube&quot;,&quot;label&quot;:&quot;text:youtube&quot;}">
1260 |         <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 19.17 13.6" aria-hidden="true" class="d-block" width="23" height="16"><path d="M18.77 2.13A2.4 2.4 0 0 0 17.09.42C15.59 0 9.58 0 9.58 0a57.55 57.55 0 0 0-7.5.4A2.49 2.49 0 0 0 .39 2.13 26.27 26.27 0 0 0 0 6.8a26.15 26.15 0 0 0 .39 4.67 2.43 2.43 0 0 0 1.69 1.71c1.52.42 7.5.42 7.5.42a57.69 57.69 0 0 0 7.51-.4 2.4 2.4 0 0 0 1.68-1.71 25.63 25.63 0 0 0 .4-4.67 24 24 0 0 0-.4-4.69zM7.67 9.71V3.89l5 2.91z" fill="currentColor"></path></svg>
1261 |         <span class="sr-only">GitHub on YouTube</span>
1262 |       </a>
1263 |     </li>
1264 |     <li>
1265 |       <a href="https://x.com/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to X&quot;,&quot;label&quot;:&quot;text:x&quot;}">
1266 |         <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 1227" fill="currentColor" aria-hidden="true" class="d-block" width="16" height="16"><path d="M714.163 519.284 1160.89 0h-105.86L667.137 450.887 357.328 0H0l468.492 681.821L0 1226.37h105.866l409.625-476.152 327.181 476.152H1200L714.137 519.284h.026ZM569.165 687.828l-47.468-67.894-377.686-540.24h162.604l304.797 435.991 47.468 67.894 396.2 566.721H892.476L569.165 687.854v-.026Z"></path></svg>
1267 |         <span class="sr-only">GitHub on X</span>
1268 |       </a>
1269 |     </li>
1270 |     <li>
1271 |       <a href="https://www.tiktok.com/@github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to tiktok&quot;,&quot;label&quot;:&quot;text:tiktok&quot;}">
1272 |         <svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 24 24" aria-hidden="true" class="d-block" width="18" height="18"><title>TikTok</title><path d="M12.525.02c1.31-.02 2.61-.01 3.91-.02.08 1.53.63 3.09 1.75 4.17 1.12 1.11 2.7 1.62 4.24 1.79v4.03c-1.44-.05-2.89-.35-4.2-.97-.57-.26-1.1-.59-1.62-.93-.01 2.92.01 5.84-.02 8.75-.08 1.4-.54 2.79-1.35 3.94-1.31 1.92-3.58 3.17-5.91 3.21-1.43.08-2.86-.31-4.08-1.03-2.02-1.19-3.44-3.37-3.65-5.71-.02-.5-.03-1-.01-1.49.18-1.9 1.12-3.72 2.58-4.96 1.66-1.44 3.98-2.13 6.15-1.72.02 1.48-.04 2.96-.04 4.44-.99-.32-2.15-.23-3.02.37-.63.41-1.11 1.04-1.36 1.75-.21.51-.15 1.07-.14 1.61.24 1.64 1.82 3.02 3.5 2.87 1.12-.01 2.19-.66 2.77-1.61.19-.33.4-.67.41-1.06.1-1.79.06-3.57.07-5.36.01-4.03-.01-8.05.02-12.07z" fill="currentColor"></path></svg>
1273 |         <span class="sr-only">GitHub on TikTok</span>
1274 |       </a>
1275 |     </li>
1276 |     <li>
1277 |       <a href="https://www.twitch.tv/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to Twitch&quot;,&quot;label&quot;:&quot;text:twitch&quot;}">
1278 |         <svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 24 24" aria-hidden="true" class="d-block" width="18" height="18"><title>Twitch</title><path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z" fill="currentColor"></path></svg>
1279 |         <span class="sr-only">GitHub on Twitch</span>
1280 |       </a>
1281 |     </li>
1282 |     <li>
1283 |       <a href="https://github.com/github" class="footer-social-icon d-block Link--outlineOffset" data-analytics-event="{&quot;category&quot;:&quot;Footer&quot;,&quot;action&quot;:&quot;go to github&#39;s org&quot;,&quot;label&quot;:&quot;text:github&quot;}">
1284 |         <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 0 16 16" width="20" aria-hidden="true" class="d-block"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path></svg>
1285 |         <span class="sr-only">GitHub’s organization on GitHub</span>
1286 |       </a>
1287 |     </li>
1288 | </ul>
1289 | 
1290 |         
1291 | 
1292 |   <locale-selector>
1293 |     <experimental-action-menu data-anchor-align="start" data-anchor-side="outside-bottom" data-view-component="true" class="footer-social-icon">
1294 |   <button role="button" aria-haspopup="true" aria-controls="locale-selector-list" aria-expanded="false" id="locale-selector-text" aria-label="English - Select language" type="button" data-view-component="true" class="d-flex flex-items-center border-0 color-bg-transparent p-0 locale-trigger Button--secondary Button--medium Button">  <span class="Button-content">
1295 |     <span class="Button-label"><span class="locale-selector-trigger">
1296 |           <svg height="16" aria-hidden="true" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-globe mr-1 color-fg-muted locale-icon">
1297 |     <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM5.78 8.75a9.64 9.64 0 0 0 1.363 4.177c.255.426.542.832.857 1.215.245-.296.551-.705.857-1.215A9.64 9.64 0 0 0 10.22 8.75Zm4.44-1.5a9.64 9.64 0 0 0-1.363-4.177c-.307-.51-.612-.919-.857-1.215a9.927 9.927 0 0 0-.857 1.215A9.64 9.64 0 0 0 5.78 7.25Zm-5.944 1.5H1.543a6.507 6.507 0 0 0 4.666 5.5c-.123-.181-.24-.365-.352-.552-.715-1.192-1.437-2.874-1.581-4.948Zm-2.733-1.5h2.733c.144-2.074.866-3.756 1.58-4.948.12-.197.237-.381.353-.552a6.507 6.507 0 0 0-4.666 5.5Zm10.181 1.5c-.144 2.074-.866 3.756-1.58 4.948-.12.197-.237.381-.353.552a6.507 6.507 0 0 0 4.666-5.5Zm2.733-1.5a6.507 6.507 0 0 0-4.666-5.5c.123.181.24.365.353.552.714 1.192 1.436 2.874 1.58 4.948Z"></path>
1298 | </svg>
1299 |           <span class="color-fg-muted Link--secondary f6">English</span>
1300 |           <svg height="16" aria-hidden="true" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-chevron-down ml-1 color-fg-muted locale-icon">
1301 |     <path d="M12.78 5.22a.749.749 0 0 1 0 1.06l-4.25 4.25a.749.749 0 0 1-1.06 0L3.22 6.28a.749.749 0 1 1 1.06-1.06L8 8.939l3.72-3.719a.749.749 0 0 1 1.06 0Z"></path>
1302 | </svg>
1303 |         </span></span>
1304 |   </span>
1305 | </button>
1306 | 
1307 |   <div class="Overlay-backdrop--anchor" data-menu-overlay>
1308 |     <div class="Overlay Overlay-whenNarrow Overlay--height-auto Overlay--width-auto" hidden>
1309 |       <div class="Overlay-body Overlay-body--paddingNone">
1310 |         <ul class="ActionList" id="locale-selector-list" role="menu" aria-labelledby="locale-selector-text">
1311 |                 <li role="none" data-view-component="true" class="ActionList-item">
1312 |     <a href="" selected="selected" style="white-space: normal;" data-action="click:locale-selector#handleSelectLocale" data-locale="en-us" role="menuitem" tabindex="-1" data-view-component="true" class="footer-social-locale ActionList-content">
1313 |       
1314 |       <span class="ActionList-item-label">
1315 |                   <div style="width: 16px; display: inline-block; text-align: center; margin-right: 8px; flex-shrink: 0;">
1316 |             <svg height="16" aria-hidden="true" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-check">
1317 |     <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"></path>
1318 | </svg>
1319 |           </div>
1320 |           <span>English</span>
1321 | 
1322 |       </span>
1323 | </a></li>
1324 |                 <li role="none" data-view-component="true" class="ActionList-item">
1325 |     <a href="" style="white-space: normal;" data-action="click:locale-selector#handleSelectLocale" data-locale="pt-br" role="menuitem" tabindex="-1" data-view-component="true" class="footer-social-locale ActionList-content">
1326 |       
1327 |       <span class="ActionList-item-label">
1328 |                   <div style="width: 16px; display: inline-block; text-align: center; margin-right: 8px; flex-shrink: 0;">
1329 |             
1330 |           </div>
1331 |           <span>Português (Brasil)</span>
1332 | 
1333 |       </span>
1334 | </a></li>
1335 |                 <li role="none" data-view-component="true" class="ActionList-item">
1336 |     <a href="" style="white-space: normal;" data-action="click:locale-selector#handleSelectLocale" data-locale="es-419" role="menuitem" tabindex="-1" data-view-component="true" class="footer-social-locale ActionList-content">
1337 |       
1338 |       <span class="ActionList-item-label">
1339 |                   <div style="width: 16px; display: inline-block; text-align: center; margin-right: 8px; flex-shrink: 0;">
1340 |             
1341 |           </div>
1342 |           <span>Español (América Latina)</span>
1343 | 
1344 |       </span>
1345 | </a></li>
1346 |                 <li role="none" data-view-component="true" class="ActionList-item">
1347 |     <a href="" style="white-space: normal;" data-action="click:locale-selector#handleSelectLocale" data-locale="ja" role="menuitem" tabindex="-1" data-view-component="true" class="footer-social-locale ActionList-content">
1348 |       
1349 |       <span class="ActionList-item-label">
1350 |                   <div style="width: 16px; display: inline-block; text-align: center; margin-right: 8px; flex-shrink: 0;">
1351 |             
1352 |           </div>
1353 |           <span>日本語</span>
1354 | 
1355 |       </span>
1356 | </a></li>
1357 |                 <li role="none" data-view-component="true" class="ActionList-item">
1358 |     <a href="" style="white-space: normal;" data-action="click:locale-selector#handleSelectLocale" data-locale="ko-kr" role="menuitem" tabindex="-1" data-view-component="true" class="footer-social-locale ActionList-content">
1359 |       
1360 |       <span class="ActionList-item-label">
1361 |                   <div style="width: 16px; display: inline-block; text-align: center; margin-right: 8px; flex-shrink: 0;">
1362 |             
1363 |           </div>
1364 |           <span>한국어</span>
1365 | 
1366 |       </span>
1367 | </a></li>
1368 |         </ul>
1369 |       </div>
1370 |     </div>
1371 |   </div>
1372 | </experimental-action-menu>  </locale-selector>
1373 | 
1374 |       </nav>
1375 |     </div>
1376 |   </div>
1377 | </footer>
1378 | 
1379 | 
1380 | 
1381 |     <ghcc-consent id="ghcc" class="position-fixed bottom-0 left-0" style="z-index: 999999"
1382 |       data-locale="en"
1383 |       data-initial-cookie-consent-allowed=""
1384 |       data-cookie-consent-required="false"
1385 |     ></ghcc-consent>
1386 | 
1387 | 
1388 | 
1389 | 
1390 |   <div id="ajax-error-message" class="ajax-error-message flash flash-error" hidden>
1391 |     <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-alert">
1392 |     <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path>
1393 | </svg>
1394 |     <button type="button" class="flash-close js-ajax-error-dismiss" aria-label="Dismiss error">
1395 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x">
1396 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
1397 | </svg>
1398 |     </button>
1399 |     You can’t perform that action at this time.
1400 |   </div>
1401 | 
1402 |     <template id="site-details-dialog">
1403 |   <details class="details-reset details-overlay details-overlay-dark lh-default color-fg-default hx_rsm" open>
1404 |     <summary role="button" aria-label="Close dialog"></summary>
1405 |     <details-dialog class="Box Box--overlay d-flex flex-column anim-fade-in fast hx_rsm-dialog hx_rsm-modal">
1406 |       <button class="Box-btn-octicon m-0 btn-octicon position-absolute right-0 top-0" type="button" aria-label="Close dialog" data-close-dialog>
1407 |         <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-x">
1408 |     <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"></path>
1409 | </svg>
1410 |       </button>
1411 |       <div class="octocat-spinner tmp-my-6 js-details-dialog-spinner"></div>
1412 |     </details-dialog>
1413 |   </details>
1414 | </template>
1415 | 
1416 |     <div class="Popover js-hovercard-content position-absolute" style="display: none; outline: none;">
1417 |   <div class="Popover-message Popover-message--bottom-left Popover-message--large Box color-shadow-large" style="width:360px;">
1418 |   </div>
1419 | </div>
1420 | 
1421 |     <template id="snippet-clipboard-copy-button">
1422 |   <div class="zeroclipboard-container position-absolute right-0 top-0">
1423 |     <clipboard-copy aria-label="Copy" class="ClipboardButton btn js-clipboard-copy m-2 p-0" data-copy-feedback="Copied!" data-tooltip-direction="w">
1424 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-copy js-clipboard-copy-icon m-2">
1425 |     <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"></path><path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"></path>
1426 | </svg>
1427 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-check js-clipboard-check-icon color-fg-success d-none m-2">
1428 |     <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"></path>
1429 | </svg>
1430 |     </clipboard-copy>
1431 |   </div>
1432 | </template>
1433 | <template id="snippet-clipboard-copy-button-unpositioned">
1434 |   <div class="zeroclipboard-container">
1435 |     <clipboard-copy aria-label="Copy" class="ClipboardButton btn btn-invisible js-clipboard-copy m-2 p-0 d-flex flex-justify-center flex-items-center" data-copy-feedback="Copied!" data-tooltip-direction="w">
1436 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-copy js-clipboard-copy-icon">
1437 |     <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"></path><path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"></path>
1438 | </svg>
1439 |       <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" data-view-component="true" class="octicon octicon-check js-clipboard-check-icon color-fg-success d-none">
1440 |     <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"></path>
1441 | </svg>
1442 |     </clipboard-copy>
1443 |   </div>
1444 | </template>
1445 | 
1446 | 
1447 | 
1448 | 
1449 |     </div>
1450 |     <div id="js-global-screen-reader-notice" class="sr-only mt-n1" aria-live="polite" aria-atomic="true" ></div>
1451 |     <div id="js-global-screen-reader-notice-assertive" class="sr-only mt-n1" aria-live="assertive" aria-atomic="true"></div>
1452 |   </body>
1453 | </html>
1454 | 
```

### File: `assets\icons\icon.png`

- Size: 189831 bytes
- Modified: 2026-03-02 16:05:01 UTC

```text
<Binary file or unsupported encoding: 189831 bytes>
```

### File: `help.txt`

- Size: 1813 bytes
- Modified: 2026-03-07 04:47:06 UTC

```txt
   1 | CLI tool to aggregate directory contents into a single markdown file optimized for LLM consumption
   2 | 
   3 | Usage: context-builder.exe [OPTIONS]
   4 | 
   5 | Options:
   6 |   -d, --input <INPUT>            Directory path to process [default: .]
   7 |   -o, --output <OUTPUT>          Output file path [default: output.md]
   8 |   -f, --filter <FILTER>          File extensions to include (e.g., --filter rs,toml)
   9 |   -i, --ignore <IGNORE>          Folder or file names to ignore (e.g., --ignore target --ignore lock)
  10 |       --preview                  Preview mode: only print the file tree to the console, don't generate the documentation file
  11 |       --token-count              Token count mode: estimate the total token count of the final document
  12 |       --line-numbers             Add line numbers to code blocks in the output
  13 |   -y, --yes                      Automatically answer yes to all prompts
  14 |       --max-tokens <MAX_TOKENS>  Maximum token budget for the output. Files are truncated/skipped when exceeded
  15 |       --diff-only                Output only diffs (omit full file contents; requires auto-diff & timestamped output)
  16 |       --clear-cache              Clear the cached project state and exit
  17 |       --init                     Initialize a new context-builder.toml config file in the current directory
  18 |       --signatures               Extract function/class signatures only (requires tree-sitter feature)
  19 |       --structure                Extract code structure (imports, exports, symbol counts) - requires tree-sitter feature
  20 |       --truncate <MODE>          Truncation mode for max-tokens: "smart" (AST boundaries) or "byte" [default: smart]
  21 |       --visibility <VISIBILITY>  Filter signatures by visibility: "all", "public", or "private" [default: all]
  22 |   -h, --help                     Print help
  23 |   -V, --version                  Print version
```

### File: `log.txt`

- Size: 2565 bytes
- Modified: 2026-03-07 12:19:16 UTC

```txt
   1 |     Checking spedimage v2.0.0 (E:\SpedImage)
   2 | error: couldn't read `src\app\../assets/icons/icon.png`: The system cannot find the path specified. (os error 3)
   3 |   --> src\app\types.rs:11:29
   4 |    |
   5 | 11 | pub const APP_ICON: &[u8] = include_bytes!("../assets/icons/icon.png");
   6 |    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   7 |    |
   8 | help: there is a file with the same name in a different directory
   9 |    |
  10 | 11 | pub const APP_ICON: &[u8] = include_bytes!("../../assets/icons/icon.png");
  11 |    |                                                +++
  12 | 
  13 | warning: unused imports: `APP_ICON` and `WakeUp`
  14 |  --> src\app\actions.rs:2:47
  15 |   |
  16 | 2 | use crate::app::types::{send_event, AppEvent, WakeUp, APP_ICON};
  17 |   |                                               ^^^^^^  ^^^^^^^^
  18 |   |
  19 |   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
  20 | 
  21 | warning: unused import: `std::sync::Arc`
  22 |  --> src\app\actions.rs:7:5
  23 |   |
  24 | 7 | use std::sync::Arc;
  25 |   |     ^^^^^^^^^^^^^^
  26 | 
  27 | warning: unused import: `ElementState`
  28 |  --> src\app\actions.rs:9:20
  29 |   |
  30 | 9 | use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
  31 |   |                    ^^^^^^^^^^^^
  32 | 
  33 | warning: unused imports: `Icon` and `Window`
  34 |   --> src\app\actions.rs:13:33
  35 |    |
  36 | 13 | use winit::window::{Fullscreen, Icon, Window};
  37 |    |                                 ^^^^  ^^^^^^
  38 | 
  39 | warning: unused import: `crate::image_backend::ImageData`
  40 |  --> src\app\events.rs:4:5
  41 |   |
  42 | 4 | use crate::image_backend::ImageData;
  43 |   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  44 | 
  45 | warning: unused import: `Path`
  46 |  --> src\app\events.rs:6:17
  47 |   |
  48 | 6 | use std::path::{Path, PathBuf};
  49 |   |                 ^^^^
  50 | 
  51 | warning: unused import: `winit::dpi::PhysicalPosition`
  52 |  --> src\app\events.rs:9:5
  53 |   |
  54 | 9 | use winit::dpi::PhysicalPosition;
  55 |   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  56 | 
  57 | warning: unused import: `Fullscreen`
  58 |   --> src\app\events.rs:12:21
  59 |    |
  60 | 12 | use winit::window::{Fullscreen, Icon, Window, WindowId};
  61 |    |                     ^^^^^^^^^^
  62 | 
  63 | warning: unused import: `std::os::windows::ffi::OsStrExt`
  64 |   --> src\app\actions.rs:16:5
  65 |    |
  66 | 16 | use std::os::windows::ffi::OsStrExt;
  67 |    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  68 | 
  69 | warning: unused import: `winit::raw_window_handle::HasWindowHandle`
  70 |   --> src\app\actions.rs:12:5
  71 |    |
  72 | 12 | use winit::raw_window_handle::HasWindowHandle;
  73 |    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  74 | 
  75 | warning: `spedimage` (lib) generated 10 warnings
  76 | error: could not compile `spedimage` (lib) due to 1 previous error; 10 warnings emitted
```

### File: `Cargo.lock`

- Size: 130102 bytes
- Modified: 2026-03-07 05:20:16 UTC

```toml
   1 | # This file is automatically @generated by Cargo.
   2 | # It is not intended for manual editing.
   3 | version = 3
   4 | 
   5 | [[package]]
   6 | name = "ab_glyph"
   7 | version = "0.2.32"
   8 | source = "registry+https://github.com/rust-lang/crates.io-index"
   9 | checksum = "01c0457472c38ea5bd1c3b5ada5e368271cb550be7a4ca4a0b4634e9913f6cc2"
  10 | dependencies = [
  11 |  "ab_glyph_rasterizer",
  12 |  "owned_ttf_parser",
  13 | ]
  14 | 
  15 | [[package]]
  16 | name = "ab_glyph_rasterizer"
  17 | version = "0.1.10"
  18 | source = "registry+https://github.com/rust-lang/crates.io-index"
  19 | checksum = "366ffbaa4442f4684d91e2cd7c5ea7c4ed8add41959a31447066e279e432b618"
  20 | 
  21 | [[package]]
  22 | name = "addr2line"
  23 | version = "0.25.1"
  24 | source = "registry+https://github.com/rust-lang/crates.io-index"
  25 | checksum = "1b5d307320b3181d6d7954e663bd7c774a838b8220fe0593c86d9fb09f498b4b"
  26 | dependencies = [
  27 |  "gimli",
  28 | ]
  29 | 
  30 | [[package]]
  31 | name = "adler2"
  32 | version = "2.0.1"
  33 | source = "registry+https://github.com/rust-lang/crates.io-index"
  34 | checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"
  35 | 
  36 | [[package]]
  37 | name = "adler32"
  38 | version = "1.2.0"
  39 | source = "registry+https://github.com/rust-lang/crates.io-index"
  40 | checksum = "aae1277d39aeec15cb388266ecc24b11c80469deae6067e17a1a7aa9e5c1f234"
  41 | 
  42 | [[package]]
  43 | name = "ahash"
  44 | version = "0.8.12"
  45 | source = "registry+https://github.com/rust-lang/crates.io-index"
  46 | checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
  47 | dependencies = [
  48 |  "cfg-if",
  49 |  "getrandom 0.3.4",
  50 |  "once_cell",
  51 |  "version_check",
  52 |  "zerocopy",
  53 | ]
  54 | 
  55 | [[package]]
  56 | name = "aho-corasick"
  57 | version = "1.1.4"
  58 | source = "registry+https://github.com/rust-lang/crates.io-index"
  59 | checksum = "ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301"
  60 | dependencies = [
  61 |  "memchr",
  62 | ]
  63 | 
  64 | [[package]]
  65 | name = "alloc-no-stdlib"
  66 | version = "2.0.4"
  67 | source = "registry+https://github.com/rust-lang/crates.io-index"
  68 | checksum = "cc7bb162ec39d46ab1ca8c77bf72e890535becd1751bb45f64c597edb4c8c6b3"
  69 | 
  70 | [[package]]
  71 | name = "alloc-stdlib"
  72 | version = "0.2.2"
  73 | source = "registry+https://github.com/rust-lang/crates.io-index"
  74 | checksum = "94fb8275041c72129eb51b7d0322c29b8387a0386127718b096429201a5d6ece"
  75 | dependencies = [
  76 |  "alloc-no-stdlib",
  77 | ]
  78 | 
  79 | [[package]]
  80 | name = "allocator-api2"
  81 | version = "0.2.21"
  82 | source = "registry+https://github.com/rust-lang/crates.io-index"
  83 | checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"
  84 | 
  85 | [[package]]
  86 | name = "android-activity"
  87 | version = "0.6.0"
  88 | source = "registry+https://github.com/rust-lang/crates.io-index"
  89 | checksum = "ef6978589202a00cd7e118380c448a08b6ed394c3a8df3a430d0898e3a42d046"
  90 | dependencies = [
  91 |  "android-properties",
  92 |  "bitflags 2.11.0",
  93 |  "cc",
  94 |  "cesu8",
  95 |  "jni",
  96 |  "jni-sys",
  97 |  "libc",
  98 |  "log",
  99 |  "ndk",
 100 |  "ndk-context",
 101 |  "ndk-sys",
 102 |  "num_enum",
 103 |  "thiserror 1.0.69",
 104 | ]
 105 | 
 106 | [[package]]
 107 | name = "android-properties"
 108 | version = "0.2.2"
 109 | source = "registry+https://github.com/rust-lang/crates.io-index"
 110 | checksum = "fc7eb209b1518d6bb87b283c20095f5228ecda460da70b44f0802523dea6da04"
 111 | 
 112 | [[package]]
 113 | name = "android_system_properties"
 114 | version = "0.1.5"
 115 | source = "registry+https://github.com/rust-lang/crates.io-index"
 116 | checksum = "819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311"
 117 | dependencies = [
 118 |  "libc",
 119 | ]
 120 | 
 121 | [[package]]
 122 | name = "anyhow"
 123 | version = "1.0.102"
 124 | source = "registry+https://github.com/rust-lang/crates.io-index"
 125 | checksum = "7f202df86484c868dbad7eaa557ef785d5c66295e41b460ef922eca0723b842c"
 126 | 
 127 | [[package]]
 128 | name = "approx"
 129 | version = "0.5.1"
 130 | source = "registry+https://github.com/rust-lang/crates.io-index"
 131 | checksum = "cab112f0a86d568ea0e627cc1d6be74a1e9cd55214684db5561995f6dad897c6"
 132 | dependencies = [
 133 |  "num-traits",
 134 | ]
 135 | 
 136 | [[package]]
 137 | name = "arrayref"
 138 | version = "0.3.9"
 139 | source = "registry+https://github.com/rust-lang/crates.io-index"
 140 | checksum = "76a2e8124351fda1ef8aaaa3bbd7ebbcb486bbcd4225aca0aa0d84bb2db8fecb"
 141 | 
 142 | [[package]]
 143 | name = "arrayvec"
 144 | version = "0.7.6"
 145 | source = "registry+https://github.com/rust-lang/crates.io-index"
 146 | checksum = "7c02d123df017efcdfbd739ef81735b36c5ba83ec3c59c80a9d7ecc718f92e50"
 147 | 
 148 | [[package]]
 149 | name = "as-raw-xcb-connection"
 150 | version = "1.0.1"
 151 | source = "registry+https://github.com/rust-lang/crates.io-index"
 152 | checksum = "175571dd1d178ced59193a6fc02dde1b972eb0bc56c892cde9beeceac5bf0f6b"
 153 | 
 154 | [[package]]
 155 | name = "ash"
 156 | version = "0.38.0+1.3.281"
 157 | source = "registry+https://github.com/rust-lang/crates.io-index"
 158 | checksum = "0bb44936d800fea8f016d7f2311c6a4f97aebd5dc86f09906139ec848cf3a46f"
 159 | dependencies = [
 160 |  "libloading",
 161 | ]
 162 | 
 163 | [[package]]
 164 | name = "ashpd"
 165 | version = "0.8.1"
 166 | source = "registry+https://github.com/rust-lang/crates.io-index"
 167 | checksum = "dd884d7c72877a94102c3715f3b1cd09ff4fac28221add3e57cfbe25c236d093"
 168 | dependencies = [
 169 |  "async-fs",
 170 |  "async-net",
 171 |  "enumflags2",
 172 |  "futures-channel",
 173 |  "futures-util",
 174 |  "rand 0.8.5",
 175 |  "serde",
 176 |  "serde_repr",
 177 |  "url",
 178 |  "zbus",
 179 | ]
 180 | 
 181 | [[package]]
 182 | name = "async-broadcast"
 183 | version = "0.7.2"
 184 | source = "registry+https://github.com/rust-lang/crates.io-index"
 185 | checksum = "435a87a52755b8f27fcf321ac4f04b2802e337c8c4872923137471ec39c37532"
 186 | dependencies = [
 187 |  "event-listener",
 188 |  "event-listener-strategy",
 189 |  "futures-core",
 190 |  "pin-project-lite",
 191 | ]
 192 | 
 193 | [[package]]
 194 | name = "async-channel"
 195 | version = "2.5.0"
 196 | source = "registry+https://github.com/rust-lang/crates.io-index"
 197 | checksum = "924ed96dd52d1b75e9c1a3e6275715fd320f5f9439fb5a4a11fa51f4221158d2"
 198 | dependencies = [
 199 |  "concurrent-queue",
 200 |  "event-listener-strategy",
 201 |  "futures-core",
 202 |  "pin-project-lite",
 203 | ]
 204 | 
 205 | [[package]]
 206 | name = "async-executor"
 207 | version = "1.14.0"
 208 | source = "registry+https://github.com/rust-lang/crates.io-index"
 209 | checksum = "c96bf972d85afc50bf5ab8fe2d54d1586b4e0b46c97c50a0c9e71e2f7bcd812a"
 210 | dependencies = [
 211 |  "async-task",
 212 |  "concurrent-queue",
 213 |  "fastrand",
 214 |  "futures-lite",
 215 |  "pin-project-lite",
 216 |  "slab",
 217 | ]
 218 | 
 219 | [[package]]
 220 | name = "async-fs"
 221 | version = "2.2.0"
 222 | source = "registry+https://github.com/rust-lang/crates.io-index"
 223 | checksum = "8034a681df4aed8b8edbd7fbe472401ecf009251c8b40556b304567052e294c5"
 224 | dependencies = [
 225 |  "async-lock",
 226 |  "blocking",
 227 |  "futures-lite",
 228 | ]
 229 | 
 230 | [[package]]
 231 | name = "async-io"
 232 | version = "2.6.0"
 233 | source = "registry+https://github.com/rust-lang/crates.io-index"
 234 | checksum = "456b8a8feb6f42d237746d4b3e9a178494627745c3c56c6ea55d92ba50d026fc"
 235 | dependencies = [
 236 |  "autocfg",
 237 |  "cfg-if",
 238 |  "concurrent-queue",
 239 |  "futures-io",
 240 |  "futures-lite",
 241 |  "parking",
 242 |  "polling",
 243 |  "rustix 1.1.4",
 244 |  "slab",
 245 |  "windows-sys 0.61.2",
 246 | ]
 247 | 
 248 | [[package]]
 249 | name = "async-lock"
 250 | version = "3.4.2"
 251 | source = "registry+https://github.com/rust-lang/crates.io-index"
 252 | checksum = "290f7f2596bd5b78a9fec8088ccd89180d7f9f55b94b0576823bbbdc72ee8311"
 253 | dependencies = [
 254 |  "event-listener",
 255 |  "event-listener-strategy",
 256 |  "pin-project-lite",
 257 | ]
 258 | 
 259 | [[package]]
 260 | name = "async-net"
 261 | version = "2.0.0"
 262 | source = "registry+https://github.com/rust-lang/crates.io-index"
 263 | checksum = "b948000fad4873c1c9339d60f2623323a0cfd3816e5181033c6a5cb68b2accf7"
 264 | dependencies = [
 265 |  "async-io",
 266 |  "blocking",
 267 |  "futures-lite",
 268 | ]
 269 | 
 270 | [[package]]
 271 | name = "async-process"
 272 | version = "2.5.0"
 273 | source = "registry+https://github.com/rust-lang/crates.io-index"
 274 | checksum = "fc50921ec0055cdd8a16de48773bfeec5c972598674347252c0399676be7da75"
 275 | dependencies = [
 276 |  "async-channel",
 277 |  "async-io",
 278 |  "async-lock",
 279 |  "async-signal",
 280 |  "async-task",
 281 |  "blocking",
 282 |  "cfg-if",
 283 |  "event-listener",
 284 |  "futures-lite",
 285 |  "rustix 1.1.4",
 286 | ]
 287 | 
 288 | [[package]]
 289 | name = "async-recursion"
 290 | version = "1.1.1"
 291 | source = "registry+https://github.com/rust-lang/crates.io-index"
 292 | checksum = "3b43422f69d8ff38f95f1b2bb76517c91589a924d1559a0e935d7c8ce0274c11"
 293 | dependencies = [
 294 |  "proc-macro2",
 295 |  "quote",
 296 |  "syn",
 297 | ]
 298 | 
 299 | [[package]]
 300 | name = "async-signal"
 301 | version = "0.2.13"
 302 | source = "registry+https://github.com/rust-lang/crates.io-index"
 303 | checksum = "43c070bbf59cd3570b6b2dd54cd772527c7c3620fce8be898406dd3ed6adc64c"
 304 | dependencies = [
 305 |  "async-io",
 306 |  "async-lock",
 307 |  "atomic-waker",
 308 |  "cfg-if",
 309 |  "futures-core",
 310 |  "futures-io",
 311 |  "rustix 1.1.4",
 312 |  "signal-hook-registry",
 313 |  "slab",
 314 |  "windows-sys 0.61.2",
 315 | ]
 316 | 
 317 | [[package]]
 318 | name = "async-task"
 319 | version = "4.7.1"
 320 | source = "registry+https://github.com/rust-lang/crates.io-index"
 321 | checksum = "8b75356056920673b02621b35afd0f7dda9306d03c79a30f5c56c44cf256e3de"
 322 | 
 323 | [[package]]
 324 | name = "async-trait"
 325 | version = "0.1.89"
 326 | source = "registry+https://github.com/rust-lang/crates.io-index"
 327 | checksum = "9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb"
 328 | dependencies = [
 329 |  "proc-macro2",
 330 |  "quote",
 331 |  "syn",
 332 | ]
 333 | 
 334 | [[package]]
 335 | name = "atomic-waker"
 336 | version = "1.1.2"
 337 | source = "registry+https://github.com/rust-lang/crates.io-index"
 338 | checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"
 339 | 
 340 | [[package]]
 341 | name = "autocfg"
 342 | version = "1.5.0"
 343 | source = "registry+https://github.com/rust-lang/crates.io-index"
 344 | checksum = "c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8"
 345 | 
 346 | [[package]]
 347 | name = "backtrace"
 348 | version = "0.3.76"
 349 | source = "registry+https://github.com/rust-lang/crates.io-index"
 350 | checksum = "bb531853791a215d7c62a30daf0dde835f381ab5de4589cfe7c649d2cbe92bd6"
 351 | dependencies = [
 352 |  "addr2line",
 353 |  "cfg-if",
 354 |  "libc",
 355 |  "miniz_oxide",
 356 |  "object",
 357 |  "rustc-demangle",
 358 |  "windows-link",
 359 | ]
 360 | 
 361 | [[package]]
 362 | name = "base64"
 363 | version = "0.22.1"
 364 | source = "registry+https://github.com/rust-lang/crates.io-index"
 365 | checksum = "72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6"
 366 | 
 367 | [[package]]
 368 | name = "bit-set"
 369 | version = "0.8.0"
 370 | source = "registry+https://github.com/rust-lang/crates.io-index"
 371 | checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
 372 | dependencies = [
 373 |  "bit-vec",
 374 | ]
 375 | 
 376 | [[package]]
 377 | name = "bit-vec"
 378 | version = "0.8.0"
 379 | source = "registry+https://github.com/rust-lang/crates.io-index"
 380 | checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"
 381 | 
 382 | [[package]]
 383 | name = "bitflags"
 384 | version = "1.3.2"
 385 | source = "registry+https://github.com/rust-lang/crates.io-index"
 386 | checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"
 387 | 
 388 | [[package]]
 389 | name = "bitflags"
 390 | version = "2.11.0"
 391 | source = "registry+https://github.com/rust-lang/crates.io-index"
 392 | checksum = "843867be96c8daad0d758b57df9392b6d8d271134fce549de6ce169ff98a92af"
 393 | dependencies = [
 394 |  "serde_core",
 395 | ]
 396 | 
 397 | [[package]]
 398 | name = "bitstream-io"
 399 | version = "4.9.0"
 400 | source = "registry+https://github.com/rust-lang/crates.io-index"
 401 | checksum = "60d4bd9d1db2c6bdf285e223a7fa369d5ce98ec767dec949c6ca62863ce61757"
 402 | dependencies = [
 403 |  "core2",
 404 | ]
 405 | 
 406 | [[package]]
 407 | name = "block"
 408 | version = "0.1.6"
 409 | source = "registry+https://github.com/rust-lang/crates.io-index"
 410 | checksum = "0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a"
 411 | 
 412 | [[package]]
 413 | name = "block-buffer"
 414 | version = "0.10.4"
 415 | source = "registry+https://github.com/rust-lang/crates.io-index"
 416 | checksum = "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71"
 417 | dependencies = [
 418 |  "generic-array",
 419 | ]
 420 | 
 421 | [[package]]
 422 | name = "block2"
 423 | version = "0.5.1"
 424 | source = "registry+https://github.com/rust-lang/crates.io-index"
 425 | checksum = "2c132eebf10f5cad5289222520a4a058514204aed6d791f1cf4fe8088b82d15f"
 426 | dependencies = [
 427 |  "objc2",
 428 | ]
 429 | 
 430 | [[package]]
 431 | name = "blocking"
 432 | version = "1.6.2"
 433 | source = "registry+https://github.com/rust-lang/crates.io-index"
 434 | checksum = "e83f8d02be6967315521be875afa792a316e28d57b5a2d401897e2a7921b7f21"
 435 | dependencies = [
 436 |  "async-channel",
 437 |  "async-task",
 438 |  "futures-io",
 439 |  "futures-lite",
 440 |  "piper",
 441 | ]
 442 | 
 443 | [[package]]
 444 | name = "brotli-decompressor"
 445 | version = "5.0.0"
 446 | source = "registry+https://github.com/rust-lang/crates.io-index"
 447 | checksum = "874bb8112abecc98cbd6d81ea4fa7e94fb9449648c93cc89aa40c81c24d7de03"
 448 | dependencies = [
 449 |  "alloc-no-stdlib",
 450 |  "alloc-stdlib",
 451 | ]
 452 | 
 453 | [[package]]
 454 | name = "bumpalo"
 455 | version = "3.20.2"
 456 | source = "registry+https://github.com/rust-lang/crates.io-index"
 457 | checksum = "5d20789868f4b01b2f2caec9f5c4e0213b41e3e5702a50157d699ae31ced2fcb"
 458 | 
 459 | [[package]]
 460 | name = "bytemuck"
 461 | version = "1.25.0"
 462 | source = "registry+https://github.com/rust-lang/crates.io-index"
 463 | checksum = "c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec"
 464 | dependencies = [
 465 |  "bytemuck_derive",
 466 | ]
 467 | 
 468 | [[package]]
 469 | name = "bytemuck_derive"
 470 | version = "1.10.2"
 471 | source = "registry+https://github.com/rust-lang/crates.io-index"
 472 | checksum = "f9abbd1bc6865053c427f7198e6af43bfdedc55ab791faed4fbd361d789575ff"
 473 | dependencies = [
 474 |  "proc-macro2",
 475 |  "quote",
 476 |  "syn",
 477 | ]
 478 | 
 479 | [[package]]
 480 | name = "byteorder"
 481 | version = "1.5.0"
 482 | source = "registry+https://github.com/rust-lang/crates.io-index"
 483 | checksum = "1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b"
 484 | 
 485 | [[package]]
 486 | name = "byteorder-lite"
 487 | version = "0.1.0"
 488 | source = "registry+https://github.com/rust-lang/crates.io-index"
 489 | checksum = "8f1fe948ff07f4bd06c30984e69f5b4899c516a3ef74f34df92a2df2ab535495"
 490 | 
 491 | [[package]]
 492 | name = "bytes"
 493 | version = "1.11.1"
 494 | source = "registry+https://github.com/rust-lang/crates.io-index"
 495 | checksum = "1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33"
 496 | 
 497 | [[package]]
 498 | name = "calloop"
 499 | version = "0.13.0"
 500 | source = "registry+https://github.com/rust-lang/crates.io-index"
 501 | checksum = "b99da2f8558ca23c71f4fd15dc57c906239752dd27ff3c00a1d56b685b7cbfec"
 502 | dependencies = [
 503 |  "bitflags 2.11.0",
 504 |  "log",
 505 |  "polling",
 506 |  "rustix 0.38.44",
 507 |  "slab",
 508 |  "thiserror 1.0.69",
 509 | ]
 510 | 
 511 | [[package]]
 512 | name = "calloop-wayland-source"
 513 | version = "0.3.0"
 514 | source = "registry+https://github.com/rust-lang/crates.io-index"
 515 | checksum = "95a66a987056935f7efce4ab5668920b5d0dac4a7c99991a67395f13702ddd20"
 516 | dependencies = [
 517 |  "calloop",
 518 |  "rustix 0.38.44",
 519 |  "wayland-backend",
 520 |  "wayland-client",
 521 | ]
 522 | 
 523 | [[package]]
 524 | name = "cc"
 525 | version = "1.2.56"
 526 | source = "registry+https://github.com/rust-lang/crates.io-index"
 527 | checksum = "aebf35691d1bfb0ac386a69bac2fde4dd276fb618cf8bf4f5318fe285e821bb2"
 528 | dependencies = [
 529 |  "find-msvc-tools",
 530 |  "jobserver",
 531 |  "libc",
 532 |  "shlex",
 533 | ]
 534 | 
 535 | [[package]]
 536 | name = "cesu8"
 537 | version = "1.1.0"
 538 | source = "registry+https://github.com/rust-lang/crates.io-index"
 539 | checksum = "6d43a04d8753f35258c91f8ec639f792891f748a1edbd759cf1dcea3382ad83c"
 540 | 
 541 | [[package]]
 542 | name = "cfg-if"
 543 | version = "1.0.4"
 544 | source = "registry+https://github.com/rust-lang/crates.io-index"
 545 | checksum = "9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801"
 546 | 
 547 | [[package]]
 548 | name = "cfg_aliases"
 549 | version = "0.2.1"
 550 | source = "registry+https://github.com/rust-lang/crates.io-index"
 551 | checksum = "613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724"
 552 | 
 553 | [[package]]
 554 | name = "chrono"
 555 | version = "0.4.44"
 556 | source = "registry+https://github.com/rust-lang/crates.io-index"
 557 | checksum = "c673075a2e0e5f4a1dde27ce9dee1ea4558c7ffe648f576438a20ca1d2acc4b0"
 558 | dependencies = [
 559 |  "iana-time-zone",
 560 |  "js-sys",
 561 |  "num-traits",
 562 |  "wasm-bindgen",
 563 |  "windows-link",
 564 | ]
 565 | 
 566 | [[package]]
 567 | name = "codespan-reporting"
 568 | version = "0.12.0"
 569 | source = "registry+https://github.com/rust-lang/crates.io-index"
 570 | checksum = "fe6d2e5af09e8c8ad56c969f2157a3d4238cebc7c55f0a517728c38f7b200f81"
 571 | dependencies = [
 572 |  "serde",
 573 |  "termcolor",
 574 |  "unicode-width",
 575 | ]
 576 | 
 577 | [[package]]
 578 | name = "color_quant"
 579 | version = "1.1.0"
 580 | source = "registry+https://github.com/rust-lang/crates.io-index"
 581 | checksum = "3d7b894f5411737b7867f4827955924d7c254fc9f4d91a6aad6b097804b1018b"
 582 | 
 583 | [[package]]
 584 | name = "combine"
 585 | version = "4.6.7"
 586 | source = "registry+https://github.com/rust-lang/crates.io-index"
 587 | checksum = "ba5a308b75df32fe02788e748662718f03fde005016435c444eea572398219fd"
 588 | dependencies = [
 589 |  "bytes",
 590 |  "memchr",
 591 | ]
 592 | 
 593 | [[package]]
 594 | name = "concurrent-queue"
 595 | version = "2.5.0"
 596 | source = "registry+https://github.com/rust-lang/crates.io-index"
 597 | checksum = "4ca0197aee26d1ae37445ee532fefce43251d24cc7c166799f4d46817f1d3973"
 598 | dependencies = [
 599 |  "crossbeam-utils",
 600 | ]
 601 | 
 602 | [[package]]
 603 | name = "core-foundation"
 604 | version = "0.9.4"
 605 | source = "registry+https://github.com/rust-lang/crates.io-index"
 606 | checksum = "91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f"
 607 | dependencies = [
 608 |  "core-foundation-sys",
 609 |  "libc",
 610 | ]
 611 | 
 612 | [[package]]
 613 | name = "core-foundation"
 614 | version = "0.10.1"
 615 | source = "registry+https://github.com/rust-lang/crates.io-index"
 616 | checksum = "b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6"
 617 | dependencies = [
 618 |  "core-foundation-sys",
 619 |  "libc",
 620 | ]
 621 | 
 622 | [[package]]
 623 | name = "core-foundation-sys"
 624 | version = "0.8.7"
 625 | source = "registry+https://github.com/rust-lang/crates.io-index"
 626 | checksum = "773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"
 627 | 
 628 | [[package]]
 629 | name = "core-graphics"
 630 | version = "0.23.2"
 631 | source = "registry+https://github.com/rust-lang/crates.io-index"
 632 | checksum = "c07782be35f9e1140080c6b96f0d44b739e2278479f64e02fdab4e32dfd8b081"
 633 | dependencies = [
 634 |  "bitflags 1.3.2",
 635 |  "core-foundation 0.9.4",
 636 |  "core-graphics-types 0.1.3",
 637 |  "foreign-types",
 638 |  "libc",
 639 | ]
 640 | 
 641 | [[package]]
 642 | name = "core-graphics-types"
 643 | version = "0.1.3"
 644 | source = "registry+https://github.com/rust-lang/crates.io-index"
 645 | checksum = "45390e6114f68f718cc7a830514a96f903cccd70d02a8f6d9f643ac4ba45afaf"
 646 | dependencies = [
 647 |  "bitflags 1.3.2",
 648 |  "core-foundation 0.9.4",
 649 |  "libc",
 650 | ]
 651 | 
 652 | [[package]]
 653 | name = "core-graphics-types"
 654 | version = "0.2.0"
 655 | source = "registry+https://github.com/rust-lang/crates.io-index"
 656 | checksum = "3d44a101f213f6c4cdc1853d4b78aef6db6bdfa3468798cc1d9912f4735013eb"
 657 | dependencies = [
 658 |  "bitflags 2.11.0",
 659 |  "core-foundation 0.10.1",
 660 |  "libc",
 661 | ]
 662 | 
 663 | [[package]]
 664 | name = "core2"
 665 | version = "0.4.0"
 666 | source = "registry+https://github.com/rust-lang/crates.io-index"
 667 | checksum = "b49ba7ef1ad6107f8824dbe97de947cbaac53c44e7f9756a1fba0d37c1eec505"
 668 | dependencies = [
 669 |  "memchr",
 670 | ]
 671 | 
 672 | [[package]]
 673 | name = "core_maths"
 674 | version = "0.1.1"
 675 | source = "registry+https://github.com/rust-lang/crates.io-index"
 676 | checksum = "77745e017f5edba1a9c1d854f6f3a52dac8a12dd5af5d2f54aecf61e43d80d30"
 677 | dependencies = [
 678 |  "libm",
 679 | ]
 680 | 
 681 | [[package]]
 682 | name = "cpufeatures"
 683 | version = "0.2.17"
 684 | source = "registry+https://github.com/rust-lang/crates.io-index"
 685 | checksum = "59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280"
 686 | dependencies = [
 687 |  "libc",
 688 | ]
 689 | 
 690 | [[package]]
 691 | name = "crc32fast"
 692 | version = "1.5.0"
 693 | source = "registry+https://github.com/rust-lang/crates.io-index"
 694 | checksum = "9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511"
 695 | dependencies = [
 696 |  "cfg-if",
 697 | ]
 698 | 
 699 | [[package]]
 700 | name = "crossbeam-channel"
 701 | version = "0.5.15"
 702 | source = "registry+https://github.com/rust-lang/crates.io-index"
 703 | checksum = "82b8f8f868b36967f9606790d1903570de9ceaf870a7bf9fbbd3016d636a2cb2"
 704 | dependencies = [
 705 |  "crossbeam-utils",
 706 | ]
 707 | 
 708 | [[package]]
 709 | name = "crossbeam-deque"
 710 | version = "0.8.6"
 711 | source = "registry+https://github.com/rust-lang/crates.io-index"
 712 | checksum = "9dd111b7b7f7d55b72c0a6ae361660ee5853c9af73f70c3c2ef6858b950e2e51"
 713 | dependencies = [
 714 |  "crossbeam-epoch",
 715 |  "crossbeam-utils",
 716 | ]
 717 | 
 718 | [[package]]
 719 | name = "crossbeam-epoch"
 720 | version = "0.9.18"
 721 | source = "registry+https://github.com/rust-lang/crates.io-index"
 722 | checksum = "5b82ac4a3c2ca9c3460964f020e1402edd5753411d7737aa39c3714ad1b5420e"
 723 | dependencies = [
 724 |  "crossbeam-utils",
 725 | ]
 726 | 
 727 | [[package]]
 728 | name = "crossbeam-utils"
 729 | version = "0.8.21"
 730 | source = "registry+https://github.com/rust-lang/crates.io-index"
 731 | checksum = "d0a5c400df2834b80a4c3327b3aad3a4c4cd4de0629063962b03235697506a28"
 732 | 
 733 | [[package]]
 734 | name = "crunchy"
 735 | version = "0.2.4"
 736 | source = "registry+https://github.com/rust-lang/crates.io-index"
 737 | checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"
 738 | 
 739 | [[package]]
 740 | name = "crypto-common"
 741 | version = "0.1.7"
 742 | source = "registry+https://github.com/rust-lang/crates.io-index"
 743 | checksum = "78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a"
 744 | dependencies = [
 745 |  "generic-array",
 746 |  "typenum",
 747 | ]
 748 | 
 749 | [[package]]
 750 | name = "cursor-icon"
 751 | version = "1.2.0"
 752 | source = "registry+https://github.com/rust-lang/crates.io-index"
 753 | checksum = "f27ae1dd37df86211c42e150270f82743308803d90a6f6e6651cd730d5e1732f"
 754 | 
 755 | [[package]]
 756 | name = "dary_heap"
 757 | version = "0.3.8"
 758 | source = "registry+https://github.com/rust-lang/crates.io-index"
 759 | checksum = "06d2e3287df1c007e74221c49ca10a95d557349e54b3a75dc2fb14712c751f04"
 760 | 
 761 | [[package]]
 762 | name = "data-url"
 763 | version = "0.3.2"
 764 | source = "registry+https://github.com/rust-lang/crates.io-index"
 765 | checksum = "be1e0bca6c3637f992fc1cc7cbc52a78c1ef6db076dbf1059c4323d6a2048376"
 766 | 
 767 | [[package]]
 768 | name = "digest"
 769 | version = "0.10.7"
 770 | source = "registry+https://github.com/rust-lang/crates.io-index"
 771 | checksum = "9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292"
 772 | dependencies = [
 773 |  "block-buffer",
 774 |  "crypto-common",
 775 | ]
 776 | 
 777 | [[package]]
 778 | name = "dispatch"
 779 | version = "0.2.0"
 780 | source = "registry+https://github.com/rust-lang/crates.io-index"
 781 | checksum = "bd0c93bb4b0c6d9b77f4435b0ae98c24d17f1c45b2ff844c6151a07256ca923b"
 782 | 
 783 | [[package]]
 784 | name = "displaydoc"
 785 | version = "0.2.5"
 786 | source = "registry+https://github.com/rust-lang/crates.io-index"
 787 | checksum = "97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0"
 788 | dependencies = [
 789 |  "proc-macro2",
 790 |  "quote",
 791 |  "syn",
 792 | ]
 793 | 
 794 | [[package]]
 795 | name = "dlib"
 796 | version = "0.5.3"
 797 | source = "registry+https://github.com/rust-lang/crates.io-index"
 798 | checksum = "ab8ecd87370524b461f8557c119c405552c396ed91fc0a8eec68679eab26f94a"
 799 | dependencies = [
 800 |  "libloading",
 801 | ]
 802 | 
 803 | [[package]]
 804 | name = "document-features"
 805 | version = "0.2.12"
 806 | source = "registry+https://github.com/rust-lang/crates.io-index"
 807 | checksum = "d4b8a88685455ed29a21542a33abd9cb6510b6b129abadabdcef0f4c55bc8f61"
 808 | dependencies = [
 809 |  "litrs",
 810 | ]
 811 | 
 812 | [[package]]
 813 | name = "downcast-rs"
 814 | version = "1.2.1"
 815 | source = "registry+https://github.com/rust-lang/crates.io-index"
 816 | checksum = "75b325c5dbd37f80359721ad39aca5a29fb04c89279657cffdda8736d0c0b9d2"
 817 | 
 818 | [[package]]
 819 | name = "dpi"
 820 | version = "0.1.2"
 821 | source = "registry+https://github.com/rust-lang/crates.io-index"
 822 | checksum = "d8b14ccef22fc6f5a8f4d7d768562a182c04ce9a3b3157b91390b52ddfdf1a76"
 823 | 
 824 | [[package]]
 825 | name = "either"
 826 | version = "1.15.0"
 827 | source = "registry+https://github.com/rust-lang/crates.io-index"
 828 | checksum = "48c757948c5ede0e46177b7add2e67155f70e33c07fea8284df6576da70b3719"
 829 | 
 830 | [[package]]
 831 | name = "endi"
 832 | version = "1.1.1"
 833 | source = "registry+https://github.com/rust-lang/crates.io-index"
 834 | checksum = "66b7e2430c6dff6a955451e2cfc438f09cea1965a9d6f87f7e3b90decc014099"
 835 | 
 836 | [[package]]
 837 | name = "enumflags2"
 838 | version = "0.7.12"
 839 | source = "registry+https://github.com/rust-lang/crates.io-index"
 840 | checksum = "1027f7680c853e056ebcec683615fb6fbbc07dbaa13b4d5d9442b146ded4ecef"
 841 | dependencies = [
 842 |  "enumflags2_derive",
 843 |  "serde",
 844 | ]
 845 | 
 846 | [[package]]
 847 | name = "enumflags2_derive"
 848 | version = "0.7.12"
 849 | source = "registry+https://github.com/rust-lang/crates.io-index"
 850 | checksum = "67c78a4d8fdf9953a5c9d458f9efe940fd97a0cab0941c075a813ac594733827"
 851 | dependencies = [
 852 |  "proc-macro2",
 853 |  "quote",
 854 |  "syn",
 855 | ]
 856 | 
 857 | [[package]]
 858 | name = "enumn"
 859 | version = "0.1.14"
 860 | source = "registry+https://github.com/rust-lang/crates.io-index"
 861 | checksum = "2f9ed6b3789237c8a0c1c505af1c7eb2c560df6186f01b098c3a1064ea532f38"
 862 | dependencies = [
 863 |  "proc-macro2",
 864 |  "quote",
 865 |  "syn",
 866 | ]
 867 | 
 868 | [[package]]
 869 | name = "equivalent"
 870 | version = "1.0.2"
 871 | source = "registry+https://github.com/rust-lang/crates.io-index"
 872 | checksum = "877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"
 873 | 
 874 | [[package]]
 875 | name = "errno"
 876 | version = "0.3.14"
 877 | source = "registry+https://github.com/rust-lang/crates.io-index"
 878 | checksum = "39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"
 879 | dependencies = [
 880 |  "libc",
 881 |  "windows-sys 0.61.2",
 882 | ]
 883 | 
 884 | [[package]]
 885 | name = "euclid"
 886 | version = "0.22.13"
 887 | source = "registry+https://github.com/rust-lang/crates.io-index"
 888 | checksum = "df61bf483e837f88d5c2291dcf55c67be7e676b3a51acc48db3a7b163b91ed63"
 889 | dependencies = [
 890 |  "num-traits",
 891 | ]
 892 | 
 893 | [[package]]
 894 | name = "event-listener"
 895 | version = "5.4.1"
 896 | source = "registry+https://github.com/rust-lang/crates.io-index"
 897 | checksum = "e13b66accf52311f30a0db42147dadea9850cb48cd070028831ae5f5d4b856ab"
 898 | dependencies = [
 899 |  "concurrent-queue",
 900 |  "parking",
 901 |  "pin-project-lite",
 902 | ]
 903 | 
 904 | [[package]]
 905 | name = "event-listener-strategy"
 906 | version = "0.5.4"
 907 | source = "registry+https://github.com/rust-lang/crates.io-index"
 908 | checksum = "8be9f3dfaaffdae2972880079a491a1a8bb7cbed0b8dd7a347f668b4150a3b93"
 909 | dependencies = [
 910 |  "event-listener",
 911 |  "pin-project-lite",
 912 | ]
 913 | 
 914 | [[package]]
 915 | name = "fast_image_resize"
 916 | version = "3.0.4"
 917 | source = "registry+https://github.com/rust-lang/crates.io-index"
 918 | checksum = "c9d450fac8a334ad72825596173f0f7767ff04dd6e3d59c49c894c4bc2957e8b"
 919 | dependencies = [
 920 |  "cfg-if",
 921 |  "num-traits",
 922 |  "thiserror 1.0.69",
 923 | ]
 924 | 
 925 | [[package]]
 926 | name = "fastrand"
 927 | version = "2.3.0"
 928 | source = "registry+https://github.com/rust-lang/crates.io-index"
 929 | checksum = "37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be"
 930 | 
 931 | [[package]]
 932 | name = "fax"
 933 | version = "0.2.6"
 934 | source = "registry+https://github.com/rust-lang/crates.io-index"
 935 | checksum = "f05de7d48f37cd6730705cbca900770cab77a89f413d23e100ad7fad7795a0ab"
 936 | dependencies = [
 937 |  "fax_derive",
 938 | ]
 939 | 
 940 | [[package]]
 941 | name = "fax_derive"
 942 | version = "0.2.0"
 943 | source = "registry+https://github.com/rust-lang/crates.io-index"
 944 | checksum = "a0aca10fb742cb43f9e7bb8467c91aa9bcb8e3ffbc6a6f7389bb93ffc920577d"
 945 | dependencies = [
 946 |  "proc-macro2",
 947 |  "quote",
 948 |  "syn",
 949 | ]
 950 | 
 951 | [[package]]
 952 | name = "fdeflate"
 953 | version = "0.3.7"
 954 | source = "registry+https://github.com/rust-lang/crates.io-index"
 955 | checksum = "1e6853b52649d4ac5c0bd02320cddc5ba956bdb407c4b75a2c6b75bf51500f8c"
 956 | dependencies = [
 957 |  "simd-adler32",
 958 | ]
 959 | 
 960 | [[package]]
 961 | name = "filetime"
 962 | version = "0.2.27"
 963 | source = "registry+https://github.com/rust-lang/crates.io-index"
 964 | checksum = "f98844151eee8917efc50bd9e8318cb963ae8b297431495d3f758616ea5c57db"
 965 | dependencies = [
 966 |  "cfg-if",
 967 |  "libc",
 968 |  "libredox",
 969 | ]
 970 | 
 971 | [[package]]
 972 | name = "find-msvc-tools"
 973 | version = "0.1.9"
 974 | source = "registry+https://github.com/rust-lang/crates.io-index"
 975 | checksum = "5baebc0774151f905a1a2cc41989300b1e6fbb29aff0ceffa1064fdd3088d582"
 976 | 
 977 | [[package]]
 978 | name = "flate2"
 979 | version = "1.1.9"
 980 | source = "registry+https://github.com/rust-lang/crates.io-index"
 981 | checksum = "843fba2746e448b37e26a819579957415c8cef339bf08564fe8b7ddbd959573c"
 982 | dependencies = [
 983 |  "crc32fast",
 984 |  "miniz_oxide",
 985 | ]
 986 | 
 987 | [[package]]
 988 | name = "float-cmp"
 989 | version = "0.9.0"
 990 | source = "registry+https://github.com/rust-lang/crates.io-index"
 991 | checksum = "98de4bbd547a563b716d8dfa9aad1cb19bfab00f4fa09a6a4ed21dbcf44ce9c4"
 992 | 
 993 | [[package]]
 994 | name = "foldhash"
 995 | version = "0.1.5"
 996 | source = "registry+https://github.com/rust-lang/crates.io-index"
 997 | checksum = "d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2"
 998 | 
 999 | [[package]]
1000 | name = "foldhash"
1001 | version = "0.2.0"
1002 | source = "registry+https://github.com/rust-lang/crates.io-index"
1003 | checksum = "77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb"
1004 | 
1005 | [[package]]
1006 | name = "fontconfig-parser"
1007 | version = "0.5.8"
1008 | source = "registry+https://github.com/rust-lang/crates.io-index"
1009 | checksum = "bbc773e24e02d4ddd8395fd30dc147524273a83e54e0f312d986ea30de5f5646"
1010 | dependencies = [
1011 |  "roxmltree 0.20.0",
1012 | ]
1013 | 
1014 | [[package]]
1015 | name = "fontdb"
1016 | version = "0.23.0"
1017 | source = "registry+https://github.com/rust-lang/crates.io-index"
1018 | checksum = "457e789b3d1202543297a350643cf459f836cade38934e7a4cf6a39e7cde2905"
1019 | dependencies = [
1020 |  "fontconfig-parser",
1021 |  "log",
1022 |  "memmap2",
1023 |  "slotmap",
1024 |  "tinyvec",
1025 |  "ttf-parser",
1026 | ]
1027 | 
1028 | [[package]]
1029 | name = "foreign-types"
1030 | version = "0.5.0"
1031 | source = "registry+https://github.com/rust-lang/crates.io-index"
1032 | checksum = "d737d9aa519fb7b749cbc3b962edcf310a8dd1f4b67c91c4f83975dbdd17d965"
1033 | dependencies = [
1034 |  "foreign-types-macros",
1035 |  "foreign-types-shared",
1036 | ]
1037 | 
1038 | [[package]]
1039 | name = "foreign-types-macros"
1040 | version = "0.2.3"
1041 | source = "registry+https://github.com/rust-lang/crates.io-index"
1042 | checksum = "1a5c6c585bc94aaf2c7b51dd4c2ba22680844aba4c687be581871a6f518c5742"
1043 | dependencies = [
1044 |  "proc-macro2",
1045 |  "quote",
1046 |  "syn",
1047 | ]
1048 | 
1049 | [[package]]
1050 | name = "foreign-types-shared"
1051 | version = "0.3.1"
1052 | source = "registry+https://github.com/rust-lang/crates.io-index"
1053 | checksum = "aa9a19cbb55df58761df49b23516a86d432839add4af60fc256da840f66ed35b"
1054 | 
1055 | [[package]]
1056 | name = "form_urlencoded"
1057 | version = "1.2.2"
1058 | source = "registry+https://github.com/rust-lang/crates.io-index"
1059 | checksum = "cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf"
1060 | dependencies = [
1061 |  "percent-encoding",
1062 | ]
1063 | 
1064 | [[package]]
1065 | name = "fsevent-sys"
1066 | version = "4.1.0"
1067 | source = "registry+https://github.com/rust-lang/crates.io-index"
1068 | checksum = "76ee7a02da4d231650c7cea31349b889be2f45ddb3ef3032d2ec8185f6313fd2"
1069 | dependencies = [
1070 |  "libc",
1071 | ]
1072 | 
1073 | [[package]]
1074 | name = "futures-channel"
1075 | version = "0.3.32"
1076 | source = "registry+https://github.com/rust-lang/crates.io-index"
1077 | checksum = "07bbe89c50d7a535e539b8c17bc0b49bdb77747034daa8087407d655f3f7cc1d"
1078 | dependencies = [
1079 |  "futures-core",
1080 | ]
1081 | 
1082 | [[package]]
1083 | name = "futures-core"
1084 | version = "0.3.32"
1085 | source = "registry+https://github.com/rust-lang/crates.io-index"
1086 | checksum = "7e3450815272ef58cec6d564423f6e755e25379b217b0bc688e295ba24df6b1d"
1087 | 
1088 | [[package]]
1089 | name = "futures-io"
1090 | version = "0.3.32"
1091 | source = "registry+https://github.com/rust-lang/crates.io-index"
1092 | checksum = "cecba35d7ad927e23624b22ad55235f2239cfa44fd10428eecbeba6d6a717718"
1093 | 
1094 | [[package]]
1095 | name = "futures-lite"
1096 | version = "2.6.1"
1097 | source = "registry+https://github.com/rust-lang/crates.io-index"
1098 | checksum = "f78e10609fe0e0b3f4157ffab1876319b5b0db102a2c60dc4626306dc46b44ad"
1099 | dependencies = [
1100 |  "fastrand",
1101 |  "futures-core",
1102 |  "futures-io",
1103 |  "parking",
1104 |  "pin-project-lite",
1105 | ]
1106 | 
1107 | [[package]]
1108 | name = "futures-macro"
1109 | version = "0.3.32"
1110 | source = "registry+https://github.com/rust-lang/crates.io-index"
1111 | checksum = "e835b70203e41293343137df5c0664546da5745f82ec9b84d40be8336958447b"
1112 | dependencies = [
1113 |  "proc-macro2",
1114 |  "quote",
1115 |  "syn",
1116 | ]
1117 | 
1118 | [[package]]
1119 | name = "futures-sink"
1120 | version = "0.3.32"
1121 | source = "registry+https://github.com/rust-lang/crates.io-index"
1122 | checksum = "c39754e157331b013978ec91992bde1ac089843443c49cbc7f46150b0fad0893"
1123 | 
1124 | [[package]]
1125 | name = "futures-task"
1126 | version = "0.3.32"
1127 | source = "registry+https://github.com/rust-lang/crates.io-index"
1128 | checksum = "037711b3d59c33004d3856fbdc83b99d4ff37a24768fa1be9ce3538a1cde4393"
1129 | 
1130 | [[package]]
1131 | name = "futures-util"
1132 | version = "0.3.32"
1133 | source = "registry+https://github.com/rust-lang/crates.io-index"
1134 | checksum = "389ca41296e6190b48053de0321d02a77f32f8a5d2461dd38762c0593805c6d6"
1135 | dependencies = [
1136 |  "futures-core",
1137 |  "futures-io",
1138 |  "futures-macro",
1139 |  "futures-sink",
1140 |  "futures-task",
1141 |  "memchr",
1142 |  "pin-project-lite",
1143 |  "slab",
1144 | ]
1145 | 
1146 | [[package]]
1147 | name = "generic-array"
1148 | version = "0.14.7"
1149 | source = "registry+https://github.com/rust-lang/crates.io-index"
1150 | checksum = "85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a"
1151 | dependencies = [
1152 |  "typenum",
1153 |  "version_check",
1154 | ]
1155 | 
1156 | [[package]]
1157 | name = "gethostname"
1158 | version = "1.1.0"
1159 | source = "registry+https://github.com/rust-lang/crates.io-index"
1160 | checksum = "1bd49230192a3797a9a4d6abe9b3eed6f7fa4c8a8a4947977c6f80025f92cbd8"
1161 | dependencies = [
1162 |  "rustix 1.1.4",
1163 |  "windows-link",
1164 | ]
1165 | 
1166 | [[package]]
1167 | name = "getrandom"
1168 | version = "0.2.17"
1169 | source = "registry+https://github.com/rust-lang/crates.io-index"
1170 | checksum = "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0"
1171 | dependencies = [
1172 |  "cfg-if",
1173 |  "libc",
1174 |  "wasi",
1175 | ]
1176 | 
1177 | [[package]]
1178 | name = "getrandom"
1179 | version = "0.3.4"
1180 | source = "registry+https://github.com/rust-lang/crates.io-index"
1181 | checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
1182 | dependencies = [
1183 |  "cfg-if",
1184 |  "libc",
1185 |  "r-efi",
1186 |  "wasip2",
1187 | ]
1188 | 
1189 | [[package]]
1190 | name = "getrandom"
1191 | version = "0.4.1"
1192 | source = "registry+https://github.com/rust-lang/crates.io-index"
1193 | checksum = "139ef39800118c7683f2fd3c98c1b23c09ae076556b435f8e9064ae108aaeeec"
1194 | dependencies = [
1195 |  "cfg-if",
1196 |  "libc",
1197 |  "r-efi",
1198 |  "wasip2",
1199 |  "wasip3",
1200 | ]
1201 | 
1202 | [[package]]
1203 | name = "gif"
1204 | version = "0.14.1"
1205 | source = "registry+https://github.com/rust-lang/crates.io-index"
1206 | checksum = "f5df2ba84018d80c213569363bdcd0c64e6933c67fe4c1d60ecf822971a3c35e"
1207 | dependencies = [
1208 |  "color_quant",
1209 |  "weezl",
1210 | ]
1211 | 
1212 | [[package]]
1213 | name = "gimli"
1214 | version = "0.32.3"
1215 | source = "registry+https://github.com/rust-lang/crates.io-index"
1216 | checksum = "e629b9b98ef3dd8afe6ca2bd0f89306cec16d43d907889945bc5d6687f2f13c7"
1217 | 
1218 | [[package]]
1219 | name = "gl_generator"
1220 | version = "0.14.0"
1221 | source = "registry+https://github.com/rust-lang/crates.io-index"
1222 | checksum = "1a95dfc23a2b4a9a2f5ab41d194f8bfda3cabec42af4e39f08c339eb2a0c124d"
1223 | dependencies = [
1224 |  "khronos_api",
1225 |  "log",
1226 |  "xml-rs",
1227 | ]
1228 | 
1229 | [[package]]
1230 | name = "glob"
1231 | version = "0.3.3"
1232 | source = "registry+https://github.com/rust-lang/crates.io-index"
1233 | checksum = "0cc23270f6e1808e30a928bdc84dea0b9b4136a8bc82338574f23baf47bbd280"
1234 | 
1235 | [[package]]
1236 | name = "glow"
1237 | version = "0.16.0"
1238 | source = "registry+https://github.com/rust-lang/crates.io-index"
1239 | checksum = "c5e5ea60d70410161c8bf5da3fdfeaa1c72ed2c15f8bbb9d19fe3a4fad085f08"
1240 | dependencies = [
1241 |  "js-sys",
1242 |  "slotmap",
1243 |  "wasm-bindgen",
1244 |  "web-sys",
1245 | ]
1246 | 
1247 | [[package]]
1248 | name = "glutin_wgl_sys"
1249 | version = "0.6.1"
1250 | source = "registry+https://github.com/rust-lang/crates.io-index"
1251 | checksum = "2c4ee00b289aba7a9e5306d57c2d05499b2e5dc427f84ac708bd2c090212cf3e"
1252 | dependencies = [
1253 |  "gl_generator",
1254 | ]
1255 | 
1256 | [[package]]
1257 | name = "glyph_brush"
1258 | version = "0.7.12"
1259 | source = "registry+https://github.com/rust-lang/crates.io-index"
1260 | checksum = "0060f4ed4ef64a5876d9836d7d6c9ed43a463f3ca431682bec1c326064c8c93e"
1261 | dependencies = [
1262 |  "glyph_brush_draw_cache",
1263 |  "glyph_brush_layout",
1264 |  "ordered-float 5.1.0",
1265 |  "rustc-hash 2.1.1",
1266 |  "twox-hash",
1267 | ]
1268 | 
1269 | [[package]]
1270 | name = "glyph_brush_draw_cache"
1271 | version = "0.1.6"
1272 | source = "registry+https://github.com/rust-lang/crates.io-index"
1273 | checksum = "4bb6c910def52365fef3f439a6b50a4d5c11b28eec4cf6c191f6dfea18e88d7f"
1274 | dependencies = [
1275 |  "ab_glyph",
1276 |  "crossbeam-channel",
1277 |  "crossbeam-deque",
1278 |  "linked-hash-map",
1279 |  "rayon",
1280 |  "rustc-hash 2.1.1",
1281 | ]
1282 | 
1283 | [[package]]
1284 | name = "glyph_brush_layout"
1285 | version = "0.2.4"
1286 | source = "registry+https://github.com/rust-lang/crates.io-index"
1287 | checksum = "7b1e288bfd2f6c0313f78bf5aa538356ad481a3bb97e9b7f93220ab0066c5992"
1288 | dependencies = [
1289 |  "ab_glyph",
1290 |  "approx",
1291 |  "xi-unicode",
1292 | ]
1293 | 
1294 | [[package]]
1295 | name = "gpu-alloc"
1296 | version = "0.6.0"
1297 | source = "registry+https://github.com/rust-lang/crates.io-index"
1298 | checksum = "fbcd2dba93594b227a1f57ee09b8b9da8892c34d55aa332e034a228d0fe6a171"
1299 | dependencies = [
1300 |  "bitflags 2.11.0",
1301 |  "gpu-alloc-types",
1302 | ]
1303 | 
1304 | [[package]]
1305 | name = "gpu-alloc-types"
1306 | version = "0.3.0"
1307 | source = "registry+https://github.com/rust-lang/crates.io-index"
1308 | checksum = "98ff03b468aa837d70984d55f5d3f846f6ec31fe34bbb97c4f85219caeee1ca4"
1309 | dependencies = [
1310 |  "bitflags 2.11.0",
1311 | ]
1312 | 
1313 | [[package]]
1314 | name = "gpu-allocator"
1315 | version = "0.27.0"
1316 | source = "registry+https://github.com/rust-lang/crates.io-index"
1317 | checksum = "c151a2a5ef800297b4e79efa4f4bec035c5f51d5ae587287c9b952bdf734cacd"
1318 | dependencies = [
1319 |  "log",
1320 |  "presser",
1321 |  "thiserror 1.0.69",
1322 |  "windows",
1323 | ]
1324 | 
1325 | [[package]]
1326 | name = "gpu-descriptor"
1327 | version = "0.3.2"
1328 | source = "registry+https://github.com/rust-lang/crates.io-index"
1329 | checksum = "b89c83349105e3732062a895becfc71a8f921bb71ecbbdd8ff99263e3b53a0ca"
1330 | dependencies = [
1331 |  "bitflags 2.11.0",
1332 |  "gpu-descriptor-types",
1333 |  "hashbrown 0.15.5",
1334 | ]
1335 | 
1336 | [[package]]
1337 | name = "gpu-descriptor-types"
1338 | version = "0.2.0"
1339 | source = "registry+https://github.com/rust-lang/crates.io-index"
1340 | checksum = "fdf242682df893b86f33a73828fb09ca4b2d3bb6cc95249707fc684d27484b91"
1341 | dependencies = [
1342 |  "bitflags 2.11.0",
1343 | ]
1344 | 
1345 | [[package]]
1346 | name = "half"
1347 | version = "2.7.1"
1348 | source = "registry+https://github.com/rust-lang/crates.io-index"
1349 | checksum = "6ea2d84b969582b4b1864a92dc5d27cd2b77b622a8d79306834f1be5ba20d84b"
1350 | dependencies = [
1351 |  "cfg-if",
1352 |  "crunchy",
1353 |  "num-traits",
1354 |  "zerocopy",
1355 | ]
1356 | 
1357 | [[package]]
1358 | name = "hashbrown"
1359 | version = "0.15.5"
1360 | source = "registry+https://github.com/rust-lang/crates.io-index"
1361 | checksum = "9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1"
1362 | dependencies = [
1363 |  "allocator-api2",
1364 |  "equivalent",
1365 |  "foldhash 0.1.5",
1366 | ]
1367 | 
1368 | [[package]]
1369 | name = "hashbrown"
1370 | version = "0.16.1"
1371 | source = "registry+https://github.com/rust-lang/crates.io-index"
1372 | checksum = "841d1cc9bed7f9236f321df977030373f4a4163ae1a7dbfe1a51a2c1a51d9100"
1373 | dependencies = [
1374 |  "allocator-api2",
1375 |  "equivalent",
1376 |  "foldhash 0.2.0",
1377 | ]
1378 | 
1379 | [[package]]
1380 | name = "heck"
1381 | version = "0.5.0"
1382 | source = "registry+https://github.com/rust-lang/crates.io-index"
1383 | checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"
1384 | 
1385 | [[package]]
1386 | name = "hermit-abi"
1387 | version = "0.5.2"
1388 | source = "registry+https://github.com/rust-lang/crates.io-index"
1389 | checksum = "fc0fef456e4baa96da950455cd02c081ca953b141298e41db3fc7e36b1da849c"
1390 | 
1391 | [[package]]
1392 | name = "hex"
1393 | version = "0.4.3"
1394 | source = "registry+https://github.com/rust-lang/crates.io-index"
1395 | checksum = "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"
1396 | 
1397 | [[package]]
1398 | name = "hexf-parse"
1399 | version = "0.2.1"
1400 | source = "registry+https://github.com/rust-lang/crates.io-index"
1401 | checksum = "dfa686283ad6dd069f105e5ab091b04c62850d3e4cf5d67debad1933f55023df"
1402 | 
1403 | [[package]]
1404 | name = "iana-time-zone"
1405 | version = "0.1.65"
1406 | source = "registry+https://github.com/rust-lang/crates.io-index"
1407 | checksum = "e31bc9ad994ba00e440a8aa5c9ef0ec67d5cb5e5cb0cc7f8b744a35b389cc470"
1408 | dependencies = [
1409 |  "android_system_properties",
1410 |  "core-foundation-sys",
1411 |  "iana-time-zone-haiku",
1412 |  "js-sys",
1413 |  "log",
1414 |  "wasm-bindgen",
1415 |  "windows-core",
1416 | ]
1417 | 
1418 | [[package]]
1419 | name = "iana-time-zone-haiku"
1420 | version = "0.1.2"
1421 | source = "registry+https://github.com/rust-lang/crates.io-index"
1422 | checksum = "f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f"
1423 | dependencies = [
1424 |  "cc",
1425 | ]
1426 | 
1427 | [[package]]
1428 | name = "icu_collections"
1429 | version = "2.1.1"
1430 | source = "registry+https://github.com/rust-lang/crates.io-index"
1431 | checksum = "4c6b649701667bbe825c3b7e6388cb521c23d88644678e83c0c4d0a621a34b43"
1432 | dependencies = [
1433 |  "displaydoc",
1434 |  "potential_utf",
1435 |  "yoke",
1436 |  "zerofrom",
1437 |  "zerovec",
1438 | ]
1439 | 
1440 | [[package]]
1441 | name = "icu_locale_core"
1442 | version = "2.1.1"
1443 | source = "registry+https://github.com/rust-lang/crates.io-index"
1444 | checksum = "edba7861004dd3714265b4db54a3c390e880ab658fec5f7db895fae2046b5bb6"
1445 | dependencies = [
1446 |  "displaydoc",
1447 |  "litemap",
1448 |  "tinystr",
1449 |  "writeable",
1450 |  "zerovec",
1451 | ]
1452 | 
1453 | [[package]]
1454 | name = "icu_normalizer"
1455 | version = "2.1.1"
1456 | source = "registry+https://github.com/rust-lang/crates.io-index"
1457 | checksum = "5f6c8828b67bf8908d82127b2054ea1b4427ff0230ee9141c54251934ab1b599"
1458 | dependencies = [
1459 |  "icu_collections",
1460 |  "icu_normalizer_data",
1461 |  "icu_properties",
1462 |  "icu_provider",
1463 |  "smallvec",
1464 |  "zerovec",
1465 | ]
1466 | 
1467 | [[package]]
1468 | name = "icu_normalizer_data"
1469 | version = "2.1.1"
1470 | source = "registry+https://github.com/rust-lang/crates.io-index"
1471 | checksum = "7aedcccd01fc5fe81e6b489c15b247b8b0690feb23304303a9e560f37efc560a"
1472 | 
1473 | [[package]]
1474 | name = "icu_properties"
1475 | version = "2.1.2"
1476 | source = "registry+https://github.com/rust-lang/crates.io-index"
1477 | checksum = "020bfc02fe870ec3a66d93e677ccca0562506e5872c650f893269e08615d74ec"
1478 | dependencies = [
1479 |  "icu_collections",
1480 |  "icu_locale_core",
1481 |  "icu_properties_data",
1482 |  "icu_provider",
1483 |  "zerotrie",
1484 |  "zerovec",
1485 | ]
1486 | 
1487 | [[package]]
1488 | name = "icu_properties_data"
1489 | version = "2.1.2"
1490 | source = "registry+https://github.com/rust-lang/crates.io-index"
1491 | checksum = "616c294cf8d725c6afcd8f55abc17c56464ef6211f9ed59cccffe534129c77af"
1492 | 
1493 | [[package]]
1494 | name = "icu_provider"
1495 | version = "2.1.1"
1496 | source = "registry+https://github.com/rust-lang/crates.io-index"
1497 | checksum = "85962cf0ce02e1e0a629cc34e7ca3e373ce20dda4c4d7294bbd0bf1fdb59e614"
1498 | dependencies = [
1499 |  "displaydoc",
1500 |  "icu_locale_core",
1501 |  "writeable",
1502 |  "yoke",
1503 |  "zerofrom",
1504 |  "zerotrie",
1505 |  "zerovec",
1506 | ]
1507 | 
1508 | [[package]]
1509 | name = "id-arena"
1510 | version = "2.3.0"
1511 | source = "registry+https://github.com/rust-lang/crates.io-index"
1512 | checksum = "3d3067d79b975e8844ca9eb072e16b31c3c1c36928edf9c6789548c524d0d954"
1513 | 
1514 | [[package]]
1515 | name = "idna"
1516 | version = "1.1.0"
1517 | source = "registry+https://github.com/rust-lang/crates.io-index"
1518 | checksum = "3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de"
1519 | dependencies = [
1520 |  "idna_adapter",
1521 |  "smallvec",
1522 |  "utf8_iter",
1523 | ]
1524 | 
1525 | [[package]]
1526 | name = "idna_adapter"
1527 | version = "1.2.1"
1528 | source = "registry+https://github.com/rust-lang/crates.io-index"
1529 | checksum = "3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344"
1530 | dependencies = [
1531 |  "icu_normalizer",
1532 |  "icu_properties",
1533 | ]
1534 | 
1535 | [[package]]
1536 | name = "image"
1537 | version = "0.25.9"
1538 | source = "registry+https://github.com/rust-lang/crates.io-index"
1539 | checksum = "e6506c6c10786659413faa717ceebcb8f70731c0a60cbae39795fdf114519c1a"
1540 | dependencies = [
1541 |  "bytemuck",
1542 |  "byteorder-lite",
1543 |  "color_quant",
1544 |  "gif",
1545 |  "image-webp",
1546 |  "moxcms",
1547 |  "num-traits",
1548 |  "png",
1549 |  "tiff",
1550 |  "zune-core 0.5.1",
1551 |  "zune-jpeg 0.5.12",
1552 | ]
1553 | 
1554 | [[package]]
1555 | name = "image-webp"
1556 | version = "0.2.4"
1557 | source = "registry+https://github.com/rust-lang/crates.io-index"
1558 | checksum = "525e9ff3e1a4be2fbea1fdf0e98686a6d98b4d8f937e1bf7402245af1909e8c3"
1559 | dependencies = [
1560 |  "byteorder-lite",
1561 |  "quick-error",
1562 | ]
1563 | 
1564 | [[package]]
1565 | name = "imagesize"
1566 | version = "0.14.0"
1567 | source = "registry+https://github.com/rust-lang/crates.io-index"
1568 | checksum = "09e54e57b4c48b40f7aec75635392b12b3421fa26fe8b4332e63138ed278459c"
1569 | 
1570 | [[package]]
1571 | name = "indexmap"
1572 | version = "2.13.0"
1573 | source = "registry+https://github.com/rust-lang/crates.io-index"
1574 | checksum = "7714e70437a7dc3ac8eb7e6f8df75fd8eb422675fc7678aff7364301092b1017"
1575 | dependencies = [
1576 |  "equivalent",
1577 |  "hashbrown 0.16.1",
1578 |  "serde",
1579 |  "serde_core",
1580 | ]
1581 | 
1582 | [[package]]
1583 | name = "inotify"
1584 | version = "0.10.2"
1585 | source = "registry+https://github.com/rust-lang/crates.io-index"
1586 | checksum = "fdd168d97690d0b8c412d6b6c10360277f4d7ee495c5d0d5d5fe0854923255cc"
1587 | dependencies = [
1588 |  "bitflags 1.3.2",
1589 |  "inotify-sys",
1590 |  "libc",
1591 | ]
1592 | 
1593 | [[package]]
1594 | name = "inotify-sys"
1595 | version = "0.1.5"
1596 | source = "registry+https://github.com/rust-lang/crates.io-index"
1597 | checksum = "e05c02b5e89bff3b946cedeca278abc628fe811e604f027c45a8aa3cf793d0eb"
1598 | dependencies = [
1599 |  "libc",
1600 | ]
1601 | 
1602 | [[package]]
1603 | name = "instant"
1604 | version = "0.1.13"
1605 | source = "registry+https://github.com/rust-lang/crates.io-index"
1606 | checksum = "e0242819d153cba4b4b05a5a8f2a7e9bbf97b6055b2a002b395c96b5ff3c0222"
1607 | dependencies = [
1608 |  "cfg-if",
1609 | ]
1610 | 
1611 | [[package]]
1612 | name = "itertools"
1613 | version = "0.14.0"
1614 | source = "registry+https://github.com/rust-lang/crates.io-index"
1615 | checksum = "2b192c782037fadd9cfa75548310488aabdbf3d2da73885b31bd0abd03351285"
1616 | dependencies = [
1617 |  "either",
1618 | ]
1619 | 
1620 | [[package]]
1621 | name = "itoa"
1622 | version = "1.0.17"
1623 | source = "registry+https://github.com/rust-lang/crates.io-index"
1624 | checksum = "92ecc6618181def0457392ccd0ee51198e065e016d1d527a7ac1b6dc7c1f09d2"
1625 | 
1626 | [[package]]
1627 | name = "jni"
1628 | version = "0.21.1"
1629 | source = "registry+https://github.com/rust-lang/crates.io-index"
1630 | checksum = "1a87aa2bb7d2af34197c04845522473242e1aa17c12f4935d5856491a7fb8c97"
1631 | dependencies = [
1632 |  "cesu8",
1633 |  "cfg-if",
1634 |  "combine",
1635 |  "jni-sys",
1636 |  "log",
1637 |  "thiserror 1.0.69",
1638 |  "walkdir",
1639 |  "windows-sys 0.45.0",
1640 | ]
1641 | 
1642 | [[package]]
1643 | name = "jni-sys"
1644 | version = "0.3.0"
1645 | source = "registry+https://github.com/rust-lang/crates.io-index"
1646 | checksum = "8eaf4bc02d17cbdd7ff4c7438cafcdf7fb9a4613313ad11b4f8fefe7d3fa0130"
1647 | 
1648 | [[package]]
1649 | name = "jobserver"
1650 | version = "0.1.34"
1651 | source = "registry+https://github.com/rust-lang/crates.io-index"
1652 | checksum = "9afb3de4395d6b3e67a780b6de64b51c978ecf11cb9a462c66be7d4ca9039d33"
1653 | dependencies = [
1654 |  "getrandom 0.3.4",
1655 |  "libc",
1656 | ]
1657 | 
1658 | [[package]]
1659 | name = "js-sys"
1660 | version = "0.3.91"
1661 | source = "registry+https://github.com/rust-lang/crates.io-index"
1662 | checksum = "b49715b7073f385ba4bc528e5747d02e66cb39c6146efb66b781f131f0fb399c"
1663 | dependencies = [
1664 |  "once_cell",
1665 |  "wasm-bindgen",
1666 | ]
1667 | 
1668 | [[package]]
1669 | name = "jxl-bitstream"
1670 | version = "1.1.0"
1671 | source = "registry+https://github.com/rust-lang/crates.io-index"
1672 | checksum = "b480e752277e29eb4054f69546887a9b84656fe78c08f54ba5850ced98a378fe"
1673 | dependencies = [
1674 |  "tracing",
1675 | ]
1676 | 
1677 | [[package]]
1678 | name = "jxl-coding"
1679 | version = "1.0.1"
1680 | source = "registry+https://github.com/rust-lang/crates.io-index"
1681 | checksum = "cd972bcd125e776f1eb241ac50e39f956095a1c2770c64736c968f8946bd9a3c"
1682 | dependencies = [
1683 |  "jxl-bitstream",
1684 |  "tracing",
1685 | ]
1686 | 
1687 | [[package]]
1688 | name = "jxl-color"
1689 | version = "0.11.0"
1690 | source = "registry+https://github.com/rust-lang/crates.io-index"
1691 | checksum = "f316b1358c1711755b3ee8e8cb5c4a1dad12e796233088a7a513440782de80b2"
1692 | dependencies = [
1693 |  "jxl-bitstream",
1694 |  "jxl-coding",
1695 |  "jxl-grid",
1696 |  "jxl-image",
1697 |  "jxl-oxide-common",
1698 |  "jxl-threadpool",
1699 |  "tracing",
1700 | ]
1701 | 
1702 | [[package]]
1703 | name = "jxl-frame"
1704 | version = "0.13.3"
1705 | source = "registry+https://github.com/rust-lang/crates.io-index"
1706 | checksum = "2d967c6fd669c7c01060b5022d8835fa82fd46b06ffc98b549f17600a097c2b3"
1707 | dependencies = [
1708 |  "jxl-bitstream",
1709 |  "jxl-coding",
1710 |  "jxl-grid",
1711 |  "jxl-image",
1712 |  "jxl-modular",
1713 |  "jxl-oxide-common",
1714 |  "jxl-threadpool",
1715 |  "jxl-vardct",
1716 |  "tracing",
1717 | ]
1718 | 
1719 | [[package]]
1720 | name = "jxl-grid"
1721 | version = "0.6.1"
1722 | source = "registry+https://github.com/rust-lang/crates.io-index"
1723 | checksum = "a0e0ef92d5d60e76bf41098e57e985f523185e08fad54268da448637feca6989"
1724 | dependencies = [
1725 |  "tracing",
1726 | ]
1727 | 
1728 | [[package]]
1729 | name = "jxl-image"
1730 | version = "0.13.0"
1731 | source = "registry+https://github.com/rust-lang/crates.io-index"
1732 | checksum = "c5f752d62577c702a94dbbce4045caf08cb58639e8a4d56464b40ecf33ffe565"
1733 | dependencies = [
1734 |  "jxl-bitstream",
1735 |  "jxl-grid",
1736 |  "jxl-oxide-common",
1737 |  "tracing",
1738 | ]
1739 | 
1740 | [[package]]
1741 | name = "jxl-jbr"
1742 | version = "0.2.1"
1743 | source = "registry+https://github.com/rust-lang/crates.io-index"
1744 | checksum = "e35d032bcec660647828527ff42c6f5776d2fd44b8357f9f6d9ac6dc07218e46"
1745 | dependencies = [
1746 |  "brotli-decompressor",
1747 |  "jxl-bitstream",
1748 |  "jxl-frame",
1749 |  "jxl-grid",
1750 |  "jxl-image",
1751 |  "jxl-modular",
1752 |  "jxl-oxide-common",
1753 |  "jxl-threadpool",
1754 |  "jxl-vardct",
1755 |  "tracing",
1756 | ]
1757 | 
1758 | [[package]]
1759 | name = "jxl-modular"
1760 | version = "0.11.2"
1761 | source = "registry+https://github.com/rust-lang/crates.io-index"
1762 | checksum = "da758b2f989aafd9eeb39489fe43d7be5a3a0d2ad61cf1bad705eb6990a6053c"
1763 | dependencies = [
1764 |  "jxl-bitstream",
1765 |  "jxl-coding",
1766 |  "jxl-grid",
1767 |  "jxl-oxide-common",
1768 |  "jxl-threadpool",
1769 |  "tracing",
1770 | ]
1771 | 
1772 | [[package]]
1773 | name = "jxl-oxide"
1774 | version = "0.12.5"
1775 | source = "registry+https://github.com/rust-lang/crates.io-index"
1776 | checksum = "ee8ecd2678ed70c1eda42b811ccb2e25ab836edeb18e7f1178c1f917ed36b772"
1777 | dependencies = [
1778 |  "brotli-decompressor",
1779 |  "jxl-bitstream",
1780 |  "jxl-color",
1781 |  "jxl-frame",
1782 |  "jxl-grid",
1783 |  "jxl-image",
1784 |  "jxl-jbr",
1785 |  "jxl-oxide-common",
1786 |  "jxl-render",
1787 |  "jxl-threadpool",
1788 |  "tracing",
1789 | ]
1790 | 
1791 | [[package]]
1792 | name = "jxl-oxide-common"
1793 | version = "1.0.0"
1794 | source = "registry+https://github.com/rust-lang/crates.io-index"
1795 | checksum = "b62394c5021b3a9e7e0dbb2d639d555d019090c9946c39f6d3b09d390db4157b"
1796 | dependencies = [
1797 |  "jxl-bitstream",
1798 | ]
1799 | 
1800 | [[package]]
1801 | name = "jxl-render"
1802 | version = "0.12.3"
1803 | source = "registry+https://github.com/rust-lang/crates.io-index"
1804 | checksum = "aa0c3100918bd3c41bb0f8ce1f4f1664e48f3032ff8eeab0d6a2cfc3276f462d"
1805 | dependencies = [
1806 |  "bytemuck",
1807 |  "jxl-bitstream",
1808 |  "jxl-coding",
1809 |  "jxl-color",
1810 |  "jxl-frame",
1811 |  "jxl-grid",
1812 |  "jxl-image",
1813 |  "jxl-modular",
1814 |  "jxl-oxide-common",
1815 |  "jxl-threadpool",
1816 |  "jxl-vardct",
1817 |  "tracing",
1818 | ]
1819 | 
1820 | [[package]]
1821 | name = "jxl-threadpool"
1822 | version = "1.0.0"
1823 | source = "registry+https://github.com/rust-lang/crates.io-index"
1824 | checksum = "25f15eb830aa77a7f21148d72e153562a26bfe570139bd4922eab1908dd499d3"
1825 | dependencies = [
1826 |  "rayon",
1827 |  "rayon-core",
1828 |  "tracing",
1829 | ]
1830 | 
1831 | [[package]]
1832 | name = "jxl-vardct"
1833 | version = "0.11.1"
1834 | source = "registry+https://github.com/rust-lang/crates.io-index"
1835 | checksum = "ce72a18c6d3a47172ab6c479be2bdb56f22066b5d7092663f03b4490820b4511"
1836 | dependencies = [
1837 |  "jxl-bitstream",
1838 |  "jxl-coding",
1839 |  "jxl-grid",
1840 |  "jxl-modular",
1841 |  "jxl-oxide-common",
1842 |  "jxl-threadpool",
1843 |  "tracing",
1844 | ]
1845 | 
1846 | [[package]]
1847 | name = "kamadak-exif"
1848 | version = "0.5.5"
1849 | source = "registry+https://github.com/rust-lang/crates.io-index"
1850 | checksum = "ef4fc70d0ab7e5b6bafa30216a6b48705ea964cdfc29c050f2412295eba58077"
1851 | dependencies = [
1852 |  "mutate_once",
1853 | ]
1854 | 
1855 | [[package]]
1856 | name = "khronos-egl"
1857 | version = "6.0.0"
1858 | source = "registry+https://github.com/rust-lang/crates.io-index"
1859 | checksum = "6aae1df220ece3c0ada96b8153459b67eebe9ae9212258bb0134ae60416fdf76"
1860 | dependencies = [
1861 |  "libc",
1862 |  "libloading",
1863 |  "pkg-config",
1864 | ]
1865 | 
1866 | [[package]]
1867 | name = "khronos_api"
1868 | version = "3.1.0"
1869 | source = "registry+https://github.com/rust-lang/crates.io-index"
1870 | checksum = "e2db585e1d738fc771bf08a151420d3ed193d9d895a36df7f6f8a9456b911ddc"
1871 | 
1872 | [[package]]
1873 | name = "kqueue"
1874 | version = "1.1.1"
1875 | source = "registry+https://github.com/rust-lang/crates.io-index"
1876 | checksum = "eac30106d7dce88daf4a3fcb4879ea939476d5074a9b7ddd0fb97fa4bed5596a"
1877 | dependencies = [
1878 |  "kqueue-sys",
1879 |  "libc",
1880 | ]
1881 | 
1882 | [[package]]
1883 | name = "kqueue-sys"
1884 | version = "1.0.4"
1885 | source = "registry+https://github.com/rust-lang/crates.io-index"
1886 | checksum = "ed9625ffda8729b85e45cf04090035ac368927b8cebc34898e7c120f52e4838b"
1887 | dependencies = [
1888 |  "bitflags 1.3.2",
1889 |  "libc",
1890 | ]
1891 | 
1892 | [[package]]
1893 | name = "kurbo"
1894 | version = "0.13.0"
1895 | source = "registry+https://github.com/rust-lang/crates.io-index"
1896 | checksum = "7564e90fe3c0d5771e1f0bc95322b21baaeaa0d9213fa6a0b61c99f8b17b3bfb"
1897 | dependencies = [
1898 |  "arrayvec",
1899 |  "euclid",
1900 |  "smallvec",
1901 | ]
1902 | 
1903 | [[package]]
1904 | name = "lazy_static"
1905 | version = "1.5.0"
1906 | source = "registry+https://github.com/rust-lang/crates.io-index"
1907 | checksum = "bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe"
1908 | 
1909 | [[package]]
1910 | name = "leb128fmt"
1911 | version = "0.1.0"
1912 | source = "registry+https://github.com/rust-lang/crates.io-index"
1913 | checksum = "09edd9e8b54e49e587e4f6295a7d29c3ea94d469cb40ab8ca70b288248a81db2"
1914 | 
1915 | [[package]]
1916 | name = "libc"
1917 | version = "0.2.182"
1918 | source = "registry+https://github.com/rust-lang/crates.io-index"
1919 | checksum = "6800badb6cb2082ffd7b6a67e6125bb39f18782f793520caee8cb8846be06112"
1920 | 
1921 | [[package]]
1922 | name = "libflate"
1923 | version = "2.2.1"
1924 | source = "registry+https://github.com/rust-lang/crates.io-index"
1925 | checksum = "e3248b8d211bd23a104a42d81b4fa8bb8ac4a3b75e7a43d85d2c9ccb6179cd74"
1926 | dependencies = [
1927 |  "adler32",
1928 |  "core2",
1929 |  "crc32fast",
1930 |  "dary_heap",
1931 |  "libflate_lz77",
1932 | ]
1933 | 
1934 | [[package]]
1935 | name = "libflate_lz77"
1936 | version = "2.2.0"
1937 | source = "registry+https://github.com/rust-lang/crates.io-index"
1938 | checksum = "a599cb10a9cd92b1300debcef28da8f70b935ec937f44fcd1b70a7c986a11c5c"
1939 | dependencies = [
1940 |  "core2",
1941 |  "hashbrown 0.16.1",
1942 |  "rle-decode-fast",
1943 | ]
1944 | 
1945 | [[package]]
1946 | name = "libloading"
1947 | version = "0.8.9"
1948 | source = "registry+https://github.com/rust-lang/crates.io-index"
1949 | checksum = "d7c4b02199fee7c5d21a5ae7d8cfa79a6ef5bb2fc834d6e9058e89c825efdc55"
1950 | dependencies = [
1951 |  "cfg-if",
1952 |  "windows-link",
1953 | ]
1954 | 
1955 | [[package]]
1956 | name = "libm"
1957 | version = "0.2.16"
1958 | source = "registry+https://github.com/rust-lang/crates.io-index"
1959 | checksum = "b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981"
1960 | 
1961 | [[package]]
1962 | name = "libredox"
1963 | version = "0.1.14"
1964 | source = "registry+https://github.com/rust-lang/crates.io-index"
1965 | checksum = "1744e39d1d6a9948f4f388969627434e31128196de472883b39f148769bfe30a"
1966 | dependencies = [
1967 |  "bitflags 2.11.0",
1968 |  "libc",
1969 |  "plain",
1970 |  "redox_syscall 0.7.3",
1971 | ]
1972 | 
1973 | [[package]]
1974 | name = "linked-hash-map"
1975 | version = "0.5.6"
1976 | source = "registry+https://github.com/rust-lang/crates.io-index"
1977 | checksum = "0717cef1bc8b636c6e1c1bbdefc09e6322da8a9321966e8928ef80d20f7f770f"
1978 | 
1979 | [[package]]
1980 | name = "linux-raw-sys"
1981 | version = "0.4.15"
1982 | source = "registry+https://github.com/rust-lang/crates.io-index"
1983 | checksum = "d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab"
1984 | 
1985 | [[package]]
1986 | name = "linux-raw-sys"
1987 | version = "0.12.1"
1988 | source = "registry+https://github.com/rust-lang/crates.io-index"
1989 | checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"
1990 | 
1991 | [[package]]
1992 | name = "litemap"
1993 | version = "0.8.1"
1994 | source = "registry+https://github.com/rust-lang/crates.io-index"
1995 | checksum = "6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77"
1996 | 
1997 | [[package]]
1998 | name = "litrs"
1999 | version = "1.0.0"
2000 | source = "registry+https://github.com/rust-lang/crates.io-index"
2001 | checksum = "11d3d7f243d5c5a8b9bb5d6dd2b1602c0cb0b9db1621bafc7ed66e35ff9fe092"
2002 | 
2003 | [[package]]
2004 | name = "lock_api"
2005 | version = "0.4.14"
2006 | source = "registry+https://github.com/rust-lang/crates.io-index"
2007 | checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
2008 | dependencies = [
2009 |  "scopeguard",
2010 | ]
2011 | 
2012 | [[package]]
2013 | name = "log"
2014 | version = "0.4.29"
2015 | source = "registry+https://github.com/rust-lang/crates.io-index"
2016 | checksum = "5e5032e24019045c762d3c0f28f5b6b8bbf38563a65908389bf7978758920897"
2017 | 
2018 | [[package]]
2019 | name = "lru"
2020 | version = "0.12.5"
2021 | source = "registry+https://github.com/rust-lang/crates.io-index"
2022 | checksum = "234cf4f4a04dc1f57e24b96cc0cd600cf2af460d4161ac5ecdd0af8e1f3b2a38"
2023 | dependencies = [
2024 |  "hashbrown 0.15.5",
2025 | ]
2026 | 
2027 | [[package]]
2028 | name = "malloc_buf"
2029 | version = "0.0.6"
2030 | source = "registry+https://github.com/rust-lang/crates.io-index"
2031 | checksum = "62bb907fe88d54d8d9ce32a3cceab4218ed2f6b7d35617cafe9adf84e43919cb"
2032 | dependencies = [
2033 |  "libc",
2034 | ]
2035 | 
2036 | [[package]]
2037 | name = "matchers"
2038 | version = "0.2.0"
2039 | source = "registry+https://github.com/rust-lang/crates.io-index"
2040 | checksum = "d1525a2a28c7f4fa0fc98bb91ae755d1e2d1505079e05539e35bc876b5d65ae9"
2041 | dependencies = [
2042 |  "regex-automata",
2043 | ]
2044 | 
2045 | [[package]]
2046 | name = "md5"
2047 | version = "0.8.0"
2048 | source = "registry+https://github.com/rust-lang/crates.io-index"
2049 | checksum = "ae960838283323069879657ca3de837e9f7bbb4c7bf6ea7f1b290d5e9476d2e0"
2050 | 
2051 | [[package]]
2052 | name = "memchr"
2053 | version = "2.8.0"
2054 | source = "registry+https://github.com/rust-lang/crates.io-index"
2055 | checksum = "f8ca58f447f06ed17d5fc4043ce1b10dd205e060fb3ce5b979b8ed8e59ff3f79"
2056 | 
2057 | [[package]]
2058 | name = "memmap2"
2059 | version = "0.9.10"
2060 | source = "registry+https://github.com/rust-lang/crates.io-index"
2061 | checksum = "714098028fe011992e1c3962653c96b2d578c4b4bce9036e15ff220319b1e0e3"
2062 | dependencies = [
2063 |  "libc",
2064 | ]
2065 | 
2066 | [[package]]
2067 | name = "memoffset"
2068 | version = "0.9.1"
2069 | source = "registry+https://github.com/rust-lang/crates.io-index"
2070 | checksum = "488016bfae457b036d996092f6cb448677611ce4449e970ceaf42695203f218a"
2071 | dependencies = [
2072 |  "autocfg",
2073 | ]
2074 | 
2075 | [[package]]
2076 | name = "metal"
2077 | version = "0.32.0"
2078 | source = "registry+https://github.com/rust-lang/crates.io-index"
2079 | checksum = "00c15a6f673ff72ddcc22394663290f870fb224c1bfce55734a75c414150e605"
2080 | dependencies = [
2081 |  "bitflags 2.11.0",
2082 |  "block",
2083 |  "core-graphics-types 0.2.0",
2084 |  "foreign-types",
2085 |  "log",
2086 |  "objc",
2087 |  "paste",
2088 | ]
2089 | 
2090 | [[package]]
2091 | name = "miniz_oxide"
2092 | version = "0.8.9"
2093 | source = "registry+https://github.com/rust-lang/crates.io-index"
2094 | checksum = "1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316"
2095 | dependencies = [
2096 |  "adler2",
2097 |  "simd-adler32",
2098 | ]
2099 | 
2100 | [[package]]
2101 | name = "mio"
2102 | version = "1.1.1"
2103 | source = "registry+https://github.com/rust-lang/crates.io-index"
2104 | checksum = "a69bcab0ad47271a0234d9422b131806bf3968021e5dc9328caf2d4cd58557fc"
2105 | dependencies = [
2106 |  "libc",
2107 |  "log",
2108 |  "wasi",
2109 |  "windows-sys 0.61.2",
2110 | ]
2111 | 
2112 | [[package]]
2113 | name = "moxcms"
2114 | version = "0.7.11"
2115 | source = "registry+https://github.com/rust-lang/crates.io-index"
2116 | checksum = "ac9557c559cd6fc9867e122e20d2cbefc9ca29d80d027a8e39310920ed2f0a97"
2117 | dependencies = [
2118 |  "num-traits",
2119 |  "pxfm",
2120 | ]
2121 | 
2122 | [[package]]
2123 | name = "multiversion"
2124 | version = "0.8.0"
2125 | source = "registry+https://github.com/rust-lang/crates.io-index"
2126 | checksum = "7edb7f0ff51249dfda9ab96b5823695e15a052dc15074c9dbf3d118afaf2c201"
2127 | dependencies = [
2128 |  "multiversion-macros",
2129 |  "target-features",
2130 | ]
2131 | 
2132 | [[package]]
2133 | name = "multiversion-macros"
2134 | version = "0.8.0"
2135 | source = "registry+https://github.com/rust-lang/crates.io-index"
2136 | checksum = "b093064383341eb3271f42e381cb8f10a01459478446953953c75d24bd339fc0"
2137 | dependencies = [
2138 |  "proc-macro2",
2139 |  "quote",
2140 |  "syn",
2141 |  "target-features",
2142 | ]
2143 | 
2144 | [[package]]
2145 | name = "mutate_once"
2146 | version = "0.1.2"
2147 | source = "registry+https://github.com/rust-lang/crates.io-index"
2148 | checksum = "13d2233c9842d08cfe13f9eac96e207ca6a2ea10b80259ebe8ad0268be27d2af"
2149 | 
2150 | [[package]]
2151 | name = "naga"
2152 | version = "26.0.0"
2153 | source = "registry+https://github.com/rust-lang/crates.io-index"
2154 | checksum = "916cbc7cb27db60be930a4e2da243cf4bc39569195f22fd8ee419cd31d5b662c"
2155 | dependencies = [
2156 |  "arrayvec",
2157 |  "bit-set",
2158 |  "bitflags 2.11.0",
2159 |  "cfg-if",
2160 |  "cfg_aliases",
2161 |  "codespan-reporting",
2162 |  "half",
2163 |  "hashbrown 0.15.5",
2164 |  "hexf-parse",
2165 |  "indexmap",
2166 |  "libm",
2167 |  "log",
2168 |  "num-traits",
2169 |  "once_cell",
2170 |  "rustc-hash 1.1.0",
2171 |  "spirv",
2172 |  "thiserror 2.0.18",
2173 |  "unicode-ident",
2174 | ]
2175 | 
2176 | [[package]]
2177 | name = "ndk"
2178 | version = "0.9.0"
2179 | source = "registry+https://github.com/rust-lang/crates.io-index"
2180 | checksum = "c3f42e7bbe13d351b6bead8286a43aac9534b82bd3cc43e47037f012ebfd62d4"
2181 | dependencies = [
2182 |  "bitflags 2.11.0",
2183 |  "jni-sys",
2184 |  "log",
2185 |  "ndk-sys",
2186 |  "num_enum",
2187 |  "raw-window-handle",
2188 |  "thiserror 1.0.69",
2189 | ]
2190 | 
2191 | [[package]]
2192 | name = "ndk-context"
2193 | version = "0.1.1"
2194 | source = "registry+https://github.com/rust-lang/crates.io-index"
2195 | checksum = "27b02d87554356db9e9a873add8782d4ea6e3e58ea071a9adb9a2e8ddb884a8b"
2196 | 
2197 | [[package]]
2198 | name = "ndk-sys"
2199 | version = "0.6.0+11769913"
2200 | source = "registry+https://github.com/rust-lang/crates.io-index"
2201 | checksum = "ee6cda3051665f1fb8d9e08fc35c96d5a244fb1be711a03b71118828afc9a873"
2202 | dependencies = [
2203 |  "jni-sys",
2204 | ]
2205 | 
2206 | [[package]]
2207 | name = "nix"
2208 | version = "0.29.0"
2209 | source = "registry+https://github.com/rust-lang/crates.io-index"
2210 | checksum = "71e2746dc3a24dd78b3cfcb7be93368c6de9963d30f43a6a73998a9cf4b17b46"
2211 | dependencies = [
2212 |  "bitflags 2.11.0",
2213 |  "cfg-if",
2214 |  "cfg_aliases",
2215 |  "libc",
2216 |  "memoffset",
2217 | ]
2218 | 
2219 | [[package]]
2220 | name = "notify"
2221 | version = "7.0.0"
2222 | source = "registry+https://github.com/rust-lang/crates.io-index"
2223 | checksum = "c533b4c39709f9ba5005d8002048266593c1cfaf3c5f0739d5b8ab0c6c504009"
2224 | dependencies = [
2225 |  "bitflags 2.11.0",
2226 |  "filetime",
2227 |  "fsevent-sys",
2228 |  "inotify",
2229 |  "kqueue",
2230 |  "libc",
2231 |  "log",
2232 |  "mio",
2233 |  "notify-types",
2234 |  "walkdir",
2235 |  "windows-sys 0.52.0",
2236 | ]
2237 | 
2238 | [[package]]
2239 | name = "notify-types"
2240 | version = "1.0.1"
2241 | source = "registry+https://github.com/rust-lang/crates.io-index"
2242 | checksum = "585d3cb5e12e01aed9e8a1f70d5c6b5e86fe2a6e48fc8cd0b3e0b8df6f6eb174"
2243 | dependencies = [
2244 |  "instant",
2245 | ]
2246 | 
2247 | [[package]]
2248 | name = "nu-ansi-term"
2249 | version = "0.50.3"
2250 | source = "registry+https://github.com/rust-lang/crates.io-index"
2251 | checksum = "7957b9740744892f114936ab4a57b3f487491bbeafaf8083688b16841a4240e5"
2252 | dependencies = [
2253 |  "windows-sys 0.61.2",
2254 | ]
2255 | 
2256 | [[package]]
2257 | name = "num"
2258 | version = "0.4.3"
2259 | source = "registry+https://github.com/rust-lang/crates.io-index"
2260 | checksum = "35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23"
2261 | dependencies = [
2262 |  "num-bigint",
2263 |  "num-complex",
2264 |  "num-integer",
2265 |  "num-iter",
2266 |  "num-rational",
2267 |  "num-traits",
2268 | ]
2269 | 
2270 | [[package]]
2271 | name = "num-bigint"
2272 | version = "0.4.6"
2273 | source = "registry+https://github.com/rust-lang/crates.io-index"
2274 | checksum = "a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9"
2275 | dependencies = [
2276 |  "num-integer",
2277 |  "num-traits",
2278 | ]
2279 | 
2280 | [[package]]
2281 | name = "num-complex"
2282 | version = "0.4.6"
2283 | source = "registry+https://github.com/rust-lang/crates.io-index"
2284 | checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
2285 | dependencies = [
2286 |  "num-traits",
2287 | ]
2288 | 
2289 | [[package]]
2290 | name = "num-integer"
2291 | version = "0.1.46"
2292 | source = "registry+https://github.com/rust-lang/crates.io-index"
2293 | checksum = "7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f"
2294 | dependencies = [
2295 |  "num-traits",
2296 | ]
2297 | 
2298 | [[package]]
2299 | name = "num-iter"
2300 | version = "0.1.45"
2301 | source = "registry+https://github.com/rust-lang/crates.io-index"
2302 | checksum = "1429034a0490724d0075ebb2bc9e875d6503c3cf69e235a8941aa757d83ef5bf"
2303 | dependencies = [
2304 |  "autocfg",
2305 |  "num-integer",
2306 |  "num-traits",
2307 | ]
2308 | 
2309 | [[package]]
2310 | name = "num-rational"
2311 | version = "0.4.2"
2312 | source = "registry+https://github.com/rust-lang/crates.io-index"
2313 | checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824"
2314 | dependencies = [
2315 |  "num-bigint",
2316 |  "num-integer",
2317 |  "num-traits",
2318 | ]
2319 | 
2320 | [[package]]
2321 | name = "num-traits"
2322 | version = "0.2.19"
2323 | source = "registry+https://github.com/rust-lang/crates.io-index"
2324 | checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
2325 | dependencies = [
2326 |  "autocfg",
2327 |  "libm",
2328 | ]
2329 | 
2330 | [[package]]
2331 | name = "num_enum"
2332 | version = "0.7.5"
2333 | source = "registry+https://github.com/rust-lang/crates.io-index"
2334 | checksum = "b1207a7e20ad57b847bbddc6776b968420d38292bbfe2089accff5e19e82454c"
2335 | dependencies = [
2336 |  "num_enum_derive",
2337 |  "rustversion",
2338 | ]
2339 | 
2340 | [[package]]
2341 | name = "num_enum_derive"
2342 | version = "0.7.5"
2343 | source = "registry+https://github.com/rust-lang/crates.io-index"
2344 | checksum = "ff32365de1b6743cb203b710788263c44a03de03802daf96092f2da4fe6ba4d7"
2345 | dependencies = [
2346 |  "proc-macro-crate",
2347 |  "proc-macro2",
2348 |  "quote",
2349 |  "syn",
2350 | ]
2351 | 
2352 | [[package]]
2353 | name = "objc"
2354 | version = "0.2.7"
2355 | source = "registry+https://github.com/rust-lang/crates.io-index"
2356 | checksum = "915b1b472bc21c53464d6c8461c9d3af805ba1ef837e1cac254428f4a77177b1"
2357 | dependencies = [
2358 |  "malloc_buf",
2359 | ]
2360 | 
2361 | [[package]]
2362 | name = "objc-foundation"
2363 | version = "0.1.1"
2364 | source = "registry+https://github.com/rust-lang/crates.io-index"
2365 | checksum = "1add1b659e36c9607c7aab864a76c7a4c2760cd0cd2e120f3fb8b952c7e22bf9"
2366 | dependencies = [
2367 |  "block",
2368 |  "objc",
2369 |  "objc_id",
2370 | ]
2371 | 
2372 | [[package]]
2373 | name = "objc-sys"
2374 | version = "0.3.5"
2375 | source = "registry+https://github.com/rust-lang/crates.io-index"
2376 | checksum = "cdb91bdd390c7ce1a8607f35f3ca7151b65afc0ff5ff3b34fa350f7d7c7e4310"
2377 | 
2378 | [[package]]
2379 | name = "objc2"
2380 | version = "0.5.2"
2381 | source = "registry+https://github.com/rust-lang/crates.io-index"
2382 | checksum = "46a785d4eeff09c14c487497c162e92766fbb3e4059a71840cecc03d9a50b804"
2383 | dependencies = [
2384 |  "objc-sys",
2385 |  "objc2-encode",
2386 | ]
2387 | 
2388 | [[package]]
2389 | name = "objc2-app-kit"
2390 | version = "0.2.2"
2391 | source = "registry+https://github.com/rust-lang/crates.io-index"
2392 | checksum = "e4e89ad9e3d7d297152b17d39ed92cd50ca8063a89a9fa569046d41568891eff"
2393 | dependencies = [
2394 |  "bitflags 2.11.0",
2395 |  "block2",
2396 |  "libc",
2397 |  "objc2",
2398 |  "objc2-core-data",
2399 |  "objc2-core-image",
2400 |  "objc2-foundation",
2401 |  "objc2-quartz-core",
2402 | ]
2403 | 
2404 | [[package]]
2405 | name = "objc2-cloud-kit"
2406 | version = "0.2.2"
2407 | source = "registry+https://github.com/rust-lang/crates.io-index"
2408 | checksum = "74dd3b56391c7a0596a295029734d3c1c5e7e510a4cb30245f8221ccea96b009"
2409 | dependencies = [
2410 |  "bitflags 2.11.0",
2411 |  "block2",
2412 |  "objc2",
2413 |  "objc2-core-location",
2414 |  "objc2-foundation",
2415 | ]
2416 | 
2417 | [[package]]
2418 | name = "objc2-contacts"
2419 | version = "0.2.2"
2420 | source = "registry+https://github.com/rust-lang/crates.io-index"
2421 | checksum = "a5ff520e9c33812fd374d8deecef01d4a840e7b41862d849513de77e44aa4889"
2422 | dependencies = [
2423 |  "block2",
2424 |  "objc2",
2425 |  "objc2-foundation",
2426 | ]
2427 | 
2428 | [[package]]
2429 | name = "objc2-core-data"
2430 | version = "0.2.2"
2431 | source = "registry+https://github.com/rust-lang/crates.io-index"
2432 | checksum = "617fbf49e071c178c0b24c080767db52958f716d9eabdf0890523aeae54773ef"
2433 | dependencies = [
2434 |  "bitflags 2.11.0",
2435 |  "block2",
2436 |  "objc2",
2437 |  "objc2-foundation",
2438 | ]
2439 | 
2440 | [[package]]
2441 | name = "objc2-core-image"
2442 | version = "0.2.2"
2443 | source = "registry+https://github.com/rust-lang/crates.io-index"
2444 | checksum = "55260963a527c99f1819c4f8e3b47fe04f9650694ef348ffd2227e8196d34c80"
2445 | dependencies = [
2446 |  "block2",
2447 |  "objc2",
2448 |  "objc2-foundation",
2449 |  "objc2-metal",
2450 | ]
2451 | 
2452 | [[package]]
2453 | name = "objc2-core-location"
2454 | version = "0.2.2"
2455 | source = "registry+https://github.com/rust-lang/crates.io-index"
2456 | checksum = "000cfee34e683244f284252ee206a27953279d370e309649dc3ee317b37e5781"
2457 | dependencies = [
2458 |  "block2",
2459 |  "objc2",
2460 |  "objc2-contacts",
2461 |  "objc2-foundation",
2462 | ]
2463 | 
2464 | [[package]]
2465 | name = "objc2-encode"
2466 | version = "4.1.0"
2467 | source = "registry+https://github.com/rust-lang/crates.io-index"
2468 | checksum = "ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33"
2469 | 
2470 | [[package]]
2471 | name = "objc2-foundation"
2472 | version = "0.2.2"
2473 | source = "registry+https://github.com/rust-lang/crates.io-index"
2474 | checksum = "0ee638a5da3799329310ad4cfa62fbf045d5f56e3ef5ba4149e7452dcf89d5a8"
2475 | dependencies = [
2476 |  "bitflags 2.11.0",
2477 |  "block2",
2478 |  "dispatch",
2479 |  "libc",
2480 |  "objc2",
2481 | ]
2482 | 
2483 | [[package]]
2484 | name = "objc2-link-presentation"
2485 | version = "0.2.2"
2486 | source = "registry+https://github.com/rust-lang/crates.io-index"
2487 | checksum = "a1a1ae721c5e35be65f01a03b6d2ac13a54cb4fa70d8a5da293d7b0020261398"
2488 | dependencies = [
2489 |  "block2",
2490 |  "objc2",
2491 |  "objc2-app-kit",
2492 |  "objc2-foundation",
2493 | ]
2494 | 
2495 | [[package]]
2496 | name = "objc2-metal"
2497 | version = "0.2.2"
2498 | source = "registry+https://github.com/rust-lang/crates.io-index"
2499 | checksum = "dd0cba1276f6023976a406a14ffa85e1fdd19df6b0f737b063b95f6c8c7aadd6"
2500 | dependencies = [
2501 |  "bitflags 2.11.0",
2502 |  "block2",
2503 |  "objc2",
2504 |  "objc2-foundation",
2505 | ]
2506 | 
2507 | [[package]]
2508 | name = "objc2-quartz-core"
2509 | version = "0.2.2"
2510 | source = "registry+https://github.com/rust-lang/crates.io-index"
2511 | checksum = "e42bee7bff906b14b167da2bac5efe6b6a07e6f7c0a21a7308d40c960242dc7a"
2512 | dependencies = [
2513 |  "bitflags 2.11.0",
2514 |  "block2",
2515 |  "objc2",
2516 |  "objc2-foundation",
2517 |  "objc2-metal",
2518 | ]
2519 | 
2520 | [[package]]
2521 | name = "objc2-symbols"
2522 | version = "0.2.2"
2523 | source = "registry+https://github.com/rust-lang/crates.io-index"
2524 | checksum = "0a684efe3dec1b305badae1a28f6555f6ddd3bb2c2267896782858d5a78404dc"
2525 | dependencies = [
2526 |  "objc2",
2527 |  "objc2-foundation",
2528 | ]
2529 | 
2530 | [[package]]
2531 | name = "objc2-ui-kit"
2532 | version = "0.2.2"
2533 | source = "registry+https://github.com/rust-lang/crates.io-index"
2534 | checksum = "b8bb46798b20cd6b91cbd113524c490f1686f4c4e8f49502431415f3512e2b6f"
2535 | dependencies = [
2536 |  "bitflags 2.11.0",
2537 |  "block2",
2538 |  "objc2",
2539 |  "objc2-cloud-kit",
2540 |  "objc2-core-data",
2541 |  "objc2-core-image",
2542 |  "objc2-core-location",
2543 |  "objc2-foundation",
2544 |  "objc2-link-presentation",
2545 |  "objc2-quartz-core",
2546 |  "objc2-symbols",
2547 |  "objc2-uniform-type-identifiers",
2548 |  "objc2-user-notifications",
2549 | ]
2550 | 
2551 | [[package]]
2552 | name = "objc2-uniform-type-identifiers"
2553 | version = "0.2.2"
2554 | source = "registry+https://github.com/rust-lang/crates.io-index"
2555 | checksum = "44fa5f9748dbfe1ca6c0b79ad20725a11eca7c2218bceb4b005cb1be26273bfe"
2556 | dependencies = [
2557 |  "block2",
2558 |  "objc2",
2559 |  "objc2-foundation",
2560 | ]
2561 | 
2562 | [[package]]
2563 | name = "objc2-user-notifications"
2564 | version = "0.2.2"
2565 | source = "registry+https://github.com/rust-lang/crates.io-index"
2566 | checksum = "76cfcbf642358e8689af64cee815d139339f3ed8ad05103ed5eaf73db8d84cb3"
2567 | dependencies = [
2568 |  "bitflags 2.11.0",
2569 |  "block2",
2570 |  "objc2",
2571 |  "objc2-core-location",
2572 |  "objc2-foundation",
2573 | ]
2574 | 
2575 | [[package]]
2576 | name = "objc_id"
2577 | version = "0.1.1"
2578 | source = "registry+https://github.com/rust-lang/crates.io-index"
2579 | checksum = "c92d4ddb4bd7b50d730c215ff871754d0da6b2178849f8a2a2ab69712d0c073b"
2580 | dependencies = [
2581 |  "objc",
2582 | ]
2583 | 
2584 | [[package]]
2585 | name = "object"
2586 | version = "0.37.3"
2587 | source = "registry+https://github.com/rust-lang/crates.io-index"
2588 | checksum = "ff76201f031d8863c38aa7f905eca4f53abbfa15f609db4277d44cd8938f33fe"
2589 | dependencies = [
2590 |  "memchr",
2591 | ]
2592 | 
2593 | [[package]]
2594 | name = "once_cell"
2595 | version = "1.21.3"
2596 | source = "registry+https://github.com/rust-lang/crates.io-index"
2597 | checksum = "42f5e15c9953c5e4ccceeb2e7382a716482c34515315f7b03532b8b4e8393d2d"
2598 | 
2599 | [[package]]
2600 | name = "orbclient"
2601 | version = "0.3.50"
2602 | source = "registry+https://github.com/rust-lang/crates.io-index"
2603 | checksum = "52ad2c6bae700b7aa5d1cc30c59bdd3a1c180b09dbaea51e2ae2b8e1cf211fdd"
2604 | dependencies = [
2605 |  "libc",
2606 |  "libredox",
2607 | ]
2608 | 
2609 | [[package]]
2610 | name = "ordered-float"
2611 | version = "4.6.0"
2612 | source = "registry+https://github.com/rust-lang/crates.io-index"
2613 | checksum = "7bb71e1b3fa6ca1c61f383464aaf2bb0e2f8e772a1f01d486832464de363b951"
2614 | dependencies = [
2615 |  "num-traits",
2616 | ]
2617 | 
2618 | [[package]]
2619 | name = "ordered-float"
2620 | version = "5.1.0"
2621 | source = "registry+https://github.com/rust-lang/crates.io-index"
2622 | checksum = "7f4779c6901a562440c3786d08192c6fbda7c1c2060edd10006b05ee35d10f2d"
2623 | dependencies = [
2624 |  "num-traits",
2625 | ]
2626 | 
2627 | [[package]]
2628 | name = "ordered-stream"
2629 | version = "0.2.0"
2630 | source = "registry+https://github.com/rust-lang/crates.io-index"
2631 | checksum = "9aa2b01e1d916879f73a53d01d1d6cee68adbb31d6d9177a8cfce093cced1d50"
2632 | dependencies = [
2633 |  "futures-core",
2634 |  "pin-project-lite",
2635 | ]
2636 | 
2637 | [[package]]
2638 | name = "owned_ttf_parser"
2639 | version = "0.25.1"
2640 | source = "registry+https://github.com/rust-lang/crates.io-index"
2641 | checksum = "36820e9051aca1014ddc75770aab4d68bc1e9e632f0f5627c4086bc216fb583b"
2642 | dependencies = [
2643 |  "ttf-parser",
2644 | ]
2645 | 
2646 | [[package]]
2647 | name = "parking"
2648 | version = "2.2.1"
2649 | source = "registry+https://github.com/rust-lang/crates.io-index"
2650 | checksum = "f38d5652c16fde515bb1ecef450ab0f6a219d619a7274976324d5e377f7dceba"
2651 | 
2652 | [[package]]
2653 | name = "parking_lot"
2654 | version = "0.12.5"
2655 | source = "registry+https://github.com/rust-lang/crates.io-index"
2656 | checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
2657 | dependencies = [
2658 |  "lock_api",
2659 |  "parking_lot_core",
2660 | ]
2661 | 
2662 | [[package]]
2663 | name = "parking_lot_core"
2664 | version = "0.9.12"
2665 | source = "registry+https://github.com/rust-lang/crates.io-index"
2666 | checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
2667 | dependencies = [
2668 |  "cfg-if",
2669 |  "libc",
2670 |  "redox_syscall 0.5.18",
2671 |  "smallvec",
2672 |  "windows-link",
2673 | ]
2674 | 
2675 | [[package]]
2676 | name = "paste"
2677 | version = "1.0.15"
2678 | source = "registry+https://github.com/rust-lang/crates.io-index"
2679 | checksum = "57c0d7b74b563b49d38dae00a0c37d4d6de9b432382b2892f0574ddcae73fd0a"
2680 | 
2681 | [[package]]
2682 | name = "percent-encoding"
2683 | version = "2.3.2"
2684 | source = "registry+https://github.com/rust-lang/crates.io-index"
2685 | checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"
2686 | 
2687 | [[package]]
2688 | name = "pico-args"
2689 | version = "0.5.0"
2690 | source = "registry+https://github.com/rust-lang/crates.io-index"
2691 | checksum = "5be167a7af36ee22fe3115051bc51f6e6c7054c9348e28deb4f49bd6f705a315"
2692 | 
2693 | [[package]]
2694 | name = "pin-project"
2695 | version = "1.1.11"
2696 | source = "registry+https://github.com/rust-lang/crates.io-index"
2697 | checksum = "f1749c7ed4bcaf4c3d0a3efc28538844fb29bcdd7d2b67b2be7e20ba861ff517"
2698 | dependencies = [
2699 |  "pin-project-internal",
2700 | ]
2701 | 
2702 | [[package]]
2703 | name = "pin-project-internal"
2704 | version = "1.1.11"
2705 | source = "registry+https://github.com/rust-lang/crates.io-index"
2706 | checksum = "d9b20ed30f105399776b9c883e68e536ef602a16ae6f596d2c473591d6ad64c6"
2707 | dependencies = [
2708 |  "proc-macro2",
2709 |  "quote",
2710 |  "syn",
2711 | ]
2712 | 
2713 | [[package]]
2714 | name = "pin-project-lite"
2715 | version = "0.2.17"
2716 | source = "registry+https://github.com/rust-lang/crates.io-index"
2717 | checksum = "a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"
2718 | 
2719 | [[package]]
2720 | name = "piper"
2721 | version = "0.2.5"
2722 | source = "registry+https://github.com/rust-lang/crates.io-index"
2723 | checksum = "c835479a4443ded371d6c535cbfd8d31ad92c5d23ae9770a61bc155e4992a3c1"
2724 | dependencies = [
2725 |  "atomic-waker",
2726 |  "fastrand",
2727 |  "futures-io",
2728 | ]
2729 | 
2730 | [[package]]
2731 | name = "pkg-config"
2732 | version = "0.3.32"
2733 | source = "registry+https://github.com/rust-lang/crates.io-index"
2734 | checksum = "7edddbd0b52d732b21ad9a5fab5c704c14cd949e5e9a1ec5929a24fded1b904c"
2735 | 
2736 | [[package]]
2737 | name = "plain"
2738 | version = "0.2.3"
2739 | source = "registry+https://github.com/rust-lang/crates.io-index"
2740 | checksum = "b4596b6d070b27117e987119b4dac604f3c58cfb0b191112e24771b2faeac1a6"
2741 | 
2742 | [[package]]
2743 | name = "png"
2744 | version = "0.18.1"
2745 | source = "registry+https://github.com/rust-lang/crates.io-index"
2746 | checksum = "60769b8b31b2a9f263dae2776c37b1b28ae246943cf719eb6946a1db05128a61"
2747 | dependencies = [
2748 |  "bitflags 2.11.0",
2749 |  "crc32fast",
2750 |  "fdeflate",
2751 |  "flate2",
2752 |  "miniz_oxide",
2753 | ]
2754 | 
2755 | [[package]]
2756 | name = "polling"
2757 | version = "3.11.0"
2758 | source = "registry+https://github.com/rust-lang/crates.io-index"
2759 | checksum = "5d0e4f59085d47d8241c88ead0f274e8a0cb551f3625263c05eb8dd897c34218"
2760 | dependencies = [
2761 |  "cfg-if",
2762 |  "concurrent-queue",
2763 |  "hermit-abi",
2764 |  "pin-project-lite",
2765 |  "rustix 1.1.4",
2766 |  "windows-sys 0.61.2",
2767 | ]
2768 | 
2769 | [[package]]
2770 | name = "pollster"
2771 | version = "0.3.0"
2772 | source = "registry+https://github.com/rust-lang/crates.io-index"
2773 | checksum = "22686f4785f02a4fcc856d3b3bb19bf6c8160d103f7a99cc258bddd0251dc7f2"
2774 | 
2775 | [[package]]
2776 | name = "portable-atomic"
2777 | version = "1.13.1"
2778 | source = "registry+https://github.com/rust-lang/crates.io-index"
2779 | checksum = "c33a9471896f1c69cecef8d20cbe2f7accd12527ce60845ff44c153bb2a21b49"
2780 | 
2781 | [[package]]
2782 | name = "portable-atomic-util"
2783 | version = "0.2.5"
2784 | source = "registry+https://github.com/rust-lang/crates.io-index"
2785 | checksum = "7a9db96d7fa8782dd8c15ce32ffe8680bbd1e978a43bf51a34d39483540495f5"
2786 | dependencies = [
2787 |  "portable-atomic",
2788 | ]
2789 | 
2790 | [[package]]
2791 | name = "potential_utf"
2792 | version = "0.1.4"
2793 | source = "registry+https://github.com/rust-lang/crates.io-index"
2794 | checksum = "b73949432f5e2a09657003c25bca5e19a0e9c84f8058ca374f49e0ebe605af77"
2795 | dependencies = [
2796 |  "zerovec",
2797 | ]
2798 | 
2799 | [[package]]
2800 | name = "ppv-lite86"
2801 | version = "0.2.21"
2802 | source = "registry+https://github.com/rust-lang/crates.io-index"
2803 | checksum = "85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9"
2804 | dependencies = [
2805 |  "zerocopy",
2806 | ]
2807 | 
2808 | [[package]]
2809 | name = "presser"
2810 | version = "0.3.1"
2811 | source = "registry+https://github.com/rust-lang/crates.io-index"
2812 | checksum = "e8cf8e6a8aa66ce33f63993ffc4ea4271eb5b0530a9002db8455ea6050c77bfa"
2813 | 
2814 | [[package]]
2815 | name = "prettyplease"
2816 | version = "0.2.37"
2817 | source = "registry+https://github.com/rust-lang/crates.io-index"
2818 | checksum = "479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b"
2819 | dependencies = [
2820 |  "proc-macro2",
2821 |  "syn",
2822 | ]
2823 | 
2824 | [[package]]
2825 | name = "proc-macro-crate"
2826 | version = "3.4.0"
2827 | source = "registry+https://github.com/rust-lang/crates.io-index"
2828 | checksum = "219cb19e96be00ab2e37d6e299658a0cfa83e52429179969b0f0121b4ac46983"
2829 | dependencies = [
2830 |  "toml_edit 0.23.10+spec-1.0.0",
2831 | ]
2832 | 
2833 | [[package]]
2834 | name = "proc-macro2"
2835 | version = "1.0.106"
2836 | source = "registry+https://github.com/rust-lang/crates.io-index"
2837 | checksum = "8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934"
2838 | dependencies = [
2839 |  "unicode-ident",
2840 | ]
2841 | 
2842 | [[package]]
2843 | name = "profiling"
2844 | version = "1.0.17"
2845 | source = "registry+https://github.com/rust-lang/crates.io-index"
2846 | checksum = "3eb8486b569e12e2c32ad3e204dbaba5e4b5b216e9367044f25f1dba42341773"
2847 | 
2848 | [[package]]
2849 | name = "pxfm"
2850 | version = "0.1.28"
2851 | source = "registry+https://github.com/rust-lang/crates.io-index"
2852 | checksum = "b5a041e753da8b807c9255f28de81879c78c876392ff2469cde94799b2896b9d"
2853 | 
2854 | [[package]]
2855 | name = "quick-error"
2856 | version = "2.0.1"
2857 | source = "registry+https://github.com/rust-lang/crates.io-index"
2858 | checksum = "a993555f31e5a609f617c12db6250dedcac1b0a85076912c436e6fc9b2c8e6a3"
2859 | 
2860 | [[package]]
2861 | name = "quick-xml"
2862 | version = "0.38.4"
2863 | source = "registry+https://github.com/rust-lang/crates.io-index"
2864 | checksum = "b66c2058c55a409d601666cffe35f04333cf1013010882cec174a7467cd4e21c"
2865 | dependencies = [
2866 |  "memchr",
2867 | ]
2868 | 
2869 | [[package]]
2870 | name = "quote"
2871 | version = "1.0.44"
2872 | source = "registry+https://github.com/rust-lang/crates.io-index"
2873 | checksum = "21b2ebcf727b7760c461f091f9f0f539b77b8e87f2fd88131e7f1b433b3cece4"
2874 | dependencies = [
2875 |  "proc-macro2",
2876 | ]
2877 | 
2878 | [[package]]
2879 | name = "r-efi"
2880 | version = "5.3.0"
2881 | source = "registry+https://github.com/rust-lang/crates.io-index"
2882 | checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"
2883 | 
2884 | [[package]]
2885 | name = "rand"
2886 | version = "0.8.5"
2887 | source = "registry+https://github.com/rust-lang/crates.io-index"
2888 | checksum = "34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404"
2889 | dependencies = [
2890 |  "libc",
2891 |  "rand_chacha 0.3.1",
2892 |  "rand_core 0.6.4",
2893 | ]
2894 | 
2895 | [[package]]
2896 | name = "rand"
2897 | version = "0.9.2"
2898 | source = "registry+https://github.com/rust-lang/crates.io-index"
2899 | checksum = "6db2770f06117d490610c7488547d543617b21bfa07796d7a12f6f1bd53850d1"
2900 | dependencies = [
2901 |  "rand_chacha 0.9.0",
2902 |  "rand_core 0.9.5",
2903 | ]
2904 | 
2905 | [[package]]
2906 | name = "rand_chacha"
2907 | version = "0.3.1"
2908 | source = "registry+https://github.com/rust-lang/crates.io-index"
2909 | checksum = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88"
2910 | dependencies = [
2911 |  "ppv-lite86",
2912 |  "rand_core 0.6.4",
2913 | ]
2914 | 
2915 | [[package]]
2916 | name = "rand_chacha"
2917 | version = "0.9.0"
2918 | source = "registry+https://github.com/rust-lang/crates.io-index"
2919 | checksum = "d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb"
2920 | dependencies = [
2921 |  "ppv-lite86",
2922 |  "rand_core 0.9.5",
2923 | ]
2924 | 
2925 | [[package]]
2926 | name = "rand_core"
2927 | version = "0.6.4"
2928 | source = "registry+https://github.com/rust-lang/crates.io-index"
2929 | checksum = "ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c"
2930 | dependencies = [
2931 |  "getrandom 0.2.17",
2932 | ]
2933 | 
2934 | [[package]]
2935 | name = "rand_core"
2936 | version = "0.9.5"
2937 | source = "registry+https://github.com/rust-lang/crates.io-index"
2938 | checksum = "76afc826de14238e6e8c374ddcc1fa19e374fd8dd986b0d2af0d02377261d83c"
2939 | dependencies = [
2940 |  "getrandom 0.3.4",
2941 | ]
2942 | 
2943 | [[package]]
2944 | name = "range-alloc"
2945 | version = "0.1.5"
2946 | source = "registry+https://github.com/rust-lang/crates.io-index"
2947 | checksum = "ca45419789ae5a7899559e9512e58ca889e41f04f1f2445e9f4b290ceccd1d08"
2948 | 
2949 | [[package]]
2950 | name = "raw-window-handle"
2951 | version = "0.6.2"
2952 | source = "registry+https://github.com/rust-lang/crates.io-index"
2953 | checksum = "20675572f6f24e9e76ef639bc5552774ed45f1c30e2951e1e99c59888861c539"
2954 | 
2955 | [[package]]
2956 | name = "rawler"
2957 | version = "0.7.2"
2958 | source = "registry+https://github.com/rust-lang/crates.io-index"
2959 | checksum = "04f4cc35c23969a4a834e0b117c7da41ace812eb9053b5effc3fc5c77d114677"
2960 | dependencies = [
2961 |  "backtrace",
2962 |  "bitstream-io",
2963 |  "byteorder",
2964 |  "chrono",
2965 |  "enumn",
2966 |  "glob",
2967 |  "hex",
2968 |  "image",
2969 |  "itertools",
2970 |  "jxl-oxide",
2971 |  "lazy_static",
2972 |  "libflate",
2973 |  "log",
2974 |  "md5",
2975 |  "memmap2",
2976 |  "multiversion",
2977 |  "num",
2978 |  "num_enum",
2979 |  "rayon",
2980 |  "rustc_version",
2981 |  "serde",
2982 |  "thiserror 2.0.18",
2983 |  "toml",
2984 |  "uuid",
2985 |  "weezl",
2986 |  "zerocopy",
2987 | ]
2988 | 
2989 | [[package]]
2990 | name = "rayon"
2991 | version = "1.11.0"
2992 | source = "registry+https://github.com/rust-lang/crates.io-index"
2993 | checksum = "368f01d005bf8fd9b1206fb6fa653e6c4a81ceb1466406b81792d87c5677a58f"
2994 | dependencies = [
2995 |  "either",
2996 |  "rayon-core",
2997 | ]
2998 | 
2999 | [[package]]
3000 | name = "rayon-core"
3001 | version = "1.13.0"
3002 | source = "registry+https://github.com/rust-lang/crates.io-index"
3003 | checksum = "22e18b0f0062d30d4230b2e85ff77fdfe4326feb054b9783a3460d8435c8ab91"
3004 | dependencies = [
3005 |  "crossbeam-deque",
3006 |  "crossbeam-utils",
3007 | ]
3008 | 
3009 | [[package]]
3010 | name = "redox_syscall"
3011 | version = "0.4.1"
3012 | source = "registry+https://github.com/rust-lang/crates.io-index"
3013 | checksum = "4722d768eff46b75989dd134e5c353f0d6296e5aaa3132e776cbdb56be7731aa"
3014 | dependencies = [
3015 |  "bitflags 1.3.2",
3016 | ]
3017 | 
3018 | [[package]]
3019 | name = "redox_syscall"
3020 | version = "0.5.18"
3021 | source = "registry+https://github.com/rust-lang/crates.io-index"
3022 | checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
3023 | dependencies = [
3024 |  "bitflags 2.11.0",
3025 | ]
3026 | 
3027 | [[package]]
3028 | name = "redox_syscall"
3029 | version = "0.7.3"
3030 | source = "registry+https://github.com/rust-lang/crates.io-index"
3031 | checksum = "6ce70a74e890531977d37e532c34d45e9055d2409ed08ddba14529471ed0be16"
3032 | dependencies = [
3033 |  "bitflags 2.11.0",
3034 | ]
3035 | 
3036 | [[package]]
3037 | name = "regex-automata"
3038 | version = "0.4.14"
3039 | source = "registry+https://github.com/rust-lang/crates.io-index"
3040 | checksum = "6e1dd4122fc1595e8162618945476892eefca7b88c52820e74af6262213cae8f"
3041 | dependencies = [
3042 |  "aho-corasick",
3043 |  "memchr",
3044 |  "regex-syntax",
3045 | ]
3046 | 
3047 | [[package]]
3048 | name = "regex-syntax"
3049 | version = "0.8.10"
3050 | source = "registry+https://github.com/rust-lang/crates.io-index"
3051 | checksum = "dc897dd8d9e8bd1ed8cdad82b5966c3e0ecae09fb1907d58efaa013543185d0a"
3052 | 
3053 | [[package]]
3054 | name = "renderdoc-sys"
3055 | version = "1.1.0"
3056 | source = "registry+https://github.com/rust-lang/crates.io-index"
3057 | checksum = "19b30a45b0cd0bcca8037f3d0dc3421eaf95327a17cad11964fb8179b4fc4832"
3058 | 
3059 | [[package]]
3060 | name = "resvg"
3061 | version = "0.47.0"
3062 | source = "registry+https://github.com/rust-lang/crates.io-index"
3063 | checksum = "9be183ad6a216aa96f33e4c8033b0988b8b3ea6fd2359d19af5bac4643fd8e81"
3064 | dependencies = [
3065 |  "gif",
3066 |  "image-webp",
3067 |  "log",
3068 |  "pico-args",
3069 |  "rgb",
3070 |  "svgtypes",
3071 |  "tiny-skia 0.12.0",
3072 |  "usvg",
3073 |  "zune-jpeg 0.5.12",
3074 | ]
3075 | 
3076 | [[package]]
3077 | name = "rfd"
3078 | version = "0.14.1"
3079 | source = "registry+https://github.com/rust-lang/crates.io-index"
3080 | checksum = "25a73a7337fc24366edfca76ec521f51877b114e42dab584008209cca6719251"
3081 | dependencies = [
3082 |  "ashpd",
3083 |  "block",
3084 |  "dispatch",
3085 |  "js-sys",
3086 |  "log",
3087 |  "objc",
3088 |  "objc-foundation",
3089 |  "objc_id",
3090 |  "pollster",
3091 |  "raw-window-handle",
3092 |  "urlencoding",
3093 |  "wasm-bindgen",
3094 |  "wasm-bindgen-futures",
3095 |  "web-sys",
3096 |  "windows-sys 0.48.0",
3097 | ]
3098 | 
3099 | [[package]]
3100 | name = "rgb"
3101 | version = "0.8.53"
3102 | source = "registry+https://github.com/rust-lang/crates.io-index"
3103 | checksum = "47b34b781b31e5d73e9fbc8689c70551fd1ade9a19e3e28cfec8580a79290cc4"
3104 | dependencies = [
3105 |  "bytemuck",
3106 | ]
3107 | 
3108 | [[package]]
3109 | name = "rle-decode-fast"
3110 | version = "1.0.3"
3111 | source = "registry+https://github.com/rust-lang/crates.io-index"
3112 | checksum = "3582f63211428f83597b51b2ddb88e2a91a9d52d12831f9d08f5e624e8977422"
3113 | 
3114 | [[package]]
3115 | name = "roxmltree"
3116 | version = "0.20.0"
3117 | source = "registry+https://github.com/rust-lang/crates.io-index"
3118 | checksum = "6c20b6793b5c2fa6553b250154b78d6d0db37e72700ae35fad9387a46f487c97"
3119 | 
3120 | [[package]]
3121 | name = "roxmltree"
3122 | version = "0.21.1"
3123 | source = "registry+https://github.com/rust-lang/crates.io-index"
3124 | checksum = "f1964b10c76125c36f8afe190065a4bf9a87bf324842c05701330bba9f1cacbb"
3125 | dependencies = [
3126 |  "memchr",
3127 | ]
3128 | 
3129 | [[package]]
3130 | name = "rustc-demangle"
3131 | version = "0.1.27"
3132 | source = "registry+https://github.com/rust-lang/crates.io-index"
3133 | checksum = "b50b8869d9fc858ce7266cce0194bd74df58b9d0e3f6df3a9fc8eb470d95c09d"
3134 | 
3135 | [[package]]
3136 | name = "rustc-hash"
3137 | version = "1.1.0"
3138 | source = "registry+https://github.com/rust-lang/crates.io-index"
3139 | checksum = "08d43f7aa6b08d49f382cde6a7982047c3426db949b1424bc4b7ec9ae12c6ce2"
3140 | 
3141 | [[package]]
3142 | name = "rustc-hash"
3143 | version = "2.1.1"
3144 | source = "registry+https://github.com/rust-lang/crates.io-index"
3145 | checksum = "357703d41365b4b27c590e3ed91eabb1b663f07c4c084095e60cbed4362dff0d"
3146 | 
3147 | [[package]]
3148 | name = "rustc_version"
3149 | version = "0.4.1"
3150 | source = "registry+https://github.com/rust-lang/crates.io-index"
3151 | checksum = "cfcb3a22ef46e85b45de6ee7e79d063319ebb6594faafcf1c225ea92ab6e9b92"
3152 | dependencies = [
3153 |  "semver",
3154 | ]
3155 | 
3156 | [[package]]
3157 | name = "rustix"
3158 | version = "0.38.44"
3159 | source = "registry+https://github.com/rust-lang/crates.io-index"
3160 | checksum = "fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154"
3161 | dependencies = [
3162 |  "bitflags 2.11.0",
3163 |  "errno",
3164 |  "libc",
3165 |  "linux-raw-sys 0.4.15",
3166 |  "windows-sys 0.59.0",
3167 | ]
3168 | 
3169 | [[package]]
3170 | name = "rustix"
3171 | version = "1.1.4"
3172 | source = "registry+https://github.com/rust-lang/crates.io-index"
3173 | checksum = "b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"
3174 | dependencies = [
3175 |  "bitflags 2.11.0",
3176 |  "errno",
3177 |  "libc",
3178 |  "linux-raw-sys 0.12.1",
3179 |  "windows-sys 0.61.2",
3180 | ]
3181 | 
3182 | [[package]]
3183 | name = "rustversion"
3184 | version = "1.0.22"
3185 | source = "registry+https://github.com/rust-lang/crates.io-index"
3186 | checksum = "b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d"
3187 | 
3188 | [[package]]
3189 | name = "rustybuzz"
3190 | version = "0.20.1"
3191 | source = "registry+https://github.com/rust-lang/crates.io-index"
3192 | checksum = "fd3c7c96f8a08ee34eff8857b11b49b07d71d1c3f4e88f8a88d4c9e9f90b1702"
3193 | dependencies = [
3194 |  "bitflags 2.11.0",
3195 |  "bytemuck",
3196 |  "core_maths",
3197 |  "log",
3198 |  "smallvec",
3199 |  "ttf-parser",
3200 |  "unicode-bidi-mirroring",
3201 |  "unicode-ccc",
3202 |  "unicode-properties",
3203 |  "unicode-script",
3204 | ]
3205 | 
3206 | [[package]]
3207 | name = "same-file"
3208 | version = "1.0.6"
3209 | source = "registry+https://github.com/rust-lang/crates.io-index"
3210 | checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
3211 | dependencies = [
3212 |  "winapi-util",
3213 | ]
3214 | 
3215 | [[package]]
3216 | name = "scoped-tls"
3217 | version = "1.0.1"
3218 | source = "registry+https://github.com/rust-lang/crates.io-index"
3219 | checksum = "e1cf6437eb19a8f4a6cc0f7dca544973b0b78843adbfeb3683d1a94a0024a294"
3220 | 
3221 | [[package]]
3222 | name = "scopeguard"
3223 | version = "1.2.0"
3224 | source = "registry+https://github.com/rust-lang/crates.io-index"
3225 | checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"
3226 | 
3227 | [[package]]
3228 | name = "sctk-adwaita"
3229 | version = "0.10.1"
3230 | source = "registry+https://github.com/rust-lang/crates.io-index"
3231 | checksum = "b6277f0217056f77f1d8f49f2950ac6c278c0d607c45f5ee99328d792ede24ec"
3232 | dependencies = [
3233 |  "ab_glyph",
3234 |  "log",
3235 |  "memmap2",
3236 |  "smithay-client-toolkit",
3237 |  "tiny-skia 0.11.4",
3238 | ]
3239 | 
3240 | [[package]]
3241 | name = "semver"
3242 | version = "1.0.27"
3243 | source = "registry+https://github.com/rust-lang/crates.io-index"
3244 | checksum = "d767eb0aabc880b29956c35734170f26ed551a859dbd361d140cdbeca61ab1e2"
3245 | 
3246 | [[package]]
3247 | name = "serde"
3248 | version = "1.0.228"
3249 | source = "registry+https://github.com/rust-lang/crates.io-index"
3250 | checksum = "9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e"
3251 | dependencies = [
3252 |  "serde_core",
3253 |  "serde_derive",
3254 | ]
3255 | 
3256 | [[package]]
3257 | name = "serde_core"
3258 | version = "1.0.228"
3259 | source = "registry+https://github.com/rust-lang/crates.io-index"
3260 | checksum = "41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad"
3261 | dependencies = [
3262 |  "serde_derive",
3263 | ]
3264 | 
3265 | [[package]]
3266 | name = "serde_derive"
3267 | version = "1.0.228"
3268 | source = "registry+https://github.com/rust-lang/crates.io-index"
3269 | checksum = "d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79"
3270 | dependencies = [
3271 |  "proc-macro2",
3272 |  "quote",
3273 |  "syn",
3274 | ]
3275 | 
3276 | [[package]]
3277 | name = "serde_json"
3278 | version = "1.0.149"
3279 | source = "registry+https://github.com/rust-lang/crates.io-index"
3280 | checksum = "83fc039473c5595ace860d8c4fafa220ff474b3fc6bfdb4293327f1a37e94d86"
3281 | dependencies = [
3282 |  "itoa",
3283 |  "memchr",
3284 |  "serde",
3285 |  "serde_core",
3286 |  "zmij",
3287 | ]
3288 | 
3289 | [[package]]
3290 | name = "serde_repr"
3291 | version = "0.1.20"
3292 | source = "registry+https://github.com/rust-lang/crates.io-index"
3293 | checksum = "175ee3e80ae9982737ca543e96133087cbd9a485eecc3bc4de9c1a37b47ea59c"
3294 | dependencies = [
3295 |  "proc-macro2",
3296 |  "quote",
3297 |  "syn",
3298 | ]
3299 | 
3300 | [[package]]
3301 | name = "serde_spanned"
3302 | version = "0.6.9"
3303 | source = "registry+https://github.com/rust-lang/crates.io-index"
3304 | checksum = "bf41e0cfaf7226dca15e8197172c295a782857fcb97fad1808a166870dee75a3"
3305 | dependencies = [
3306 |  "serde",
3307 | ]
3308 | 
3309 | [[package]]
3310 | name = "sha1"
3311 | version = "0.10.6"
3312 | source = "registry+https://github.com/rust-lang/crates.io-index"
3313 | checksum = "e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba"
3314 | dependencies = [
3315 |  "cfg-if",
3316 |  "cpufeatures",
3317 |  "digest",
3318 | ]
3319 | 
3320 | [[package]]
3321 | name = "sharded-slab"
3322 | version = "0.1.7"
3323 | source = "registry+https://github.com/rust-lang/crates.io-index"
3324 | checksum = "f40ca3c46823713e0d4209592e8d6e826aa57e928f09752619fc696c499637f6"
3325 | dependencies = [
3326 |  "lazy_static",
3327 | ]
3328 | 
3329 | [[package]]
3330 | name = "shlex"
3331 | version = "1.3.0"
3332 | source = "registry+https://github.com/rust-lang/crates.io-index"
3333 | checksum = "0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64"
3334 | 
3335 | [[package]]
3336 | name = "signal-hook-registry"
3337 | version = "1.4.8"
3338 | source = "registry+https://github.com/rust-lang/crates.io-index"
3339 | checksum = "c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b"
3340 | dependencies = [
3341 |  "errno",
3342 |  "libc",
3343 | ]
3344 | 
3345 | [[package]]
3346 | name = "simd-adler32"
3347 | version = "0.3.8"
3348 | source = "registry+https://github.com/rust-lang/crates.io-index"
3349 | checksum = "e320a6c5ad31d271ad523dcf3ad13e2767ad8b1cb8f047f75a8aeaf8da139da2"
3350 | 
3351 | [[package]]
3352 | name = "simplecss"
3353 | version = "0.2.2"
3354 | source = "registry+https://github.com/rust-lang/crates.io-index"
3355 | checksum = "7a9c6883ca9c3c7c90e888de77b7a5c849c779d25d74a1269b0218b14e8b136c"
3356 | dependencies = [
3357 |  "log",
3358 | ]
3359 | 
3360 | [[package]]
3361 | name = "siphasher"
3362 | version = "1.0.2"
3363 | source = "registry+https://github.com/rust-lang/crates.io-index"
3364 | checksum = "b2aa850e253778c88a04c3d7323b043aeda9d3e30d5971937c1855769763678e"
3365 | 
3366 | [[package]]
3367 | name = "slab"
3368 | version = "0.4.12"
3369 | source = "registry+https://github.com/rust-lang/crates.io-index"
3370 | checksum = "0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"
3371 | 
3372 | [[package]]
3373 | name = "slotmap"
3374 | version = "1.1.1"
3375 | source = "registry+https://github.com/rust-lang/crates.io-index"
3376 | checksum = "bdd58c3c93c3d278ca835519292445cb4b0d4dc59ccfdf7ceadaab3f8aeb4038"
3377 | dependencies = [
3378 |  "version_check",
3379 | ]
3380 | 
3381 | [[package]]
3382 | name = "smallvec"
3383 | version = "1.15.1"
3384 | source = "registry+https://github.com/rust-lang/crates.io-index"
3385 | checksum = "67b1b7a3b5fe4f1376887184045fcf45c69e92af734b7aaddc05fb777b6fbd03"
3386 | 
3387 | [[package]]
3388 | name = "smithay-client-toolkit"
3389 | version = "0.19.2"
3390 | source = "registry+https://github.com/rust-lang/crates.io-index"
3391 | checksum = "3457dea1f0eb631b4034d61d4d8c32074caa6cd1ab2d59f2327bd8461e2c0016"
3392 | dependencies = [
3393 |  "bitflags 2.11.0",
3394 |  "calloop",
3395 |  "calloop-wayland-source",
3396 |  "cursor-icon",
3397 |  "libc",
3398 |  "log",
3399 |  "memmap2",
3400 |  "rustix 0.38.44",
3401 |  "thiserror 1.0.69",
3402 |  "wayland-backend",
3403 |  "wayland-client",
3404 |  "wayland-csd-frame",
3405 |  "wayland-cursor",
3406 |  "wayland-protocols",
3407 |  "wayland-protocols-wlr",
3408 |  "wayland-scanner",
3409 |  "xkeysym",
3410 | ]
3411 | 
3412 | [[package]]
3413 | name = "smol_str"
3414 | version = "0.2.2"
3415 | source = "registry+https://github.com/rust-lang/crates.io-index"
3416 | checksum = "dd538fb6910ac1099850255cf94a94df6551fbdd602454387d0adb2d1ca6dead"
3417 | dependencies = [
3418 |  "serde",
3419 | ]
3420 | 
3421 | [[package]]
3422 | name = "spedimage"
3423 | version = "2.0.0"
3424 | dependencies = [
3425 |  "anyhow",
3426 |  "bytemuck",
3427 |  "chrono",
3428 |  "fast_image_resize",
3429 |  "image",
3430 |  "kamadak-exif",
3431 |  "lru",
3432 |  "notify",
3433 |  "pollster",
3434 |  "rawler",
3435 |  "rayon",
3436 |  "resvg",
3437 |  "rfd",
3438 |  "thiserror 2.0.18",
3439 |  "tracing",
3440 |  "tracing-subscriber",
3441 |  "wgpu",
3442 |  "wgpu_glyph",
3443 |  "windows",
3444 |  "winit",
3445 |  "winreg",
3446 | ]
3447 | 
3448 | [[package]]
3449 | name = "spirv"
3450 | version = "0.3.0+sdk-1.3.268.0"
3451 | source = "registry+https://github.com/rust-lang/crates.io-index"
3452 | checksum = "eda41003dc44290527a59b13432d4a0379379fa074b70174882adfbdfd917844"
3453 | dependencies = [
3454 |  "bitflags 2.11.0",
3455 | ]
3456 | 
3457 | [[package]]
3458 | name = "stable_deref_trait"
3459 | version = "1.2.1"
3460 | source = "registry+https://github.com/rust-lang/crates.io-index"
3461 | checksum = "6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596"
3462 | 
3463 | [[package]]
3464 | name = "static_assertions"
3465 | version = "1.1.0"
3466 | source = "registry+https://github.com/rust-lang/crates.io-index"
3467 | checksum = "a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f"
3468 | 
3469 | [[package]]
3470 | name = "strict-num"
3471 | version = "0.1.1"
3472 | source = "registry+https://github.com/rust-lang/crates.io-index"
3473 | checksum = "6637bab7722d379c8b41ba849228d680cc12d0a45ba1fa2b48f2a30577a06731"
3474 | dependencies = [
3475 |  "float-cmp",
3476 | ]
3477 | 
3478 | [[package]]
3479 | name = "svgtypes"
3480 | version = "0.16.1"
3481 | source = "registry+https://github.com/rust-lang/crates.io-index"
3482 | checksum = "695b5790b3131dafa99b3bbfd25a216edb3d216dad9ca208d4657bfb8f2abc3d"
3483 | dependencies = [
3484 |  "kurbo",
3485 |  "siphasher",
3486 | ]
3487 | 
3488 | [[package]]
3489 | name = "syn"
3490 | version = "2.0.117"
3491 | source = "registry+https://github.com/rust-lang/crates.io-index"
3492 | checksum = "e665b8803e7b1d2a727f4023456bbbbe74da67099c585258af0ad9c5013b9b99"
3493 | dependencies = [
3494 |  "proc-macro2",
3495 |  "quote",
3496 |  "unicode-ident",
3497 | ]
3498 | 
3499 | [[package]]
3500 | name = "synstructure"
3501 | version = "0.13.2"
3502 | source = "registry+https://github.com/rust-lang/crates.io-index"
3503 | checksum = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
3504 | dependencies = [
3505 |  "proc-macro2",
3506 |  "quote",
3507 |  "syn",
3508 | ]
3509 | 
3510 | [[package]]
3511 | name = "target-features"
3512 | version = "0.1.6"
3513 | source = "registry+https://github.com/rust-lang/crates.io-index"
3514 | checksum = "c1bbb9f3c5c463a01705937a24fdabc5047929ac764b2d5b9cf681c1f5041ed5"
3515 | 
3516 | [[package]]
3517 | name = "tempfile"
3518 | version = "3.26.0"
3519 | source = "registry+https://github.com/rust-lang/crates.io-index"
3520 | checksum = "82a72c767771b47409d2345987fda8628641887d5466101319899796367354a0"
3521 | dependencies = [
3522 |  "fastrand",
3523 |  "getrandom 0.4.1",
3524 |  "once_cell",
3525 |  "rustix 1.1.4",
3526 |  "windows-sys 0.61.2",
3527 | ]
3528 | 
3529 | [[package]]
3530 | name = "termcolor"
3531 | version = "1.4.1"
3532 | source = "registry+https://github.com/rust-lang/crates.io-index"
3533 | checksum = "06794f8f6c5c898b3275aebefa6b8a1cb24cd2c6c79397ab15774837a0bc5755"
3534 | dependencies = [
3535 |  "winapi-util",
3536 | ]
3537 | 
3538 | [[package]]
3539 | name = "thiserror"
3540 | version = "1.0.69"
3541 | source = "registry+https://github.com/rust-lang/crates.io-index"
3542 | checksum = "b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52"
3543 | dependencies = [
3544 |  "thiserror-impl 1.0.69",
3545 | ]
3546 | 
3547 | [[package]]
3548 | name = "thiserror"
3549 | version = "2.0.18"
3550 | source = "registry+https://github.com/rust-lang/crates.io-index"
3551 | checksum = "4288b5bcbc7920c07a1149a35cf9590a2aa808e0bc1eafaade0b80947865fbc4"
3552 | dependencies = [
3553 |  "thiserror-impl 2.0.18",
3554 | ]
3555 | 
3556 | [[package]]
3557 | name = "thiserror-impl"
3558 | version = "1.0.69"
3559 | source = "registry+https://github.com/rust-lang/crates.io-index"
3560 | checksum = "4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1"
3561 | dependencies = [
3562 |  "proc-macro2",
3563 |  "quote",
3564 |  "syn",
3565 | ]
3566 | 
3567 | [[package]]
3568 | name = "thiserror-impl"
3569 | version = "2.0.18"
3570 | source = "registry+https://github.com/rust-lang/crates.io-index"
3571 | checksum = "ebc4ee7f67670e9b64d05fa4253e753e016c6c95ff35b89b7941d6b856dec1d5"
3572 | dependencies = [
3573 |  "proc-macro2",
3574 |  "quote",
3575 |  "syn",
3576 | ]
3577 | 
3578 | [[package]]
3579 | name = "thread_local"
3580 | version = "1.1.9"
3581 | source = "registry+https://github.com/rust-lang/crates.io-index"
3582 | checksum = "f60246a4944f24f6e018aa17cdeffb7818b76356965d03b07d6a9886e8962185"
3583 | dependencies = [
3584 |  "cfg-if",
3585 | ]
3586 | 
3587 | [[package]]
3588 | name = "tiff"
3589 | version = "0.10.3"
3590 | source = "registry+https://github.com/rust-lang/crates.io-index"
3591 | checksum = "af9605de7fee8d9551863fd692cce7637f548dbd9db9180fcc07ccc6d26c336f"
3592 | dependencies = [
3593 |  "fax",
3594 |  "flate2",
3595 |  "half",
3596 |  "quick-error",
3597 |  "weezl",
3598 |  "zune-jpeg 0.4.21",
3599 | ]
3600 | 
3601 | [[package]]
3602 | name = "tiny-skia"
3603 | version = "0.11.4"
3604 | source = "registry+https://github.com/rust-lang/crates.io-index"
3605 | checksum = "83d13394d44dae3207b52a326c0c85a8bf87f1541f23b0d143811088497b09ab"
3606 | dependencies = [
3607 |  "arrayref",
3608 |  "arrayvec",
3609 |  "bytemuck",
3610 |  "cfg-if",
3611 |  "log",
3612 |  "tiny-skia-path 0.11.4",
3613 | ]
3614 | 
3615 | [[package]]
3616 | name = "tiny-skia"
3617 | version = "0.12.0"
3618 | source = "registry+https://github.com/rust-lang/crates.io-index"
3619 | checksum = "47ffee5eaaf5527f630fb0e356b90ebdec84d5d18d937c5e440350f88c5a91ea"
3620 | dependencies = [
3621 |  "arrayref",
3622 |  "arrayvec",
3623 |  "bytemuck",
3624 |  "cfg-if",
3625 |  "log",
3626 |  "png",
3627 |  "tiny-skia-path 0.12.0",
3628 | ]
3629 | 
3630 | [[package]]
3631 | name = "tiny-skia-path"
3632 | version = "0.11.4"
3633 | source = "registry+https://github.com/rust-lang/crates.io-index"
3634 | checksum = "9c9e7fc0c2e86a30b117d0462aa261b72b7a99b7ebd7deb3a14ceda95c5bdc93"
3635 | dependencies = [
3636 |  "arrayref",
3637 |  "bytemuck",
3638 |  "strict-num",
3639 | ]
3640 | 
3641 | [[package]]
3642 | name = "tiny-skia-path"
3643 | version = "0.12.0"
3644 | source = "registry+https://github.com/rust-lang/crates.io-index"
3645 | checksum = "edca365c3faccca67d06593c5980fa6c57687de727a03131735bb85f01fdeeb9"
3646 | dependencies = [
3647 |  "arrayref",
3648 |  "bytemuck",
3649 |  "strict-num",
3650 | ]
3651 | 
3652 | [[package]]
3653 | name = "tinystr"
3654 | version = "0.8.2"
3655 | source = "registry+https://github.com/rust-lang/crates.io-index"
3656 | checksum = "42d3e9c45c09de15d06dd8acf5f4e0e399e85927b7f00711024eb7ae10fa4869"
3657 | dependencies = [
3658 |  "displaydoc",
3659 |  "zerovec",
3660 | ]
3661 | 
3662 | [[package]]
3663 | name = "tinyvec"
3664 | version = "1.10.0"
3665 | source = "registry+https://github.com/rust-lang/crates.io-index"
3666 | checksum = "bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa"
3667 | dependencies = [
3668 |  "tinyvec_macros",
3669 | ]
3670 | 
3671 | [[package]]
3672 | name = "tinyvec_macros"
3673 | version = "0.1.1"
3674 | source = "registry+https://github.com/rust-lang/crates.io-index"
3675 | checksum = "1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20"
3676 | 
3677 | [[package]]
3678 | name = "toml"
3679 | version = "0.8.23"
3680 | source = "registry+https://github.com/rust-lang/crates.io-index"
3681 | checksum = "dc1beb996b9d83529a9e75c17a1686767d148d70663143c7854d8b4a09ced362"
3682 | dependencies = [
3683 |  "serde",
3684 |  "serde_spanned",
3685 |  "toml_datetime 0.6.11",
3686 |  "toml_edit 0.22.27",
3687 | ]
3688 | 
3689 | [[package]]
3690 | name = "toml_datetime"
3691 | version = "0.6.11"
3692 | source = "registry+https://github.com/rust-lang/crates.io-index"
3693 | checksum = "22cddaf88f4fbc13c51aebbf5f8eceb5c7c5a9da2ac40a13519eb5b0a0e8f11c"
3694 | dependencies = [
3695 |  "serde",
3696 | ]
3697 | 
3698 | [[package]]
3699 | name = "toml_datetime"
3700 | version = "0.7.5+spec-1.1.0"
3701 | source = "registry+https://github.com/rust-lang/crates.io-index"
3702 | checksum = "92e1cfed4a3038bc5a127e35a2d360f145e1f4b971b551a2ba5fd7aedf7e1347"
3703 | dependencies = [
3704 |  "serde_core",
3705 | ]
3706 | 
3707 | [[package]]
3708 | name = "toml_edit"
3709 | version = "0.22.27"
3710 | source = "registry+https://github.com/rust-lang/crates.io-index"
3711 | checksum = "41fe8c660ae4257887cf66394862d21dbca4a6ddd26f04a3560410406a2f819a"
3712 | dependencies = [
3713 |  "indexmap",
3714 |  "serde",
3715 |  "serde_spanned",
3716 |  "toml_datetime 0.6.11",
3717 |  "toml_write",
3718 |  "winnow",
3719 | ]
3720 | 
3721 | [[package]]
3722 | name = "toml_edit"
3723 | version = "0.23.10+spec-1.0.0"
3724 | source = "registry+https://github.com/rust-lang/crates.io-index"
3725 | checksum = "84c8b9f757e028cee9fa244aea147aab2a9ec09d5325a9b01e0a49730c2b5269"
3726 | dependencies = [
3727 |  "indexmap",
3728 |  "toml_datetime 0.7.5+spec-1.1.0",
3729 |  "toml_parser",
3730 |  "winnow",
3731 | ]
3732 | 
3733 | [[package]]
3734 | name = "toml_parser"
3735 | version = "1.0.9+spec-1.1.0"
3736 | source = "registry+https://github.com/rust-lang/crates.io-index"
3737 | checksum = "702d4415e08923e7e1ef96cd5727c0dfed80b4d2fa25db9647fe5eb6f7c5a4c4"
3738 | dependencies = [
3739 |  "winnow",
3740 | ]
3741 | 
3742 | [[package]]
3743 | name = "toml_write"
3744 | version = "0.1.2"
3745 | source = "registry+https://github.com/rust-lang/crates.io-index"
3746 | checksum = "5d99f8c9a7727884afe522e9bd5edbfc91a3312b36a77b5fb8926e4c31a41801"
3747 | 
3748 | [[package]]
3749 | name = "tracing"
3750 | version = "0.1.44"
3751 | source = "registry+https://github.com/rust-lang/crates.io-index"
3752 | checksum = "63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100"
3753 | dependencies = [
3754 |  "pin-project-lite",
3755 |  "tracing-attributes",
3756 |  "tracing-core",
3757 | ]
3758 | 
3759 | [[package]]
3760 | name = "tracing-attributes"
3761 | version = "0.1.31"
3762 | source = "registry+https://github.com/rust-lang/crates.io-index"
3763 | checksum = "7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da"
3764 | dependencies = [
3765 |  "proc-macro2",
3766 |  "quote",
3767 |  "syn",
3768 | ]
3769 | 
3770 | [[package]]
3771 | name = "tracing-core"
3772 | version = "0.1.36"
3773 | source = "registry+https://github.com/rust-lang/crates.io-index"
3774 | checksum = "db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a"
3775 | dependencies = [
3776 |  "once_cell",
3777 |  "valuable",
3778 | ]
3779 | 
3780 | [[package]]
3781 | name = "tracing-log"
3782 | version = "0.2.0"
3783 | source = "registry+https://github.com/rust-lang/crates.io-index"
3784 | checksum = "ee855f1f400bd0e5c02d150ae5de3840039a3f54b025156404e34c23c03f47c3"
3785 | dependencies = [
3786 |  "log",
3787 |  "once_cell",
3788 |  "tracing-core",
3789 | ]
3790 | 
3791 | [[package]]
3792 | name = "tracing-subscriber"
3793 | version = "0.3.22"
3794 | source = "registry+https://github.com/rust-lang/crates.io-index"
3795 | checksum = "2f30143827ddab0d256fd843b7a66d164e9f271cfa0dde49142c5ca0ca291f1e"
3796 | dependencies = [
3797 |  "matchers",
3798 |  "nu-ansi-term",
3799 |  "once_cell",
3800 |  "regex-automata",
3801 |  "sharded-slab",
3802 |  "smallvec",
3803 |  "thread_local",
3804 |  "tracing",
3805 |  "tracing-core",
3806 |  "tracing-log",
3807 | ]
3808 | 
3809 | [[package]]
3810 | name = "ttf-parser"
3811 | version = "0.25.1"
3812 | source = "registry+https://github.com/rust-lang/crates.io-index"
3813 | checksum = "d2df906b07856748fa3f6e0ad0cbaa047052d4a7dd609e231c4f72cee8c36f31"
3814 | dependencies = [
3815 |  "core_maths",
3816 | ]
3817 | 
3818 | [[package]]
3819 | name = "twox-hash"
3820 | version = "2.1.2"
3821 | source = "registry+https://github.com/rust-lang/crates.io-index"
3822 | checksum = "9ea3136b675547379c4bd395ca6b938e5ad3c3d20fad76e7fe85f9e0d011419c"
3823 | dependencies = [
3824 |  "rand 0.9.2",
3825 | ]
3826 | 
3827 | [[package]]
3828 | name = "typenum"
3829 | version = "1.19.0"
3830 | source = "registry+https://github.com/rust-lang/crates.io-index"
3831 | checksum = "562d481066bde0658276a35467c4af00bdc6ee726305698a55b86e61d7ad82bb"
3832 | 
3833 | [[package]]
3834 | name = "uds_windows"
3835 | version = "1.1.0"
3836 | source = "registry+https://github.com/rust-lang/crates.io-index"
3837 | checksum = "89daebc3e6fd160ac4aa9fc8b3bf71e1f74fbf92367ae71fb83a037e8bf164b9"
3838 | dependencies = [
3839 |  "memoffset",
3840 |  "tempfile",
3841 |  "winapi",
3842 | ]
3843 | 
3844 | [[package]]
3845 | name = "unicode-bidi"
3846 | version = "0.3.18"
3847 | source = "registry+https://github.com/rust-lang/crates.io-index"
3848 | checksum = "5c1cb5db39152898a79168971543b1cb5020dff7fe43c8dc468b0885f5e29df5"
3849 | 
3850 | [[package]]
3851 | name = "unicode-bidi-mirroring"
3852 | version = "0.4.0"
3853 | source = "registry+https://github.com/rust-lang/crates.io-index"
3854 | checksum = "5dfa6e8c60bb66d49db113e0125ee8711b7647b5579dc7f5f19c42357ed039fe"
3855 | 
3856 | [[package]]
3857 | name = "unicode-ccc"
3858 | version = "0.4.0"
3859 | source = "registry+https://github.com/rust-lang/crates.io-index"
3860 | checksum = "ce61d488bcdc9bc8b5d1772c404828b17fc481c0a582b5581e95fb233aef503e"
3861 | 
3862 | [[package]]
3863 | name = "unicode-ident"
3864 | version = "1.0.24"
3865 | source = "registry+https://github.com/rust-lang/crates.io-index"
3866 | checksum = "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75"
3867 | 
3868 | [[package]]
3869 | name = "unicode-properties"
3870 | version = "0.1.4"
3871 | source = "registry+https://github.com/rust-lang/crates.io-index"
3872 | checksum = "7df058c713841ad818f1dc5d3fd88063241cc61f49f5fbea4b951e8cf5a8d71d"
3873 | 
3874 | [[package]]
3875 | name = "unicode-script"
3876 | version = "0.5.8"
3877 | source = "registry+https://github.com/rust-lang/crates.io-index"
3878 | checksum = "383ad40bb927465ec0ce7720e033cb4ca06912855fc35db31b5755d0de75b1ee"
3879 | 
3880 | [[package]]
3881 | name = "unicode-segmentation"
3882 | version = "1.12.0"
3883 | source = "registry+https://github.com/rust-lang/crates.io-index"
3884 | checksum = "f6ccf251212114b54433ec949fd6a7841275f9ada20dddd2f29e9ceea4501493"
3885 | 
3886 | [[package]]
3887 | name = "unicode-vo"
3888 | version = "0.1.0"
3889 | source = "registry+https://github.com/rust-lang/crates.io-index"
3890 | checksum = "b1d386ff53b415b7fe27b50bb44679e2cc4660272694b7b6f3326d8480823a94"
3891 | 
3892 | [[package]]
3893 | name = "unicode-width"
3894 | version = "0.1.14"
3895 | source = "registry+https://github.com/rust-lang/crates.io-index"
3896 | checksum = "7dd6e30e90baa6f72411720665d41d89b9a3d039dc45b8faea1ddd07f617f6af"
3897 | 
3898 | [[package]]
3899 | name = "unicode-xid"
3900 | version = "0.2.6"
3901 | source = "registry+https://github.com/rust-lang/crates.io-index"
3902 | checksum = "ebc1c04c71510c7f702b52b7c350734c9ff1295c464a03335b00bb84fc54f853"
3903 | 
3904 | [[package]]
3905 | name = "url"
3906 | version = "2.5.8"
3907 | source = "registry+https://github.com/rust-lang/crates.io-index"
3908 | checksum = "ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed"
3909 | dependencies = [
3910 |  "form_urlencoded",
3911 |  "idna",
3912 |  "percent-encoding",
3913 |  "serde",
3914 |  "serde_derive",
3915 | ]
3916 | 
3917 | [[package]]
3918 | name = "urlencoding"
3919 | version = "2.1.3"
3920 | source = "registry+https://github.com/rust-lang/crates.io-index"
3921 | checksum = "daf8dba3b7eb870caf1ddeed7bc9d2a049f3cfdfae7cb521b087cc33ae4c49da"
3922 | 
3923 | [[package]]
3924 | name = "usvg"
3925 | version = "0.47.0"
3926 | source = "registry+https://github.com/rust-lang/crates.io-index"
3927 | checksum = "d46cf96c5f498d36b7a9693bc6a7075c0bb9303189d61b2249b0dc3d309c07de"
3928 | dependencies = [
3929 |  "base64",
3930 |  "data-url",
3931 |  "flate2",
3932 |  "fontdb",
3933 |  "imagesize",
3934 |  "kurbo",
3935 |  "log",
3936 |  "pico-args",
3937 |  "roxmltree 0.21.1",
3938 |  "rustybuzz",
3939 |  "simplecss",
3940 |  "siphasher",
3941 |  "strict-num",
3942 |  "svgtypes",
3943 |  "tiny-skia-path 0.12.0",
3944 |  "ttf-parser",
3945 |  "unicode-bidi",
3946 |  "unicode-script",
3947 |  "unicode-vo",
3948 |  "xmlwriter",
3949 | ]
3950 | 
3951 | [[package]]
3952 | name = "utf8_iter"
3953 | version = "1.0.4"
3954 | source = "registry+https://github.com/rust-lang/crates.io-index"
3955 | checksum = "b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be"
3956 | 
3957 | [[package]]
3958 | name = "uuid"
3959 | version = "1.21.0"
3960 | source = "registry+https://github.com/rust-lang/crates.io-index"
3961 | checksum = "b672338555252d43fd2240c714dc444b8c6fb0a5c5335e65a07bba7742735ddb"
3962 | dependencies = [
3963 |  "getrandom 0.4.1",
3964 |  "js-sys",
3965 |  "serde_core",
3966 |  "wasm-bindgen",
3967 | ]
3968 | 
3969 | [[package]]
3970 | name = "valuable"
3971 | version = "0.1.1"
3972 | source = "registry+https://github.com/rust-lang/crates.io-index"
3973 | checksum = "ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65"
3974 | 
3975 | [[package]]
3976 | name = "version_check"
3977 | version = "0.9.5"
3978 | source = "registry+https://github.com/rust-lang/crates.io-index"
3979 | checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"
3980 | 
3981 | [[package]]
3982 | name = "walkdir"
3983 | version = "2.5.0"
3984 | source = "registry+https://github.com/rust-lang/crates.io-index"
3985 | checksum = "29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b"
3986 | dependencies = [
3987 |  "same-file",
3988 |  "winapi-util",
3989 | ]
3990 | 
3991 | [[package]]
3992 | name = "wasi"
3993 | version = "0.11.1+wasi-snapshot-preview1"
3994 | source = "registry+https://github.com/rust-lang/crates.io-index"
3995 | checksum = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"
3996 | 
3997 | [[package]]
3998 | name = "wasip2"
3999 | version = "1.0.2+wasi-0.2.9"
4000 | source = "registry+https://github.com/rust-lang/crates.io-index"
4001 | checksum = "9517f9239f02c069db75e65f174b3da828fe5f5b945c4dd26bd25d89c03ebcf5"
4002 | dependencies = [
4003 |  "wit-bindgen",
4004 | ]
4005 | 
4006 | [[package]]
4007 | name = "wasip3"
4008 | version = "0.4.0+wasi-0.3.0-rc-2026-01-06"
4009 | source = "registry+https://github.com/rust-lang/crates.io-index"
4010 | checksum = "5428f8bf88ea5ddc08faddef2ac4a67e390b88186c703ce6dbd955e1c145aca5"
4011 | dependencies = [
4012 |  "wit-bindgen",
4013 | ]
4014 | 
4015 | [[package]]
4016 | name = "wasm-bindgen"
4017 | version = "0.2.114"
4018 | source = "registry+https://github.com/rust-lang/crates.io-index"
4019 | checksum = "6532f9a5c1ece3798cb1c2cfdba640b9b3ba884f5db45973a6f442510a87d38e"
4020 | dependencies = [
4021 |  "cfg-if",
4022 |  "once_cell",
4023 |  "rustversion",
4024 |  "wasm-bindgen-macro",
4025 |  "wasm-bindgen-shared",
4026 | ]
4027 | 
4028 | [[package]]
4029 | name = "wasm-bindgen-futures"
4030 | version = "0.4.64"
4031 | source = "registry+https://github.com/rust-lang/crates.io-index"
4032 | checksum = "e9c5522b3a28661442748e09d40924dfb9ca614b21c00d3fd135720e48b67db8"
4033 | dependencies = [
4034 |  "cfg-if",
4035 |  "futures-util",
4036 |  "js-sys",
4037 |  "once_cell",
4038 |  "wasm-bindgen",
4039 |  "web-sys",
4040 | ]
4041 | 
4042 | [[package]]
4043 | name = "wasm-bindgen-macro"
4044 | version = "0.2.114"
4045 | source = "registry+https://github.com/rust-lang/crates.io-index"
4046 | checksum = "18a2d50fcf105fb33bb15f00e7a77b772945a2ee45dcf454961fd843e74c18e6"
4047 | dependencies = [
4048 |  "quote",
4049 |  "wasm-bindgen-macro-support",
4050 | ]
4051 | 
4052 | [[package]]
4053 | name = "wasm-bindgen-macro-support"
4054 | version = "0.2.114"
4055 | source = "registry+https://github.com/rust-lang/crates.io-index"
4056 | checksum = "03ce4caeaac547cdf713d280eda22a730824dd11e6b8c3ca9e42247b25c631e3"
4057 | dependencies = [
4058 |  "bumpalo",
4059 |  "proc-macro2",
4060 |  "quote",
4061 |  "syn",
4062 |  "wasm-bindgen-shared",
4063 | ]
4064 | 
4065 | [[package]]
4066 | name = "wasm-bindgen-shared"
4067 | version = "0.2.114"
4068 | source = "registry+https://github.com/rust-lang/crates.io-index"
4069 | checksum = "75a326b8c223ee17883a4251907455a2431acc2791c98c26279376490c378c16"
4070 | dependencies = [
4071 |  "unicode-ident",
4072 | ]
4073 | 
4074 | [[package]]
4075 | name = "wasm-encoder"
4076 | version = "0.244.0"
4077 | source = "registry+https://github.com/rust-lang/crates.io-index"
4078 | checksum = "990065f2fe63003fe337b932cfb5e3b80e0b4d0f5ff650e6985b1048f62c8319"
4079 | dependencies = [
4080 |  "leb128fmt",
4081 |  "wasmparser",
4082 | ]
4083 | 
4084 | [[package]]
4085 | name = "wasm-metadata"
4086 | version = "0.244.0"
4087 | source = "registry+https://github.com/rust-lang/crates.io-index"
4088 | checksum = "bb0e353e6a2fbdc176932bbaab493762eb1255a7900fe0fea1a2f96c296cc909"
4089 | dependencies = [
4090 |  "anyhow",
4091 |  "indexmap",
4092 |  "wasm-encoder",
4093 |  "wasmparser",
4094 | ]
4095 | 
4096 | [[package]]
4097 | name = "wasmparser"
4098 | version = "0.244.0"
4099 | source = "registry+https://github.com/rust-lang/crates.io-index"
4100 | checksum = "47b807c72e1bac69382b3a6fb3dbe8ea4c0ed87ff5629b8685ae6b9a611028fe"
4101 | dependencies = [
4102 |  "bitflags 2.11.0",
4103 |  "hashbrown 0.15.5",
4104 |  "indexmap",
4105 |  "semver",
4106 | ]
4107 | 
4108 | [[package]]
4109 | name = "wayland-backend"
4110 | version = "0.3.12"
4111 | source = "registry+https://github.com/rust-lang/crates.io-index"
4112 | checksum = "fee64194ccd96bf648f42a65a7e589547096dfa702f7cadef84347b66ad164f9"
4113 | dependencies = [
4114 |  "cc",
4115 |  "downcast-rs",
4116 |  "rustix 1.1.4",
4117 |  "scoped-tls",
4118 |  "smallvec",
4119 |  "wayland-sys",
4120 | ]
4121 | 
4122 | [[package]]
4123 | name = "wayland-client"
4124 | version = "0.31.12"
4125 | source = "registry+https://github.com/rust-lang/crates.io-index"
4126 | checksum = "b8e6faa537fbb6c186cb9f1d41f2f811a4120d1b57ec61f50da451a0c5122bec"
4127 | dependencies = [
4128 |  "bitflags 2.11.0",
4129 |  "rustix 1.1.4",
4130 |  "wayland-backend",
4131 |  "wayland-scanner",
4132 | ]
4133 | 
4134 | [[package]]
4135 | name = "wayland-csd-frame"
4136 | version = "0.3.0"
4137 | source = "registry+https://github.com/rust-lang/crates.io-index"
4138 | checksum = "625c5029dbd43d25e6aa9615e88b829a5cad13b2819c4ae129fdbb7c31ab4c7e"
4139 | dependencies = [
4140 |  "bitflags 2.11.0",
4141 |  "cursor-icon",
4142 |  "wayland-backend",
4143 | ]
4144 | 
4145 | [[package]]
4146 | name = "wayland-cursor"
4147 | version = "0.31.12"
4148 | source = "registry+https://github.com/rust-lang/crates.io-index"
4149 | checksum = "5864c4b5b6064b06b1e8b74ead4a98a6c45a285fe7a0e784d24735f011fdb078"
4150 | dependencies = [
4151 |  "rustix 1.1.4",
4152 |  "wayland-client",
4153 |  "xcursor",
4154 | ]
4155 | 
4156 | [[package]]
4157 | name = "wayland-protocols"
4158 | version = "0.32.10"
4159 | source = "registry+https://github.com/rust-lang/crates.io-index"
4160 | checksum = "baeda9ffbcfc8cd6ddaade385eaf2393bd2115a69523c735f12242353c3df4f3"
4161 | dependencies = [
4162 |  "bitflags 2.11.0",
4163 |  "wayland-backend",
4164 |  "wayland-client",
4165 |  "wayland-scanner",
4166 | ]
4167 | 
4168 | [[package]]
4169 | name = "wayland-protocols-plasma"
4170 | version = "0.3.10"
4171 | source = "registry+https://github.com/rust-lang/crates.io-index"
4172 | checksum = "aa98634619300a535a9a97f338aed9a5ff1e01a461943e8346ff4ae26007306b"
4173 | dependencies = [
4174 |  "bitflags 2.11.0",
4175 |  "wayland-backend",
4176 |  "wayland-client",
4177 |  "wayland-protocols",
4178 |  "wayland-scanner",
4179 | ]
4180 | 
4181 | [[package]]
4182 | name = "wayland-protocols-wlr"
4183 | version = "0.3.10"
4184 | source = "registry+https://github.com/rust-lang/crates.io-index"
4185 | checksum = "e9597cdf02cf0c34cd5823786dce6b5ae8598f05c2daf5621b6e178d4f7345f3"
4186 | dependencies = [
4187 |  "bitflags 2.11.0",
4188 |  "wayland-backend",
4189 |  "wayland-client",
4190 |  "wayland-protocols",
4191 |  "wayland-scanner",
4192 | ]
4193 | 
4194 | [[package]]
4195 | name = "wayland-scanner"
4196 | version = "0.31.8"
4197 | source = "registry+https://github.com/rust-lang/crates.io-index"
4198 | checksum = "5423e94b6a63e68e439803a3e153a9252d5ead12fd853334e2ad33997e3889e3"
4199 | dependencies = [
4200 |  "proc-macro2",
4201 |  "quick-xml",
4202 |  "quote",
4203 | ]
4204 | 
4205 | [[package]]
4206 | name = "wayland-sys"
4207 | version = "0.31.8"
4208 | source = "registry+https://github.com/rust-lang/crates.io-index"
4209 | checksum = "1e6dbfc3ac5ef974c92a2235805cc0114033018ae1290a72e474aa8b28cbbdfd"
4210 | dependencies = [
4211 |  "dlib",
4212 |  "log",
4213 |  "once_cell",
4214 |  "pkg-config",
4215 | ]
4216 | 
4217 | [[package]]
4218 | name = "web-sys"
4219 | version = "0.3.91"
4220 | source = "registry+https://github.com/rust-lang/crates.io-index"
4221 | checksum = "854ba17bb104abfb26ba36da9729addc7ce7f06f5c0f90f3c391f8461cca21f9"
4222 | dependencies = [
4223 |  "js-sys",
4224 |  "wasm-bindgen",
4225 | ]
4226 | 
4227 | [[package]]
4228 | name = "web-time"
4229 | version = "1.1.0"
4230 | source = "registry+https://github.com/rust-lang/crates.io-index"
4231 | checksum = "5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb"
4232 | dependencies = [
4233 |  "js-sys",
4234 |  "wasm-bindgen",
4235 | ]
4236 | 
4237 | [[package]]
4238 | name = "weezl"
4239 | version = "0.1.12"
4240 | source = "registry+https://github.com/rust-lang/crates.io-index"
4241 | checksum = "a28ac98ddc8b9274cb41bb4d9d4d5c425b6020c50c46f25559911905610b4a88"
4242 | 
4243 | [[package]]
4244 | name = "wgpu"
4245 | version = "26.0.1"
4246 | source = "registry+https://github.com/rust-lang/crates.io-index"
4247 | checksum = "70b6ff82bbf6e9206828e1a3178e851f8c20f1c9028e74dd3a8090741ccd5798"
4248 | dependencies = [
4249 |  "arrayvec",
4250 |  "bitflags 2.11.0",
4251 |  "cfg-if",
4252 |  "cfg_aliases",
4253 |  "document-features",
4254 |  "hashbrown 0.15.5",
4255 |  "js-sys",
4256 |  "log",
4257 |  "naga",
4258 |  "parking_lot",
4259 |  "portable-atomic",
4260 |  "profiling",
4261 |  "raw-window-handle",
4262 |  "smallvec",
4263 |  "static_assertions",
4264 |  "wasm-bindgen",
4265 |  "wasm-bindgen-futures",
4266 |  "web-sys",
4267 |  "wgpu-core",
4268 |  "wgpu-hal",
4269 |  "wgpu-types",
4270 | ]
4271 | 
4272 | [[package]]
4273 | name = "wgpu-core"
4274 | version = "26.0.1"
4275 | source = "registry+https://github.com/rust-lang/crates.io-index"
4276 | checksum = "d5f62f1053bd28c2268f42916f31588f81f64796e2ff91b81293515017ca8bd9"
4277 | dependencies = [
4278 |  "arrayvec",
4279 |  "bit-set",
4280 |  "bit-vec",
4281 |  "bitflags 2.11.0",
4282 |  "cfg_aliases",
4283 |  "document-features",
4284 |  "hashbrown 0.15.5",
4285 |  "indexmap",
4286 |  "log",
4287 |  "naga",
4288 |  "once_cell",
4289 |  "parking_lot",
4290 |  "portable-atomic",
4291 |  "profiling",
4292 |  "raw-window-handle",
4293 |  "rustc-hash 1.1.0",
4294 |  "smallvec",
4295 |  "thiserror 2.0.18",
4296 |  "wgpu-core-deps-apple",
4297 |  "wgpu-core-deps-emscripten",
4298 |  "wgpu-core-deps-windows-linux-android",
4299 |  "wgpu-hal",
4300 |  "wgpu-types",
4301 | ]
4302 | 
4303 | [[package]]
4304 | name = "wgpu-core-deps-apple"
4305 | version = "26.0.0"
4306 | source = "registry+https://github.com/rust-lang/crates.io-index"
4307 | checksum = "18ae5fbde6a4cbebae38358aa73fcd6e0f15c6144b67ef5dc91ded0db125dbdf"
4308 | dependencies = [
4309 |  "wgpu-hal",
4310 | ]
4311 | 
4312 | [[package]]
4313 | name = "wgpu-core-deps-emscripten"
4314 | version = "26.0.0"
4315 | source = "registry+https://github.com/rust-lang/crates.io-index"
4316 | checksum = "d7670e390f416006f746b4600fdd9136455e3627f5bd763abf9a65daa216dd2d"
4317 | dependencies = [
4318 |  "wgpu-hal",
4319 | ]
4320 | 
4321 | [[package]]
4322 | name = "wgpu-core-deps-windows-linux-android"
4323 | version = "26.0.0"
4324 | source = "registry+https://github.com/rust-lang/crates.io-index"
4325 | checksum = "720a5cb9d12b3d337c15ff0e24d3e97ed11490ff3f7506e7f3d98c68fa5d6f14"
4326 | dependencies = [
4327 |  "wgpu-hal",
4328 | ]
4329 | 
4330 | [[package]]
4331 | name = "wgpu-hal"
4332 | version = "26.0.6"
4333 | source = "registry+https://github.com/rust-lang/crates.io-index"
4334 | checksum = "a8d0e67224cc7305b3b4eb2cc57ca4c4c3afc665c1d1bee162ea806e19c47bdd"
4335 | dependencies = [
4336 |  "android_system_properties",
4337 |  "arrayvec",
4338 |  "ash",
4339 |  "bit-set",
4340 |  "bitflags 2.11.0",
4341 |  "block",
4342 |  "bytemuck",
4343 |  "cfg-if",
4344 |  "cfg_aliases",
4345 |  "core-graphics-types 0.2.0",
4346 |  "glow",
4347 |  "glutin_wgl_sys",
4348 |  "gpu-alloc",
4349 |  "gpu-allocator",
4350 |  "gpu-descriptor",
4351 |  "hashbrown 0.15.5",
4352 |  "js-sys",
4353 |  "khronos-egl",
4354 |  "libc",
4355 |  "libloading",
4356 |  "log",
4357 |  "metal",
4358 |  "naga",
4359 |  "ndk-sys",
4360 |  "objc",
4361 |  "ordered-float 4.6.0",
4362 |  "parking_lot",
4363 |  "portable-atomic",
4364 |  "portable-atomic-util",
4365 |  "profiling",
4366 |  "range-alloc",
4367 |  "raw-window-handle",
4368 |  "renderdoc-sys",
4369 |  "smallvec",
4370 |  "thiserror 2.0.18",
4371 |  "wasm-bindgen",
4372 |  "web-sys",
4373 |  "wgpu-types",
4374 |  "windows",
4375 |  "windows-core",
4376 | ]
4377 | 
4378 | [[package]]
4379 | name = "wgpu-types"
4380 | version = "26.0.0"
4381 | source = "registry+https://github.com/rust-lang/crates.io-index"
4382 | checksum = "eca7a8d8af57c18f57d393601a1fb159ace8b2328f1b6b5f80893f7d672c9ae2"
4383 | dependencies = [
4384 |  "bitflags 2.11.0",
4385 |  "bytemuck",
4386 |  "js-sys",
4387 |  "log",
4388 |  "thiserror 2.0.18",
4389 |  "web-sys",
4390 | ]
4391 | 
4392 | [[package]]
4393 | name = "wgpu_glyph"
4394 | version = "0.26.0"
4395 | source = "registry+https://github.com/rust-lang/crates.io-index"
4396 | checksum = "ddda099db5a12f3cc8087eab88d9c2cf69a2df7d31b10602bef110c01f708c65"
4397 | dependencies = [
4398 |  "bytemuck",
4399 |  "glyph_brush",
4400 |  "log",
4401 |  "wgpu",
4402 | ]
4403 | 
4404 | [[package]]
4405 | name = "winapi"
4406 | version = "0.3.9"
4407 | source = "registry+https://github.com/rust-lang/crates.io-index"
4408 | checksum = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"
4409 | dependencies = [
4410 |  "winapi-i686-pc-windows-gnu",
4411 |  "winapi-x86_64-pc-windows-gnu",
4412 | ]
4413 | 
4414 | [[package]]
4415 | name = "winapi-i686-pc-windows-gnu"
4416 | version = "0.4.0"
4417 | source = "registry+https://github.com/rust-lang/crates.io-index"
4418 | checksum = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"
4419 | 
4420 | [[package]]
4421 | name = "winapi-util"
4422 | version = "0.1.11"
4423 | source = "registry+https://github.com/rust-lang/crates.io-index"
4424 | checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
4425 | dependencies = [
4426 |  "windows-sys 0.61.2",
4427 | ]
4428 | 
4429 | [[package]]
4430 | name = "winapi-x86_64-pc-windows-gnu"
4431 | version = "0.4.0"
4432 | source = "registry+https://github.com/rust-lang/crates.io-index"
4433 | checksum = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"
4434 | 
4435 | [[package]]
4436 | name = "windows"
4437 | version = "0.58.0"
4438 | source = "registry+https://github.com/rust-lang/crates.io-index"
4439 | checksum = "dd04d41d93c4992d421894c18c8b43496aa748dd4c081bac0dc93eb0489272b6"
4440 | dependencies = [
4441 |  "windows-core",
4442 |  "windows-targets 0.52.6",
4443 | ]
4444 | 
4445 | [[package]]
4446 | name = "windows-core"
4447 | version = "0.58.0"
4448 | source = "registry+https://github.com/rust-lang/crates.io-index"
4449 | checksum = "6ba6d44ec8c2591c134257ce647b7ea6b20335bf6379a27dac5f1641fcf59f99"
4450 | dependencies = [
4451 |  "windows-implement",
4452 |  "windows-interface",
4453 |  "windows-result",
4454 |  "windows-strings",
4455 |  "windows-targets 0.52.6",
4456 | ]
4457 | 
4458 | [[package]]
4459 | name = "windows-implement"
4460 | version = "0.58.0"
4461 | source = "registry+https://github.com/rust-lang/crates.io-index"
4462 | checksum = "2bbd5b46c938e506ecbce286b6628a02171d56153ba733b6c741fc627ec9579b"
4463 | dependencies = [
4464 |  "proc-macro2",
4465 |  "quote",
4466 |  "syn",
4467 | ]
4468 | 
4469 | [[package]]
4470 | name = "windows-interface"
4471 | version = "0.58.0"
4472 | source = "registry+https://github.com/rust-lang/crates.io-index"
4473 | checksum = "053c4c462dc91d3b1504c6fe5a726dd15e216ba718e84a0e46a88fbe5ded3515"
4474 | dependencies = [
4475 |  "proc-macro2",
4476 |  "quote",
4477 |  "syn",
4478 | ]
4479 | 
4480 | [[package]]
4481 | name = "windows-link"
4482 | version = "0.2.1"
4483 | source = "registry+https://github.com/rust-lang/crates.io-index"
4484 | checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"
4485 | 
4486 | [[package]]
4487 | name = "windows-result"
4488 | version = "0.2.0"
4489 | source = "registry+https://github.com/rust-lang/crates.io-index"
4490 | checksum = "1d1043d8214f791817bab27572aaa8af63732e11bf84aa21a45a78d6c317ae0e"
4491 | dependencies = [
4492 |  "windows-targets 0.52.6",
4493 | ]
4494 | 
4495 | [[package]]
4496 | name = "windows-strings"
4497 | version = "0.1.0"
4498 | source = "registry+https://github.com/rust-lang/crates.io-index"
4499 | checksum = "4cd9b125c486025df0eabcb585e62173c6c9eddcec5d117d3b6e8c30e2ee4d10"
4500 | dependencies = [
4501 |  "windows-result",
4502 |  "windows-targets 0.52.6",
4503 | ]
4504 | 
4505 | [[package]]
4506 | name = "windows-sys"
4507 | version = "0.45.0"
4508 | source = "registry+https://github.com/rust-lang/crates.io-index"
4509 | checksum = "75283be5efb2831d37ea142365f009c02ec203cd29a3ebecbc093d52315b66d0"
4510 | dependencies = [
4511 |  "windows-targets 0.42.2",
4512 | ]
4513 | 
4514 | [[package]]
4515 | name = "windows-sys"
4516 | version = "0.48.0"
4517 | source = "registry+https://github.com/rust-lang/crates.io-index"
4518 | checksum = "677d2418bec65e3338edb076e806bc1ec15693c5d0104683f2efe857f61056a9"
4519 | dependencies = [
4520 |  "windows-targets 0.48.5",
4521 | ]
4522 | 
4523 | [[package]]
4524 | name = "windows-sys"
4525 | version = "0.52.0"
4526 | source = "registry+https://github.com/rust-lang/crates.io-index"
4527 | checksum = "282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d"
4528 | dependencies = [
4529 |  "windows-targets 0.52.6",
4530 | ]
4531 | 
4532 | [[package]]
4533 | name = "windows-sys"
4534 | version = "0.59.0"
4535 | source = "registry+https://github.com/rust-lang/crates.io-index"
4536 | checksum = "1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b"
4537 | dependencies = [
4538 |  "windows-targets 0.52.6",
4539 | ]
4540 | 
4541 | [[package]]
4542 | name = "windows-sys"
4543 | version = "0.61.2"
4544 | source = "registry+https://github.com/rust-lang/crates.io-index"
4545 | checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
4546 | dependencies = [
4547 |  "windows-link",
4548 | ]
4549 | 
4550 | [[package]]
4551 | name = "windows-targets"
4552 | version = "0.42.2"
4553 | source = "registry+https://github.com/rust-lang/crates.io-index"
4554 | checksum = "8e5180c00cd44c9b1c88adb3693291f1cd93605ded80c250a75d472756b4d071"
4555 | dependencies = [
4556 |  "windows_aarch64_gnullvm 0.42.2",
4557 |  "windows_aarch64_msvc 0.42.2",
4558 |  "windows_i686_gnu 0.42.2",
4559 |  "windows_i686_msvc 0.42.2",
4560 |  "windows_x86_64_gnu 0.42.2",
4561 |  "windows_x86_64_gnullvm 0.42.2",
4562 |  "windows_x86_64_msvc 0.42.2",
4563 | ]
4564 | 
4565 | [[package]]
4566 | name = "windows-targets"
4567 | version = "0.48.5"
4568 | source = "registry+https://github.com/rust-lang/crates.io-index"
4569 | checksum = "9a2fa6e2155d7247be68c096456083145c183cbbbc2764150dda45a87197940c"
4570 | dependencies = [
4571 |  "windows_aarch64_gnullvm 0.48.5",
4572 |  "windows_aarch64_msvc 0.48.5",
4573 |  "windows_i686_gnu 0.48.5",
4574 |  "windows_i686_msvc 0.48.5",
4575 |  "windows_x86_64_gnu 0.48.5",
4576 |  "windows_x86_64_gnullvm 0.48.5",
4577 |  "windows_x86_64_msvc 0.48.5",
4578 | ]
4579 | 
4580 | [[package]]
4581 | name = "windows-targets"
4582 | version = "0.52.6"
4583 | source = "registry+https://github.com/rust-lang/crates.io-index"
4584 | checksum = "9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973"
4585 | dependencies = [
4586 |  "windows_aarch64_gnullvm 0.52.6",
4587 |  "windows_aarch64_msvc 0.52.6",
4588 |  "windows_i686_gnu 0.52.6",
4589 |  "windows_i686_gnullvm",
4590 |  "windows_i686_msvc 0.52.6",
4591 |  "windows_x86_64_gnu 0.52.6",
4592 |  "windows_x86_64_gnullvm 0.52.6",
4593 |  "windows_x86_64_msvc 0.52.6",
4594 | ]
4595 | 
4596 | [[package]]
4597 | name = "windows_aarch64_gnullvm"
4598 | version = "0.42.2"
4599 | source = "registry+https://github.com/rust-lang/crates.io-index"
4600 | checksum = "597a5118570b68bc08d8d59125332c54f1ba9d9adeedeef5b99b02ba2b0698f8"
4601 | 
4602 | [[package]]
4603 | name = "windows_aarch64_gnullvm"
4604 | version = "0.48.5"
4605 | source = "registry+https://github.com/rust-lang/crates.io-index"
4606 | checksum = "2b38e32f0abccf9987a4e3079dfb67dcd799fb61361e53e2882c3cbaf0d905d8"
4607 | 
4608 | [[package]]
4609 | name = "windows_aarch64_gnullvm"
4610 | version = "0.52.6"
4611 | source = "registry+https://github.com/rust-lang/crates.io-index"
4612 | checksum = "32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3"
4613 | 
4614 | [[package]]
4615 | name = "windows_aarch64_msvc"
4616 | version = "0.42.2"
4617 | source = "registry+https://github.com/rust-lang/crates.io-index"
4618 | checksum = "e08e8864a60f06ef0d0ff4ba04124db8b0fb3be5776a5cd47641e942e58c4d43"
4619 | 
4620 | [[package]]
4621 | name = "windows_aarch64_msvc"
4622 | version = "0.48.5"
4623 | source = "registry+https://github.com/rust-lang/crates.io-index"
4624 | checksum = "dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc"
4625 | 
4626 | [[package]]
4627 | name = "windows_aarch64_msvc"
4628 | version = "0.52.6"
4629 | source = "registry+https://github.com/rust-lang/crates.io-index"
4630 | checksum = "09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469"
4631 | 
4632 | [[package]]
4633 | name = "windows_i686_gnu"
4634 | version = "0.42.2"
4635 | source = "registry+https://github.com/rust-lang/crates.io-index"
4636 | checksum = "c61d927d8da41da96a81f029489353e68739737d3beca43145c8afec9a31a84f"
4637 | 
4638 | [[package]]
4639 | name = "windows_i686_gnu"
4640 | version = "0.48.5"
4641 | source = "registry+https://github.com/rust-lang/crates.io-index"
4642 | checksum = "a75915e7def60c94dcef72200b9a8e58e5091744960da64ec734a6c6e9b3743e"
4643 | 
4644 | [[package]]
4645 | name = "windows_i686_gnu"
4646 | version = "0.52.6"
4647 | source = "registry+https://github.com/rust-lang/crates.io-index"
4648 | checksum = "8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b"
4649 | 
4650 | [[package]]
4651 | name = "windows_i686_gnullvm"
4652 | version = "0.52.6"
4653 | source = "registry+https://github.com/rust-lang/crates.io-index"
4654 | checksum = "0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66"
4655 | 
4656 | [[package]]
4657 | name = "windows_i686_msvc"
4658 | version = "0.42.2"
4659 | source = "registry+https://github.com/rust-lang/crates.io-index"
4660 | checksum = "44d840b6ec649f480a41c8d80f9c65108b92d89345dd94027bfe06ac444d1060"
4661 | 
4662 | [[package]]
4663 | name = "windows_i686_msvc"
4664 | version = "0.48.5"
4665 | source = "registry+https://github.com/rust-lang/crates.io-index"
4666 | checksum = "8f55c233f70c4b27f66c523580f78f1004e8b5a8b659e05a4eb49d4166cca406"
4667 | 
4668 | [[package]]
4669 | name = "windows_i686_msvc"
4670 | version = "0.52.6"
4671 | source = "registry+https://github.com/rust-lang/crates.io-index"
4672 | checksum = "240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66"
4673 | 
4674 | [[package]]
4675 | name = "windows_x86_64_gnu"
4676 | version = "0.42.2"
4677 | source = "registry+https://github.com/rust-lang/crates.io-index"
4678 | checksum = "8de912b8b8feb55c064867cf047dda097f92d51efad5b491dfb98f6bbb70cb36"
4679 | 
4680 | [[package]]
4681 | name = "windows_x86_64_gnu"
4682 | version = "0.48.5"
4683 | source = "registry+https://github.com/rust-lang/crates.io-index"
4684 | checksum = "53d40abd2583d23e4718fddf1ebec84dbff8381c07cae67ff7768bbf19c6718e"
4685 | 
4686 | [[package]]
4687 | name = "windows_x86_64_gnu"
4688 | version = "0.52.6"
4689 | source = "registry+https://github.com/rust-lang/crates.io-index"
4690 | checksum = "147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78"
4691 | 
4692 | [[package]]
4693 | name = "windows_x86_64_gnullvm"
4694 | version = "0.42.2"
4695 | source = "registry+https://github.com/rust-lang/crates.io-index"
4696 | checksum = "26d41b46a36d453748aedef1486d5c7a85db22e56aff34643984ea85514e94a3"
4697 | 
4698 | [[package]]
4699 | name = "windows_x86_64_gnullvm"
4700 | version = "0.48.5"
4701 | source = "registry+https://github.com/rust-lang/crates.io-index"
4702 | checksum = "0b7b52767868a23d5bab768e390dc5f5c55825b6d30b86c844ff2dc7414044cc"
4703 | 
4704 | [[package]]
4705 | name = "windows_x86_64_gnullvm"
4706 | version = "0.52.6"
4707 | source = "registry+https://github.com/rust-lang/crates.io-index"
4708 | checksum = "24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d"
4709 | 
4710 | [[package]]
4711 | name = "windows_x86_64_msvc"
4712 | version = "0.42.2"
4713 | source = "registry+https://github.com/rust-lang/crates.io-index"
4714 | checksum = "9aec5da331524158c6d1a4ac0ab1541149c0b9505fde06423b02f5ef0106b9f0"
4715 | 
4716 | [[package]]
4717 | name = "windows_x86_64_msvc"
4718 | version = "0.48.5"
4719 | source = "registry+https://github.com/rust-lang/crates.io-index"
4720 | checksum = "ed94fce61571a4006852b7389a063ab983c02eb1bb37b47f8272ce92d06d9538"
4721 | 
4722 | [[package]]
4723 | name = "windows_x86_64_msvc"
4724 | version = "0.52.6"
4725 | source = "registry+https://github.com/rust-lang/crates.io-index"
4726 | checksum = "589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec"
4727 | 
4728 | [[package]]
4729 | name = "winit"
4730 | version = "0.30.13"
4731 | source = "registry+https://github.com/rust-lang/crates.io-index"
4732 | checksum = "a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d"
4733 | dependencies = [
4734 |  "ahash",
4735 |  "android-activity",
4736 |  "atomic-waker",
4737 |  "bitflags 2.11.0",
4738 |  "block2",
4739 |  "bytemuck",
4740 |  "calloop",
4741 |  "cfg_aliases",
4742 |  "concurrent-queue",
4743 |  "core-foundation 0.9.4",
4744 |  "core-graphics",
4745 |  "cursor-icon",
4746 |  "dpi",
4747 |  "js-sys",
4748 |  "libc",
4749 |  "memmap2",
4750 |  "ndk",
4751 |  "objc2",
4752 |  "objc2-app-kit",
4753 |  "objc2-foundation",
4754 |  "objc2-ui-kit",
4755 |  "orbclient",
4756 |  "percent-encoding",
4757 |  "pin-project",
4758 |  "raw-window-handle",
4759 |  "redox_syscall 0.4.1",
4760 |  "rustix 0.38.44",
4761 |  "sctk-adwaita",
4762 |  "smithay-client-toolkit",
4763 |  "smol_str",
4764 |  "tracing",
4765 |  "unicode-segmentation",
4766 |  "wasm-bindgen",
4767 |  "wasm-bindgen-futures",
4768 |  "wayland-backend",
4769 |  "wayland-client",
4770 |  "wayland-protocols",
4771 |  "wayland-protocols-plasma",
4772 |  "web-sys",
4773 |  "web-time",
4774 |  "windows-sys 0.52.0",
4775 |  "x11-dl",
4776 |  "x11rb",
4777 |  "xkbcommon-dl",
4778 | ]
4779 | 
4780 | [[package]]
4781 | name = "winnow"
4782 | version = "0.7.14"
4783 | source = "registry+https://github.com/rust-lang/crates.io-index"
4784 | checksum = "5a5364e9d77fcdeeaa6062ced926ee3381faa2ee02d3eb83a5c27a8825540829"
4785 | dependencies = [
4786 |  "memchr",
4787 | ]
4788 | 
4789 | [[package]]
4790 | name = "winreg"
4791 | version = "0.51.0"
4792 | source = "registry+https://github.com/rust-lang/crates.io-index"
4793 | checksum = "937f3df7948156640f46aacef17a70db0de5917bda9c92b0f751f3a955b588fc"
4794 | dependencies = [
4795 |  "cfg-if",
4796 |  "windows-sys 0.48.0",
4797 | ]
4798 | 
4799 | [[package]]
4800 | name = "wit-bindgen"
4801 | version = "0.51.0"
4802 | source = "registry+https://github.com/rust-lang/crates.io-index"
4803 | checksum = "d7249219f66ced02969388cf2bb044a09756a083d0fab1e566056b04d9fbcaa5"
4804 | dependencies = [
4805 |  "wit-bindgen-rust-macro",
4806 | ]
4807 | 
4808 | [[package]]
4809 | name = "wit-bindgen-core"
4810 | version = "0.51.0"
4811 | source = "registry+https://github.com/rust-lang/crates.io-index"
4812 | checksum = "ea61de684c3ea68cb082b7a88508a8b27fcc8b797d738bfc99a82facf1d752dc"
4813 | dependencies = [
4814 |  "anyhow",
4815 |  "heck",
4816 |  "wit-parser",
4817 | ]
4818 | 
4819 | [[package]]
4820 | name = "wit-bindgen-rust"
4821 | version = "0.51.0"
4822 | source = "registry+https://github.com/rust-lang/crates.io-index"
4823 | checksum = "b7c566e0f4b284dd6561c786d9cb0142da491f46a9fbed79ea69cdad5db17f21"
4824 | dependencies = [
4825 |  "anyhow",
4826 |  "heck",
4827 |  "indexmap",
4828 |  "prettyplease",
4829 |  "syn",
4830 |  "wasm-metadata",
4831 |  "wit-bindgen-core",
4832 |  "wit-component",
4833 | ]
4834 | 
4835 | [[package]]
4836 | name = "wit-bindgen-rust-macro"
4837 | version = "0.51.0"
4838 | source = "registry+https://github.com/rust-lang/crates.io-index"
4839 | checksum = "0c0f9bfd77e6a48eccf51359e3ae77140a7f50b1e2ebfe62422d8afdaffab17a"
4840 | dependencies = [
4841 |  "anyhow",
4842 |  "prettyplease",
4843 |  "proc-macro2",
4844 |  "quote",
4845 |  "syn",
4846 |  "wit-bindgen-core",
4847 |  "wit-bindgen-rust",
4848 | ]
4849 | 
4850 | [[package]]
4851 | name = "wit-component"
4852 | version = "0.244.0"
4853 | source = "registry+https://github.com/rust-lang/crates.io-index"
4854 | checksum = "9d66ea20e9553b30172b5e831994e35fbde2d165325bec84fc43dbf6f4eb9cb2"
4855 | dependencies = [
4856 |  "anyhow",
4857 |  "bitflags 2.11.0",
4858 |  "indexmap",
4859 |  "log",
4860 |  "serde",
4861 |  "serde_derive",
4862 |  "serde_json",
4863 |  "wasm-encoder",
4864 |  "wasm-metadata",
4865 |  "wasmparser",
4866 |  "wit-parser",
4867 | ]
4868 | 
4869 | [[package]]
4870 | name = "wit-parser"
4871 | version = "0.244.0"
4872 | source = "registry+https://github.com/rust-lang/crates.io-index"
4873 | checksum = "ecc8ac4bc1dc3381b7f59c34f00b67e18f910c2c0f50015669dde7def656a736"
4874 | dependencies = [
4875 |  "anyhow",
4876 |  "id-arena",
4877 |  "indexmap",
4878 |  "log",
4879 |  "semver",
4880 |  "serde",
4881 |  "serde_derive",
4882 |  "serde_json",
4883 |  "unicode-xid",
4884 |  "wasmparser",
4885 | ]
4886 | 
4887 | [[package]]
4888 | name = "writeable"
4889 | version = "0.6.2"
4890 | source = "registry+https://github.com/rust-lang/crates.io-index"
4891 | checksum = "9edde0db4769d2dc68579893f2306b26c6ecfbe0ef499b013d731b7b9247e0b9"
4892 | 
4893 | [[package]]
4894 | name = "x11-dl"
4895 | version = "2.21.0"
4896 | source = "registry+https://github.com/rust-lang/crates.io-index"
4897 | checksum = "38735924fedd5314a6e548792904ed8c6de6636285cb9fec04d5b1db85c1516f"
4898 | dependencies = [
4899 |  "libc",
4900 |  "once_cell",
4901 |  "pkg-config",
4902 | ]
4903 | 
4904 | [[package]]
4905 | name = "x11rb"
4906 | version = "0.13.2"
4907 | source = "registry+https://github.com/rust-lang/crates.io-index"
4908 | checksum = "9993aa5be5a26815fe2c3eacfc1fde061fc1a1f094bf1ad2a18bf9c495dd7414"
4909 | dependencies = [
4910 |  "as-raw-xcb-connection",
4911 |  "gethostname",
4912 |  "libc",
4913 |  "libloading",
4914 |  "once_cell",
4915 |  "rustix 1.1.4",
4916 |  "x11rb-protocol",
4917 | ]
4918 | 
4919 | [[package]]
4920 | name = "x11rb-protocol"
4921 | version = "0.13.2"
4922 | source = "registry+https://github.com/rust-lang/crates.io-index"
4923 | checksum = "ea6fc2961e4ef194dcbfe56bb845534d0dc8098940c7e5c012a258bfec6701bd"
4924 | 
4925 | [[package]]
4926 | name = "xcursor"
4927 | version = "0.3.10"
4928 | source = "registry+https://github.com/rust-lang/crates.io-index"
4929 | checksum = "bec9e4a500ca8864c5b47b8b482a73d62e4237670e5b5f1d6b9e3cae50f28f2b"
4930 | 
4931 | [[package]]
4932 | name = "xdg-home"
4933 | version = "1.3.0"
4934 | source = "registry+https://github.com/rust-lang/crates.io-index"
4935 | checksum = "ec1cdab258fb55c0da61328dc52c8764709b249011b2cad0454c72f0bf10a1f6"
4936 | dependencies = [
4937 |  "libc",
4938 |  "windows-sys 0.59.0",
4939 | ]
4940 | 
4941 | [[package]]
4942 | name = "xi-unicode"
4943 | version = "0.3.0"
4944 | source = "registry+https://github.com/rust-lang/crates.io-index"
4945 | checksum = "a67300977d3dc3f8034dae89778f502b6ba20b269527b3223ba59c0cf393bb8a"
4946 | 
4947 | [[package]]
4948 | name = "xkbcommon-dl"
4949 | version = "0.4.2"
4950 | source = "registry+https://github.com/rust-lang/crates.io-index"
4951 | checksum = "d039de8032a9a8856a6be89cea3e5d12fdd82306ab7c94d74e6deab2460651c5"
4952 | dependencies = [
4953 |  "bitflags 2.11.0",
4954 |  "dlib",
4955 |  "log",
4956 |  "once_cell",
4957 |  "xkeysym",
4958 | ]
4959 | 
4960 | [[package]]
4961 | name = "xkeysym"
4962 | version = "0.2.1"
4963 | source = "registry+https://github.com/rust-lang/crates.io-index"
4964 | checksum = "b9cc00251562a284751c9973bace760d86c0276c471b4be569fe6b068ee97a56"
4965 | 
4966 | [[package]]
4967 | name = "xml-rs"
4968 | version = "0.8.28"
4969 | source = "registry+https://github.com/rust-lang/crates.io-index"
4970 | checksum = "3ae8337f8a065cfc972643663ea4279e04e7256de865aa66fe25cec5fb912d3f"
4971 | 
4972 | [[package]]
4973 | name = "xmlwriter"
4974 | version = "0.1.0"
4975 | source = "registry+https://github.com/rust-lang/crates.io-index"
4976 | checksum = "ec7a2a501ed189703dba8b08142f057e887dfc4b2cc4db2d343ac6376ba3e0b9"
4977 | 
4978 | [[package]]
4979 | name = "yoke"
4980 | version = "0.8.1"
4981 | source = "registry+https://github.com/rust-lang/crates.io-index"
4982 | checksum = "72d6e5c6afb84d73944e5cedb052c4680d5657337201555f9f2a16b7406d4954"
4983 | dependencies = [
4984 |  "stable_deref_trait",
4985 |  "yoke-derive",
4986 |  "zerofrom",
4987 | ]
4988 | 
4989 | [[package]]
4990 | name = "yoke-derive"
4991 | version = "0.8.1"
4992 | source = "registry+https://github.com/rust-lang/crates.io-index"
4993 | checksum = "b659052874eb698efe5b9e8cf382204678a0086ebf46982b79d6ca3182927e5d"
4994 | dependencies = [
4995 |  "proc-macro2",
4996 |  "quote",
4997 |  "syn",
4998 |  "synstructure",
4999 | ]
5000 | 
5001 | [[package]]
5002 | name = "zbus"
5003 | version = "4.4.0"
5004 | source = "registry+https://github.com/rust-lang/crates.io-index"
5005 | checksum = "bb97012beadd29e654708a0fdb4c84bc046f537aecfde2c3ee0a9e4b4d48c725"
5006 | dependencies = [
5007 |  "async-broadcast",
5008 |  "async-executor",
5009 |  "async-fs",
5010 |  "async-io",
5011 |  "async-lock",
5012 |  "async-process",
5013 |  "async-recursion",
5014 |  "async-task",
5015 |  "async-trait",
5016 |  "blocking",
5017 |  "enumflags2",
5018 |  "event-listener",
5019 |  "futures-core",
5020 |  "futures-sink",
5021 |  "futures-util",
5022 |  "hex",
5023 |  "nix",
5024 |  "ordered-stream",
5025 |  "rand 0.8.5",
5026 |  "serde",
5027 |  "serde_repr",
5028 |  "sha1",
5029 |  "static_assertions",
5030 |  "tracing",
5031 |  "uds_windows",
5032 |  "windows-sys 0.52.0",
5033 |  "xdg-home",
5034 |  "zbus_macros",
5035 |  "zbus_names",
5036 |  "zvariant",
5037 | ]
5038 | 
5039 | [[package]]
5040 | name = "zbus_macros"
5041 | version = "4.4.0"
5042 | source = "registry+https://github.com/rust-lang/crates.io-index"
5043 | checksum = "267db9407081e90bbfa46d841d3cbc60f59c0351838c4bc65199ecd79ab1983e"
5044 | dependencies = [
5045 |  "proc-macro-crate",
5046 |  "proc-macro2",
5047 |  "quote",
5048 |  "syn",
5049 |  "zvariant_utils",
5050 | ]
5051 | 
5052 | [[package]]
5053 | name = "zbus_names"
5054 | version = "3.0.0"
5055 | source = "registry+https://github.com/rust-lang/crates.io-index"
5056 | checksum = "4b9b1fef7d021261cc16cba64c351d291b715febe0fa10dc3a443ac5a5022e6c"
5057 | dependencies = [
5058 |  "serde",
5059 |  "static_assertions",
5060 |  "zvariant",
5061 | ]
5062 | 
5063 | [[package]]
5064 | name = "zerocopy"
5065 | version = "0.8.40"
5066 | source = "registry+https://github.com/rust-lang/crates.io-index"
5067 | checksum = "a789c6e490b576db9f7e6b6d661bcc9799f7c0ac8352f56ea20193b2681532e5"
5068 | dependencies = [
5069 |  "zerocopy-derive",
5070 | ]
5071 | 
5072 | [[package]]
5073 | name = "zerocopy-derive"
5074 | version = "0.8.40"
5075 | source = "registry+https://github.com/rust-lang/crates.io-index"
5076 | checksum = "f65c489a7071a749c849713807783f70672b28094011623e200cb86dcb835953"
5077 | dependencies = [
5078 |  "proc-macro2",
5079 |  "quote",
5080 |  "syn",
5081 | ]
5082 | 
5083 | [[package]]
5084 | name = "zerofrom"
5085 | version = "0.1.6"
5086 | source = "registry+https://github.com/rust-lang/crates.io-index"
5087 | checksum = "50cc42e0333e05660c3587f3bf9d0478688e15d870fab3346451ce7f8c9fbea5"
5088 | dependencies = [
5089 |  "zerofrom-derive",
5090 | ]
5091 | 
5092 | [[package]]
5093 | name = "zerofrom-derive"
5094 | version = "0.1.6"
5095 | source = "registry+https://github.com/rust-lang/crates.io-index"
5096 | checksum = "d71e5d6e06ab090c67b5e44993ec16b72dcbaabc526db883a360057678b48502"
5097 | dependencies = [
5098 |  "proc-macro2",
5099 |  "quote",
5100 |  "syn",
5101 |  "synstructure",
5102 | ]
5103 | 
5104 | [[package]]
5105 | name = "zerotrie"
5106 | version = "0.2.3"
5107 | source = "registry+https://github.com/rust-lang/crates.io-index"
5108 | checksum = "2a59c17a5562d507e4b54960e8569ebee33bee890c70aa3fe7b97e85a9fd7851"
5109 | dependencies = [
5110 |  "displaydoc",
5111 |  "yoke",
5112 |  "zerofrom",
5113 | ]
5114 | 
5115 | [[package]]
5116 | name = "zerovec"
5117 | version = "0.11.5"
5118 | source = "registry+https://github.com/rust-lang/crates.io-index"
5119 | checksum = "6c28719294829477f525be0186d13efa9a3c602f7ec202ca9e353d310fb9a002"
5120 | dependencies = [
5121 |  "yoke",
5122 |  "zerofrom",
5123 |  "zerovec-derive",
5124 | ]
5125 | 
5126 | [[package]]
5127 | name = "zerovec-derive"
5128 | version = "0.11.2"
5129 | source = "registry+https://github.com/rust-lang/crates.io-index"
5130 | checksum = "eadce39539ca5cb3985590102671f2567e659fca9666581ad3411d59207951f3"
5131 | dependencies = [
5132 |  "proc-macro2",
5133 |  "quote",
5134 |  "syn",
5135 | ]
5136 | 
5137 | [[package]]
5138 | name = "zmij"
5139 | version = "1.0.21"
5140 | source = "registry+https://github.com/rust-lang/crates.io-index"
5141 | checksum = "b8848ee67ecc8aedbaf3e4122217aff892639231befc6a1b58d29fff4c2cabaa"
5142 | 
5143 | [[package]]
5144 | name = "zune-core"
5145 | version = "0.4.12"
5146 | source = "registry+https://github.com/rust-lang/crates.io-index"
5147 | checksum = "3f423a2c17029964870cfaabb1f13dfab7d092a62a29a89264f4d36990ca414a"
5148 | 
5149 | [[package]]
5150 | name = "zune-core"
5151 | version = "0.5.1"
5152 | source = "registry+https://github.com/rust-lang/crates.io-index"
5153 | checksum = "cb8a0807f7c01457d0379ba880ba6322660448ddebc890ce29bb64da71fb40f9"
5154 | 
5155 | [[package]]
5156 | name = "zune-jpeg"
5157 | version = "0.4.21"
5158 | source = "registry+https://github.com/rust-lang/crates.io-index"
5159 | checksum = "29ce2c8a9384ad323cf564b67da86e21d3cfdff87908bc1223ed5c99bc792713"
5160 | dependencies = [
5161 |  "zune-core 0.4.12",
5162 | ]
5163 | 
5164 | [[package]]
5165 | name = "zune-jpeg"
5166 | version = "0.5.12"
5167 | source = "registry+https://github.com/rust-lang/crates.io-index"
5168 | checksum = "410e9ecef634c709e3831c2cfdb8d9c32164fae1c67496d5b68fff728eec37fe"
5169 | dependencies = [
5170 |  "zune-core 0.5.1",
5171 | ]
5172 | 
5173 | [[package]]
5174 | name = "zvariant"
5175 | version = "4.2.0"
5176 | source = "registry+https://github.com/rust-lang/crates.io-index"
5177 | checksum = "2084290ab9a1c471c38fc524945837734fbf124487e105daec2bb57fd48c81fe"
5178 | dependencies = [
5179 |  "endi",
5180 |  "enumflags2",
5181 |  "serde",
5182 |  "static_assertions",
5183 |  "url",
5184 |  "zvariant_derive",
5185 | ]
5186 | 
5187 | [[package]]
5188 | name = "zvariant_derive"
5189 | version = "4.2.0"
5190 | source = "registry+https://github.com/rust-lang/crates.io-index"
5191 | checksum = "73e2ba546bda683a90652bac4a279bc146adad1386f25379cf73200d2002c449"
5192 | dependencies = [
5193 |  "proc-macro-crate",
5194 |  "proc-macro2",
5195 |  "quote",
5196 |  "syn",
5197 |  "zvariant_utils",
5198 | ]
5199 | 
5200 | [[package]]
5201 | name = "zvariant_utils"
5202 | version = "2.1.0"
5203 | source = "registry+https://github.com/rust-lang/crates.io-index"
5204 | checksum = "c51bcff7cc3dbb5055396bcf774748c3dab426b4b8659046963523cee4808340"
5205 | dependencies = [
5206 |  "proc-macro2",
5207 |  "quote",
5208 |  "syn",
5209 | ]
```
