# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project: SpedImage

Ultra-Lightweight GPU-Accelerated Image Viewer built in Rust with WGPU for real-time image processing and rendering.

## Build & Run Commands

### Development Mode
```bash
cargo build                    # Debug build
cargo test                     # Run all tests
cargo clippy                  # Lint checking
```

### Release Build (Production)
```bash
cargo build --release
cargo run --release
```

### Run with an image on startup
```bash
cargo run --release -- <image_path>
```

## Architecture Overview

### Data Flow
1. `main.rs` - CLI entry point, initializes logging and starts the event loop
2. `app.rs` (`SpedImageApp`) - Main application state coordinator managing:
   - Event loop using winit
   - Image loading via `ImageBackend` (pure Rust image decoding)
   - WGPU renderer for GPU-accelerated rendering
   - UI state management

### Core Components

| File | Responsibility |
|------|---------------|
| `src/app.rs` | Main application loop, event handling, state coordination |
| `src/gpu_renderer.rs` | WGPU pipeline, shader compilation, real-time adjustments via WGSL shaders |
| `src/image_backend.rs` | Image decoding (JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC), format detection |
| `src/ui.rs` | UI state (sidebar, crop overlay, thumbnail strip), user-facing state |
| `src/main.rs` | Entry point, CLI argument handling, logging setup |

### Key Dependencies (Cargo.toml)
- **WGPU** - GPU rendering (Vulkan/Metal/DX12/OpenGL)
- **winit** - Windowing system
- **egui** - UI framework
- **zune-image** - Image decoding and processing
- **tracing** - Structured logging with environment filter
- **quick_cache** - Lightweight prefetch caching

### Features Flags
```toml
# Cargo.toml
# Note: The project currently manages features directly via dependencies.
```

## Technical Details

### Logging Format
The application uses structured logging via the `tracing` crate:
- Debug builds: `spedimage=debug,winit=warn,wgpu=warn`
- Release: `spedimage=info,winit=warn,wgpu=warn`
- Can be overridden with `RUST_LOG=<level>` environment variable

### GPU Rendering Pipeline
- WGPU handles the actual rendering via custom WGSL shaders
- All image adjustments (brightness, contrast, saturation) are applied on GPU - zero CPU work
- Rotation is shader-based (90° increments only)
- Thumbnail strip preview renders adjacent images for browsing

### Memory Management
- Zero-copy texture loading into WGPU
- Images are decoded in-memory using `zune-image`, then transferred to GPU textures
- Background threads preload thumbnails from neighboring images in a directory using `quick_cache`

## Keyboard Shortcuts (User-facing)
- `A/W` - Previous image
- `D/S` - Next image
- `R` - Rotate 90° clockwise
- `C` - Toggle crop mode
- `O` - Open file dialog
- `Ctrl+S` - Save modified image
- `F` - Toggle sidebar visibility

---

## File Structure

```
spedimage/
├── src/
│   ├── main.rs          # CLI entry point, logging setup
│   ├── lib.rs           # Module declarations for library crate
│   ├── app/             # Application logic and state management
│   ├── image/           # Image loading and processing
│   ├── render/          # WGPU rendering and shaders
│   └── ui.rs            # UI layout and state
├── assets/               # Assets, fonts, and icons
└── Cargo.toml           # Rust manifest with dependencies
```

## Image Formats Supported
The pure Rust `image` crate decodes: JPEG, PNG, GIF, BMP, TIFF, WebP, HEIF (optional), RAW (optional). Shaders support AVIF as well.
