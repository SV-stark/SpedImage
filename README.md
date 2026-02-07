# 🖼️ SpedImage

**The Ultra-Lightweight, Hardware-Accelerated Image Viewer for Modern Systems.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

SpedImage is a high-performance, cross-platform image viewer designed for speed and efficiency. Written in pure C99 with SDL2 hardware acceleration, it delivers a near-instant viewing experience while maintaining an incredibly small footprint.

## 🚀 Why SpedImage?

In an era of bloated software, SpedImage focuses on three core principles:
1.  **Speed**: Instant startup and smooth hardware-accelerated panning/zooming.
2.  **Effiency**: Minimal RAM usage (<30MB typical) and tiny binary size (~3-5MB).
3.  **Simplicity**: A distraction-free interface that puts your images front and center.

---

## 📊 Comparison

| Feature | SpedImage | Common System Viewers | Pro Photo Suites |
| :--- | :---: | :---: | :---: |
| **Startup Speed** | ⚡ Instant | 🐢 Moderate | 🐌 Slow |
| **RAM Usage** | 🧊 Ultra-Low (<30MB) | 📦 High (150MB+) | 🐘 Extreme (500MB+) |
| **Binary Size** | 📉 ~5MB | 📂 100MB+ | 🏗️ 1GB+ |
| **Hardware Accel.** | ✅ Yes (SDL2) | ⚠️ Partial | ✅ Yes |
| **Portable** | ✅ Yes | ❌ No | ❌ No |
| **Focus** | Speed/Viewing | General Use | Heavy Editing |

---

## ✨ Features

### 🖼️ High-Performance Viewing
- **Hardware Acceleration**: Smooth 60fps rendering via SDL2.
- **Vast Format Support**:
    - **Standard**: JPEG, PNG, BMP, GIF, WebP, TIFF, PSD, TGA.
    - **Vector**: Full SVG rendering support.
    - **Raw/Pro**: Optional support for HEIC, AVIF, and various RAW formats (CR2, NEF, ARW, DNG).
- **Smart Caching**: Configurable LRU cache for fast navigation through large directories.

### 🛠️ Integrated Editing Tools
- **Precision Cropping**: Visual selection with interactive draggable handles.
- **Rotation**: Lossless 90° increments and free rotation controls.
- **Image Adjustments**: Real-time brightness and contrast sliders.
- **Export Power**: Save your edits directly to JPEG, PNG, BMP, or TGA.

### ⌨️ Seamless Navigation
- **Fluid Zooming**: Mouse wheel and keyboard-driven zoom with focal point tracking.
- **Display Modes**: Toggle between Fit-to-Window, 1:1 Scale, and distraction-free Fullscreen.
- **Slideshow**: Automated playback with configurable delay.
- **Sidebar Integration**: Quickly browse files in the current directory.

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| **`←`** / **`→`** | Previous / Next Image |
| **`+`** / **`-`** | Zoom In / Out |
| **`0`** | Fit to Window |
| **`F`** / **`F11`** | Toggle Fullscreen |
| **`C`** | Enter Crop Mode |
| **`R`** | Rotate 90° Clockwise |
| **`B`** | Open Brightness/Contrast Tool |
| **`Space`** | Toggle Slideshow |
| **`F1`** | Show/Hide Sidebar |
| **`Delete`** | Delete Current Image |
| **`Esc`** / **`Q`** | Exit Tool / Quit Application |

---

## 🛠️ Building from Source

### 🐧 Linux
**Prerequisites:** SDL2 development libraries.
```bash
# Ubuntu/Debian: sudo apt-get install build-essential libsdl2-dev
# Fedora: sudo dnf install gcc SDL2-devel
# Arch: sudo pacman -S base-devel sdl2

cd build
make -f Makefile.linux
```

### 🪟 Windows
**Prerequisites:** MSYS2 with MinGW toolchain.
```bash
# Install toolchain in MSYS2: 
# pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-SDL2

cd build
make -f Makefile.win
```

---

## 📖 Usage

SpedImage is designed for ease of use via the command line or file associations.

```bash
# Open a specific image
spedimage portrait.jpg

# Open an entire directory for browsing
spedimage /path/to/gallery/
```

---

## 📂 Project Structure

```text
spedimage/
├── src/           # Core logic (loading, UI, editor)
├── include/       # External header-only libraries
├── build/         # Build scripts and Makefiles
└── README.md      # You are here
```

---

## 📐 Technical Architecture

SpedImage is built for longevity and portability:
- **Language**: Standard C99.
- **Backend**: SDL2 for windowing and hardware-accelerated blitting.
- **Decoding**: Bloat-free `stb_image` and `nanosvg` implementations.
- **Memory**: Efficient LRU (Least Recently Used) image caching logic.

---

## 📜 License & Credits

SpedImage is distributed under the **MIT License**. See the [LICENSE](LICENSE) file for the full license text.

**Special Thanks to:**
- **Sean Barrett** for `stb_image`.
- **Mikko Mononen** for `nanosvg`.
- **Sam Lantinga** and the SDL2 contributors.
