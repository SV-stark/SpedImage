# 🖼️ SpedImage (C++ Edition)

**The Ultra-Lightweight, GPU-Accelerated Image Viewer with Native Performance.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

SpedImage is a high-performance, cross-platform image viewer designed for speed and efficiency. Rebuilt in **Modern C++20**, it utilizes a **Hybrid Backend Architecture** to leverage OS-native codecs (WIC on Windows) and high-performance system libraries (Linux) for instant loading, while employing **OpenGL Shaders** for real-time, zero-copy image editing.

## 🚀 Key Features

### ⚡ Hybrid Native Backend
*   **Windows (WIC)**: Uses the Windows Imaging Component for hardware-accelerated decoding.
    *   **HEIC/AVIF Support**: Automatically uses native extensions or falls back to bundled `libheif` for iPhone photos.
*   **Linux (Native)**: Built on a high-performance stack:
    *   **JPEG**: `libjpeg-turbo` (SIMD optimized).
    *   **PNG**: `libspng` (Statically linked, 2x faster than libpng).
    *   **HEIC**: System or bundled `libheif`.

### 🎨 GPU-Accelerated Editing
All adjustments are applied in real-time using **OpenGL Shaders**—no CPU processing required.
*   **Instant Adjustments**: Brightness, Contrast, and Saturation sliders work instantly on 4K/8K images.
*   **Lossless Rotation**: Shader-based rotation (90° increments).
*   **Interactive Crop**: Drag-and-drop cropping tool with visual overlay.
*   **Save & Export**: Save your edits to PNG/JPG (`Ctrl+S`).

### 🛠️ Modern UI
*   **Fluid Navigation**: Smooth zooming and panning.
*   **File Browser**: Integrated sidebar for quick navigation.
*   **Dark Mode**: A clean, professional, and dark-themed interface built with **Dear ImGui**.

---

## 🛠️ Building from Source

**Prerequisites:**
-   **CMake** (3.14+)
-   **C++ Compiler** (C++20 compliant: MSVC 2019+, GCC 10+, Clang 11+)
-   **Git**

### 🪟 Windows

1.  **Clone**:
    ```powershell
    git clone https://github.com/yourusername/spedimage.git
    cd spedimage
    ```
2.  **Build**:
    ```powershell
    cmake -B build
    cmake --build build --config Release
    ```
3.  **Run**: `./build/Release/SpedImage.exe`

### 🐧 Linux

1.  **Install Dependencies**:
    We provide a script to install necessary development libraries (`libjpeg-dev`, `libheif-dev`, etc.) for Debian, Ubuntu, Fedora, and Arch.
    ```bash
    ./install_deps.sh
    ```
2.  **Build**:
    ```bash
    cmake -B build
    cmake --build build --config Release
    ```
3.  **Run**: `./build/SpedImage`

---

## 📂 Project Structure

```text
spedimage/
├── src/
│   ├── main.cpp             # Entry point
│   ├── App.cpp              # Main application loop & logic
│   ├── GuiLayer.cpp         # ImGui rendering & UI
│   ├── Editor.cpp           # GPU Renderer (FBO, Shaders)
│   ├── Image.cpp            # Image resource wrapper
│   ├── ImageBackend.h       # Backend interface
│   ├── ImageBackend_Win32.cpp # WIC implementation
│   └── ImageBackend_Linux.cpp # LibJpegTurbo/LibSpng implementation
├── assets/                  # Fonts and icons
├── CMakeLists.txt           # Build configuration
└── README.md                # Documentation
```

## 📐 Technical Architecture
*   **Language**: C++20
*   **Windowing**: GLFW
*   **UI**: Dear ImGui (Docking Branch)
*   **Rendering**: OpenGL 3.3 Core Profile
*   **Image Decoding**: WIC (Win32), LibJpeg-Turbo/LibSpng (Linux)
*   **Image Encoding**: `stb_image_write`

---

## 📜 License
SpedImage is distributed under the **MIT License**.
