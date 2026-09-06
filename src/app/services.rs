use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, MAX_THUMB_THREADS, MAX_THUMBNAILS, THUMB_LOAD_SIZE, send_event};
use crate::image::ImageBackend;
use std::path::{Path, PathBuf};

/// Natural numerical comparison (e.g. "img1" < "img2" < "img10"), matching Windows Explorer
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ca), Some(&cb)) if ca.is_ascii_digit() && cb.is_ascii_digit() => {
                let mut num_a = 0u64;
                while let Some(&c) = a_chars.peek() {
                    if let Some(d) = c.to_digit(10) {
                        num_a = num_a.saturating_mul(10).saturating_add(d as u64);
                        a_chars.next();
                    } else {
                        break;
                    }
                }
                let mut num_b = 0u64;
                while let Some(&c) = b_chars.peek() {
                    if let Some(d) = c.to_digit(10) {
                        num_b = num_b.saturating_mul(10).saturating_add(d as u64);
                        b_chars.next();
                    } else {
                        break;
                    }
                }
                if num_a != num_b {
                    return num_a.cmp(&num_b);
                }
            }
            (Some(&ca), Some(&cb)) => {
                let lower_a = ca.to_lowercase().next().unwrap_or(ca);
                let lower_b = cb.to_lowercase().next().unwrap_or(cb);
                if lower_a != lower_b {
                    return lower_a.cmp(&lower_b);
                }
                a_chars.next();
                b_chars.next();
            }
        }
    }
}

impl SpedImageApp {
    pub(crate) fn load_directory_async(&self, dir: PathBuf) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let pool = self.thread_pool().clone();

        pool.spawn(move || {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if ImageBackend::is_supported(&path) {
                        files.push(crate::ui::FileEntry::new(path));
                    }
                }
                files.sort_by(|a, b| natural_cmp(&a.name, &b.name));

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

        // Position lookup shared by retention, re-indexing and workers.
        use rustc_hash::FxHashMap;
        let index: FxHashMap<&PathBuf, usize> =
            files.iter().enumerate().map(|(i, p)| (p, i)).collect();

        // Incremental refresh: keep textures for files still present,
        // drop only removed ones, decode only new ones.
        if let Some(ref mut r) = self.renderer {
            r.thumbnails.retain(|t| index.contains_key(&t.path));
            for thumb in r.thumbnails.iter_mut() {
                if let Some(i) = index.get(&thumb.path) {
                    thumb.order = *i;
                }
            }
            r.thumbnails.sort_by_key(|t| t.order);
            r.last_thumb_state = None;
        }

        let existing: std::collections::HashSet<&PathBuf> = self
            .renderer
            .as_ref()
            .map(|r| r.thumbnails.iter().map(|t| &t.path).collect())
            .unwrap_or_default();

        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let pool = self.thumbnail_pool().clone();

        // Current generation for thumbnail batch cancellation
        use std::sync::atomic::Ordering;
        let generation = self
            .navigation
            .thumb_generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        let current_gen = self.navigation.thumb_generation.clone();

        // Prioritize work closest to currently viewed image
        let center = self.ui_state.current_file_index.unwrap_or(0);
        let mut work_order: Vec<(PathBuf, usize)> = files
            .iter()
            .enumerate()
            .filter(|(_, p)| !existing.contains(p))
            .map(|(i, p)| (p.clone(), i))
            .collect();
        work_order.sort_by_key(|(_, i)| (*i as isize - center as isize).abs());

        // Work queue of (path, order) pairs that still need a texture.
        let (tx_work, rx_work) = crossbeam_channel::unbounded();
        for (path, order) in work_order.into_iter().take(MAX_THUMBNAILS) {
            tx_work.send((path, order)).ok();
        }
        drop(tx_work); // Close producer so workers exit when queue is empty

        // Spawn a fixed number of workers
        for _ in 0..MAX_THUMB_THREADS {
            let rx = rx_work.clone();
            let tx = tx.clone();
            let proxy = proxy.clone();
            let gen_check = current_gen.clone();

            pool.spawn(move || {
                while let Ok((path_clone, order)) = rx.recv() {
                    // Early exit check
                    if gen_check.load(Ordering::Relaxed) != generation {
                        break;
                    }

                    if let Ok(frames) = ImageBackend::load_and_downsample_with(
                        &path_clone,
                        THUMB_LOAD_SIZE,
                        THUMB_LOAD_SIZE,
                        crate::image::LoadOptions {
                            frames: crate::image::FrameLimit::First,
                            bake_orientation: true,
                        },
                    ) && let Some(frame) = frames.first()
                        && let Some(ref p) = proxy
                    {
                        // Check again after expensive load
                        if gen_check.load(Ordering::Relaxed) != generation {
                            break;
                        }

                        send_event(
                            &tx,
                            p,
                            AppEvent::ThumbnailLoaded(
                                path_clone,
                                frame.rgba_data.clone(),
                                frame.width,
                                frame.height,
                                order,
                            ),
                        );
                    }
                }
            });
        }
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
                if let Ok(_events) = res
                    && let Some(ref p) = proxy
                {
                    send_event(&tx, p, AppEvent::DirectoryChanged(dir_clone.clone()));
                }
            })
            .ok();

        if let Some(ref mut d) = debouncer {
            let _ = d.watcher().watch(&dir, RecursiveMode::NonRecursive);
        }
        self.file_watcher = debouncer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_cmp() {
        use std::cmp::Ordering;
        assert_eq!(natural_cmp("img1.jpg", "img2.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("img2.jpg", "img10.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("img10.jpg", "img2.jpg"), Ordering::Greater);
        assert_eq!(natural_cmp("photo.png", "photo.png"), Ordering::Equal);
        assert_eq!(natural_cmp("Photo.png", "photo.png"), Ordering::Equal);
        assert_eq!(natural_cmp("a10b", "a2b"), Ordering::Greater);
        assert_eq!(natural_cmp("100", "20"), Ordering::Greater);
    }
}
