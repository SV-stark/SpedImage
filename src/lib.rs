//! SpedImage - Ultra-Lightweight GPU-Accelerated Image Viewer
//!
//! A high-performance, cross-platform image viewer built with Rust + WGPU.
//! Features GPU-accelerated image processing and a modern native UI.

pub mod app;
pub mod gpu_renderer;
pub mod image_backend;
pub mod ui;

pub use app::SpedImageApp;
pub use gpu_renderer::Renderer;
pub use image_backend::ImageBackend;
