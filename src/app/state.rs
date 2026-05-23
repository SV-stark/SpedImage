use crate::app::constants;
use crate::app::types::{AppEvent, KeyModifiers, WakeUp};
use crate::image::ImageData;
use crate::render::Renderer;
use crate::ui::UiState;
use crossbeam_channel::{Receiver, Sender};
use moka::sync::Cache;
use notify_debouncer_full::notify;
use notify_debouncer_full::{Debouncer, FileIdMap};
use rayon::ThreadPool;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

pub struct NavigationState {
    pub(crate) held_key: Option<char>,
    pub(crate) last_advance_time: Option<std::time::Instant>,
    pub(crate) prefetch_cache: Arc<Cache<PathBuf, Arc<Vec<ImageData>>>>,
    pub(crate) load_generation: Arc<AtomicU64>,
    pub(crate) cancelled_generation: Arc<AtomicU64>,
    pub(crate) thumb_scroll: f32,
    pub(crate) thumb_velocity: f32,
    #[allow(dead_code)]
    pub(crate) thumb_target_scroll: f32,
}

pub struct AnimationState {
    pub(crate) frame_idx: usize,
    pub(crate) frame_delays: Vec<u32>,
    pub(crate) next_frame_time: Option<std::time::Instant>,
    pub(crate) transition_start: Option<std::time::Instant>,
    pub(crate) transition_factor: f32,
}

pub struct SlideshowState {
    pub(crate) active: bool,
    pub(crate) interval: std::time::Duration,
    pub(crate) next_time: Option<std::time::Instant>,
}

pub struct ThumbnailState {
    pub(crate) paths: Vec<PathBuf>,
}

pub struct SpedImageApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) renderer: Option<Renderer>,
    pub(crate) current_image: Option<ImageData>,
    pub(crate) ui_state: UiState,
    pub(crate) navigation: NavigationState,
    pub(crate) animation: AnimationState,
    pub(crate) slideshow: SlideshowState,
    pub(crate) thumbnails: ThumbnailState,
    pub(crate) modifiers: KeyModifiers,
    pub(crate) mouse_drag_start: Option<winit::dpi::PhysicalPosition<f64>>,
    pub(crate) last_cursor_pos: winit::dpi::PhysicalPosition<f64>,
    pub(crate) dirty: bool,
    pub(crate) loading: bool,
    pub(crate) initial_path: Option<PathBuf>,

    pub(crate) event_tx: Sender<AppEvent>,
    pub(crate) event_rx: Receiver<AppEvent>,
    pub(crate) event_proxy: Option<EventLoopProxy<WakeUp>>,
    pub(crate) thread_pool: Arc<ThreadPool>,
    pub(crate) prefetch_pool: Arc<ThreadPool>,
    pub(crate) thumbnail_pool: Arc<ThreadPool>,
    pub(crate) file_watcher: Option<Debouncer<notify::RecommendedWatcher, FileIdMap>>,
}

impl SpedImageApp {
    pub fn new(proxy: EventLoopProxy<WakeUp>) -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        Self {
            window: None,
            renderer: None,
            current_image: None,
            ui_state: UiState::default(),
            navigation: NavigationState {
                held_key: None,
                last_advance_time: None,
                prefetch_cache: Arc::new(
                    Cache::builder()
                        .max_capacity(constants::PREFETCH_CACHE_BYTES)
                        .weigher(|_k, v: &Arc<Vec<ImageData>>| {
                            let mut size = 0;
                            for frame in v.iter() {
                                size += frame.rgba_data.len() as u32;
                            }
                            size
                        })
                        .build(),
                ),
                load_generation: Arc::new(AtomicU64::new(0)),
                cancelled_generation: Arc::new(AtomicU64::new(0)),
                thumb_scroll: 0.0,
                thumb_velocity: 0.0,
                thumb_target_scroll: 0.0,
            },
            animation: AnimationState {
                frame_idx: 0,
                frame_delays: Vec::new(),
                next_frame_time: None,
                transition_start: None,
                transition_factor: 1.0,
            },
            slideshow: SlideshowState {
                active: false,
                interval: constants::DEFAULT_SLIDESHOW_INTERVAL,
                next_time: None,
            },
            thumbnails: ThumbnailState { paths: Vec::new() },
            modifiers: KeyModifiers::default(),
            mouse_drag_start: None,
            last_cursor_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            dirty: true,
            loading: false,
            initial_path: None,
            event_tx,
            event_rx,
            event_proxy: Some(proxy),
            thread_pool: Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(2)
                    .thread_name(|i| format!("spedimage-main-{}", i))
                    .build()
                    .expect("Failed to initialize Rayon thread pool (main). This is fatal."),
            ),
            prefetch_pool: Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(1)
                    .thread_name(|i| format!("spedimage-prefetch-{}", i))
                    .build()
                    .expect("Failed to initialize Rayon thread pool (prefetch). This is fatal."),
            ),
            thumbnail_pool: Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(4)
                    .thread_name(|i| format!("spedimage-thumbnail-{}", i))
                    .build()
                    .expect("Failed to initialize Rayon thread pool (thumbnail). This is fatal."),
            ),
            file_watcher: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use winit::event_loop::{EventLoop, EventLoopProxy};

    static SHARED_PROXY: Mutex<Option<EventLoopProxy<WakeUp>>> = Mutex::new(None);

    fn get_test_proxy() -> EventLoopProxy<WakeUp> {
        let mut proxy_lock = SHARED_PROXY.lock().unwrap();
        if proxy_lock.is_none() {
            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                let mut builder = EventLoop::<WakeUp>::with_user_event();
                builder.with_any_thread(true);
                let event_loop = builder.build().unwrap();
                *proxy_lock = Some(event_loop.create_proxy());
            }
            #[cfg(not(target_os = "windows"))]
            {
                let event_loop = EventLoop::<WakeUp>::with_user_event().build().unwrap();
                *proxy_lock = Some(event_loop.create_proxy());
            }
        }
        proxy_lock.clone().unwrap()
    }

    #[test]
    fn test_app_creation() {
        let proxy = get_test_proxy();
        let app = SpedImageApp::new(proxy);

        assert!(app.window.is_none());
        assert!(app.renderer.is_none());
        assert!(app.current_image.is_none());
        assert!(app.ui_state.files.is_empty());
        assert_eq!(app.ui_state.current_file_index, None);
        assert!(app.dirty);
        assert!(!app.loading);
        assert!(app.initial_path.is_none());
        assert!(app.navigation.held_key.is_none());
        assert!(app.animation.frame_delays.is_empty());
        assert_eq!(app.animation.transition_factor, 1.0);
        assert!(!app.slideshow.active);
        assert!(app.thumbnails.paths.is_empty());
        assert!(!app.modifiers.ctrl);
        assert!(!app.modifiers.shift);
        assert!(!app.modifiers.alt);
    }

    #[test]
    fn test_navigation_state_creation() {
        let proxy = get_test_proxy();
        let app = SpedImageApp::new(proxy);

        assert_eq!(app.navigation.thumb_scroll, 0.0);
        assert_eq!(app.navigation.thumb_velocity, 0.0);
        assert_eq!(app.navigation.thumb_target_scroll, 0.0);
        assert!(app.navigation.last_advance_time.is_none());
        assert!(app.navigation.held_key.is_none());
        assert_eq!(
            app.navigation
                .load_generation
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn test_animation_state_creation() {
        let proxy = get_test_proxy();
        let app = SpedImageApp::new(proxy);

        assert_eq!(app.animation.frame_idx, 0);
        assert!(app.animation.frame_delays.is_empty());
        assert!(app.animation.next_frame_time.is_none());
        assert!(app.animation.transition_start.is_none());
        assert!((app.animation.transition_factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_slideshow_state_creation() {
        let proxy = get_test_proxy();
        let app = SpedImageApp::new(proxy);

        assert!(!app.slideshow.active);
        assert_eq!(app.slideshow.interval, std::time::Duration::from_secs(3));
        assert!(app.slideshow.next_time.is_none());
    }
}
