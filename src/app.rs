//! Application - Main application state and event loop
//!
//! Coordinates the WGPU renderer, image backend, and UI components.

use crate::gpu_renderer::Renderer;
use crate::image_backend::{ImageBackend, ImageData};
use crate::ui::UiState;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Fullscreen, Icon, Window, WindowId},
};

/// Wakeup token sent through EventLoopProxy to wake the sleeping event loop.
/// The actual payload travels through a regular mpsc channel.
#[derive(Debug)]
pub struct WakeUp;

const APP_ICON: &[u8] = include_bytes!("../assets/icons/icon.png");

pub enum AppEvent {
    ImageLoaded(Vec<ImageData>),
    ImageError(String),
    OpenPath(PathBuf),
    Prefetched(PathBuf, Vec<ImageData>), // prefetch for adjacent images
    SaveComplete(PathBuf),
    SaveError(String),
}

/// Helper: send an AppEvent through the data channel, then wake the event loop.
fn send_event(tx: &Sender<AppEvent>, proxy: &EventLoopProxy<WakeUp>, event: AppEvent) {
    tx.send(event).ok();
    proxy.send_event(WakeUp).ok();
}

pub struct SpedImageApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    ui_state: UiState,
    current_image: Option<ImageData>,
    current_frame_delays: Vec<u32>,
    current_frame_idx: usize,
    next_frame_time: Option<std::time::Instant>,
    loading: bool,
    dirty: bool,
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,
    event_proxy: Option<EventLoopProxy<WakeUp>>,
    // (2) Panning
    mouse_drag_start: Option<PhysicalPosition<f64>>,
    last_cursor_pos: PhysicalPosition<f64>,
    // (4) Help overlay
    show_help: bool,
    // Sidebar overlay (file list)
    show_sidebar: bool,
    // (6) Prefetch cache: (path → frames)
    prefetch_cache: std::collections::HashMap<PathBuf, Vec<ImageData>>,
    prefetch_cache_bytes: usize,
    prefetch_active: Arc<AtomicUsize>,
    // (5) CLI / open-with initial file
    initial_path: Option<PathBuf>,
    // Keyboard modifier state (tracked via ModifiersChanged)
    ctrl_pressed: bool,
}

impl SpedImageApp {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            window: None,
            renderer: None,
            ui_state: UiState::default(),
            current_image: None,
            current_frame_delays: Vec::new(),
            current_frame_idx: 0,
            next_frame_time: None,
            loading: false,
            dirty: true,
            event_tx: tx,
            event_rx: rx,
            event_proxy: None,
            mouse_drag_start: None,
            last_cursor_pos: PhysicalPosition::new(0.0, 0.0),
            show_help: false,
            show_sidebar: false,
            prefetch_cache: std::collections::HashMap::new(),
            prefetch_cache_bytes: 0,
            prefetch_active: Arc::new(AtomicUsize::new(0)),
            initial_path: None,
            ctrl_pressed: false,
        }
    }

    pub fn run(initial_path: Option<PathBuf>) -> Result<()> {
        let event_loop: EventLoop<WakeUp> = EventLoop::with_user_event().build()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = Self::new();
        app.event_proxy = Some(event_loop.create_proxy());
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
            let ctrl = self.ctrl_pressed;
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
        // Check prefetch cache first
        if let Some(cached_frames) = self.prefetch_cache.remove(&path) {
            let cached_bytes: usize = cached_frames.iter().map(|f| f.rgba_data.len()).sum();
            self.prefetch_cache_bytes = self.prefetch_cache_bytes.saturating_sub(cached_bytes);
            tracing::info!("Cache hit for {:?}", path);
            // Deliver from cache directly – still go through channel so process_events
            // handles it uniformly, and the proxy wakes the event loop.
            if let Some(ref proxy) = self.event_proxy {
                send_event(&self.event_tx, proxy, AppEvent::ImageLoaded(cached_frames));
            }
            self.prefetch_adjacent(&path);
            return;
        }

        tracing::info!("Loading image: {:?}", path);
        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        // Update window title immediately
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
        let proxy = self.event_proxy.clone();
        let path2 = path.clone();
        std::thread::spawn(move || {
            let event = match ImageBackend::load_and_downsample(&path2, max_w, max_h) {
                Ok(data) => AppEvent::ImageLoaded(data),
                Err(e) => AppEvent::ImageError(e.to_string()),
            };
            // Use EventLoopProxy to wake the event loop so RedrawRequested fires immediately.
            if let Some(ref proxy) = proxy {
                send_event(&tx, proxy, event);
            }
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
            // Limit concurrent prefetch threads
            const MAX_CONCURRENT_PREFETCH: usize = 2;
            if self.prefetch_active.load(Ordering::Relaxed) >= MAX_CONCURRENT_PREFETCH {
                continue;
            }
            self.prefetch_active.fetch_add(1, Ordering::Relaxed);
            let active = self.prefetch_active.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            let p = path.clone();
            std::thread::spawn(move || {
                let result = ImageBackend::load_and_downsample(&p, max_w, max_h);
                active.fetch_sub(1, Ordering::Relaxed);
                if let Ok(frames) = result {
                    if let Some(ref proxy) = proxy {
                        send_event(&tx, proxy, AppEvent::Prefetched(p, frames));
                    }
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
                self.current_frame_delays.clear();
                self.ui_state
                    .load_directory(path.parent().unwrap_or(&path).to_path_buf());
                self.dirty = true;
                // Navigate to the next (or first available) image so screen isn't blank
                self.next_image();
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

            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
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
                            img = img.brighten(b as i32);
                        }
                    }

                    // Saturation — per-pixel HSL conversion
                    if (adjustments.saturation - 1.0).abs() > 0.01 {
                        let sat = adjustments.saturation;
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            let r = px[0] as f32 / 255.0;
                            let g = px[1] as f32 / 255.0;
                            let b = px[2] as f32 / 255.0;
                            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                            px[0] = ((gray + (r - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[1] = ((gray + (g - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[2] = ((gray + (b - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                        }
                        img = image::DynamicImage::ImageRgba8(rgba);
                    }

                    ImageBackend::save(&save_path_clone, &img, 90)?;
                    Ok(())
                })();

                let event = match result {
                    Ok(()) => {
                        tracing::info!("Saved edited image to {:?}", save_path_clone);
                        AppEvent::SaveComplete(save_path_clone)
                    }
                    Err(e) => {
                        tracing::error!("Failed to save image: {}", e);
                        AppEvent::SaveError(e.to_string())
                    }
                };
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, event);
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
            self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        }
        self.dirty = true;
    }

    fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    fn reset_adjustments(&mut self) {
        self.ui_state.reset_adjustments();
        self.ui_state.set_status("Adjustments reset");
        self.dirty = true;
    }

    fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        self.dirty = true;
    }

    fn open_file_dialog(&mut self) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &ImageBackend::supported_extensions())
                .pick_file()
            {
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, AppEvent::OpenPath(path));
                }
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
                AppEvent::SaveComplete(path) => {
                    self.dirty = true;
                    self.ui_state.set_status(format!(
                        "Saved: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
                AppEvent::SaveError(e) => {
                    self.dirty = true;
                    self.ui_state.set_status(format!("Save failed: {}", e));
                }
                AppEvent::Prefetched(path, frames) => {
                    // Memory-budget-based prefetch cache eviction (100 MB limit)
                    const MAX_PREFETCH_BYTES: usize = 100 * 1024 * 1024;
                    let new_bytes: usize = frames.iter().map(|f| f.rgba_data.len()).sum();
                    while self.prefetch_cache_bytes + new_bytes > MAX_PREFETCH_BYTES
                        && !self.prefetch_cache.is_empty()
                    {
                        if let Some(k) = self.prefetch_cache.keys().next().cloned() {
                            if let Some(evicted) = self.prefetch_cache.remove(&k) {
                                let evicted_bytes: usize =
                                    evicted.iter().map(|f| f.rgba_data.len()).sum();
                                self.prefetch_cache_bytes =
                                    self.prefetch_cache_bytes.saturating_sub(evicted_bytes);
                            }
                        }
                    }
                    self.prefetch_cache_bytes += new_bytes;
                    self.prefetch_cache.insert(path, frames);
                }
                AppEvent::ImageLoaded(mut frames) => {
                    self.ui_state.reset_adjustments();
                    self.dirty = true;
                    if frames.is_empty() {
                        continue;
                    }
                    let mut first_frame = frames.remove(0);
                    let path = PathBuf::from(&first_frame.path);

                    // Extract frame delays before consuming pixel data
                    let frame_delays: Vec<u32> = frames.iter().map(|f| f.frame_delay_ms).collect();

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
                        if !frame_delays.is_empty() {
                            if let Err(e) = renderer.preload_gif_textures(&frames) {
                                tracing::warn!("Failed to preload GIF textures: {}", e);
                            }
                        } else {
                            // Explicitly destroy old GIF textures
                            for (tex, _) in renderer.gif_textures.drain(..) {
                                tex.destroy();
                            }
                        }
                    }

                    // Free CPU-side pixel data; GPU owns the textures now
                    first_frame.rgba_data.clear();
                    first_frame.rgba_data.shrink_to_fit();
                    drop(frames);

                    if !frame_delays.is_empty() && first_frame.frame_delay_ms > 0 {
                        self.next_frame_time = Some(
                            std::time::Instant::now()
                                + std::time::Duration::from_millis(
                                    first_frame.frame_delay_ms as u64,
                                ),
                        );
                    } else if frame_delays.is_empty() {
                        self.next_frame_time = None;
                    }

                    let size_mb = first_frame.file_size_bytes as f64 / 1_048_576.0;
                    let frame_info = if frame_delays.is_empty() {
                        String::new()
                    } else {
                        format!("  |  {} frames", frame_delays.len() + 1)
                    };
                    self.ui_state.set_status(format!(
                        "{}  |  {}×{}  |  {size_mb:.1} MB{}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        first_frame.width,
                        first_frame.height,
                        frame_info
                    ));

                    self.current_image = Some(first_frame);
                    self.current_frame_delays = frame_delays;
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

impl ApplicationHandler<WakeUp> for SpedImageApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Build window icon from embedded PNG
            let icon = image::load_from_memory(APP_ICON).ok().and_then(|img| {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                Icon::from_rgba(rgba.into_raw(), w, h).ok()
            });

            // (11) Graceful error recovery — no unwrap crashes
            let mut attrs = Window::default_attributes()
                .with_title("SpedImage")
                .with_decorations(true);
            if let Some(icon) = icon {
                attrs = attrs.with_window_icon(Some(icon));
            }
            let window = match event_loop.create_window(attrs) {
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

    /// Called when the background thread sends a WakeUp token via EventLoopProxy.
    /// The actual AppEvent payload is in the mpsc channel; we just trigger a redraw.
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: WakeUp) {
        self.dirty = true;
        if let Some(window) = &self.window {
            window.request_redraw();
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
            WindowEvent::ModifiersChanged(mods) => {
                self.ctrl_pressed = mods.state().control_key();
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
                    // Check if click was on UI Navigation bounds:
                    if let Some(ref w) = self.window {
                        let size = w.inner_size();
                        if size.width > 0 {
                            let mouse_x_ratio = self.last_cursor_pos.x / size.width as f64;
                            if mouse_x_ratio < 0.1 {
                                // Clicked far left: prev image
                                self.prev_image();
                                return; // don't start dragging
                            } else if mouse_x_ratio > 0.9 {
                                // Clicked far right: next image
                                self.next_image();
                                return;
                            }
                        }
                    }

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
                    let show_sidebar = self.show_sidebar;
                    let sidebar_files: Vec<String> = if show_sidebar {
                        self.ui_state.files.iter().map(|f| f.name.clone()).collect()
                    } else {
                        Vec::new()
                    };

                    if let Some(ref mut renderer) = self.renderer {
                        if self.loading {
                            let _ = renderer.render_loading();
                        } else {
                            // render_frame acquires the surface texture once, draws image then
                            // UI overlay into it, and presents exactly once — fixing the
                            // double-present black screen bug.
                            let _ = renderer.render_frame(
                                &self.ui_state.adjustments,
                                is_cropping,
                                crop_rect,
                                status_opt.as_deref(),
                                show_help,
                                if show_sidebar {
                                    Some(&sidebar_files)
                                } else {
                                    None
                                },
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
            if now >= next_time && !self.current_frame_delays.is_empty() {
                let total = self.current_frame_delays.len() + 1; // +1 for first frame
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
                    self.current_frame_delays
                        .get(self.current_frame_idx - 1)
                        .copied()
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

impl Drop for SpedImageApp {
    fn drop(&mut self) {
        // Eagerly release prefetch cache memory
        self.prefetch_cache.clear();
        self.prefetch_cache_bytes = 0;
        self.current_frame_delays.clear();
        self.current_image = None;
    }
}
