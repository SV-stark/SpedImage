//! Application - Main application state and event loop
//!
//! Coordinates the WGPU renderer, image backend, and UI components.

use crate::gpu_renderer::Renderer;
use crate::image_backend::{ImageBackend, ImageData};
use crate::ui::UiState;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub enum AppEvent {
    ImageLoaded(Vec<ImageData>),
    ImageError(String),
    OpenPath(PathBuf),
}

pub struct SpedImageApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    ui_state: UiState,
    current_image: Option<ImageData>,
    current_frames: Vec<ImageData>,
    current_frame_idx: usize,
    next_frame_time: Option<std::time::Instant>,
    loading: bool,
    dirty: bool,
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,
}

impl SpedImageApp {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            window: None,
            renderer: None,
            ui_state: UiState::default(),
            current_image: None,
            current_frames: Vec::new(),
            current_frame_idx: 0,
            next_frame_time: None,
            loading: false,
            dirty: true,
            event_tx: tx,
            event_rx: rx,
        }
    }

    pub fn run() -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = Self::new();
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    fn handle_keyboard(&mut self, event: KeyEvent) {
        let key = event.logical_key;

        if let Some(c) = key.to_text() {
            match c {
                "d" | "D" => self.next_image(),
                "a" | "A" => self.prev_image(),
                "w" | "W" => self.prev_image(),
                "s" | "S" => self.next_image(),
                "r" | "R" => self.rotate_image(),
                "o" | "O" => self.open_file_dialog(),
                "f" | "F" => self.toggle_sidebar(),
                "1" => self.reset_adjustments(),
                "c" | "C" => self.toggle_crop(),
                "+" | "=" => self.zoom_in(),
                "-" => self.zoom_out(),
                "0" => self.zoom_fit(),
                "q" | "Q" => {
                    if self.ui_state.is_cropping {
                        self.cancel_crop();
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.zoom_in();
                } else if y < 0.0 {
                    self.zoom_out();
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                if pos.y > 0.0 {
                    self.zoom_in();
                } else if pos.y < 0.0 {
                    self.zoom_out();
                }
            }
        }
    }

    fn load_image(&mut self, path: PathBuf) {
        tracing::info!("Loading image: {:?}", path);
        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        let tx = self.event_tx.clone();
        
        // Get current window size for downsampling, default to 4K if unavailable
        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let size = w.inner_size();
                (size.width, size.height)
            },
            None => (3840, 2160)
        };

        std::thread::spawn(move || {
            match ImageBackend::load_and_downsample(&path, max_w, max_h) {
                Ok(data) => tx.send(AppEvent::ImageLoaded(data)).ok(),
                Err(e) => tx.send(AppEvent::ImageError(e.to_string())).ok(),
            };
        });
    }

    fn save_image(&mut self) {
        if let Some(ref image) = self.current_image {
            let path = PathBuf::from(&image.path);
            let mut save_path = path.clone();

            if let Some(stem) = path.file_stem() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                save_path.set_file_name(format!("{}_edited.{}", stem.to_string_lossy(), ext));
            }

            if let Err(e) = std::fs::copy(&path, &save_path) {
                tracing::error!("Failed to save: {}", e);
                self.ui_state.set_status(format!("Save error: {}", e));
            } else {
                self.ui_state.set_status(format!(
                    "Saved: {}",
                    save_path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
            self.dirty = true;
        }
    }

    fn next_image(&mut self) {
        self.ui_state.next_file();
        if let Some(file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    fn prev_image(&mut self) {
        self.ui_state.prev_file();
        if let Some(file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    fn rotate_image(&mut self) {
        self.ui_state.rotate_90();
        self.dirty = true;
    }
    
    fn toggle_crop(&mut self) {
        self.ui_state.is_cropping = !self.ui_state.is_cropping;
        if !self.ui_state.is_cropping {
            self.ui_state.crop_rect = [0.0, 0.0, 1.0, 1.0];
        }
        self.dirty = true;
    }
    
    fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.ui_state.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }
    
    fn reset_adjustments(&mut self) {
        self.ui_state.reset_adjustments();
        self.ui_state.set_status("Adjustments reset");
        self.dirty = true;
    }
    
    fn toggle_sidebar(&mut self) {
        self.ui_state.show_sidebar = !self.ui_state.show_sidebar;
        self.dirty = true;
    }

    fn open_file_dialog(&mut self) {
        let tx = self.event_tx.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["jpg","jpeg","png","gif","bmp","tiff","webp","heic","avif"])
                .pick_file()
            {
                tx.send(AppEvent::OpenPath(path)).ok();
            }
        });
    }

    fn zoom_in(&mut self) {
        self.ui_state.adjustments.crop_rect[2] *= 0.9;
        self.ui_state.adjustments.crop_rect[3] *= 0.9;
        self.ui_state.adjustments.crop_rect[2] = self.ui_state.adjustments.crop_rect[2].max(0.05);
        self.ui_state.adjustments.crop_rect[3] = self.ui_state.adjustments.crop_rect[3].max(0.05);
        self.dirty = true;
    }

    fn zoom_out(&mut self) {
        self.ui_state.adjustments.crop_rect[2] *= 1.1;
        self.ui_state.adjustments.crop_rect[3] *= 1.1;
        self.ui_state.adjustments.crop_rect[2] = self.ui_state.adjustments.crop_rect[2].min(1.0);
        self.ui_state.adjustments.crop_rect[3] = self.ui_state.adjustments.crop_rect[3].min(1.0);
        self.dirty = true;
    }

    fn zoom_fit(&mut self) {
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.dirty = true;
            match event {
                AppEvent::ImageLoaded(mut frames) => {
                    if frames.is_empty() {
                        continue;
                    }
                    let mut first_frame = frames.remove(0);
                    let path = PathBuf::from(&first_frame.path);
                    if let Some(parent) = path.parent() {
                        self.ui_state.load_directory(parent.to_path_buf());
                    }

                    for (i, f) in self.ui_state.files.iter().enumerate() {
                        if f.path == path {
                            self.ui_state.current_file_index = Some(i);
                            break;
                        }
                    }

                    if let Some(ref mut renderer) = self.renderer {
                        if let Err(e) = renderer.load_image(&first_frame) {
                            tracing::error!("Failed to load image to GPU: {}", e);
                            self.ui_state.set_status("Failed to load image");
                            return;
                        }
                    }

                    // P2 Resource Optimization - drop CPU memory ONLY if static image
                    if frames.is_empty() {
                        first_frame.rgba_data = Vec::new();
                        first_frame.rgba_data.shrink_to_fit();
                        self.next_frame_time = None;
                    } else {
                        // It's animated
                        if first_frame.frame_delay_ms > 0 {
                            self.next_frame_time = Some(std::time::Instant::now() + std::time::Duration::from_millis(first_frame.frame_delay_ms as u64));
                        }
                    }

                    let size_mb = first_frame.file_size_bytes as f64 / 1_048_576.0;
                    self.ui_state.set_status(format!(
                        "Loaded: {}  |  {}x{}  |  {size_mb:.1} MB{}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        first_frame.width,
                        first_frame.height,
                        if frames.is_empty() { "".to_string() } else { format!("  |  {} frames", frames.len() + 1) }
                    ));
                    
                    self.current_image = Some(first_frame);
                    self.current_frames = frames;
                    self.current_frame_idx = 0;
                    self.loading = false;
                }
                AppEvent::ImageError(e) => {
                    tracing::error!("Failed to load image: {}", e);
                    self.ui_state.set_status(format!("Error: {}", e));
                    self.loading = false;
                }
                AppEvent::OpenPath(path) => {
                    self.load_image(path);
                }
            }
        }
    }
}

impl Default for SpedImageApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for SpedImageApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("SpedImage"))
                    .unwrap(),
            );
            self.window = Some(window.clone());

            let renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();
            self.renderer = Some(renderer);
            self.dirty = true;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(size);
                    self.dirty = true;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.handle_keyboard(event);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    if self.ui_state.is_cropping {
                        // Handle crop
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.process_events();
                if self.dirty {
                    if let Some(ref renderer) = self.renderer {
                        if self.loading {
                            let _ = renderer.render_loading();
                        } else {
                            let _ = renderer.render(&self.ui_state.adjustments);
                        }
                    }
                    self.dirty = false;
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.process_events();
        
        if let Some(next_time) = self.next_frame_time {
            let now = std::time::Instant::now();
            if now >= next_time {
                if !self.current_frames.is_empty() {
                    self.current_frame_idx = (self.current_frame_idx + 1) % (self.current_frames.len() + 1);
                    
                    let delay = if self.current_frame_idx == 0 {
                        let frame = self.current_image.as_ref().unwrap();
                        if let Some(ref mut renderer) = self.renderer {
                            let _ = renderer.load_image(frame);
                        }
                        frame.frame_delay_ms
                    } else {
                        let frame = &self.current_frames[self.current_frame_idx - 1];
                        if let Some(ref mut renderer) = self.renderer {
                            let _ = renderer.load_image(frame);
                        }
                        frame.frame_delay_ms
                    };
                    
                    self.next_frame_time = Some(now + std::time::Duration::from_millis(delay.max(10) as u64));
                    self.dirty = true;
                }
            }
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(self.next_frame_time.unwrap()));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }

        if self.dirty {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}
