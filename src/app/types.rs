use crate::image::ImageData;
use crossbeam_channel::Sender;
use std::path::PathBuf;
use std::sync::Arc;
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
    SaveComplete(crate::image::SaveOutcome),
    SaveError(String),
    /// A thumbnail has finished loading: (path, rgba_bytes, width, height, order)
    ThumbnailLoaded(PathBuf, Arc<Vec<u8>>, u32, u32, usize),
    /// Full-resolution re-decode for progressive refinement of the current image.
    HighResReady(PathBuf, Box<ImageData>),
    /// The full-resolution re-decode failed or was too large to keep.
    HighResFailed(PathBuf),
    /// Async GPU renderer initialized (or failed). Boxed to keep enum small.
    RendererReady(Result<Box<crate::render::Renderer>, String>),
    SetStatus(String),
    FileRenamed(PathBuf, PathBuf),
    DirectoryLoaded(PathBuf, Vec<crate::ui::FileEntry>),
    DirectoryError(String),
    ConfirmDelete(PathBuf),
    ConfirmBatchDelete(Vec<PathBuf>),
    HistogramComputed(PathBuf, Box<([u32; 256], [u32; 256], [u32; 256])>),
    DirectoryChanged(PathBuf),
    TriggerOpenFileDialog,
    /// The sort key or direction in `AppConfig` changed; re-list the folder.
    SortChanged,
    /// The "match monitor colour profile" preference was toggled.
    DisplayProfileToggled,
    /// Navigate to the previous or next image in the folder.
    PrevImage,
    NextImage,
    /// Undo / Redo non-destructive image edits.
    UndoEdit,
    RedoEdit,
    /// Snapshot captured before a UI-driven edit (e.g., slider drag start).
    RecordEditSnapshot(Box<crate::render::ImageAdjustments>),
    /// Run one-click histogram-driven Auto Enhance.
    AutoEnhance,
    /// Toggle live pixel eyedropper / colour inspector.
    ToggleColorPicker,
    /// Pause/resume or step animated GIF / WebP / multi-page TIFF playback.
    ToggleAnimationPause,
    StepAnimationFrame,
    StepAnimationFramePrev,
    /// Copy / Paste non-destructive edits across images.
    CopyAdjustments,
    PasteAdjustments,
    /// Cycle WGSL Channel / Clipping Inspector mode.
    CycleInspectMode,
    /// Pan viewport center to normalized image coordinates (from Zoom Mini-Map).
    PanToNormalized(f32, f32),
    /// Async EXIF load completed for the given image path.
    ExifLoaded(PathBuf, Option<String>, Option<(f64, f64)>),
}

/// Helper: send an AppEvent through the data channel, then wake the event loop.
pub fn send_event(tx: &Sender<AppEvent>, proxy: &EventLoopProxy<WakeUp>, event: AppEvent) {
    tx.send(event).ok();
    proxy.send_event(WakeUp).ok();
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants, clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers_default() {
        let mods = KeyModifiers::default();
        assert!(!mods.ctrl);
        assert!(!mods.shift);
        assert!(!mods.alt);
    }

    #[test]
    fn test_key_modifiers_set_individually() {
        let mut mods = KeyModifiers::default();
        mods.ctrl = true;
        assert!(mods.ctrl);
        assert!(!mods.shift);
        assert!(!mods.alt);

        mods.shift = true;
        assert!(mods.ctrl);
        assert!(mods.shift);
        assert!(!mods.alt);

        mods.alt = true;
        assert!(mods.ctrl);
        assert!(mods.shift);
        assert!(mods.alt);
    }

    #[test]
    fn test_key_modifiers_copy() {
        let mut mods = KeyModifiers::default();
        mods.ctrl = true;
        mods.shift = true;
        let copied = mods;
        assert!(copied.ctrl);
        assert!(copied.shift);
        assert!(!copied.alt);
    }

    #[test]
    fn test_thumb_constants() {
        assert!(THUMB_LOAD_SIZE > 0);
        assert!(MAX_THUMB_THREADS > 0);
        assert!(MAX_THUMBNAILS > 0);
        assert!(MAX_THUMBNAILS >= MAX_THUMB_THREADS);
    }

    #[test]
    fn test_wake_up_debug() {
        let w = WakeUp;
        // Just verify it implements Debug (compile-time check via derive)
        let _ = format!("{:?}", w);
    }
}
