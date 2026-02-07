# 🖼️ SpedImage (C++ Edition)

**The Ultra-Lightweight, Hardware-Accelerated Image Viewer for Modern Systems.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

SpedImage is a high-performance, cross-platform image viewer designed for speed and efficiency. Rebuilt in **Modern C++20**, it leverages **Dear ImGui** and **OpenGL** to deliver a professional, fluid interface while maintaining a tiny footprint.

## 🚀 Why SpedImage?

In an era of bloated software, SpedImage focuses on three core principles:
1.  **Speed**: Instant startup and smooth hardware-accelerated panning/zooming.
2.  **Efficiency**: Minimal RAM usage and tiny binary size (<1MB executable).
3.  **Modern Power**: A robust, debuggable UI built with industry-standard tools.

---

## 📊 Comparison

| Feature | SpedImage | Common System Viewers | Pro Photo Suites |
| :--- | :---: | :---: | :---: |
| **Startup Speed** | ⚡ Instant | 🐢 Moderate | 🐌 Slow |
| **RAM Usage** | 🧊 Ultra-Low | 📦 High (150MB+) | 🐘 Extreme (500MB+) |
| **UI Tech** | **Dear ImGui** (GPU) | Heavy Frameworks | Custom / Qt |
| **Hardware Accel.** | ✅ OpenGL 3.3+ | ⚠️ Partial | ✅ Yes |
| **Portable** | ✅ Yes | ❌ No | ❌ No |

---

## ✨ Features

### 🖼️ High-Performance Viewing
-   **Hardware Acceleration**: Smooth 60fps rendering via OpenGL.
-   **Vast Format Support**:
    -   **Standard**: JPEG, PNG, BMP, GIF, WebP, TGA.
    -   **Vector**: Full SVG rendering support.
-   **Smart Caching**: Efficient texture management using RAII.

### 🛠️ Integrated Editing Tools
-   **Precision Cropping**: Visual selection with interactive handles.
-   **Rotation**: Lossless 90° increments.
-   **Image Adjustments**: Real-time brightness and contrast sliders.
-   **Export Power**: Save your edits directly to standard formats.

### ⌨️ Seamless Navigation
-   **Fluid Zooming**: Mouse wheel and keyboard-driven zoom.
-   **Modern UI**: Floating toolbars, dockable panels, and file browser sidebar.
-   **Slideshow**: Automated playback with configurable delay.

---

## 🛠️ Building from Source

**Prerequisites:**
-   **CMake** (3.14+)
-   **C++ Compiler** (GCC 10+, Clang 11+, or MSVC 2019+ with C++20 support)
-   **Git** (to fetch dependencies)

### 🐧 Linux / 🪟 Windows / 🍎 macOS

SpedImage uses **CMake FetchContent** to automatically download and build dependencies (GLFW, Dear ImGui).

```bash
# 1. Clone the repository
git clone https://github.com/yourusername/spedimage.git
cd spedimage

# 2. Configure the project
cmake -B build

# 3. Build the project
cmake --build build --config Release
```

The executable will be located in `build/Release/SpedImage.exe` (Windows) or `build/SpedImage` (Linux/Mac).

---

## 📂 Project Structure

```text
spedimage/
├── src/
│   ├── main.cpp       # Entry point
│   ├── App.cpp        # Main application loop
│   ├── GuiLayer.cpp   # ImGui rendering logic
│   ├── Image.cpp      # RAII wrapper for OpenGL textures
│   └── Editor.cpp     # Image processing logic
├── assets/            # Fonts and icons
├── CMakeLists.txt     # Build configuration
└── README.md          # You are here
```

---

## 📐 Technical Architecture

SpedImage is built for longevity and maintainability:
-   **Language**: **C++20** (Concepts, Smart Pointers, auto).
-   **Windowing**: **GLFW** for robust cross-platform window management.
-   **UI**: **Dear ImGui** for immediate-mode, GPU-accelerated interface.
-   **Rendering**: **OpenGL 3.3+** for high-performance 2D blitting.
-   **Decoding**: `stb_image` (pixels) and `nanosvg` (vectors).

---

## 📜 License

SpedImage is distributed under the **MIT License**.
