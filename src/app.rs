//! Application - Main application state and event loop
//!
//! Coordinates the WGPU renderer, image backend, and UI components.

use crate::gpu_renderer::{Renderer, STRIP_HEIGHT_PX};
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

/// Thumbnail size used for background loading (must match THUMB_SIZE in gpu_renderer).
const THUMB_LOAD_SIZE: u32 = 80;
/// Max concurrently-running thumbnail background threads.
const MAX_THUMB_THREADS: usize = 4;
/// Max thumbnails kept in GPU at once (older ones are not evicted but new ones stop loading).
const MAX_THUMBNAILS: usize = 200;

pub enum AppEvent {
    ImageLoaded(Vec<ImageData>),
    ImageError(String),
    OpenPath(PathBuf),
    Prefetched(PathBuf, Vec<ImageData>), // prefetch for adjacent images
    SaveComplete(PathBuf),
    SaveError(String),
    /// A thumbnail has finished loading: (path, rgba_bytes, width, height)
    ThumbnailLoaded(PathBuf, Vec<u8>, u32, u32),
    SetStatus(String),
    FileRenamed(PathBuf, PathBuf),
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
    shift_pressed: bool,
    // Thumbnail strip
    show_thumbnail_strip: bool,
    thumb_active: Arc<AtomicUsize>, // count of running thumbnail threads
    /// Ordered list of file paths for which thumbnails have been requested,
    /// in the same order as ui_state.files (may lag behind while loading).
    thumb_paths: Vec<PathBuf>,

    // (8) Slideshow Mode
    slideshow_active: bool,
    slideshow_interval: std::time::Duration,
    slideshow_next_time: Option<std::time::Instant>,
    // (9) Color Picker - track Alt modifier
    alt_pressed: bool,
    // (10) Histogram panel
    show_histogram: bool,
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
            show_thumbnail_strip: true,
            thumb_active: Arc::new(AtomicUsize::new(0)),
            thumb_paths: Vec::new(),
            slideshow_active: false,
            slideshow_interval: std::time::Duration::from_secs(5),
            slideshow_next_time: None,
            alt_pressed: false,
            shift_pressed: false,
            show_histogram: false,
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
            Key::Named(NamedKey::ArrowLeft) => {
                self.prev_image();
                return;
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.next_image();
                return;
            }
            Key::Named(NamedKey::F1) => {
                self.show_help = !self.show_help;
                self.dirty = true;
                return;
            }
            Key::Named(NamedKey::F2) => {
                self.rename_current_image();
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
                "w" | "W" => {
                    if ctrl {
                        self.set_as_wallpaper();
                    } else {
                        self.prev_image();
                    }
                }
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
                "t" | "T" => self.toggle_thumbnail_strip(),
                "1" => self.reset_adjustments(),
                "c" | "C" if ctrl => self.copy_to_clipboard(),
                "c" | "C" => self.toggle_crop(),
                "h" | "H" => {
                    if self.alt_pressed {
                        // do nothing; alt+h is reserved
                    } else if self.shift_pressed {
                        self.show_histogram = !self.show_histogram;
                        let state = if self.show_histogram { "ON" } else { "OFF" };
                        self.ui_state.set_status(format!("Histogram: {}", state));
                        self.dirty = true;
                    } else {
                        self.toggle_hdr_toning();
                    }
                }
                "i" | "I" => {
                    self.ui_state.toggle_info();
                    self.dirty = true;
                }
                "p" | "P" if ctrl => self.print_image(),
                "+" | "=" => self.zoom_in(None),
                "-" => self.zoom_out(None),
                "0" => self.zoom_fit(),
                " " => self.toggle_slideshow(),
                "[" => self.adjust_slideshow_interval(-1),
                "]" => self.adjust_slideshow_interval(1),
                "?" => {
                    self.ui_state.toggle_help();
                    self.dirty = true;
                }
                _ => {}
            }
        }
    }

    fn toggle_slideshow(&mut self) {
        self.slideshow_active = !self.slideshow_active;
        if self.slideshow_active {
            self.slideshow_next_time = Some(std::time::Instant::now() + self.slideshow_interval);
            self.ui_state.set_status(format!(
                "Slideshow started ({}s per image)",
                self.slideshow_interval.as_secs()
            ));
        } else {
            self.slideshow_next_time = None;
            self.ui_state.set_status("Slideshow paused");
        }
        self.dirty = true;
    }

    fn adjust_slideshow_interval(&mut self, change: i32) {
        let current_secs = self.slideshow_interval.as_secs() as i32;
        let new_secs = (current_secs + change).clamp(1, 120) as u64;
        self.slideshow_interval = std::time::Duration::from_secs(new_secs);
        if self.slideshow_active {
            self.slideshow_next_time = Some(std::time::Instant::now() + self.slideshow_interval);
            self.ui_state
                .set_status(format!("Slideshow interval: {}s", new_secs));
        } else {
            self.ui_state
                .set_status(format!("Slideshow interval configured to {}s", new_secs));
        }
        self.dirty = true;
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

    /// Spawn background thumbnail-loading threads for all files in the current directory.
    /// Only files that haven't been requested yet (not in thumb_paths) are queued.
    fn load_thumbnails_for_dir(&mut self) {
        let files: Vec<PathBuf> = self.ui_state.files.iter().map(|f| f.path.clone()).collect();

        // Reset thumb_paths to match the new directory order
        self.thumb_paths = files.clone();

        // Clear existing GPU thumbnails so order matches the new directory
        if let Some(ref mut renderer) = self.renderer {
            renderer.clear_thumbnails();
        }

        let n = files.len().min(MAX_THUMBNAILS);
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let active = self.thumb_active.clone();

        for path in files.into_iter().take(n) {
            // Throttle concurrency
            while active.load(Ordering::Relaxed) >= MAX_THUMB_THREADS {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            active.fetch_add(1, Ordering::Relaxed);
            let tx2 = tx.clone();
            let proxy2 = proxy.clone();
            let act2 = active.clone();
            let p = path.clone();
            std::thread::spawn(move || {
                let result =
                    ImageBackend::load_and_downsample(&p, THUMB_LOAD_SIZE, THUMB_LOAD_SIZE);
                act2.fetch_sub(1, Ordering::Relaxed);
                if let Ok(frames) = result {
                    if let Some(first) = frames.into_iter().next() {
                        if let Some(ref proxy) = proxy2 {
                            send_event(
                                &tx2,
                                proxy,
                                AppEvent::ThumbnailLoaded(
                                    p,
                                    first.rgba_data,
                                    first.width,
                                    first.height,
                                ),
                            );
                        }
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

                    // HDR Toning (Reinhard filmic)
                    if adjustments.hdr_toning {
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            for c in 0..3 {
                                // Expose
                                let mut color = (px[c] as f32 / 255.0) * 1.6;
                                // Reinhard
                                color = color / (1.0 + color);
                                // S-curve test
                                color = color * color * (3.0 - 2.0 * color);
                                px[c] = (color.clamp(0.0, 1.0) * 255.0) as u8;
                            }
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
            self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        }
        self.dirty = true;
    }

    fn toggle_hdr_toning(&mut self) {
        self.ui_state.adjustments.hdr_toning = !self.ui_state.adjustments.hdr_toning;
        let label = if self.ui_state.adjustments.hdr_toning {
            "ON"
        } else {
            "OFF"
        };
        self.ui_state.set_status(format!("HDR Toning: {}", label));
        self.dirty = true;
    }

    fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    fn pick_color_at(&mut self, cursor: PhysicalPosition<f64>) {
        let (img, window) = match (&self.current_image, &self.window) {
            (Some(i), Some(w)) => (i, w),
            _ => return,
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        // Compute available height (subtract thumbnail strip if visible)
        let available_h = if self.show_thumbnail_strip {
            (size.height as i32 - STRIP_HEIGHT_PX as i32).max(1) as f64
        } else {
            size.height as f64
        };

        // The image is centered (letterboxed) -- reconstruct the same mapping as the GPU shader
        let image_ar = img.width as f32 / img.height as f32;
        let window_ar = size.width as f32 / available_h as f32;
        let ratio = image_ar / window_ar;

        // Extent of the visible image region in normalised window coords (range -1..1)
        let (img_half_w, img_half_h) = if ratio > 1.0 {
            (1.0_f32, 1.0 / ratio)
        } else {
            (ratio, 1.0_f32)
        };

        // Map cursor to -1..1 window space
        let wx = (cursor.x / size.width as f64) as f32 * 2.0 - 1.0;
        let wy = (cursor.y / available_h) as f32 * 2.0 - 1.0;

        // Reject clicks outside the image quad
        if wx < -img_half_w || wx > img_half_w || wy < -img_half_h || wy > img_half_h {
            self.ui_state.set_status("Color Picker: click on the image");
            self.dirty = true;
            return;
        }

        // Convert to [0,1] image UV (accounting for current crop)
        let u_img = (wx + img_half_w) / (2.0 * img_half_w);
        let v_img = (wy + img_half_h) / (2.0 * img_half_h);

        let cr = &self.ui_state.adjustments.crop_rect;
        let u = cr[0] + u_img * cr[2];
        let v = cr[1] + v_img * cr[3];
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        // Sample the RGBA pixel
        let px = (u * (img.width - 1) as f32) as usize;
        let py = (v * (img.height - 1) as f32) as usize;
        let idx = (py * img.width as usize + px) * 4;

        if idx + 3 >= img.rgba_data.len() {
            return;
        }

        let r = img.rgba_data[idx];
        let g = img.rgba_data[idx + 1];
        let b = img.rgba_data[idx + 2];

        self.ui_state.set_status(format!(
            "Color Picker: R:{} G:{} B:{}  #{:02X}{:02X}{:02X}",
            r, g, b, r, g, b
        ));
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

    fn toggle_thumbnail_strip(&mut self) {
        self.show_thumbnail_strip = !self.show_thumbnail_strip;
        self.dirty = true;
    }

    fn rename_current_image(&mut self) {
        if let Some(img) = &self.current_image {
            let old_path = std::path::PathBuf::from(&img.path);
            let filename = old_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            std::thread::spawn(move || {
                if let Some(new_path) = rfd::FileDialog::new()
                    .set_title("Rename File")
                    .set_file_name(&filename)
                    .save_file()
                {
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        if let Some(ref p) = proxy {
                            send_event(
                                &tx,
                                p,
                                AppEvent::SetStatus(format!("Rename failed: {}", e)),
                            );
                        }
                    } else if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::FileRenamed(old_path, new_path));
                    }
                }
            });
        }
    }

    fn set_as_wallpaper(&mut self) {
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                tracing::info!("Setting wallpaper: {}", img.path);
                use std::os::windows::ffi::OsStrExt;
                use windows::Win32::UI::WindowsAndMessaging::{
                    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE,
                    SPI_SETDESKWALLPAPER,
                };

                let path = std::path::Path::new(&img.path);
                let abs_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(path)
                };

                let mut path_wide: Vec<u16> = abs_path.as_os_str().encode_wide().collect();
                path_wide.push(0);

                unsafe {
                    let _ = SystemParametersInfoW(
                        SPI_SETDESKWALLPAPER,
                        0,
                        Some(path_wide.as_mut_ptr() as *mut _),
                        SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
                    );
                }
                self.ui_state.set_status("Desktop wallpaper set!");
                self.dirty = true;
            }
        }
        #[cfg(not(windows))]
        {
            self.ui_state
                .set_status("Wallpaper setting not supported on this OS");
            self.dirty = true;
        }
    }

    fn show_context_menu(&mut self) {
        #[cfg(windows)]
        {
            if let Some(ref w) = self.window {
                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
                use windows::Win32::UI::WindowsAndMessaging::{
                    AppendMenuW, CreatePopupMenu, TrackPopupMenu, MF_STRING, TPM_NONOTIFY,
                    TPM_RETURNCMD,
                };

                unsafe {
                    let hmenu = CreatePopupMenu().unwrap_or_default();
                    if hmenu.is_invalid() {
                        return;
                    }

                    let mut id = 1;
                    let items = [
                        "Open in Explorer",
                        "Copy (Ctrl+C)",
                        "Rename (F2)",
                        "Delete (Del)",
                        "Set as Wallpaper (Ctrl+W)",
                    ];

                    for item in &items {
                        let mut wide: Vec<u16> = std::ffi::OsStr::new(item).encode_wide().collect();
                        wide.push(0);
                        let _ = AppendMenuW(hmenu, MF_STRING, id, PCWSTR(wide.as_ptr()));
                        id += 1;
                    }

                    let mut pt = windows::Win32::Foundation::POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                    let hwnd = if let Ok(handle) = w.window_handle() {
                        match handle.as_raw() {
                            RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut _),
                            _ => HWND::default(),
                        }
                    } else {
                        HWND::default()
                    };

                    let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);

                    let cmd = TrackPopupMenu(
                        hmenu,
                        TPM_RETURNCMD | TPM_NONOTIFY,
                        pt.x,
                        pt.y,
                        0,
                        hwnd,
                        None,
                    );

                    let _ = windows::Win32::UI::WindowsAndMessaging::DestroyMenu(hmenu);

                    match cmd.0 {
                        1 => self.open_in_explorer(),
                        2 => self.copy_to_clipboard(),
                        3 => self.rename_current_image(),
                        4 => self.delete_current_image(),
                        5 => {
                            self.set_as_wallpaper();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn open_in_explorer(&self) {
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                let path = std::path::Path::new(&img.path);
                let abs_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(path)
                };

                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

                let arg = format!("/select,\"{}\"", abs_path.display());

                let verb: Vec<u16> = std::ffi::OsStr::new("open")
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let file: Vec<u16> = std::ffi::OsStr::new("explorer.exe")
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let params: Vec<u16> = std::ffi::OsStr::new(&arg)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                unsafe {
                    let _ = ShellExecuteW(
                        None,
                        PCWSTR(verb.as_ptr()),
                        PCWSTR(file.as_ptr()),
                        PCWSTR(params.as_ptr()),
                        None,
                        SW_SHOW,
                    );
                }
            }
        }
    }

    fn print_image(&self) {
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                tracing::info!("Printing image: {}", img.path);
                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

                let verb: Vec<u16> = std::ffi::OsStr::new("print")
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let file: Vec<u16> = std::ffi::OsStr::new(&img.path)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                unsafe {
                    ShellExecuteW(
                        None,
                        PCWSTR(verb.as_ptr()),
                        PCWSTR(file.as_ptr()),
                        None,
                        None,
                        SW_SHOW,
                    );
                }
            }
        }
        #[cfg(not(windows))]
        {
            tracing::warn!("Print not currently supported on this platform.");
        }
    }

    fn copy_to_clipboard(&mut self) {
        if let Some(img) = &self.current_image {
            let path = std::path::PathBuf::from(&img.path);
            self.ui_state.set_status("Copying to clipboard...");
            self.dirty = true;

            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
                let res = Self::do_copy_to_clipboard(&path);
                if let Some(ref p) = proxy {
                    if let Err(e) = res {
                        send_event(&tx, p, AppEvent::SetStatus(format!("Copy failed: {}", e)));
                    } else {
                        send_event(
                            &tx,
                            p,
                            AppEvent::SetStatus("Copied to clipboard".to_string()),
                        );
                    }
                }
            });
        }
    }

    fn do_copy_to_clipboard(path: &Path) -> Result<()> {
        let img = image::open(path)?;

        #[cfg(target_os = "linux")]
        {
            let mut png_data = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_data),
                image::ImageFormat::Png,
            )?;

            // Try wl-copy
            let child = std::process::Command::new("wl-copy")
                .arg("-t")
                .arg("image/png")
                .stdin(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut c) = child {
                if let Some(mut stdin) = c.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(&png_data);
                }
                let _ = c.wait();
                return Ok(());
            }

            // Try xclip
            let child = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-t", "image/png"])
                .stdin(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut c) = child {
                if let Some(mut stdin) = c.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(&png_data);
                }
                let _ = c.wait();
                return Ok(());
            }

            anyhow::bail!("Neither wl-copy nor xclip found");
        }

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
            use windows::Win32::System::DataExchange::{
                CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
            };
            use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};

            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            let mut bgra = rgba.into_raw();
            for chunk in bgra.chunks_exact_mut(4) {
                chunk.swap(0, 2); // RGBA -> BGRA
            }

            let stride = (width * 4) as usize;
            let mut flipped = vec![0u8; bgra.len()];
            for (y, row) in bgra.chunks_exact(stride).enumerate() {
                let flipped_y = (height as usize - 1) - y;
                flipped[flipped_y * stride..(flipped_y + 1) * stride].copy_from_slice(row);
            }

            let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
            let size = header_size + flipped.len();

            unsafe {
                let hmem = GlobalAlloc(GHND, size)?;
                let ptr = GlobalLock(hmem) as *mut u8;

                let header = BITMAPINFOHEADER {
                    biSize: header_size as u32,
                    biWidth: width as i32,
                    biHeight: height as i32,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: flipped.len() as u32,
                    ..Default::default()
                };

                std::ptr::copy_nonoverlapping(&header as *const _ as *const u8, ptr, header_size);
                std::ptr::copy_nonoverlapping(
                    flipped.as_ptr(),
                    ptr.add(header_size),
                    flipped.len(),
                );
                let _ = GlobalUnlock(hmem);

                // OpenClipboard needs a valid HWND, None connects to current task
                if OpenClipboard(None).is_ok() {
                    let _ = EmptyClipboard();
                    let _ = SetClipboardData(8, HANDLE(hmem.0 as *mut _)); // 8 is CF_DIB
                    let _ = CloseClipboard();
                } else {
                    anyhow::bail!("Failed to open clipboard");
                }
            }
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            anyhow::bail!("Native clipboard not implemented on this OS");
        }
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

    /// Return the index into `renderer.thumbnails` that matches the active image.
    /// Since thumbnails are inserted in the same order as `thumb_paths`, this is
    /// the position of the active path in `thumb_paths`.
    fn active_thumb_index(&self) -> Option<usize> {
        let current = self.ui_state.current_file()?;
        self.thumb_paths.iter().position(|p| p == current)
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
                AppEvent::SetStatus(msg) => {
                    self.dirty = true;
                    self.ui_state.set_status(msg);
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
                AppEvent::ThumbnailLoaded(path, rgba, width, height) => {
                    if let Some(ref mut renderer) = self.renderer {
                        // Only upload if we haven't already got this thumbnail
                        let already_have = renderer.thumbnails.iter().any(|t| t.path == path);
                        if !already_have {
                            if let Err(e) = renderer.upload_thumbnail(path, &rgba, width, height) {
                                tracing::warn!("Failed to upload thumbnail: {}", e);
                            } else {
                                self.dirty = true;
                            }
                        }
                    }
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

                    let new_dir = path.parent().map(|p| p.to_path_buf());
                    let old_dir = self
                        .ui_state
                        .current_file()
                        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

                    if let Some(parent) = path.parent() {
                        self.ui_state.load_directory(parent.to_path_buf());
                    }
                    for (i, f) in self.ui_state.files.iter().enumerate() {
                        if f.path == path {
                            self.ui_state.current_file_index = Some(i);
                            break;
                        }
                    }

                    // Reload thumbnails if we moved to a new directory
                    if new_dir != old_dir || self.thumb_paths.is_empty() {
                        self.load_thumbnails_for_dir();
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

                    let image_count = self.ui_state.files.iter().filter(|f| f.is_image).count();
                    let current_idx = self.ui_state.current_file_index.unwrap_or(0) + 1;

                    self.ui_state.set_status(format!(
                        "{}/{}  |  {}  |  {}×{}  |  {size_mb:.1} MB{}",
                        current_idx,
                        image_count,
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        first_frame.width,
                        first_frame.height,
                        frame_info
                    ));

                    first_frame.compute_histogram();
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
                AppEvent::FileRenamed(old_path, new_path) => {
                    if let Some(img) = &mut self.current_image {
                        if img.path == old_path {
                            img.path = new_path.to_string_lossy().into_owned();
                        }
                    }
                    for file in &mut self.ui_state.files {
                        if file.path == old_path {
                            file.path = new_path.clone();
                            file.name = new_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            break;
                        }
                    }
                    if let Some(frames) = self.prefetch_cache.remove(&old_path) {
                        self.prefetch_cache.insert(new_path.clone(), frames);
                    }
                    self.ui_state.set_status(format!(
                        "Renamed to {}",
                        new_path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    self.dirty = true;
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
                self.alt_pressed = mods.state().alt_key();
                self.shift_pressed = mods.state().shift_key();
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
                    let pos = self.last_cursor_pos;

                    // --- Check thumbnail strip click first ---
                    if self.show_thumbnail_strip {
                        if let Some(ref renderer) = self.renderer {
                            if let Some(thumb_slot) = renderer.thumbnail_index_at(pos.x, pos.y) {
                                // Find the file that corresponds to this thumbnail slot
                                if let Some(path) = self.thumb_paths.get(thumb_slot).cloned() {
                                    // Find the index in ui_state.files
                                    if let Some(file_idx) =
                                        self.ui_state.files.iter().position(|f| f.path == path)
                                    {
                                        if self.ctrl_pressed {
                                            // Ctrl+Click: toggle selection
                                            if self.ui_state.selected_indices.contains(&file_idx) {
                                                self.ui_state.selected_indices.remove(&file_idx);
                                            } else {
                                                self.ui_state.selected_indices.insert(file_idx);
                                            }
                                            let sel_count = self.ui_state.selected_indices.len();
                                            self.ui_state.set_status(format!(
                                                "{} item(s) selected",
                                                sel_count
                                            ));
                                            self.dirty = true;
                                        } else {
                                            // Regular click: navigate + clear selection
                                            self.ui_state.selected_indices.clear();
                                            self.ui_state.current_file_index = Some(file_idx);
                                            self.load_image(path);
                                        }
                                    }
                                }
                                return; // consumed by thumbnail strip
                            }
                        }
                    }

                    // Alt+Click: Color Picker
                    if self.alt_pressed {
                        self.pick_color_at(self.last_cursor_pos);
                        return;
                    }

                    // Check if click was within the strip area but no thumbnail (blank area)
                    if let Some(ref w) = self.window {
                        let win_h = w.inner_size().height as f64;
                        if self.show_thumbnail_strip && pos.y > win_h - STRIP_HEIGHT_PX as f64 {
                            return; // swallow clicks on the strip background
                        }
                    }

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
                (MouseButton::Back, ElementState::Released) => {
                    self.prev_image();
                }
                (MouseButton::Forward, ElementState::Released) => {
                    self.next_image();
                }
                (MouseButton::Left, ElementState::Released) => {
                    self.mouse_drag_start = None;
                }
                (MouseButton::Right, ElementState::Released) => {
                    self.mouse_drag_start = None;
                    self.show_context_menu();
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                self.process_events();
                if self.dirty {
                    // Clone status and capture flags before mut-borrowing renderer
                    let status_opt: Option<String> =
                        self.ui_state.status_message.clone().map(|msg| {
                            let mut final_msg = msg;
                            // Append zoom level to the end of the status message if we are zoomed
                            let zoom_pct = (1.0 / self.ui_state.adjustments.crop_rect[2] * 100.0)
                                .round() as u32;
                            if zoom_pct != 100 {
                                final_msg = format!("{}  |  {}%", final_msg, zoom_pct);
                            }
                            if self.slideshow_active {
                                final_msg = format!(
                                    "▶ {}s  |  {}",
                                    self.slideshow_interval.as_secs(),
                                    final_msg
                                );
                            }
                            final_msg
                        });

                    let is_cropping = self.ui_state.is_cropping;
                    let crop_rect = self.ui_state.adjustments.crop_rect;
                    let show_help = self.ui_state.show_help;
                    let show_sidebar = self.show_sidebar;
                    let show_thumbnail_strip = self.show_thumbnail_strip;
                    let show_info = self.ui_state.show_info;
                    let active_thumb_idx = self.active_thumb_index();
                    let exif_text = if show_info {
                        self.current_image
                            .as_ref()
                            .and_then(|img| img.exif_info.as_deref())
                    } else {
                        None
                    };
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
                                show_thumbnail_strip,
                                active_thumb_idx,
                                &self.ui_state.selected_indices,
                                exif_text,
                                self.show_histogram,
                                self.current_image
                                    .as_ref()
                                    .and_then(|img| img.histogram.as_ref()),
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

        // Smooth zoom interpolation
        let current = self.ui_state.adjustments.crop_rect;
        let target = self.ui_state.adjustments.crop_rect_target;
        let mut animating_zoom = false;

        let diff: f32 = current
            .iter()
            .zip(target.iter())
            .map(|(c, t)| (c - t).abs())
            .sum();
        if diff > 0.001 {
            animating_zoom = true;
            for i in 0..4 {
                self.ui_state.adjustments.crop_rect[i] =
                    current[i] + (target[i] - current[i]) * 0.2;
            }
            self.dirty = true;
        } else if diff > 0.0 {
            self.ui_state.adjustments.crop_rect = target;
            self.dirty = true;
        }

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
        }

        let now = std::time::Instant::now();
        if self.slideshow_active {
            if let Some(st) = self.slideshow_next_time {
                if now >= st {
                    self.next_image();
                    self.slideshow_next_time = Some(now + self.slideshow_interval);
                }
            }
        }

        let mut wait_until = None;
        if animating_zoom {
            wait_until = Some(now + std::time::Duration::from_millis(16));
        }
        if let Some(ft) = self.next_frame_time {
            wait_until = Some(wait_until.map_or(ft, |w| w.min(ft)));
        }
        if self.slideshow_active {
            if let Some(st) = self.slideshow_next_time {
                wait_until = Some(wait_until.map_or(st, |w| w.min(st)));
            }
        }

        if let Some(wu) = wait_until {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wu));
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
        // Clear thumbnail GPU resources
        if let Some(ref mut renderer) = self.renderer {
            renderer.clear_thumbnails();
        }
    }
}
