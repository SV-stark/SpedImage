use crate::app::types::{AppEvent, WakeUp, KeyModifiers};
use crate::gpu_renderer::Renderer;
use crate::image_backend::ImageData;
use crate::ui::UiState;
use lru::LruCache;
use notify::RecommendedWatcher;
use rayon::ThreadPool;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use winit::dpi::PhysicalPosition;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

pub struct SpedImageApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) renderer: Option<Renderer>,
    pub(crate) ui_state: UiState,
    pub(crate) current_image: Option<ImageData>,
    pub(crate) current_frame_delays: Vec<u32>,
    pub(crate) current_frame_idx: usize,
    pub(crate) next_frame_time: Option<std::time::Instant>,
    pub(crate) loading: bool,
    pub(crate) dirty: bool,
    pub(crate) event_tx: Sender<AppEvent>,
    pub(crate) event_rx: Receiver<AppEvent>,
    pub(crate) event_proxy: Option<EventLoopProxy<WakeUp>>,
    pub(crate) mouse_drag_start: Option<PhysicalPosition<f64>>,
    pub(crate) last_cursor_pos: PhysicalPosition<f64>,
    pub(crate) prefetch_cache: LruCache<PathBuf, Vec<ImageData>>,
    pub(crate) prefetch_active: Arc<AtomicUsize>,
    pub(crate) initial_path: Option<PathBuf>,
    pub(crate) held_navigation_key: Option<char>,
    pub(crate) last_advance_time: Option<std::time::Instant>,
    pub(crate) thumb_active: Arc<AtomicUsize>,
    pub(crate) thumb_paths: Vec<PathBuf>,
    pub(crate) slideshow_active: bool,
    pub(crate) slideshow_interval: std::time::Duration,
    pub(crate) slideshow_next_time: Option<std::time::Instant>,
    pub(crate) modifiers: KeyModifiers,
    #[allow(dead_code)]
    pub(crate) thread_pool: Option<Arc<ThreadPool>>,
    #[allow(dead_code)]
    pub(crate) file_watcher: Option<RecommendedWatcher>,
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
            prefetch_cache: LruCache::new(std::num::NonZeroUsize::new(50).unwrap()),
            prefetch_active: Arc::new(AtomicUsize::new(0)),
            initial_path: None,
            held_navigation_key: None,
            last_advance_time: None,
            thumb_active: Arc::new(AtomicUsize::new(0)),
            thumb_paths: Vec::new(),
            slideshow_active: false,
            slideshow_interval: std::time::Duration::from_secs(5),
            slideshow_next_time: None,
            modifiers: KeyModifiers::default(),
            thread_pool: None,
            file_watcher: None,
        }
    }
}

impl Default for SpedImageApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpedImageApp {
    fn drop(&mut self) {
        self.prefetch_cache.clear();
        self.current_frame_delays.clear();
        self.current_image = None;
        if let Some(ref mut renderer) = self.renderer {
            renderer.clear_thumbnails();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = SpedImageApp::new();
        assert!(app.window.is_none());
        assert!(app.renderer.is_none());
        assert!(!app.loading);
        assert!(app.dirty);
        assert!(!app.ui_state.show_help);
    }

    #[test]
    fn test_app_default() {
        let app1 = SpedImageApp::new();
        let app2 = SpedImageApp::default();
        // Just verify some key fields to ensure default() behaves like new()
        assert_eq!(app1.loading, app2.loading);
        assert_eq!(app1.dirty, app2.dirty);
        assert_eq!(app1.ui_state.show_help, app2.ui_state.show_help);
        assert_eq!(app1.slideshow_interval, app2.slideshow_interval);
    }

    #[test]
    fn test_key_modifiers_default() {
        let mods = KeyModifiers::default();
        assert!(!mods.ctrl);
        assert!(!mods.shift);
        assert!(!mods.alt);
    }
}
