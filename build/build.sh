#!/bin/bash
# Build script for SpedImage on Linux

set -e

echo "SpedImage Build Script for Linux"
echo "================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect architecture
ARCH=$(uname -m)
echo "Detected architecture: $ARCH"

# Check for dependencies
echo ""
echo "Checking dependencies..."

# Check for GCC
if ! command -v gcc &> /dev/null; then
    echo -e "${RED}ERROR: GCC not found!${NC}"
    echo "Please install build-essential:"
    echo "  Ubuntu/Debian: sudo apt-get install build-essential"
    echo "  Fedora: sudo dnf install gcc"
    echo "  Arch: sudo pacman -S base-devel"
    exit 1
fi
echo -e "${GREEN}✓${NC} GCC found"

# Check for SDL2
if ! pkg-config --exists sdl2; then
    echo -e "${YELLOW}WARNING: SDL2 not found in pkg-config${NC}"
    echo "Please install SDL2 development files:"
    echo "  Ubuntu/Debian: sudo apt-get install libsdl2-dev"
    echo "  Fedora: sudo dnf install SDL2-devel"
    echo "  Arch: sudo pacman -S sdl2"
    exit 1
fi
echo -e "${GREEN}✓${NC} SDL2 found"

# Build
echo ""
echo "Building SpedImage..."
cd "$(dirname "$0")"

if make -f Makefile.linux; then
    echo ""
    echo -e "${GREEN}Build successful!${NC}"
    echo ""
    echo "Executable: ./spedimage"
    echo ""
    echo "Usage:"
    echo "  ./spedimage image.jpg"
    echo ""
    echo "To install system-wide:"
    echo "  sudo make -f Makefile.linux install"
else
    echo ""
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi
