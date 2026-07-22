# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
