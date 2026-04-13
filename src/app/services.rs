use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, MAX_THUMBNAILS, THUMB_LOAD_SIZE, send_event};
use crate::image::ImageBackend;
use std::path::{Path, PathBuf};

impl SpedImageApp {
    pub(crate) fn load_directory_async(&self, dir: PathBuf) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let pool = self.thread_pool.clone();

        pool.spawn(move || {
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

        self.thumbnails.paths = files.clone();

        if let Some(ref mut r) = self.renderer {
            r.clear_thumbnails();
        }

        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let pool = self.thread_pool.clone();
        let pool_outer = self.thread_pool.clone();

        pool_outer.spawn(move || {
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

    pub(crate) fn setup_file_watcher(&mut self, path: &Path) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        use notify_debouncer_full::{new_debouncer, notify::RecursiveMode, notify::Watcher};

        let dir_clone = dir.clone();
        let mut debouncer =
            new_debouncer(std::time::Duration::from_millis(500), None, move |res| {
                if let Ok(_events) = res {
                    if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::DirectoryChanged(dir_clone.clone()));
                    }
                }
            })
            .ok();

        if let Some(ref mut d) = debouncer {
            let _ = d.watcher().watch(&dir, RecursiveMode::NonRecursive);
        }
        self.file_watcher = debouncer;
    }
}
