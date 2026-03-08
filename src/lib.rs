//! SpedImage - Ultra-Lightweight GPU-Accelerated Image Viewer
//!
//! A high-performance, cross-platform image viewer built with Rust + WGPU.
//! Features GPU-accelerated image processing and a modern native UI.

pub mod app;
pub mod image;
pub mod render;
pub mod ui;

pub use app::SpedImageApp;
pub use image::ImageBackend;
pub use render::Renderer;
