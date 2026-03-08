use crate::app::types::{AppEvent, KeyModifiers, WakeUp};
use crate::image::ImageData;
use crate::render::Renderer;
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

pub(crate) struct AnimationState {
    pub(crate) frame_delays: Vec<u32>,
    pub(crate) frame_idx: usize,
    pub(crate) next_frame_time: Option<std::time::Instant>,
}

pub(crate) struct NavigationState {
    pub(crate) prefetch_cache: LruCache<PathBuf, Vec<ImageData>>,
    pub(crate) prefetch_active: Arc<AtomicUsize>,
    pub(crate) held_key: Option<char>,
    pub(crate) last_advance_time: Option<std::time::Instant>,
}

pub(crate) struct ThumbnailState {
    pub(crate) active_count: Arc<AtomicUsize>,
    pub(crate) paths: Vec<PathBuf>,
}

pub(crate) struct SlideshowState {
    pub(crate) active: bool,
    pub(crate) interval: std::time::Duration,
    pub(crate) next_time: Option<std::time::Instant>,
}

pub struct SpedImageApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) renderer: Option<Renderer>,
    pub(crate) ui_state: UiState,
    pub(crate) current_image: Option<ImageData>,
    pub(crate) animation: AnimationState,
    pub(crate) loading: bool,
    pub(crate) dirty: bool,
    pub(crate) event_tx: Sender<AppEvent>,
    pub(crate) event_rx: Receiver<AppEvent>,
    pub(crate) event_proxy: Option<EventLoopProxy<WakeUp>>,
    pub(crate) mouse_drag_start: Option<PhysicalPosition<f64>>,
    pub(crate) last_cursor_pos: PhysicalPosition<f64>,
    pub(crate) navigation: NavigationState,
    pub(crate) initial_path: Option<PathBuf>,
    pub(crate) thumbnails: ThumbnailState,
    pub(crate) slideshow: SlideshowState,
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
            animation: AnimationState {
                frame_delays: Vec::new(),
                frame_idx: 0,
                next_frame_time: None,
            },
            loading: false,
            dirty: true,
            event_tx: tx,
            event_rx: rx,
            event_proxy: None,
            mouse_drag_start: None,
            last_cursor_pos: PhysicalPosition::new(0.0, 0.0),
            navigation: NavigationState {
                prefetch_cache: LruCache::new(std::num::NonZeroUsize::new(50).unwrap()),
                prefetch_active: Arc::new(AtomicUsize::new(0)),
                held_key: None,
                last_advance_time: None,
            },
            initial_path: None,
            thumbnails: ThumbnailState {
                active_count: Arc::new(AtomicUsize::new(0)),
                paths: Vec::new(),
            },
            slideshow: SlideshowState {
                active: false,
                interval: std::time::Duration::from_secs(5),
                next_time: None,
            },
            modifiers: KeyModifiers::default(),
            thread_pool: None,
            file_watcher: None,
        }
    }
}
 
impl SpedImageApp {

    pub fn default_with_proxy(
        proxy: winit::event_loop::EventLoopProxy<crate::app::types::WakeUp>,
        tx: std::sync::mpsc::Sender<crate::app::types::AppEvent>,
    ) -> Self {
        let mut app = Self::new();
        app.event_proxy = Some(proxy);
        app.event_tx = tx;
        app
    }
}
 
impl Default for SpedImageApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpedImageApp {
    fn drop(&mut self) {
        self.navigation.prefetch_cache.clear();
        self.animation.frame_delays.clear();
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
        assert_eq!(app1.slideshow.interval, app2.slideshow.interval);
    }

    #[test]
    fn test_key_modifiers_default() {
        let mods = KeyModifiers::default();
        assert!(!mods.ctrl);
        assert!(!mods.shift);
        assert!(!mods.alt);
    }
}
