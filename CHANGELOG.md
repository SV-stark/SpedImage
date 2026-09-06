# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-09-06

### Daily Driver Experience & Shell Integration
* **Natural Numerical Sorting**: Implemented natural alphanumeric sorting (`img1 < img2 < img10`), aligning directory navigation sequence with Windows File Explorer.
* **Seamless Folder Switching**: Resolved cross-directory image opening bug where the active folder, thumbnail strip, and file watcher remained locked to the first folder.
* **Single-Instance Focus & Un-Minimize**: On Windows, secondary process invocations now restore the primary window if minimized (`ShowWindow(SW_RESTORE)`) and bring it to foreground (`SetForegroundWindow`, `BringWindowToTop`).
* **Windows File Clipboard (`CF_HDROP`)**: Added `CF_HDROP` file drop list to clipboard alongside raw DIB bitmap (`Ctrl+C`), allowing images to be pasted directly as file objects into File Explorer, Discord, Slack, and email clients.
* **Window Maximized & State Persistence**: Persists maximized window state, window dimensions, and OSD display preference in `config.json`.

### Canvas & Visual Polish
* **Transparency Checkerboard Backdrop**: Added a GPU checkerboard pattern for transparent PNG, SVG, WebP, and ICO graphics, eliminating invisible dark artwork on black backgrounds.
* **On-Screen Display (OSD) HUD**: Added a sleek, non-intrusive floating status pill showing filename, index `[x/total]`, resolution, file size, and zoom % (toggleable via `Tab` and Preferences).
* **Dark Slate Canvas**: Replaced pitch-black out-of-bounds border with unified `#0c0e17` slate background matching the interface theme.

### Ergonomics, Navigation & Workflow
* **Mouse Navigation**: Added support for Mouse Back and Forward side buttons (`X1`/`X2`) to navigate between photos.
* **Extended Keyboard Shortcuts**: Added `PageUp`/`PageDown` (previous/next image), `Home`/`End` (jump to first/last file in folder), `Backspace` (previous image), and `Tab` (toggle OSD).
* **Rotation-Aware Intuitive Panning**: Transformed mouse delta vector by image rotation angle and flip state, ensuring mouse drag panning strictly matches screen movement.
* **Prioritized Thumbnail Scheduling**: Work queue dynamically sorts and prioritizes thumbnails centered around the currently viewed image, preventing thumbnail starvation in large folders (500+ files).
* **Fast Recycle Bin Deletion**: Deletion immediately removes file from view and moves to Recycle Bin without blocking confirmation dialogs (configurable in Preferences).
* **Save As... Dialog (`Ctrl+Shift+S`)**: Added dedicated "Save As..." dialog with format filter (PNG, JPEG, WebP) and automatic filename defaulting.
* **Configurable Double-Click Action**: Added preference to toggle between Fullscreen and 100% Zoom (actual pixels) on double-click.

## [0.9.2] - 2026-09-03

### Added & Fixed
* **Expanded Format Discovery**: Registered `.tif`, `.heic`, and `.heif` extensions in `ImageProcessor::supported_extensions()`, resolving directory filtering and file dialog omissions for TIFF, HEIC, and HEIF files.
* **Persistent Settings on `Esc` Exit**: Added graceful state preservation via `save_config_on_exit()` when exiting with the `Escape` key, ensuring window dimensions and toggled UI panels persist.
* **Clipboard Reliability**: Replaced unchecked `.unwrap()` calls in `copy_to_clipboard` and `copy_path_to_clipboard` with non-panicking error handling.
* **Portable Diagnostic Logging**: Removed hardcoded developer machine drive paths in `startup::log`, directing diagnostic traces to stderr and optionally `SPEDIMAGE_STARTUP_LOG_FILE`.
* **Zero-Allocation Histogram Acceleration**: Replaced per-chunk heap arena allocations in `compute_rgb_histogram` with stack buffers for faster parallel RGB analysis.

## [0.9.1] - 2026-08-27

### Performance & Startup Optimization
* **Pure-Rust SIMD JPEG Decoding**: Integrated `libjpeg-turbo-rs` with AVX2/SSE2 SIMD acceleration, reducing 4K JPEG decode latency from ~568ms to ~146ms with zero C/NASM dependencies.
* **Instant Window Presentation**: Window title is set to the image file name immediately upon window creation on the fast-path for sub-400ms end-to-end responsiveness and instant shell/taskbar discovery.
* **Asynchronous Parallel GPU Initialization**: Decoupled `wgpu` Renderer setup and surface configuration into background threads, running concurrently with initial image decoding.
* **Deferred Icon Decoding**: Moved application icon decompression and parsing off the critical startup path into a background task.
* **Optimized WGPU Configuration**: Direct3D 12 backend specialization on Windows, `LowPower` adapter preference for instant compositor matching, and deferred texture pool allocations.
* **Release Profile & SIMD Optimization**: Configured `opt-level = 3`, `lto = true`, and `codegen-units = 1` in release profile, accelerating decode throughput across all image formats.
* **Fast Single-Instance IPC**: Instant IPC file opening via local socket with automated window focus (`w.focus_window()`).
* **Fast-Path Diagnostics & Tracing**: Defer `color_eyre` panic hook and `tracing_subscriber` initialization behind environment flags (`RUST_BACKTRACE`, `COLORBT`, `RUST_LOG`) to eliminate cold-start overhead.
* **Streaming GIF Playback**: Background decoder stream with bounded look-ahead ring buffer and zero-latency first frame presentation.

## [0.9.0] - 2026-08-18

### Added
* **GPU High-Pass Sharpening & Clarity**: Real-time Laplacian high-pass sharpening and adaptive midtone clarity in WGSL fragment shader.
* **White Balance & Color Grading**: Interactive Color Temperature (Cool/Warm) and Tint (Green/Magenta) adjustment controls.
* **Dynamic Tone Recovery**: Real-time Highlights recovery/suppression and Shadows lift/crush controls.
* **A/B Split-Screen Comparison**: Live side-by-side comparison mode with interactive split slider and visual divider.
* **Aspect Ratio Crop Presets**: Quick framing presets (`1:1`, `16:9`, `4:3`, `3:2`, `9:16`) with centered auto-framing.

## [0.8.1] - 2026-07-30

### Added
* **Directional Predictive Prefetching**: Intelligent navigation velocity tracking (`+1, +2, +3` or `-1, -2, -3`) pre-decoding upcoming images in RAM for instant zero-delay slideshow and arrow-key transitions.

### Performance & Architecture
* **Early Pre-Downsampling Pipeline**: Re-architected image loader to downsample raw decoded RGBA pixels immediately before color profile transforms and rotation, cutting pixel processing overhead by ~12x.
* **JPEG Scale-on-Decode**: Configured native sub-sampled IDCT decoding directly in `zune-jpeg` via `DecoderOptions::set_max_width` / `set_max_height`, accelerating JPEG decode speeds by up to 8x.
* **Multi-Threaded Parallel ICC Profile Transforms**: Parallelized `qcms` color profile conversions across CPU cores using `rayon::par_chunks_mut`.
* **Instant Zero-Mipmap GPU Texture Uploads**: Streamlined WGPU texture creation in `Renderer::load_image` by eliminating CPU-side mipmap generation loops (`mip_level_count: 1`), enabling 1-step texture writes in < 1ms.
* **Fast-Path EXIF Extraction**: Separated lightweight orientation tag reading from full camera metadata string formatting, deferring tag parsing until the Info overlay is toggled.

## [0.8.0] - 2026-07-22

### Added
* **Pure Rust QOI Decoder Integration**: Native `.qoi` image file format support via the lightweight `qoi` crate.
* **OpenEXR (`.exr`) 32-bit HDR Decoding**: Added support for 32-bit High Dynamic Range `.exr` images via the pure-Rust `exr` crate.
* **Perceptual Oklab Color Science**: Integrated `palette` and `fast-srgb8` for sub-nanosecond gamma transformations and perceptual Oklab color calculations.
* **OS Single Instance Locking**: Robust single instance named OS mutex (`single-instance`) passing image paths seamlessly to the active window.
* **Fast Hash Indexing**: Integrated `rustc-hash` (`FxHashSet`) for instant integer set operations across UI selection states.
* **Bump Arena Allocations**: Integrated `bumpalo` arena allocations for thread-local histogram computations.

### Performance
* **Target CPU SIMD Vectorization**: Enabled `-C target-cpu=native` in `.cargo/config.toml` unlocking AVX2, AVX-512, FMA, and SSE4.2 vectorization across `fast_image_resize`, `bytemuck`, and histogram inner loops.
* **Resizer Buffer Reuse**: Refactored `fast_image_resize` resizer instances across mipmaps and multi-frame operations to eliminate redundant heap allocations.
* **Release Tracing Optimization**: Configured `release_max_level_info` on `tracing` to eliminate debug log checking in hot rendering loops during release builds.

## [0.6.1] - 2026-06-23

### Added
* **Smooth Viewport Zoom Lerping**: Implemented dynamically scaled scroll zooming that handles high-resolution trackpads (`PixelDelta`) and standard scroll wheels (`LineDelta`) proportionally, providing liquid-smooth zooming.

### Fixed
* **VSync Alignment**: Configured `wgpu::PresentMode::AutoVsync` to lock frame presentation to the display's refresh rate, eliminating screen tearing and micro-stuttering.
* **CPU/GPU Efficiency**: Replaced `ControlFlow::Poll` with `ControlFlow::Wait` for animation redraws, resolving the 100% CPU/GPU core usage spikes during zooming, panning, or transition events.
* **Pacing Clash**: Removed artificial 8ms wakeup timers, letting transition and scroll momentum animations run seamlessly at the monitor's native refresh rate.

## [0.6.0] - 2026-06-01

### Added
* **Interactive File Browser Sidebar**: Collapsible floating window showing adjacent files in the active directory, making navigation faster.
* **RGB Histogram Curve Overlay**: Dynamic overlay rendering real-time Red, Green, and Blue pixel distribution curves in semi-transparent layers.
* **System Clipboard Paste (`Ctrl+V`)**: Ability to paste image bitmaps directly from the system clipboard to view them instantly.
* **High-Zoom Fallback Filter**: Automatically falls back to nearest-neighbor filtering under high zoom levels ($\ge 5\times$) to prevent blurriness and keep pixel boundaries sharp for inspection.
* **Pure-Rust AVIF/AV1 Decoding**: Integrated AV1 payload parsing and decoding directly via the `heic` crate's `av1` feature, leveraging `rav1d-safe` cross-platform.
* **Full Camera RAW Support**: Integrated the `rawloader` crate to decode formats like ARW, CR2, NEF, DNG, etc. Features a fast binned 2x2 half-size preview demosaicer and black/white level normalization.

### Changed
* **HEIC Backends**: Updated the `heic` crate dependency and enabled platform HEVC decoding backends (`backend-rust` and `backend-mediafoundation`).
* **Dependency Upgrades**: Processed minor and patch updates for WGPU, egui, and other core libraries.
* **Documentation**: Overhauled the `README.md` to reflect native AVIF support and detailed camera RAW manufacturer details.

### Fixed
* **Code Style**: Reformatted the entire codebase using `cargo fmt` to match standard style guidelines.

---

## [0.5.0] - 2026-05-23

### Added
* **WGPU Native Mipmapping**: Generates mipmaps dynamically on texture uploads via `fast_image_resize` for aliasing-free rendering.
* **Active Color Profile Management**: Uses the `qcms` crate to parse and apply embedded ICC color profiles.

### Changed
* **Texture Recycling**: Transformed the main texture pipeline to reuse previous textures through a recycled buffer pool.

---

## [0.4.0] - 2026-04-27

### Added
* **Specialized Background Thread Pools**: Separated decoding and prefetching tasks into isolated thread pools.
* **Memory-Mapped I/O**: Integrated `memmap2` to load assets and files instantly.
* **Background EXIF Metadata Loading**: Decoupled EXIF parsing to keep the event loop non-blocking.
* **Single-Instance Enforcement**: Added named mutexes on Windows to enforce a single running application instance.

### Changed
* **Rust Upgrade**: Upgraded compiler target version to Rust 1.94.0.
* **Rust 2024 Edition**: Migrated the codebase to Rust Edition 2024.

### Fixed
* **Zoom Cursor Tracking**: Fixed zoom mapping to correctly follow the mouse cursor position.
* **Scroll Zooming**: Require `Ctrl` for scroll-zoom to avoid accidental zooming.
* **Legacy DLL Cleanups**: Removed legacy HEIF DLL references in the NSIS installer.

---

## [0.3.0] - 2026-04-20

### Added
* **WGPU 29 Upgrade**: Major upgrade to the latest GPU rendering backend pipeline.
* **Native HEIC/HEIF Support**: Enabled native HEIC/HEIF decoding using the pure-Rust `heic` crate.

### Changed
* **Shared Memory State**: Reorganized application structures to use an `Arc`-based model for optimal UI responsiveness.

---

## [0.2.0] - 2026-03-07

### Added
* **Premium UX overlays**: Integrated drag-and-drop loading, cross-fade transitions, and loading skeletons.
* **High-Performance Prefetching**: Prefetches adjacent files in the background using `quick_cache`.
* **Memory Allocation Optimizations**: Integrated the `mimalloc` memory allocator to boost performance.

### Fixed
* **Thumbnail Strip Rendering**: Resolved thumbnail strip rendering bugs using per-thumbnail uniform buffers.
* **PNG Color Space**: Corrected color reproduction issues by enforcing sRGB texture formats.

---

## [0.1.0] - 2026-02-15

### Added
* **Initial Rust Release**: Translated the GPU-accelerated viewer from the initial C++ codebase to memory-safe Rust.
* **GPU real-time shaders**: Implemented real-time GPU shader adjustments for brightness, contrast, saturation, and rotation.
* **Standard formats**: Full support for JPEG, PNG, GIF, BMP, SVG, and TIFF.
* **Rapid culling hotkeys**: Fast keyboard navigation (A/D/W/S, Arrows, Space, and Escape).
