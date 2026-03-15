use crate::app::state::SpedImageApp;
use crate::app::types::{send_event, AppEvent, MAX_THUMBNAILS, THUMB_LOAD_SIZE};
use crate::image::ImageBackend;
use std::path::{Path, PathBuf};

impl SpedImageApp {
    pub(crate) fn load_directory_async(&self, dir: PathBuf) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        std::thread::spawn(move || {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if ImageBackend::is_supported(&path) {
                        files.push(crate::ui::FileEntry::new(path));
                    }
                }
                files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                if let Some(ref p) = proxy {
                    send_event(&tx, p, AppEvent::DirectoryLoaded(dir, files));
                }
            }
        });
    }

    pub(crate) fn load_thumbnails_for_dir(&mut self) {
        let files: Vec<PathBuf> = self.ui_state.files.iter().map(|f| f.path.clone()).collect();
        if files.is_empty() {
            return;
        }

        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let pool = self.thread_pool.clone();

        std::thread::spawn(move || {
            for path in files.into_iter().take(MAX_THUMBNAILS) {
                let tx = tx.clone();
                let proxy = proxy.clone();
                let path_clone = path.clone();

                pool.spawn(move || {
                    if let Ok(frames) = ImageBackend::load_and_downsample(
                        &path_clone,
                        THUMB_LOAD_SIZE,
                        THUMB_LOAD_SIZE,
                    ) {
                        if let Some(frame) = frames.first() {
                            if let Some(ref p) = proxy {
                                send_event(
                                    &tx,
                                    p,
                                    AppEvent::ThumbnailLoaded(
                                        path_clone,
                                        frame.rgba_data.clone(),
                                        frame.width,
                                        frame.height,
                                    ),
                                );
                            }
                        }
                    }
                });
            }
        });
    }

    pub(crate) fn prefetch_adjacent(&self, current_path: &Path) {
        if self.ui_state.files.len() < 2 {
            return;
        }

        let idx = match self
            .ui_state
            .files
            .iter()
            .position(|f| f.path == current_path)
        {
            Some(i) => i,
            None => return,
        };

        let next_idx = (idx + 1) % self.ui_state.files.len();
        let prev_idx = if idx == 0 {
            self.ui_state.files.len() - 1
        } else {
            idx - 1
        };

        let targets = vec![
            self.ui_state.files[next_idx].path.clone(),
            self.ui_state.files[prev_idx].path.clone(),
        ];

        let (max_w, max_h) = if let Some(ref w) = self.window {
            let size = w.inner_size();
            (size.width, size.height)
        } else {
            (1920, 1080)
        };

        for path in targets {
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
                if let Ok(frames) = ImageBackend::load_and_downsample(&path, max_w, max_h) {
                    if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::Prefetched(path, frames));
                    }
                }
            });
        }
    }

    pub(crate) fn setup_file_watcher(&mut self, path: &Path) {
        let _tx = self.event_tx.clone();
        let _proxy = self.event_proxy.clone();
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        use notify_debouncer_full::{new_debouncer, notify::RecursiveMode, notify::Watcher};

        let mut debouncer =
            new_debouncer(std::time::Duration::from_millis(500), None, move |res| {
                if let Ok(_events) = res {
                    // Trigger a reload or specific event
                }
            })
            .ok();

        if let Some(ref mut d) = debouncer {
            let _ = d.watcher().watch(&dir, RecursiveMode::NonRecursive);
        }
        self.file_watcher = debouncer;
    }
}
