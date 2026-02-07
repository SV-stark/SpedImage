#!/bin/bash

# SpedImage Dependency Installer
# Installs required libraries for the Linux build.

echo "Detecting distribution..."

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$NAME
fi

if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]] || [[ "$OS" == *"Pop"* ]] || [[ "$OS" == *"Mint"* ]]; then
    echo "Detected Debian/Ubuntu based system."
    echo "Installing libjpeg-dev, libheif-dev, libxqv-dev..."
    sudo apt update
    sudo apt install -y libjpeg-dev libheif-dev libpng-dev cmake build-essential libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev xorg-dev
elif [[ "$OS" == *"Fedora"* ]]; then
    echo "Detected Fedora."
    sudo dnf install -y libjpeg-turbo-devel libheif-devel libpng-devel libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel
elif [[ "$OS" == *"Arch"* ]] || [[ "$OS" == *"Manjaro"* ]]; then
    echo "Detected Arch Linux."
    sudo pacman -S --needed base-devel libjpeg-turbo libheif libpng
else
    echo "Unsupported or unknown distribution: $OS"
    echo "Please manually install: libjpeg, libpng, libheif, and X11 development headers."
    exit 1
fi

echo "Dependencies installed successfully!"
echo "You can now build SpedImage with:"
echo "  cmake -B build"
echo "  cmake --build build --config Release"
