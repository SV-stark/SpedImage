# 🖼️ SpedImage
**Ultra-Lightweight, GPU-Accelerated Image Viewer with Native Performance.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

SpedImage is a high-performance, cross-platform image viewer rebuilt in **Rust** with **WGPU** for GPU-accelerated rendering. It provides memory-safe, zero-copy image processing with real-time adjustments.

## 🚀 Key Features

### ⚡ High-Performance Image Loading
- **Pure Rust Image Decoding**: Using the `image` crate for safe, efficient decoding
- **Wide Format Support**: JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC, AVIF
- **Memory Efficient**: Zero-copy GPU texture loading

### 🎨 GPU-Accelerated Editing
All adjustments are applied in real-time using **WGPU Shaders**—no CPU processing required.
- **Instant Adjustments**: Brightness, Contrast, and Saturation work instantly on 4K/8K images
- **Lossless Rotation**: Shader-based rotation (90° increments)
- **Interactive Crop**: Drag-and-drop cropping tool with visual overlay
- **Save & Export**: Save your edits to PNG/JPG

### 🛠️ Modern Tech Stack
- **Language**: Rust (Memory-safe, high-performance)
- **GPU API**: WGPU (Cross-platform: Vulkan/Metal/DX12/OpenGL)
- **Windowing**: winit
- **Image Processing**: Custom WGSL shaders

---

## 🛠️ Building from Source

**Prerequisites:**
- **Rust** (1.75+)
- **Cargo** (comes with Rust)

### 🪟 Windows / 🐧 Linux / 🍎 macOS

1. **Clone**:
   ```bash
   git clone https://github.com/yourusername/spedimage.git
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

## 📂 Project Structure

```
spedimage/
├── src/
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library root
│   ├── app.rs             # Main application & event loop
│   ├── gpu_renderer.rs    # WGPU rendering pipeline
│   ├── image_backend.rs    # Image loading & decoding
│   └── ui.rs              # UI state management
├── Cargo.toml             # Rust package manifest
└── README.md              # Documentation
```

---

## 📐 Technical Architecture

| Component | Technology |
|-----------|------------|
| **Language** | Rust 2021 |
| **Windowing** | winit |
| **GPU Rendering** | WGPU (Vulkan/Metal/DX12/OpenGL) |
| **Image Decoding** | `image` crate (pure Rust) |
| **Shaders** | WGSL |

### Why Rust + WGPU?
- **Memory Safety**: No buffer overflow or use-after-free bugs
- **Cross-Platform**: Single codebase compiles to Windows/Linux/macOS
- **Modern GPU API**: WGPU provides safe access to modern graphics APIs
- **Performance**: Comparable to C++ performance with better safety guarantees

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `A` / `←` | Previous image |
| `D` / `→` | Next image |
| `W` / `↑` | Previous image |
| `S` / `↓` | Next image |
| `R` | Rotate 90° |
| `C` | Toggle crop mode |
| `O` | Open file dialog |
| `Ctrl+S` | Save image |
| `F` | Toggle sidebar |
| `1` | Reset adjustments |
| `+` / `-` | Zoom in/out |
| `0` | Zoom to fit |
| `Q` | Cancel crop / Quit |

---

## 📜 License

SpedImage is distributed under the **MIT License**.
