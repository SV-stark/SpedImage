use crate::app::state::SpedImageApp;
use crate::app::types::{send_event, AppEvent, MAX_THUMBNAILS, MAX_THUMB_THREADS, THUMB_LOAD_SIZE};
use crate::image::ImageBackend;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

impl SpedImageApp {
    /// (6) Prefetch the images neighboring the current one in the folder.
    pub(crate) fn prefetch_adjacent(&mut self, current: &Path) {
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
            if self.navigation.prefetch_cache.contains(&path) {
                continue;
            }
            const MAX_CONCURRENT_PREFETCH: usize = 2;
            if self.navigation.prefetch_active.load(Ordering::Relaxed) >= MAX_CONCURRENT_PREFETCH {
                continue;
            }
            self.navigation.prefetch_active.fetch_add(1, Ordering::Relaxed);
            let active = self.navigation.prefetch_active.clone();
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

    /// Set up a file watcher to monitor directory changes
    pub(crate) fn setup_file_watcher(&mut self, dir: &Path) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        let watcher_result = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        notify::EventKind::Create(_)
                        | notify::EventKind::Modify(_)
                        | notify::EventKind::Remove(_) => {
                            if let Some(ref proxy) = proxy {
                                send_event(
                                    &tx,
                                    proxy,
                                    AppEvent::SetStatus(
                                        "Directory changed - press R to refresh".to_string(),
                                    ),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        );

        if let Ok(mut watcher) = watcher_result {
            if watcher.watch(dir, RecursiveMode::NonRecursive).is_ok() {
                self.file_watcher = Some(watcher);
                tracing::debug!("File watcher started for {:?}", dir);
            }
        }
    }

    /// Spawn background thumbnail-loading threads for all files in the current directory.
    pub(crate) fn load_thumbnails_for_dir(&mut self) {
        let files: Vec<PathBuf> = self.ui_state.files.iter().map(|f| f.path.clone()).collect();
        self.thumbnails.paths = files.clone();

        if let Some(ref mut renderer) = self.renderer {
            renderer.clear_thumbnails();
        }

        let n = files.len().min(MAX_THUMBNAILS);
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let active = self.thumbnails.active_count.clone();

        for path in files.into_iter().take(n) {
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
}
 
#[cfg(test)]
mod tests {
    use super::*;
    use winit::event_loop::EventLoopBuilder;
    use crate::app::state::SpedImageApp;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
 
    #[test]
    fn test_prefetch_logic_caching() {
        use winit::event_loop::EventLoop;
        let event_loop = EventLoop::<crate::app::types::WakeUp>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();
        let (tx, _rx) = std::sync::mpsc::channel();
 
        let mut app = SpedImageApp::default_with_proxy(proxy, tx);
 
        let test_path = PathBuf::from("test_image_123.png");
        app.navigation.prefetch_cache.put(test_path.clone(), ());
        
        let neighbors = vec![test_path.clone()];
        app.prefetch_adjacent(&test_path);
        
        // Assert no new prefetch was started because it's in cache
        assert_eq!(app.navigation.prefetch_active.load(Ordering::Relaxed), 0);
    }
}
