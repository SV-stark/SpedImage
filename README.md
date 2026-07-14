<p align="center">
  <img src="assets/icons/icon.png" width="128" height="128" alt="SpedImage Icon">
</p>

<h1 align="center">🖼️ SpedImage</h1>

<p align="center">
  <strong>Ultra-Lightweight, GPU-Accelerated Image Viewer with Native Performance.</strong>
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/Version-0.7.2-blue" alt="Version: 0.7.2"></a>
  <a href="#"><img src="https://img.shields.io/badge/Rust-1.82+-orange" alt="Rust: 1.82+"></a>
  <a href="#"><img src="https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-lightgrey" alt="Platform: Windows | Linux | macOS"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

<p align="center">
  SpedImage is a high-performance, cross-platform image viewer rebuilt in <strong>Rust</strong> with <strong>WGPU</strong> for GPU-accelerated rendering. It provides memory-safe, zero-copy image processing with real-time adjustments.
</p>

## 📋 Table of Contents
- [Key Features](#-key-features)
- [Format Support](#-format-support)
- [Usage & CLI](#-usage--cli)
- [Building from Source](#-building-from-source)
- [Project Architecture](#-project-architecture)
- [Keyboard Shortcuts](#-keyboard-shortcuts)
- [Contributing](#-contributing)

---

<h2 align="center">🚀 Key Features</h2>

### ⚡ High-Performance Image Loading
- **Memory-Mapped Decoding**: Memory-mapped file I/O (`memmap2`) for JPEG, PNG, WebP, and GIF formats, eliminating heavy buffer allocation overhead.
- **Rayon Parallelization**: Multi-threaded pixel auto-rotation, JXL float-to-u8 scaling, and fast RGB histogram calculations mapped-reduced across CPU cores.
- **Fast Startup**: Native performance with immediate viewport rendering.

### 🎨 GPU-Accelerated Editing & Modern UI
All adjustments are processed dynamically in WGSL fragment shaders.
- **Instant Adjustments**: Brightness, Contrast, and Saturation applied directly in real-time.
- **Image Flipping**: Horizontal and vertical flipping (mirroring) processed directly in the vertex shader.
- **Gamut Correction**: Automatic detection of Adobe RGB color profile tags, executing high-quality color space conversion matrices directly on the GPU.
- **HDR Toning**: Real-time filmic **Reinhard tone-mapping** for extended-range lighting (`H`).
- **Sleek Docked UI**: Borderless dashboard cards for Adjustments and File Browser styled with custom dark-slate gradients and cyan accents.
- **Double-Click Fullscreen**: Double-click anywhere on the viewport to toggle Borderless Fullscreen.

### 🔌 Native Desktop Integrations
- **Recycle Bin Integration**: Delete files safely using the native OS Recycle Bin (`Delete` key) and instantly advance to the next image.
- **Clipboard Operations**: Copy images to the clipboard (`Ctrl+C`) or paste directly from the clipboard (`Ctrl+V`).
- **Wallpaper Control**: Set the currently viewed image as your desktop background (`Ctrl+W`).
- **GPS Map Lookup**: Clickable `📍 Open in Maps` button in the metadata info card, launching coordinates directly in your default browser.
- **F2 In-App Renaming**: Safely rename files in-app with automatic directory watching and sorting.

---

<h2 align="center">🖼️ Format Support</h2>

<div align="center">

| Format | Decoding Engine | OS Support |
|--------|-----------------|------------|
| JPEG, PNG, GIF, BMP, TIFF, WebP, JXL | Pure Rust (`zune-image` / `jxl-oxide`) | All Platforms |
| RAW (CR2, NEF, ARW, DNG, etc.)* | Pure Rust (`rawloader` crate) | All Platforms |
| SVG | `resvg` crate | All Platforms |
| HEIC / AVIF | Pure Rust (`heic` crate with `av1` feature) | All Platforms |

</div>

*\* Supported RAW formats include Canon (CR2, CRW), Sony (ARW, SRF, SR2), Nikon (NEF, NRW), Fujifilm (RAF), Olympus (ORF), Pentax (PEF), Samsung (SRW), Minolta (MRW), Kodak (KDC, DCR), Panasonic/Leica (RW2), and Adobe DNG.*

---

<h2 align="center">⚡ Performance Benchmarks</h2>

<p align="center">
  Based on typical consumer systems (Apple M-series or Intel/AMD multicore CPU + mid-range GPU). Latencies and memory scales represent high-resolution (24MP+) photos.
</p>

<div align="center">

| Operation | Typical Latency | CPU Usage | Memory Impact | 
|-----------|-----------------|-----------|---------------|
| **Cold Start to Render** | < 100ms | Spike on load | Base app size (~10MB) |
| **Decoding (e.g., 24MP JPEG)** | 50-150ms | Multi-core spike | Dependent on image res |
| **GPU Upload (Zero-Copy)** | < 5ms | Near Zero | Video RAM mapped directly |
| **HDR Toning (Filmic)** | 0.0ms (0 CPU) | Zero | None |
| **Smooth Crop/Zoom Animation** | 60 FPS | Nominal (< 2%) | None |
| **Brightness/Contrast Adjust** | 0.0ms (0 CPU) | Zero | None |

</div>

---

## 💻 Usage & CLI

Launch SpedImage normally, or open a specific image directly from the command line:

```bash
# Open SpedImage in the current directory
spedimage

# Open a specific image
spedimage /path/to/image.jpg
```

---

## ⚙️ Building from Source

**Prerequisites:**
- **Rust** (1.82+)
- **Cargo** (comes with Rust)

### 🪟 Windows / 🐧 Linux / 🍎 macOS

1. **Clone**:
   ```bash
   git clone https://github.com/SV-stark/SpedImage.git
   cd spedimage
   ```

2. **Build**:
   ```bash
   cargo build --release
   ```

3. **Run**:
   ```bash
   cargo run --release
   ```

---

<h2 align="center">📐 Project Architecture</h2>

<p align="center">
  Built with a state-of-the-art native stack emphasizing <strong>Memory Safety</strong> and <strong>Performance</strong>.
</p>

<div align="center">

| Component | Technology | Description |
|-----------|------------|-------------|
| **Language** | Rust 2021 | Eliminates buffer overflows and data races. |
| **Windowing** | winit | Cross-platform, reliable event loop. |
| **GPU Rendering** | WGPU | Safe access to Vulkan/Metal/DX12/OpenGL. |
| **Image Decoding**| `image` / OS codecs | Hybrid approach for maximum format compatibility. |
| **Shaders** | WGSL | Highly optimized GPU processing blocks. |

</div>

---

<h2 align="center">⌨️ Keyboard Shortcuts</h2>

<div align="center">

| Key | Action |
|-----|--------|
| `A` / `W` | Previous image |
| `D` / `S` | Next image |
| `Right` / `Left` Arrow | Next / Previous image |
| `R` | Rotate 90° |
| `H` | Toggle HDR Toning |
| `C` | Toggle crop mode |
| `I` | Toggle image info (EXIF) |
| `O` | Open file dialog |
| `Ctrl+P` | Print image (Windows) |
| `Ctrl+S` | Save image |
| `Ctrl+F` | Open Search / Find |
| `F11` / `Double Click` | Toggle Fullscreen |
| `Ctrl+W` | Set as Desktop Wallpaper |
| `Ctrl+C` | Copy image to clipboard |
| `Ctrl+Shift+C` | Copy image file path |
| `Ctrl+V` | Paste image from clipboard |
| `F2` | Rename current file |
| `Delete` | Move current file to Recycle Bin / Trash |
| `Shift+Delete`| Batch delete selected |
| `Enter` | Toggle Zoom 100% (Actual pixels) / Zoom to fit |
| `F` | Toggle sidebar |
| `T` | Toggle thumbnail strip |
| `1` | Reset adjustments |
| `+` / `=` | Zoom in |
| `-` | Zoom out |
| `0` | Zoom to fit |
| `Esc` | Cancel crop / Quit |
| `?` | Toggle help overlay |

</div>

---

<h2 align="center">📈 Codebase Health</h2>

<p align="center">
  <img src="scorecard.png" width="100%" alt="Desloppify Scorecard">
</p>

## 🤝 Contributing
Contributions, issues, and feature requests are welcome! Feel free to check out the [issues page](https://github.com/SV-stark/SpedImage/issues) if you want to contribute.

---

## 📜 License
SpedImage is distributed under the **[MIT License](LICENSE)**.
