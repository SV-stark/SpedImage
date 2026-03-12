use crate::image::ImageData;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use winit::event_loop::EventLoopProxy;

/// Wakeup token sent through EventLoopProxy to wake the sleeping event loop.
/// The actual payload travels through a regular mpsc channel.
#[derive(Debug)]
pub struct WakeUp;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

pub const APP_ICON: &[u8] = include_bytes!("../../assets/icons/icon.png");

/// Thumbnail size used for background loading (must match THUMB_SIZE in gpu_renderer).
pub const THUMB_LOAD_SIZE: u32 = 80;
/// Max concurrently-running thumbnail background threads.
pub const MAX_THUMB_THREADS: usize = 4;
/// Max thumbnails kept in GPU at once (older ones are not evicted but new ones stop loading).
pub const MAX_THUMBNAILS: usize = 200;

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
    DirectoryLoaded(PathBuf, Vec<crate::ui::FileEntry>),
    DirectoryError(String),
}

/// Helper: send an AppEvent through the data channel, then wake the event loop.
pub fn send_event(tx: &Sender<AppEvent>, proxy: &EventLoopProxy<WakeUp>, event: AppEvent) {
    tx.send(event).ok();
    proxy.send_event(WakeUp).ok();
}
