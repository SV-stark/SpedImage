use crate::app::types::{AppEvent, KeyModifiers, WakeUp};
use crate::image::ImageData;
use crate::render::Renderer;
use crate::ui::UiState;
use lru::LruCache;
use notify_debouncer_full::{Debouncer, FileIdMap};
use notify_debouncer_full::notify;
use rayon::ThreadPool;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;
use dashmap::DashMap;

pub struct NavigationState {
    pub(crate) held_key: Option<char>,
    pub(crate) last_advance_time: Option<std::time::Instant>,
    pub(crate) prefetch_cache: Arc<DashMap<PathBuf, Vec<ImageData>>>,
    pub(crate) load_generation: Arc<AtomicU64>,
    pub(crate) thumb_scroll: f32,
}

pub struct AnimationState {
    pub(crate) frame_idx: usize,
    pub(crate) frame_delays: Vec<u32>,
    pub(crate) next_frame_time: Option<std::time::Instant>,
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
                prefetch_cache: Arc::new(DashMap::new()),
                load_generation: Arc::new(AtomicU64::new(0)),
                thumb_scroll: 0.0,
            },
            animation: AnimationState {
                frame_idx: 0,
                frame_delays: Vec::new(),
                next_frame_time: None,
            },
            slideshow: SlideshowState {
                active: false,
                interval: std::time::Duration::from_secs(3),
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
                    .num_threads(4)
                    .build()
                    .unwrap(),
            ),
            file_watcher: None,
        }
    }
}
