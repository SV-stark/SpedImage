# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.0] - 2026-10-07

### Added

**Decoders**
* **PSD, Netpbm and Radiance HDR**: Three new decoders, each a self-contained pure-Rust `zune-*` crate of 11-15 KB with no transitive dependencies, reached through zune-image's existing magic-byte sniffing so no new dispatch code was needed. PSD decodes to the flattened composite.
* **Farbfeld**: Decoded by a ~30-line hand-rolled parser rather than `zune-farbfeld`, whose `decode()` cannot succeed for any input. It allocates `4 * w * h` `u16` values and hands them to `decode_into`, which compares that element count against `output_buffer_size()` — a *byte* count that doubles for the 16-bit depth (`zune-farbfeld-0.5.2/src/decoder.rs:92-131`). The check fails for every file with "Too small output buffer size", so the crate is not in the dependency tree at all.
* **ICO and CUR**: Windows icons were listed in the file browser with no decoder behind them, so they appeared and then failed to open. Both are now parsed directly (~150 lines, no dependency): a 6-byte header, a directory of 16-byte entries, and per-entry payloads that are either a whole PNG or a headerless BMP DIB. Handles both `BITMAPCOREHEADER` and `BITMAPINFOHEADER`, the doubled height an icon DIB uses to cover the AND mask, and an all-zero alpha plane on 32bpp entries that would otherwise decode fully invisible.
* **Nine RAW Extensions the README Already Advertised**: `from_extension` matched only 7 RAW extensions while the README listed 11 camera families, so `.pef`, `.crw`, `.mrw`, `.rw2`, `.kdc`, `.dcr`, `.nrw`, `.srf` and `.sr2` files resolved to `Unknown` and were filtered out of the file browser even though `rawloader` can decode them. `rawloader::decode_file` identifies the container from its content, so the extension was only ever a routing hint. Now routed and listed.
* **File Associations for the New Formats**: `installer.nsi` registers and unregisters `.ico`/`.cur`, `.psd`, `.psb`, `.hdr`, the Netpbm extensions, `.ff` and the RAW extensions. Register and unregister lists are asserted to match by test.
* **Decode Tests for Every New Format**: ICO with both PNG and DIB payloads, CUR routing, ICO downsampling, Netpbm greyscale and colour, farbfeld, Radiance HDR, plus malformed-header, out-of-range-offset and oversized-entry cases asserting errors rather than panics.

**Editing and saving**
* **Lossless JPEG Rotation and Flips**: JPEG-to-JPEG saves whose only edits are 90° rotations and flips use libjpeg-turbo's DCT-domain transform (what `jpegtran` does). No pixel is re-encoded, EXIF and ICC data are kept, and the orientation tag is reset to 1 so viewers don't rotate twice. The source's EXIF orientation is folded into the same transform. Images with partial edge MCUs fall back to a re-encode. All 8 orientations × 12 edits are tested against the pixel pipeline.
* **Saving in Every Format the App Offers**: PNG via zune-image, JPEG via `libjpeg-turbo-rs` (quality 92, alpha composited onto white) and WebP via `image-webp`'s lossless VP8L encoder. Saves are written to a temporary file and renamed into place, so a failed encode cannot leave a truncated file. `tests/save_tests.rs` round-trips every format.

**Browsing and viewing**
* **Sort Options**: the file browser sorts by name (natural), date modified, date taken (EXIF `DateTimeOriginal`, falling back to the modification time), size or type, ascending or descending. The choice is saved. Capture dates are only read, in parallel, when sorting by them.
* **Side-by-Side Compare (`K`)**: pins the current image to the left half; browsing shows the next image on the right with shared zoom and pan. `K` or `Esc` closes it.
* **Window Position Is Remembered**, and only restored when the title bar would land on a connected monitor — a window last used on an unplugged screen would otherwise reopen where it cannot be grabbed.
* **"Loading full resolution…" Hint** while a zoomed-in image is being refined.

**Colour**
* **Monitor Colour Profile Matching**: on Windows, the ICC profile assigned to the window's monitor is read (`GetICMProfileW`). For matrix/TRC profiles, an sRGB→display matrix is applied as the last shader step. It is re-read when the window moves to another monitor, can be toggled in Preferences, and costs nothing on sRGB monitors. CPU buffers stay sRGB, so saving and the clipboard are unaffected.

**Startup**
* **Instant First Frame**: until the GPU is ready (0.6–2.5 s for DX12 device creation on integrated GPUs), the decoded image is painted with GDI. Decoding also starts before the window is created, sized from the saved window. The picture now appears ~170–220 ms after launch instead of after GPU init. The startup log gains `first frame with image` markers.

**Tooling**
* **Binary Size Gate**: `scripts/check_size.ps1`, wired into both GitHub workflows and exposed as `just size-check` / `just check-ci`.
* **`examples/verify_decode.rs`**: Decodes a file and prints sampled pixels, for checking a decoder against a real file rather than only unit tests.
* **WebP and AVIF Decode Tests**: Fixtures and regression tests covering the alpha/no-alpha buffer layouts and the AVIF explanation. `image-webp` emits 3 bytes/pixel without an alpha channel and 4 with one, which a naive decoder silently mishandles.
* **Extension/Decoder Consistency Test**: Asserts every extension in `SUPPORTED_EXTENSIONS` is one `is_supported()` accepts, so the two lists cannot drift apart again.

### Changed

* **Dependency Tree Trimmed**: The Windows release binary went from 23.3 MB to 18.3 MB (−5.0 MB, −21.5%), measured against a baseline build of the previous commit. The release dependency count dropped from 359 to 305 crates on the Windows target, net −54. The savings came from the AGPL `heic` subtree (`rav1d-safe`, `ultrahdr-core`, `archmage`, `magetypes`, `whereat`, `safe_unaligned_simd`), the duplicate `jxl-oxide` 0.12 that `zune-image`'s default features pulled in beside the 0.11 already in use, and the `image` crate that `arboard`'s default features dragged in.
* **`arboard` Dropped on Windows**: Clipboard access now goes directly through Win32 `CF_DIB` / `CF_UNICODETEXT` / `CF_HDROP`, extending the `CF_HDROP` code that already existed. This removes the `image` crate — a fourth PNG decoder, and a second TIFF crate on macOS — from the Windows build. It also fixes `Ctrl+C`, which previously opened the clipboard a second time and could drop the file drop if anything else held the clipboard mid-operation. Clipboard images on non-Windows platforms are behind the new `clipboard-image` feature.
* **`zune-image` Default Features Disabled**: The generic fallback path only ever reached PNG, BMP and WebP, since every other format had a specialised loader that claimed it first. Enabling defaults added PSD, PPM, HDR, farbfeld, QOI (duplicating the `qoi` crate), JPEG XL (a second `jxl-oxide`) and the `jpeg-encoder` *encoder* to a viewer. `psd`, `ppm` and `hdr` are now enabled individually because they turned out to be worth their 11-15 KB each; `farbfeld`, `qoi` and `jpeg-xl` remain off.
* **WebP Decoded Without `zune-image`**: `zune-image` 0.5.0's WebP codec has an ungated `use jxl_oxide::...` in it, so its `webp` feature only compiles alongside the `jpeg-xl` feature this change removes. WebP is now decoded through `image-webp` directly, which is the crate that codec wraps. Verified byte-identical to the previous path on lossy, lossless, alpha and animated samples.
* **Release Binary Is Now 18.9 MB**, up from 18.3 MB for the JPEG/WebP encoders and the lossless transform, still within the 22 MB budget enforced in CI.
* **README and CI Comments Now Cite What Is Measured**: The benchmark section explains that its headline number times a titled window appearing, not the image — about 320 ms of it is Windows creating the process before any app code runs — and points at the startup log's `first frame with image` marker instead. The stale "~10MB base app size" comment in the release workflow was replaced with the budget that is actually enforced.

### Fixed

* **GPU Renderer Never Initialised (Nothing Was Ever Drawn)**: `Renderer::new` ran on a background thread and created the WGPU surface there. winit 0.30.13 refuses to hand out a window handle off the event-loop thread on Windows, so surface creation failed on every launch and the window stayed empty with "GPU Init Error" in a status line nobody saw. The README benchmark only waited for the window *title*, so it never noticed. The surface is now created on the event-loop thread (cheap); adapter/device setup still runs in the background. The startup-log visibility probe had the same bug.
* **Saving JPEG and WebP Always Failed**: `ImageBackend::save` relied on zune-image picking an encoder by extension, but only its `png` encoder is compiled in. Ctrl+S on a `.jpg`, batch save of JPEGs, and the JPEG/WebP choices in Save As all errored. The only save test wrote a PNG, so CI never caught it.
* **Edited Copies of Unwritable Formats**: Ctrl+S and batch save kept the source extension, asking for a HEIC/RAW/SVG/TIFF encoder that does not exist. They now write `name_edited.png`, and an unsupported extension in Save As gives a clear message naming the formats that work.
* **Full-Resolution Saves Could Be Sideways**: when the on-screen buffer was already full resolution, saving reused it without applying the EXIF rotation the preview defers to the GPU.
* **Stuck Progressive-Refinement Flag**: a failed or oversized full-resolution re-decode never cleared `highres_in_flight`, so refinement never ran again for that image.
* **Damaged Settings Reset Everything**: one malformed field in `config.json` made the whole parse fail and silently reset every preference. Each field is now read on its own, missing fields take defaults, the damaged file is copied to `config.invalid.json`, and a dismissible notice says what was reset.
* **AVIF Support Removed**: The `heic` git dependency was replaced with `heic-rs`, which decodes HEIC/HEIF only. AVIF is an HEIF container holding AV1 rather than HEVC, so it needs an AV1 decoder. `.avif` files are now filtered out of the file browser, are no longer registered as a file association by the installer, and opening one explicitly reports why. HEVC-in-HEIF — what cameras and phones actually write — is unaffected.
* **Unused Dependencies Removed**: `zune-imageprocs`, `parking_lot` and `bumpalo` were declared but never referenced anywhere in the tree. The README advertised a "Bump Allocation Arena" feature that did not exist; that line has been replaced with a description of the clipboard work that actually shipped.
* **AGPL Decoder in an MIT Application**: HEIC/AVIF was decoded by a git-pinned build of `heic` licensed **AGPL-3.0-only**, linked into an MIT-licensed app and shipped in the installer. That was a licensing liability independent of build size.
* **`tga` and `ico` Were Listed With No Decoder**: `SUPPORTED_EXTENSIONS` gated the file browser while `ImageFormatType::is_supported` gated the loaders, and the two had drifted: `.tga` and `.ico` sat in the browser's list with nothing able to open them. A test had been carrying an explicit exemption for both instead of fixing them. `ico` now decodes; `tga` has no decoder anywhere in the tree and has been removed from the list, matching how `.avif` is handled.
* **README Claimed an Unverifiable Size**: The "~10MB base app size" figure was never checked by CI and was wrong by more than half — the release binary was 23.3 MB.
* **README Credited the Wrong Crates**: The format table credited PNG, GIF, BMP, TIFF, WebP and JXL to `zune-image`, but `gif`, `tiff`, `image-webp` and `jxl-oxide` each have their own dependency and dedicated loader. Each row now names the crate that actually decodes it.
* **`.avif` Files Were Listed in the Browser Anyway**: `is_supported()` maps an extension to a format, but the browser filters on a separate `SUPPORTED_EXTENSIONS` list. Marking `Avif` unsupported changed only the former, so `.avif` files kept appearing next to the images that do work, and opened into an error.
* **Stale `imagepipe` Comment**: `is_supported` was annotated "Core formats + RAW via imagepipe", but `imagepipe` is not a dependency of this crate. The comment now describes what actually happens.

## [0.10.1] - 2026-10-01

### Fixed
* **Panic on Zero-Dimension Images**: `crop_rgba` called `clamp(1, width - cx)` with `min > max` whenever a decode produced an empty image, aborting the process. `rotate_rgba` and `flip_rgba` are guarded the same way.
* **GIF Decode Panics & Out-of-Bounds Writes**: A zero-size GIF sub-frame triggered `chunks_exact(0)` (abort), and frame rectangles larger than the logical screen were trusted unchecked. Both the loader and the background streaming decoder now clamp frame geometry and skip empty sub-frames.
* **Silent Black Window on Oversized Images**: The device was created with `wgpu::Limits::default()`, capping textures at 8192px, so any larger image failed to upload — and the error was discarded with `.ok()`, leaving no explanation. Now uses `adapter.limits()` and reports upload failures in the status bar.
* **100% CPU Spin When GPU Init Failed**: `dirty` was only cleared inside the `if let Some(renderer)` block, so a failed GPU init left it set forever and the event loop re-requested redraws indefinitely.
* **Animations Freezing Mid-Transition**: The event loop set `ControlFlow::Wait` while the crop/zoom lerp was still interpolating. It now polls while anything is animating and sleeps only when visually static.
* **Ragged Buffers Reaching the Encoder**: `apply_adjustments_cpu` passed a non-multiple-of-4 buffer straight through to file encoding and GPU upload.
* **Mirrored EXIF Orientations Dropped**: Orientation values 2, 4, 5, and 7 were ignored entirely, displaying those photos sideways or mirrored. Mirrors are now always baked into the buffer (the GPU path can only express rotation); pure rotations still defer to the shader.
* **Aspect Presets Produced Wrong Ratios**: The `1:1` preset hardcoded a normalized fraction of `0.8`, yielding a *non-square* crop on any non-square image, and the other presets ignored the source aspect entirely. Ratios are now corrected against the image's own dimensions.
* **Stale Selection Indices**: Deleting a file shifted every subsequent multi-select index onto the wrong file, and a directory reload left selections pointing at unrelated files.
* **Thumbnails Beyond 200 Files Never Loaded**: The thumbnail work queue was truncated to `MAX_THUMBNAILS` *after* being distance-sorted, so in larger folders everything past the cap was permanently blank. Eviction now happens at upload time (furthest from the viewport) instead of dropping work.

### Added
* **Working Crop Mode**: Crop was advertised in the help text but non-functional — `is_cropping` was never drawn and the click handler had an empty branch. Now supports drag-to-move, corner-handle resize, and a dimmed-outside selection overlay with live pixel dimensions.
* **GPS Coordinates & Color Space**: `extract_exif_and_orientation` existed but had no callers, so the "Open in Maps" button could never appear and the Adobe RGB correction never engaged. A single EXIF parse now populates orientation, GPS, and color space together.
* **Scaled JPEG Decode**: libjpeg-turbo's scaled IDCT is used with the largest reduction that still leaves the CPU resize a pure downscale, so output quality is unchanged while the intermediate buffer for a 24 MP photo drops from 96 MB to 1.5 MB.
* **Decode Benchmarks**: Added `bench_jpeg_*`, `bench_rotate_*`, `bench_flip_*`, and `bench_crop_*` to `benches/image_processing.rs` using a deterministic synthetic corpus, so decode-path performance stays reproducible.
* **Decode-Path Integration Tests**: New `tests/loader_tests.rs` covers real encode/decode round-trips, decode-budget invariants, and malformed input (truncated JPEGs, hostile GIF geometry, garbage bytes) to confirm errors are returned rather than panics.

### Changed
* **Allocation-Free Format Detection**: `is_supported` allocated a lowercase `String` and a `Vec` per directory entry; it now matches against a `const` slice with `eq_ignore_ascii_case`.
* **Indexed Prefetch Eviction**: Cache eviction was `O(cache × files)` with a full file-list clone on every keystroke; now indexed and only clones paths.
* **Parallel Pixel Transforms**: `rotate_rgba`, `flip_rgba`, and `crop_rgba` were single-threaded; all three now parallelize with rayon.
* **Single EXIF Pass**: Non-JPEG containers previously opened and parsed the file twice. Metadata is now read once and reused.
* **Fewer Per-Frame Allocations**: Status text is borrowed instead of `to_string()`-ed each frame, and the surface is only reconfigured on an actual size change.

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
