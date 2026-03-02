//! Application - Main application state and event loop
//!
//! Coordinates the WGPU renderer, image backend, and UI components.

use crate::gpu_renderer::Renderer;
use crate::image_backend::{ImageBackend, ImageData};
use crate::ui::UiState;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Fullscreen, Window, WindowId},
};

pub enum AppEvent {
    ImageLoaded(Vec<ImageData>),
    ImageError(String),
    OpenPath(PathBuf),
    Prefetched(PathBuf, Vec<ImageData>), // (7) prefetch for adjacent images
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
    // (2) Panning
    mouse_drag_start: Option<PhysicalPosition<f64>>,
    last_cursor_pos: PhysicalPosition<f64>,
    // (4) Help overlay
    show_help: bool,
    // (6) Prefetch cache: (path → frames)
    prefetch_cache: std::collections::HashMap<PathBuf, Vec<ImageData>>,
    // (5) CLI / open-with initial file
    initial_path: Option<PathBuf>,
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
            mouse_drag_start: None,
            last_cursor_pos: PhysicalPosition::new(0.0, 0.0),
            show_help: false,
            prefetch_cache: std::collections::HashMap::new(),
            initial_path: None,
        }
    }

    pub fn run(initial_path: Option<PathBuf>) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = Self::new();
        app.initial_path = initial_path;
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        // Named keys (non-text)
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                if self.ui_state.is_cropping {
                    self.cancel_crop();
                } else if self.show_help {
                    self.show_help = false;
                    self.dirty = true;
                } else {
                    event_loop.exit();
                }
                return;
            }
            Key::Named(NamedKey::F1) => {
                self.show_help = !self.show_help;
                self.dirty = true;
                return;
            }
            Key::Named(NamedKey::F11) => {
                if let Some(ref w) = self.window {
                    let mode = if w.fullscreen().is_some() {
                        None
                    } else {
                        Some(Fullscreen::Borderless(None))
                    };
                    w.set_fullscreen(mode);
                }
                return;
            }
            Key::Named(NamedKey::Delete) => {
                self.delete_current_image();
                return;
            }
            _ => {}
        }

        if let Some(c) = event.logical_key.to_text() {
            let ctrl = event_loop.modifiers().state().control_key();
            match c {
                "d" | "D" => self.next_image(),
                "a" | "A" => self.prev_image(),
                "w" | "W" => self.prev_image(),
                "s" | "S" => {
                    if ctrl {
                        self.save_image();
                    } else {
                        self.next_image();
                    }
                }
                "r" | "R" => self.rotate_image(),
                "o" | "O" => self.open_file_dialog(),
                "f" | "F" => self.toggle_sidebar(),
                "1" => self.reset_adjustments(),
                "c" | "C" => self.toggle_crop(),
                "+" | "=" => self.zoom_in(None),
                "-" => self.zoom_out(None),
                "0" => self.zoom_fit(),
                "?" => {
                    self.ui_state.toggle_help();
                    self.dirty = true;
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, cursor_pos: PhysicalPosition<f64>) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.zoom_in(Some(cursor_pos));
                } else if y < 0.0 {
                    self.zoom_out(Some(cursor_pos));
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                if pos.y > 0.0 {
                    self.zoom_in(Some(cursor_pos));
                } else if pos.y < 0.0 {
                    self.zoom_out(Some(cursor_pos));
                }
            }
        }
    }

    fn load_image(&mut self, path: PathBuf) {
        // (6) Check prefetch cache first
        if let Some(cached_frames) = self.prefetch_cache.remove(&path) {
            tracing::info!("Cache hit for {:?}", path);
            let tx = self.event_tx.clone();
            tx.send(AppEvent::ImageLoaded(cached_frames)).ok();
            self.prefetch_adjacent(&path);
            return;
        }

        tracing::info!("Loading image: {:?}", path);
        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        // Update window title
        if let Some(ref w) = self.window {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("SpedImage");
            w.set_title(&format!("SpedImage — {}", name));
        }

        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let size = w.inner_size();
                (size.width, size.height)
            }
            None => (3840, 2160),
        };

        let tx = self.event_tx.clone();
        let path2 = path.clone();
        std::thread::spawn(move || {
            match ImageBackend::load_and_downsample(&path2, max_w, max_h) {
                Ok(data) => tx.send(AppEvent::ImageLoaded(data)).ok(),
                Err(e) => tx.send(AppEvent::ImageError(e.to_string())).ok(),
            };
        });

        self.prefetch_adjacent(&path);
    }

    /// (6) Prefetch the images neighboring the current one in the folder.
    fn prefetch_adjacent(&mut self, current: &Path) {
        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let s = w.inner_size();
                (s.width, s.height)
            }
            None => (3840, 2160),
        };

        let neighbors: Vec<PathBuf> = {
            let files = &self.ui_state.files;
            let idx = files.iter().position(|f| f.path == current);
            let mut v = Vec::new();
            if let Some(i) = idx {
                if i + 1 < files.len() {
                    v.push(files[i + 1].path.clone());
                }
                if i > 0 {
                    v.push(files[i - 1].path.clone());
                }
            }
            v
        };

        for path in neighbors {
            if self.prefetch_cache.contains_key(&path) {
                continue;
            }
            let tx = self.event_tx.clone();
            let p = path.clone();
            std::thread::spawn(move || {
                if let Ok(frames) = ImageBackend::load_and_downsample(&p, max_w, max_h) {
                    tx.send(AppEvent::Prefetched(p, frames)).ok();
                }
            });
        }
    }

    /// (8) Delete current image with confirmation
    fn delete_current_image(&mut self) {
        if let Some(ref image) = self.current_image {
            let path = PathBuf::from(&image.path);
            let confirmed = rfd::MessageDialog::new()
                .set_title("Delete Image")
                .set_description(format!(
                    "Delete {}?",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes;
            if confirmed && std::fs::remove_file(&path).is_ok() {
                self.ui_state.set_status(format!(
                    "Deleted: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                self.current_image = None;
                self.current_frames.clear();
                self.ui_state
                    .load_directory(path.parent().unwrap_or(&path).to_path_buf());
                self.dirty = true;
            }
        }
    }

    fn save_image(&mut self) {
        if let Some(ref image_data) = self.current_image {
            let path = PathBuf::from(&image_data.path);
            let mut save_path = path.clone();

            if let Some(stem) = path.file_stem() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                save_path.set_file_name(format!("{}_edited.{}", stem.to_string_lossy(), ext));
            }

            self.ui_state.set_status("Saving...");
            self.dirty = true;

            let path_clone = path.clone();
            let save_path_clone = save_path.clone();
            let adjustments = self.ui_state.adjustments;
            let tx = self.event_tx.clone();

            std::thread::spawn(move || {
                // Software fallback for adjustments since we don't have GPU readback yet
                let result = (|| -> anyhow::Result<()> {
                    let mut img = image::open(&path_clone)?;

                    // Crop
                    if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
                        let (w, h) = (img.width() as f32, img.height() as f32);
                        let crop_x = (adjustments.crop_rect[0] * w) as u32;
                        let crop_y = (adjustments.crop_rect[1] * h) as u32;
                        let crop_w = (adjustments.crop_rect[2] * w) as u32;
                        let crop_h = (adjustments.crop_rect[3] * h) as u32;
                        img = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
                    }

                    // Rotate
                    let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round() as i32;
                    let rot_normalized = if rot_deg < 0 { rot_deg + 360 } else { rot_deg };
                    match rot_normalized {
                        90 => img = img.rotate90(),
                        180 => img = img.rotate180(),
                        270 => img = img.rotate270(),
                        _ => {}
                    }

                    // Brightness & Contrast
                    if (adjustments.brightness - 1.0).abs() > 0.01
                        || (adjustments.contrast - 1.0).abs() > 0.01
                    {
                        let b = (adjustments.brightness - 1.0) * 255.0;
                        let c = adjustments.contrast;
                        img = img.adjust_contrast(c);
                        if b != 0.0 {
                            // The `img.brighten` method takes an i32 value.
                            img = img.brighten(b as i32);
                        }
                    }

                    // Saturation (image crate doesn't have a direct saturation method,
                    // we could do a custom pixel map, but for now we skip or approximate)

                    ImageBackend::save(&save_path_clone, &img, 90)?;
                    Ok(())
                })();

                // We can't easily notify the UI thread of the status update without a specific event type,
                // but we can send a "fake" event or just let the async operation finish.
                // For simplicity, we just log in the thread. A full fix would add an app event.
                if let Err(e) = result {
                    tracing::error!("Failed to save image: {}", e);
                } else {
                    tracing::info!("Saved edited image to {:?}", save_path_clone);
                }
            });

            self.ui_state.set_status(format!(
                "Saving to {}",
                save_path.file_name().unwrap_or_default().to_string_lossy()
            ));
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
                .add_filter(
                    "Images",
                    &[
                        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic", "avif",
                    ],
                )
                .pick_file()
            {
                tx.send(AppEvent::OpenPath(path)).ok();
            }
        });
    }

    fn zoom_in(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(0.9, cursor);
    }

    fn zoom_out(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(1.1, cursor);
    }

    /// (3) Zoom toward/away from cursor position.
    fn zoom_by(&mut self, factor: f32, cursor: Option<PhysicalPosition<f64>>) {
        let old_w = self.ui_state.adjustments.crop_rect[2];
        let old_h = self.ui_state.adjustments.crop_rect[3];
        let new_w = (old_w * factor).clamp(0.05, 1.0);
        let new_h = (old_h * factor).clamp(0.05, 1.0);

        if let (Some(pos), Some(ref w)) = (cursor, &self.window) {
            let win_size = w.inner_size();
            if win_size.width > 0 && win_size.height > 0 {
                // Cursor UV position within [0,1]
                let cx = (pos.x as f32 / win_size.width as f32)
                    .mul_add(old_w, self.ui_state.adjustments.crop_rect[0]);
                let cy = (pos.y as f32 / win_size.height as f32)
                    .mul_add(old_h, self.ui_state.adjustments.crop_rect[1]);
                self.ui_state.adjustments.crop_rect[0] = (cx
                    - new_w * (pos.x as f32 / win_size.width as f32))
                    .max(0.0)
                    .min(1.0 - new_w);
                self.ui_state.adjustments.crop_rect[1] = (cy
                    - new_h * (pos.y as f32 / win_size.height as f32))
                    .max(0.0)
                    .min(1.0 - new_h);
            }
        }

        self.ui_state.adjustments.crop_rect[2] = new_w;
        self.ui_state.adjustments.crop_rect[3] = new_h;
        self.dirty = true;
    }

    fn zoom_fit(&mut self) {
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::Prefetched(path, frames) => {
                    // (6) Store in prefetch cache, capped at 4 entries
                    if self.prefetch_cache.len() >= 4 {
                        if let Some(k) = self.prefetch_cache.keys().next().cloned() {
                            self.prefetch_cache.remove(&k);
                        }
                    }
                    self.prefetch_cache.insert(path, frames);
                }
                AppEvent::ImageLoaded(mut frames) => {
                    self.dirty = true;
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
                            self.loading = false;
                            return;
                        }
                        // (7) Preload all GIF frames to GPU on load
                        if !frames.is_empty() {
                            if let Err(e) = renderer.preload_gif_textures(&frames) {
                                tracing::warn!("Failed to preload GIF textures: {}", e);
                            }
                        } else {
                            renderer.gif_textures.clear();
                        }
                    }

                    if frames.is_empty() {
                        first_frame.rgba_data.clear();
                        first_frame.rgba_data.shrink_to_fit();
                        self.next_frame_time = None;
                    } else {
                        for f in frames.iter_mut() {
                            f.rgba_data.clear();
                            f.rgba_data.shrink_to_fit();
                        }
                        if first_frame.frame_delay_ms > 0 {
                            self.next_frame_time = Some(
                                std::time::Instant::now()
                                    + std::time::Duration::from_millis(
                                        first_frame.frame_delay_ms as u64,
                                    ),
                            );
                        }
                    }

                    let size_mb = first_frame.file_size_bytes as f64 / 1_048_576.0;
                    let frame_info = if frames.is_empty() {
                        String::new()
                    } else {
                        format!("  |  {} frames", frames.len() + 1)
                    };
                    self.ui_state.set_status(format!(
                        "{}  |  {}×{}  |  {size_mb:.1} MB{}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        first_frame.width,
                        first_frame.height,
                        frame_info
                    ));

                    self.current_image = Some(first_frame);
                    self.current_frames = frames;
                    self.current_frame_idx = 0;
                    self.loading = false;
                }
                AppEvent::ImageError(e) => {
                    self.dirty = true;
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
            // (11) Graceful error recovery — no unwrap crashes
            let window = match event_loop.create_window(
                Window::default_attributes()
                    .with_title("SpedImage")
                    .with_decorations(true),
            ) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(window.clone());

            match pollster::block_on(Renderer::new(window.clone())) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                }
                Err(e) => {
                    tracing::error!("Failed to initialize GPU renderer: {}", e);
                    let _ = rfd::MessageDialog::new()
                        .set_title("GPU Error")
                        .set_description(format!(
                            "Failed to initialize GPU: {}\n\nThe app will exit.",
                            e
                        ))
                        .show();
                    event_loop.exit();
                    return;
                }
            }
            self.dirty = true;
            // (5) Open CLI-provided file immediately on startup
            if let Some(path) = self.initial_path.take() {
                self.load_image(path);
            }
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
            // (12) DPI-aware: update scale_factor when monitor DPI changes
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.update_scale_factor(scale_factor);
                    self.dirty = true;
                }
            }
            // (1) Drag-and-drop file opening
            WindowEvent::DroppedFile(path) => {
                tracing::info!("File dropped: {:?}", path);
                self.load_image(path);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.handle_keyboard(event, event_loop);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta, self.last_cursor_pos);
            }
            // (2) Pan tracking
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(start) = self.mouse_drag_start {
                    if let Some(ref w) = self.window {
                        let size = w.inner_size();
                        if size.width > 0 && size.height > 0 {
                            let dx = (position.x - start.x) as f32 / size.width as f32;
                            let dy = (position.y - start.y) as f32 / size.height as f32;
                            let rect = &mut self.ui_state.adjustments.crop_rect;
                            rect[0] = (rect[0] - dx * rect[2]).clamp(0.0, 1.0 - rect[2]);
                            rect[1] = (rect[1] - dy * rect[3]).clamp(0.0, 1.0 - rect[3]);
                            self.dirty = true;
                        }
                    }
                    self.mouse_drag_start = Some(position); // update origin each move
                }
                self.last_cursor_pos = position;
            }
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    if !self.ui_state.is_cropping {
                        self.mouse_drag_start = Some(self.last_cursor_pos);
                    }
                }
                (MouseButton::Left, ElementState::Released) => {
                    self.mouse_drag_start = None;
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                self.process_events();
                if self.dirty {
                    // Clone status and capture flags before mut-borrowing renderer
                    let status_opt: Option<String> = self.ui_state.status_message.clone();
                    let is_cropping = self.ui_state.is_cropping;
                    let crop_rect = self.ui_state.adjustments.crop_rect;
                    let show_help = self.ui_state.show_help;

                    if let Some(ref mut renderer) = self.renderer {
                        if self.loading {
                            let _ = renderer.render_loading();
                        } else {
                            let _ = renderer.render(&self.ui_state.adjustments);
                            let _ = renderer.render_ui_overlay(
                                is_cropping,
                                crop_rect,
                                status_opt.as_deref(),
                                show_help,
                            );
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
            if now >= next_time && !self.current_frames.is_empty() {
                let total = self.current_frames.len() + 1; // +1 for first frame
                self.current_frame_idx = (self.current_frame_idx + 1) % total;

                let delay = if self.current_frame_idx == 0 {
                    // (7) Swap bind group cheaply — no re-upload
                    if let Some(ref mut renderer) = self.renderer {
                        renderer.swap_gif_frame(0);
                    }
                    self.current_image
                        .as_ref()
                        .map(|f| f.frame_delay_ms)
                        .unwrap_or(100)
                } else {
                    let idx = self.current_frame_idx; // avoid borrow tension
                    if let Some(ref mut renderer) = self.renderer {
                        renderer.swap_gif_frame(idx); // frames[idx-1] stored at gif_textures[idx]
                    }
                    self.current_frames
                        .get(self.current_frame_idx - 1)
                        .map(|f| f.frame_delay_ms)
                        .unwrap_or(100)
                };

                self.next_frame_time =
                    Some(now + std::time::Duration::from_millis(delay.max(10) as u64));
                self.dirty = true;
            }
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                self.next_frame_time.unwrap(),
            ));
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
