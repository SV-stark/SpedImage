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
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

/// What an in-progress crop drag is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CropDrag {
    /// Dragging inside the crop rect moves it.
    Move,
    /// Dragging the bottom-right handle resizes it.
    Resize,
}

pub struct NavigationState {
    pub(crate) held_key: Option<char>,
    pub(crate) last_advance_time: Option<std::time::Instant>,
    pub(crate) prefetch_cache: OnceLock<Arc<Cache<PathBuf, Arc<Vec<ImageData>>>>>,
    pub(crate) load_generation: Arc<AtomicU64>,
    pub(crate) thumb_generation: Arc<AtomicU64>,
    pub(crate) thumb_scroll: f32,
    pub(crate) thumb_velocity: f32,
    pub(crate) last_left_click_time: Option<std::time::Instant>,
    pub(crate) last_direction: i8,
}

pub struct AnimationState {
    pub(crate) frame_idx: usize,
    pub(crate) frame_delays: Vec<u32>,
    pub(crate) next_frame_time: Option<std::time::Instant>,
    pub(crate) transition_start: Option<std::time::Instant>,
    pub(crate) transition_factor: f32,
    /// Streamed GIF frames awaiting playback, ordered by index.
    pub(crate) pending: std::collections::VecDeque<crate::image::GifFrameMsg>,
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
    /// Corner being dragged while crop mode is active (bottom-right = resize,
    /// inside the rect = move). `None` until a crop drag starts.
    pub(crate) crop_drag: Option<CropDrag>,
    /// Set when the user zooms; fires progressive refinement after settling.
    pub(crate) zoom_settle_at: Option<std::time::Instant>,
    /// A progressive high-res decode is currently running for this image.
    pub(crate) highres_in_flight: bool,
    pub(crate) dirty: bool,
    pub(crate) loading: bool,
    pub(crate) initial_path: Option<PathBuf>,

    pub(crate) event_tx: Sender<AppEvent>,
    pub(crate) event_rx: Receiver<AppEvent>,
    /// Look-ahead channel from the streaming GIF decoder (None = idle).
    pub(crate) gif_rx: Option<crossbeam_channel::Receiver<crate::image::GifStreamMsg>>,
    pub(crate) event_proxy: Option<EventLoopProxy<WakeUp>>,
    pub(crate) thread_pool: OnceLock<Arc<ThreadPool>>,
    pub(crate) prefetch_pool: OnceLock<Arc<ThreadPool>>,
    pub(crate) thumbnail_pool: OnceLock<Arc<ThreadPool>>,
    pub(crate) file_watcher: Option<Debouncer<notify::RecommendedWatcher, FileIdMap>>,
    pub(crate) config: crate::config::AppConfig,
    /// File pinned to the left half of the side-by-side compare view.
    pub(crate) compare_pinned: Option<PathBuf>,
    /// Name of the monitor the display profile was last read for, so moving
    /// the window to another screen re-reads it.
    pub(crate) profile_monitor: Option<String>,
}

impl SpedImageApp {
    pub fn new(proxy: EventLoopProxy<WakeUp>) -> Self {
        crate::startup::log("App::new enter");
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let (config, config_warning) = crate::config::AppConfig::load_reporting();
        let ui_state = UiState {
            notice: config_warning,
            ..UiState::default()
        };

        let app = Self {
            window: None,
            renderer: None,
            current_image: None,
            ui_state,
            navigation: NavigationState {
                held_key: None,
                last_advance_time: None,
                prefetch_cache: OnceLock::new(),
                load_generation: Arc::new(AtomicU64::new(0)),
                thumb_generation: Arc::new(AtomicU64::new(0)),
                thumb_scroll: 0.0,
                thumb_velocity: 0.0,
                last_left_click_time: None,
                last_direction: 1,
            },
            animation: AnimationState {
                frame_idx: 0,
                frame_delays: Vec::new(),
                next_frame_time: None,
                transition_start: None,
                transition_factor: 1.0,
                pending: std::collections::VecDeque::new(),
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
            crop_drag: None,
            zoom_settle_at: None,
            highres_in_flight: false,
            dirty: true,
            loading: false,
            initial_path: None,
            event_tx,
            event_rx,
            gif_rx: None,
            event_proxy: Some(proxy),
            thread_pool: OnceLock::new(),
            prefetch_pool: OnceLock::new(),
            thumbnail_pool: OnceLock::new(),
            file_watcher: None,
            config,
            compare_pinned: None,
            profile_monitor: None,
        };
        crate::startup::log("App::new done");
        app
    }

    pub(crate) fn thread_pool(&self) -> Arc<ThreadPool> {
        self.thread_pool
            .get_or_init(|| {
                Arc::new(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(2)
                        .thread_name(|i| format!("spedimage-main-{}", i))
                        .build()
                        .expect("Failed to initialize Rayon thread pool (main)."),
                )
            })
            .clone()
    }

    pub(crate) fn prefetch_pool(&self) -> Arc<ThreadPool> {
        self.prefetch_pool
            .get_or_init(|| {
                Arc::new(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(1)
                        .thread_name(|i| format!("spedimage-prefetch-{}", i))
                        .build()
                        .expect("Failed to initialize Rayon thread pool (prefetch)."),
                )
            })
            .clone()
    }

    pub(crate) fn thumbnail_pool(&self) -> Arc<ThreadPool> {
        self.thumbnail_pool
            .get_or_init(|| {
                Arc::new(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(4)
                        .thread_name(|i| format!("spedimage-thumbnail-{}", i))
                        .build()
                        .expect("Failed to initialize Rayon thread pool (thumbnail)."),
                )
            })
            .clone()
    }

    pub(crate) fn prefetch_cache(&self) -> Arc<Cache<PathBuf, Arc<Vec<ImageData>>>> {
        self.navigation
            .prefetch_cache
            .get_or_init(|| {
                Arc::new(
                    Cache::builder()
                        .max_capacity(constants::PREFETCH_CACHE_BYTES)
                        .weigher(|_k, v: &Arc<Vec<ImageData>>| {
                            let mut size: u64 = 0;
                            for frame in v.iter() {
                                size = size.saturating_add(frame.rgba_data.len() as u64);
                            }
                            size.min(u32::MAX as u64) as u32
                        })
                        .build(),
                )
            })
            .clone()
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
        assert_eq!(
            app.navigation
                .thumb_generation
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert!(app.navigation.last_advance_time.is_none());
        assert!(app.navigation.held_key.is_none());
        assert!(app.navigation.last_left_click_time.is_none());
        assert_eq!(
            app.navigation
                .load_generation
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn test_app_toggle_zoom_100() {
        let proxy = get_test_proxy();
        let mut app = SpedImageApp::new(proxy);

        // Initially crop_rect_target is [0, 0, 1, 1] (fully zoomed out)
        assert_eq!(
            app.ui_state.adjustments.crop_rect_target,
            [0.0, 0.0, 1.0, 1.0]
        );

        // Call toggle_zoom_100 (should zoom in, but since current_image is None, it won't crash and will do nothing)
        app.toggle_zoom_100();
        assert_eq!(
            app.ui_state.adjustments.crop_rect_target,
            [0.0, 0.0, 1.0, 1.0]
        );

        // Mock current_image
        app.current_image = Some(ImageData {
            path: std::path::PathBuf::from("test.jpg"),
            width: 1000,
            height: 1000,
            rgba_data: Arc::new(vec![0; 4000]),
            format: crate::image::ImageFormatType::Jpeg,
            file_size_bytes: 4000,
            frame_delay_ms: 0,
            exif_info: None,
            histogram: None,
            exif_loaded: true,
            is_downsampled: false,
            orientation_deg: 0,
            gps_coords: None,
            color_space: None,
        });

        // Toggle Zoom 100 with current image: new crop_w/crop_h will be win_w/img_w.
        app.toggle_zoom_100();
        assert_ne!(app.ui_state.adjustments.crop_rect_target[2], 1.0);

        // Toggle zoom again: it should reset back to 1.0 (zoom fit)
        app.toggle_zoom_100();
        assert_eq!(
            app.ui_state.adjustments.crop_rect_target,
            [0.0, 0.0, 1.0, 1.0]
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
