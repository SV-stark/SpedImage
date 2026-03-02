//! Application - Main application state and event loop
//!
//! Coordinates the WGPU renderer, image backend, and UI components.

use crate::gpu_renderer::Renderer;
use crate::image_backend::{ImageBackend, ImageData};
use crate::ui::UiState;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub struct SpedImageApp {
    #[allow(dead_code)]
    renderer: Option<Renderer<'static>>,
    ui_state: UiState,
    current_image: Option<ImageData>,
}

impl SpedImageApp {
    pub fn new() -> Self {
        Self {
            renderer: None,
            ui_state: UiState::default(),
            current_image: None,
        }
    }

    #[allow(deprecated)]
    pub fn run() -> Result<()> {
        let event_loop = EventLoop::new()?;

        let window = Window::new(&event_loop)?;
        let window_id = window.id();

        let app = Arc::new(std::sync::Mutex::new(Self::new()));
        let app_clone = app.clone();

        // Try to initialize renderer - skip for now due to lifetime issues
        // Will initialize on first resize
        /*
        let runtime = tokio::runtime::Runtime::new();
        if let Ok(runtime) = runtime {
            if let Ok(mut app_lock) = app.try_lock() {
                let result = runtime.block_on(async {
                    Renderer::new(&window).await
                });
                if let Ok(r) = result {
                    app_lock.renderer = Some(r);
                }
            }
        }
        */

        event_loop.run(move |event, _window_target| {
            let mut app = app_clone.lock().unwrap();

            match event {
                Event::WindowEvent {
                    window_id: wid,
                    event,
                } if wid == window_id => {
                    match event {
                        WindowEvent::CloseRequested => {
                            std::process::exit(0);
                        }
                        WindowEvent::Resized(size) => {
                            if let Some(ref mut renderer) = app.renderer {
                                renderer.resize(size);
                            }
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            app.handle_keyboard(event);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            app.handle_mouse_wheel(delta);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if button == MouseButton::Left && state == ElementState::Pressed {
                                if app.ui_state.is_cropping {
                                    // Handle crop
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    if let Some(ref renderer) = app.renderer {
                        let adjustments = app.ui_state.adjustments;
                        let _ = renderer.render(&adjustments);
                    }
                }
                _ => {}
            }
        });

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

        match ImageBackend::load(&path) {
            Ok(image_data) => {
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
                    if let Err(e) = renderer.load_image(&image_data) {
                        tracing::error!("Failed to load image to GPU: {}", e);
                        self.ui_state.set_status("Failed to load image");
                        return;
                    }
                }

                self.current_image = Some(image_data);
                self.ui_state.set_status(format!(
                    "Loaded: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
            Err(e) => {
                tracing::error!("Failed to load image: {}", e);
                self.ui_state.set_status(format!("Error: {}", e));
            }
        }
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
        }
    }

    fn next_image(&mut self) {
        self.ui_state.next_file();
        if let Some(ref file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    fn prev_image(&mut self) {
        self.ui_state.prev_file();
        if let Some(ref file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    fn rotate_image(&mut self) {
        self.ui_state.rotate_90();
    }
    fn toggle_crop(&mut self) {
        self.ui_state.is_cropping = !self.ui_state.is_cropping;
        if !self.ui_state.is_cropping {
            self.ui_state.crop_rect = [0.0, 0.0, 1.0, 1.0];
        }
    }
    fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.ui_state.crop_rect = [0.0, 0.0, 1.0, 1.0];
    }
    fn reset_adjustments(&mut self) {
        self.ui_state.reset_adjustments();
        self.ui_state.set_status("Adjustments reset");
    }
    fn toggle_sidebar(&mut self) {
        self.ui_state.show_sidebar = !self.ui_state.show_sidebar;
    }

    fn open_file_dialog(&mut self) {
        if let Some(pictures) = dirs::picture_dir() {
            self.ui_state.load_directory(pictures.clone());
            self.ui_state
                .set_status(format!("Loaded: {}", pictures.display()));
        }
    }

    fn zoom_in(&mut self) {
        self.ui_state.adjustments.crop_rect[2] *= 0.9;
        self.ui_state.adjustments.crop_rect[3] *= 0.9;
    }

    fn zoom_out(&mut self) {
        self.ui_state.adjustments.crop_rect[2] *= 1.1;
        self.ui_state.adjustments.crop_rect[3] *= 1.1;
        self.ui_state.adjustments.crop_rect[2] = self.ui_state.adjustments.crop_rect[2].min(1.0);
        self.ui_state.adjustments.crop_rect[3] = self.ui_state.adjustments.crop_rect[3].min(1.0);
    }

    fn zoom_fit(&mut self) {
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
    }
}

impl Default for SpedImageApp {
    fn default() -> Self {
        Self::new()
    }
}
