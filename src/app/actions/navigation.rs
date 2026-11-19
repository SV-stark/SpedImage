use crate::app::constants;
use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, send_event};
use crate::image::ImageBackend;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use winit::dpi::PhysicalPosition;

impl SpedImageApp {
    pub(crate) fn load_image(&mut self, path: &std::path::Path) {
        self.load_image_sized(path, None);
    }

    /// Like [`Self::load_image`], decoding to fit `fit` (physical pixels)
    /// instead of the current window. Used at startup to begin decoding
    /// before the window exists.
    pub(crate) fn load_image_sized(&mut self, path: &std::path::Path, fit: Option<(u32, u32)>) {
        // Evict far-away cache entries based on current navigation index.
        // Only the paths are cloned (not the whole `FileEntry` list): this runs
        // on the UI thread for every keystroke during fast browsing.
        let current_idx = self
            .ui_state
            .files
            .iter()
            .position(|f| f.path == path)
            .filter(|_| self.ui_state.files.len() > 8);
        if let Some(current_idx) = current_idx {
            let cache = self.prefetch_cache();
            if cache.entry_count() > 0 {
                let files: Vec<PathBuf> =
                    self.ui_state.files.iter().map(|f| f.path.clone()).collect();
                self.thread_pool().spawn(move || {
                    let len = files.len() as i32;
                    let keep_range = 4; // Maintain a window of 4 images in each direction
                    // Index once: a linear scan per cached entry made this
                    // O(cache x files) string comparisons on every navigation.
                    let index: rustc_hash::FxHashMap<&PathBuf, i32> = files
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (p, i as i32))
                        .collect();
                    let mut to_remove = Vec::new();
                    for entry in cache.iter() {
                        let cached_path = entry.0;
                        if let Some(idx) = index.get(cached_path.as_ref()) {
                            let dist = (idx - current_idx as i32).abs();
                            // Handle wrap-around distance
                            let wrapped_dist = dist.min((len - dist).abs());
                            if wrapped_dist > keep_range {
                                to_remove.push(cached_path);
                            }
                        } else {
                            to_remove.push(cached_path);
                        }
                    }
                    for p in to_remove {
                        cache.invalidate(p.as_ref());
                    }
                });
            }
        }

        let generation = self
            .navigation
            .load_generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        // Compute prefetch targets based on navigation direction (+1, +2, +3 or -1, -2, -3)
        let mut prefetch_targets = Vec::new();
        if self.ui_state.files.len() >= 2
            && let Some(idx) = self.ui_state.files.iter().position(|f| f.path == path)
        {
            let len = self.ui_state.files.len();
            let offsets = if self.navigation.last_direction >= 0 {
                [1isize, 2, 3, -1]
            } else {
                [-1isize, -2, -3, 1]
            };
            for offset in offsets {
                let target_idx = (idx as isize + offset).rem_euclid(len as isize) as usize;
                let target_path = self.ui_state.files[target_idx].path.clone();
                if !prefetch_targets.contains(&target_path) && target_path != path {
                    prefetch_targets.push(target_path);
                }
            }
        }

        let (max_w, max_h) = match (fit, &self.window) {
            (Some(size), _) => size,
            (None, Some(w)) => {
                let size = w.inner_size();
                (size.width, size.height)
            }
            (None, None) => (constants::DEFAULT_MAX_WIDTH, constants::DEFAULT_MAX_HEIGHT),
        };

        let pool = self.thread_pool().clone();
        let prefetch_pool = self.prefetch_pool().clone();
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let current_gen = self.navigation.load_generation.clone();
        let cache = self.prefetch_cache();
        if let Some(cached_frames) = cache.get(path) {
            if let Some(ref proxy) = self.event_proxy {
                send_event(
                    &self.event_tx,
                    proxy,
                    AppEvent::ImageLoaded((*cached_frames).clone()),
                );
            }

            for target_path in prefetch_targets {
                if cache.get(&target_path).is_some() {
                    continue;
                }
                let tx_p = tx.clone();
                let proxy_p = proxy.clone();
                let gen_p = current_gen.clone();
                prefetch_pool.spawn(move || {
                    // Early exit check
                    if gen_p.load(Ordering::Relaxed) != generation {
                        return;
                    }

                    if let Ok(frames) = ImageBackend::load_and_downsample_with(
                        &target_path,
                        max_w,
                        max_h,
                        crate::image::LoadOptions {
                            frames: crate::image::FrameLimit::First,
                            bake_orientation: false,
                        },
                    ) && gen_p.load(Ordering::Relaxed) == generation
                        && let Some(ref p) = proxy_p
                    {
                        send_event(&tx_p, p, AppEvent::Prefetched(target_path, frames));
                    }
                });
            }
            return;
        }

        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        if let Some(ref w) = self.window {
            w.set_title(&format!(
                "SpedImage â€” {}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("SpedImage")
            ));
        }

        let path_owned = path.to_path_buf();
        let prefetch_pool_inner = prefetch_pool.clone();
        let cache_inner = self.prefetch_cache();
        let skip_prefetch = self.mouse_drag_start.is_some();

        pool.spawn(move || {
            // Early exit check
            if current_gen.load(Ordering::Relaxed) != generation {
                return;
            }

            // GIF streams frames asynchronously after frame 0; animated WebP
            // and multi-page TIFF decode all frames directly on primary load.
            let frames_limit = match path_owned
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("webp" | "tif" | "tiff") => crate::image::FrameLimit::All,
                _ => crate::image::FrameLimit::First,
            };

            let result = ImageBackend::load_and_downsample_with(
                &path_owned,
                max_w,
                max_h,
                crate::image::LoadOptions {
                    frames: frames_limit,
                    bake_orientation: false,
                },
            );

            if current_gen.load(Ordering::Relaxed) == generation {
                let event = match result {
                    Ok(data) => AppEvent::ImageLoaded(data),
                    Err(e) => AppEvent::ImageError(e.to_string()),
                };
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, event);
                }

                if skip_prefetch {
                    return;
                }

                // prefetch adjacent images
                for target_path in prefetch_targets {
                    if cache_inner.get(&target_path).is_some() {
                        continue;
                    }
                    let tx_p = tx.clone();
                    let proxy_p = proxy.clone();
                    let gen_p = current_gen.clone();
                    prefetch_pool_inner.spawn(move || {
                        // Early exit check
                        if gen_p.load(Ordering::Relaxed) != generation {
                            return;
                        }

                        if let Ok(frames) = ImageBackend::load_and_downsample_with(
                            &target_path,
                            max_w,
                            max_h,
                            crate::image::LoadOptions {
                                frames: crate::image::FrameLimit::First,
                                bake_orientation: false,
                            },
                        ) && gen_p.load(Ordering::Relaxed) == generation
                            && let Some(ref p) = proxy_p
                        {
                            send_event(&tx_p, p, AppEvent::Prefetched(target_path, frames));
                        }
                    });
                }
            }
        });
    }

    pub(crate) fn next_image(&mut self) {
        self.navigation.last_direction = 1;
        self.ui_state.next_file();
        let path = self.ui_state.current_file().cloned();
        if let Some(p) = path {
            self.load_image(&p);
        }
    }

    pub(crate) fn prev_image(&mut self) {
        self.navigation.last_direction = -1;
        self.ui_state.prev_file();
        let path = self.ui_state.current_file().cloned();
        if let Some(p) = path {
            self.load_image(&p);
        }
    }

    pub(crate) fn first_image(&mut self) {
        if !self.ui_state.files.is_empty() {
            self.navigation.last_direction = -1;
            self.ui_state.current_file_index = Some(0);
            let path = self.ui_state.current_file().cloned();
            if let Some(p) = path {
                self.load_image(&p);
            }
        }
    }

    pub(crate) fn last_image(&mut self) {
        if !self.ui_state.files.is_empty() {
            self.navigation.last_direction = 1;
            self.ui_state.current_file_index = Some(self.ui_state.files.len() - 1);
            let path = self.ui_state.current_file().cloned();
            if let Some(p) = path {
                self.load_image(&p);
            }
        }
    }

    pub(crate) fn zoom_in(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(0.8, cursor);
    }

    pub(crate) fn zoom_out(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(1.25, cursor);
    }

    pub(crate) fn set_crop_target(
        &mut self,
        new_w: f32,
        new_h: f32,
        cursor: Option<PhysicalPosition<f64>>,
    ) {
        // The current image's own view: the right half while comparing.
        let (win_w, win_h) = self.image_view_size();
        let cursor = cursor.map(|p| self.to_image_view(p));

        let img_aspect = self
            .current_image
            .as_ref()
            .map(|img| img.width as f32 / img.height as f32)
            .unwrap_or(1.0);
        let win_aspect = if win_h > 0.0 { win_w / win_h } else { 1.0 };
        let ratio = img_aspect / win_aspect;

        let (nx, ny) = if let Some(pos) = cursor {
            let mut x_ratio = pos.x as f32 / win_w;
            let mut y_ratio = pos.y as f32 / win_h;

            if ratio > 1.0 {
                let img_h_in_win = 1.0 / ratio;
                let offset = (1.0 - img_h_in_win) / 2.0;
                y_ratio = ((y_ratio - offset) * ratio).clamp(0.0, 1.0);
            } else {
                let img_w_in_win = ratio;
                let offset = (1.0 - img_w_in_win) / 2.0;
                x_ratio = ((x_ratio - offset) / ratio).clamp(0.0, 1.0);
            }
            (x_ratio, y_ratio)
        } else {
            (0.5, 0.5)
        };

        let old_w = self.ui_state.adjustments.crop_rect_target[2];
        let old_h = self.ui_state.adjustments.crop_rect_target[3];

        let cx = nx.mul_add(old_w, self.ui_state.adjustments.crop_rect_target[0]);
        let cy = ny.mul_add(old_h, self.ui_state.adjustments.crop_rect_target[1]);

        let target_x = cx - new_w * nx;
        let target_y = cy - new_h * ny;

        let min_x = 0.0f32.min(1.0 - new_w);
        let max_x = 0.0f32.max(1.0 - new_w);
        let min_y = 0.0f32.min(1.0 - new_h);
        let max_y = 0.0f32.max(1.0 - new_h);

        self.ui_state.adjustments.crop_rect_target[0] = target_x.clamp(min_x, max_x);
        self.ui_state.adjustments.crop_rect_target[1] = target_y.clamp(min_y, max_y);
        self.ui_state.adjustments.crop_rect_target[2] = new_w;
        self.ui_state.adjustments.crop_rect_target[3] = new_h;
        // Arm the progressive high-res refinement timer.
        self.zoom_settle_at = Some(std::time::Instant::now());
        self.dirty = true;
    }

    pub(crate) fn zoom_by(&mut self, factor: f32, cursor: Option<PhysicalPosition<f64>>) {
        let old_w = self.ui_state.adjustments.crop_rect_target[2];
        let old_h = self.ui_state.adjustments.crop_rect_target[3];
        // Clamp on the *larger* side and derive the other from it, so the two
        // axes always scale by the same factor. Clamping each axis separately
        // silently stretched the image once one side hit the 5x/0.01x limit.
        let limit = 5.0f32;
        let (new_w, new_h) = if old_w >= old_h {
            let new_w = (old_w * factor).clamp(0.01, limit);
            let ratio = if old_w > 0.0 { old_h / old_w } else { 1.0 };
            (new_w, (new_w * ratio).clamp(0.01, limit))
        } else {
            let new_h = (old_h * factor).clamp(0.01, limit);
            let ratio = if old_h > 0.0 { old_w / old_h } else { 1.0 };
            ((new_h * ratio).clamp(0.01, limit), new_h)
        };
        self.set_crop_target(new_w, new_h, cursor);
    }

    pub(crate) fn zoom_fit(&mut self) {
        self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    pub(crate) fn active_thumb_index(&self) -> Option<usize> {
        let current = self.ui_state.current_file()?;
        self.thumbnails.paths.iter().position(|p| p == current)
    }

    pub(crate) fn start_gif_stream(&mut self, path: PathBuf) {
        // Dropping the old receiver disconnects any previous streamer.
        self.gif_rx = None;
        self.animation.pending.clear();

        let generation = self.navigation.load_generation.load(Ordering::SeqCst);
        let gen_flag = self.navigation.load_generation.clone();
        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let size = w.inner_size();
                (size.width, size.height)
            }
            None => (constants::DEFAULT_MAX_WIDTH, constants::DEFAULT_MAX_HEIGHT),
        };

        let (tx, rx) = crossbeam_channel::bounded(constants::GIF_LOOKAHEAD);
        self.gif_rx = Some(rx);

        self.prefetch_pool().spawn(move || {
            crate::image::stream_gif(path, 1, max_w, max_h, generation, gen_flag, tx);
        });
    }

    /// Fire a full-resolution re-decode once the user settles into a zoom that
    /// magnifies beyond the preview texture's native resolution.
    pub(crate) fn maybe_request_highres(&mut self) {
        if !self.config.refinement_enabled() || self.highres_in_flight {
            return;
        }
        let Some(ref img) = self.current_image else {
            return;
        };
        if !img.is_downsampled {
            return;
        }
        // Animated formats stream their own frames; vectors re-render by scale.
        if matches!(
            img.format,
            crate::image::ImageFormatType::Gif | crate::image::ImageFormatType::Svg
        ) {
            return;
        }
        if self.window.is_none() {
            return;
        }
        let (win_w, _) = self.image_view_size();
        let shown_frac = self.ui_state.adjustments.crop_rect_target[2];
        let screen_px_per_tex_px = shown_frac * win_w / img.width as f32;
        if screen_px_per_tex_px < constants::HIGHRES_TRIGGER_SCALE {
            return;
        }

        let path = img.path.clone();
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.highres_in_flight = true;

        self.prefetch_pool().spawn(move || {
            let result = (|| -> color_eyre::eyre::Result<crate::image::ImageData> {
                let (frames, _) = crate::image::ImageLoader::load(&path, None, None)?;
                frames
                    .into_iter()
                    .next()
                    .ok_or_else(|| color_eyre::eyre::eyre!("No image frames loaded"))
            })();
            let Some(ref p) = proxy else {
                return;
            };
            match result {
                Ok(frame) if (frame.rgba_data.len() as u64) <= constants::HIGHRES_MAX_BYTES => {
                    send_event(&tx, p, AppEvent::HighResReady(path, Box::new(frame)));
                }
                // Without this the in-flight flag stayed set forever, which
                // blocked any later refinement and left the hint spinning.
                _ => send_event(&tx, p, AppEvent::HighResFailed(path)),
            }
        });
    }

    pub(crate) fn toggle_zoom_100(&mut self) {
        if self.ui_state.adjustments.crop_rect_target[2] < 0.99 {
            self.zoom_fit();
        } else {
            self.zoom_100(None);
        }
    }

    pub(crate) fn zoom_100(&mut self, cursor: Option<winit::dpi::PhysicalPosition<f64>>) {
        if let Some(ref img) = self.current_image {
            let (win_w, win_h) = self.image_view_size();

            // Sideways EXIF orientation swaps the displayed dimensions.
            let swapped = matches!(img.orientation_deg, 90 | 270);
            let (img_w, img_h) = if swapped {
                (img.height as f32, img.width as f32)
            } else {
                (img.width as f32, img.height as f32)
            };

            if img_w > 0.0 && img_h > 0.0 {
                let crop_w = win_w / img_w;
                let crop_h = win_h / img_h;
                self.set_crop_target(crop_w, crop_h, cursor);
            }
        }
    }
}
