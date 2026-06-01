# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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
