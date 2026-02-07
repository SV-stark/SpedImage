@echo off
REM Build script for SpedImage on Windows
REM Requires: MSYS2 MinGW-w64 or Visual Studio with SDL2

echo SpedImage Build Script for Windows
echo ===================================
echo.

REM Check for compiler
where gcc >nul 2>nul
if %ERRORLEVEL% == 0 (
    echo Found GCC compiler
    set COMPILER=gcc
) else (
    where cl >nul 2>nul
    if %ERRORLEVEL% == 0 (
        echo Found MSVC compiler
        set COMPILER=msvc
    ) else (
        echo ERROR: No compiler found!
        echo Please install MSYS2 MinGW-w64 or Visual Studio
        echo.
        echo MSYS2 installation:
        echo 1. Download from https://www.msys2.org/
        echo 2. Install and open MSYS2 MinGW 64-bit
        echo 3. Run: pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-SDL2
        echo 4. Run this script from MSYS2 terminal
        exit /b 1
    )
)

REM Check for SDL2
if "%COMPILER%"=="gcc" (
    pkg-config --exists sdl2
    if %ERRORLEVEL% NEQ 0 (
        echo WARNING: SDL2 not found in pkg-config
        echo Trying to use default paths...
    )
)

REM Build
echo.
echo Building SpedImage...
cd /d "%~dp0\.."

if "%COMPILER%"=="gcc" (
    cd build
    make -f Makefile.win
    if %ERRORLEVEL% NEQ 0 (
        echo.
        echo Build failed!
        exit /b 1
    )
    echo.
    echo Build successful!
    echo Executable: spedimage64.exe
) else (
    echo MSVC build not implemented in this script
    echo Please use build/Makefile.win with MSYS2
)

echo.
echo Done!
pause
