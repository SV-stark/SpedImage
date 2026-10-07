use crate::app::clipboard;
use crate::app::constants;
use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, send_event};
use crate::image::ImageBackend;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use winit::dpi::PhysicalPosition;
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::Fullscreen;

impl SpedImageApp {
    pub(crate) fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                if self.ui_state.show_search {
                    self.ui_state.show_search = false;
                    self.dirty = true;
                } else if self.ui_state.is_cropping {
                    self.cancel_crop();
                } else if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                    self.dirty = true;
                } else if self.is_comparing() {
                    self.toggle_compare();
                } else {
                    self.save_config_on_exit();
                    event_loop.exit();
                }
                return;
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.prev_image();
                return;
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.next_image();
                return;
            }
            Key::Named(NamedKey::PageUp) => {
                self.prev_image();
                return;
            }
            Key::Named(NamedKey::PageDown) => {
                self.next_image();
                return;
            }
            Key::Named(NamedKey::Home) => {
                self.first_image();
                return;
            }
            Key::Named(NamedKey::End) => {
                self.last_image();
                return;
            }
            Key::Named(NamedKey::Backspace) => {
                self.prev_image();
                return;
            }
            Key::Named(NamedKey::Tab) => {
                self.ui_state.show_osd = !self.ui_state.show_osd;
                self.config.show_osd = Some(self.ui_state.show_osd);
                self.config.save();
                self.dirty = true;
                return;
            }
            Key::Named(NamedKey::F1) => {
                self.ui_state.show_help = !self.ui_state.show_help;
                self.dirty = true;
                return;
            }
            Key::Named(NamedKey::F2) => {
                self.rename_current_image();
                return;
            }
            Key::Named(NamedKey::F11) => {
                self.toggle_fullscreen();
                return;
            }
            Key::Named(NamedKey::Enter) => {
                self.toggle_zoom_100();
                return;
            }
            Key::Named(NamedKey::Delete) => {
                if self.modifiers.shift && !self.ui_state.selected_indices.is_empty() {
                    self.batch_delete_selected();
                } else {
                    self.delete_current_image();
                }
                return;
            }
            _ => {}
        }

        if let Some(c) = event.logical_key.to_text() {
            let ctrl = self.modifiers.ctrl;
            match c {
                "d" | "D" => {
                    if event.repeat {
                        self.next_image();
                    } else {
                        self.next_image();
                        self.navigation.held_key = Some('d');
                        self.navigation.last_advance_time = Some(std::time::Instant::now());
                    }
                }
                "a" | "A" => {
                    if event.repeat {
                        self.prev_image();
                    } else {
                        self.prev_image();
                        self.navigation.held_key = Some('a');
                        self.navigation.last_advance_time = Some(std::time::Instant::now());
                    }
                }
                "w" | "W" => {
                    if ctrl {
                        self.set_as_wallpaper();
                    } else if event.repeat {
                        self.prev_image();
                    } else {
                        self.prev_image();
                        self.navigation.held_key = Some('w');
                        self.navigation.last_advance_time = Some(std::time::Instant::now());
                    }
                }
                "s" | "S" => {
                    if ctrl && self.modifiers.shift {
                        if self.ui_state.selected_indices.len() > 1 {
                            self.batch_save_selected();
                        } else {
                            self.save_image_as();
                        }
                    } else if ctrl {
                        self.save_image();
                    } else if event.repeat {
                        self.next_image();
                    } else {
                        self.next_image();
                        self.navigation.held_key = Some('s');
                        self.navigation.last_advance_time = Some(std::time::Instant::now());
                    }
                }
                "r" | "R" => self.rotate_image(),
                "o" | "O" => self.open_file_dialog(),
                "f" | "F" if !ctrl => self.toggle_sidebar(),
                "f" | "F" if ctrl => {
                    self.ui_state.show_search = !self.ui_state.show_search;
                    if self.ui_state.show_search {
                        self.ui_state.search_query.clear();
                    }
                    self.dirty = true;
                }
                "t" | "T" => self.toggle_thumbnail_strip(),
                "k" | "K" => self.toggle_compare(),
                "1" => self.reset_adjustments(),
                "z" | "Z" => {
                    self.ui_state.adjustments.pixel_perfect =
                        !self.ui_state.adjustments.pixel_perfect;
                    self.dirty = true;
                }
                "c" | "C" if ctrl && self.modifiers.shift => self.copy_path_to_clipboard(),
                "c" | "C" if ctrl => self.copy_to_clipboard(),
                "v" | "V" if ctrl => self.paste_from_clipboard(),
                "c" | "C" => self.toggle_crop(),
                "h" | "H" => {
                    if self.modifiers.shift {
                        self.ui_state.show_histogram = !self.ui_state.show_histogram;
                        self.config.show_histogram = self.ui_state.show_histogram;
                        self.config.save();
                        if self.ui_state.show_histogram
                            && let Some(ref img) = self.current_image
                            && img.histogram.is_none()
                        {
                            let path = img.path.clone();
                            let rgba = img.rgba_data.clone();
                            let tx = self.event_tx.clone();
                            let proxy = self.event_proxy.clone();
                            // Off the critical path: use the low-priority pool.
                            self.thumbnail_pool().spawn(move || {
                                let mut r_hist = [0u32; 256];
                                let mut g_hist = [0u32; 256];
                                let mut b_hist = [0u32; 256];
                                crate::image::ImageData::compute_rgb_histogram(
                                    &rgba,
                                    &mut r_hist,
                                    &mut g_hist,
                                    &mut b_hist,
                                );
                                if let Some(ref p) = proxy {
                                    send_event(
                                        &tx,
                                        p,
                                        AppEvent::HistogramComputed(
                                            path,
                                            Box::new((r_hist, g_hist, b_hist)),
                                        ),
                                    );
                                }
                            });
                        }
                        self.dirty = true;
                    } else {
                        self.toggle_hdr_toning();
                    }
                }
                "i" | "I" => {
                    self.ui_state.toggle_info();
                    self.config.show_info = self.ui_state.show_info;
                    self.config.save();
                    if self.ui_state.show_info
                        && let Some(ref mut img) = self.current_image
                    {
                        img.load_exif();
                    }
                    self.dirty = true;
                }
                "p" | "P" if ctrl => self.print_image(),
                "+" | "=" => self.zoom_in(None),
                "-" => self.zoom_out(None),
                "0" => self.zoom_fit(),
                " " => self.toggle_slideshow(),
                "[" => self.adjust_slideshow_interval(-1),
                "]" => self.adjust_slideshow_interval(1),
                "?" => {
                    self.ui_state.toggle_help();
                    self.dirty = true;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn toggle_slideshow(&mut self) {
        self.slideshow.active = !self.slideshow.active;
        if self.slideshow.active {
            self.slideshow.next_time = Some(std::time::Instant::now() + self.slideshow.interval);
        } else {
            self.slideshow.next_time = None;
        }
        self.dirty = true;
    }

    pub(crate) fn adjust_slideshow_interval(&mut self, change: i32) {
        let current_secs = self.slideshow.interval.as_secs() as i32;
        let new_secs = (current_secs + change).clamp(
            constants::MIN_SLIDESHOW_INTERVAL_SECS as i32,
            constants::MAX_SLIDESHOW_INTERVAL_SECS as i32,
        ) as u64;
        self.slideshow.interval = std::time::Duration::from_secs(new_secs);
        if self.slideshow.active {
            self.slideshow.next_time = Some(std::time::Instant::now() + self.slideshow.interval);
        }
        self.dirty = true;
    }

    pub(crate) fn handle_mouse_wheel(
        &mut self,
        delta: MouseScrollDelta,
        cursor_pos: PhysicalPosition<f64>,
    ) {
        let y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };

        if y == 0.0 {
            return;
        }

        let scroll_to_zoom = self.config.scroll_to_zoom.unwrap_or(true);

        let do_zoom = if scroll_to_zoom {
            !self.modifiers.ctrl
        } else {
            self.modifiers.ctrl
        };

        if do_zoom {
            let factor = match delta {
                MouseScrollDelta::LineDelta(_, y_lines) => {
                    // Standard scroll wheel: zoom in steps of ~12% per line tick
                    1.0 - y_lines * 0.12
                }
                MouseScrollDelta::PixelDelta(pixel_pos) => {
                    // High-resolution trackpad: zoom proportionally to scroll pixel distance
                    1.0 - (pixel_pos.y as f32 * 0.002)
                }
            };
            let factor = factor.clamp(0.5, 2.0);
            self.zoom_by(factor, Some(cursor_pos));
        } else {
            if y > 0.0 {
                self.prev_image();
            } else {
                self.next_image();
            }
        }
    }

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
                "SpedImage — {}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("SpedImage")
            ));
        }

        let path_owned = path.to_path_buf();
        let prefetch_pool_inner = prefetch_pool.clone();
        let cache_inner = self.prefetch_cache();

        pool.spawn(move || {
            // Early exit check
            if current_gen.load(Ordering::Relaxed) != generation {
                return;
            }

            let result = ImageBackend::load_and_downsample_with(
                &path_owned,
                max_w,
                max_h,
                crate::image::LoadOptions {
                    frames: crate::image::FrameLimit::First,
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

    pub(crate) fn delete_current_image(&mut self) {
        if let Some(ref image) = self.current_image {
            let path = image.path.clone();
            if self.config.confirm_delete_enabled() {
                let tx = self.event_tx.clone();
                let proxy = self.event_proxy.clone();

                self.thread_pool().spawn(move || {
                    let dialog = rfd::AsyncMessageDialog::new()
                        .set_title("Delete Image")
                        .set_description(format!(
                            "Delete {}?",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ))
                        .set_buttons(rfd::MessageButtons::YesNo);

                    pollster::block_on(async move {
                        if dialog.show().await == rfd::MessageDialogResult::Yes
                            && let Some(ref p) = proxy
                        {
                            send_event(&tx, p, AppEvent::ConfirmDelete(path));
                        }
                    });
                });
            } else if let Some(ref p) = self.event_proxy {
                send_event(&self.event_tx, p, AppEvent::ConfirmDelete(path));
            }
        }
    }

    /// Run [`crate::image::save_edited`] on the worker pool and report back.
    fn spawn_save(
        &mut self,
        source: PathBuf,
        output: PathBuf,
        displayed: Option<crate::image::ImageData>,
    ) {
        self.ui_state.set_status("Saving...");
        self.dirty = true;
        let adjustments = self.ui_state.adjustments;
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.thread_pool().spawn(move || {
            let event =
                match crate::image::save_edited(&source, &output, &adjustments, displayed.as_ref())
                {
                    Ok(outcome) => AppEvent::SaveComplete(outcome),
                    Err(e) => AppEvent::SaveError(e.to_string()),
                };
            if let Some(ref p) = proxy {
                send_event(&tx, p, event);
            }
        });
    }

    pub(crate) fn save_image_as(&mut self) {
        let Some(ref image_data) = self.current_image else {
            return;
        };
        let source = image_data.path.clone();
        let displayed = image_data.clone();
        let default_path = crate::image::edited_output_path(&source);
        let default_name = default_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "image_edited.png".to_string());
        let start_dir = source.parent().filter(|d| d.is_dir()).map(PathBuf::from);
        // List the format the default name uses first: on Windows the first
        // filter is the one preselected in the dialog.
        let default_format = crate::image::SaveFormat::from_path(&default_path)
            .unwrap_or(crate::image::SaveFormat::Png);
        let mut formats = crate::image::SaveFormat::ALL.to_vec();
        formats.sort_by_key(|f| *f != default_format);

        let adjustments = self.ui_state.adjustments;
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        // The dialog blocks, so it runs on the pool rather than the UI thread.
        self.thread_pool().spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .set_title("Save Image As...")
                .set_file_name(&default_name);
            if let Some(dir) = start_dir {
                dialog = dialog.set_directory(dir);
            }
            for f in formats {
                dialog = dialog.add_filter(f.label(), f.extensions());
            }
            let Some(output) = dialog.save_file() else {
                return;
            };
            let event =
                match crate::image::save_edited(&source, &output, &adjustments, Some(&displayed)) {
                    Ok(outcome) => AppEvent::SaveComplete(outcome),
                    Err(e) => AppEvent::SaveError(e.to_string()),
                };
            if let Some(ref p) = proxy {
                send_event(&tx, p, event);
            }
        });
    }

    pub(crate) fn save_image(&mut self) {
        let Some(ref image_data) = self.current_image else {
            return;
        };
        // A pasted clipboard image has no file next to which `_edited` could
        // be written, so ask where it should go instead.
        if !image_data.path.is_file() {
            self.save_image_as();
            return;
        }
        let source = image_data.path.clone();
        let output = crate::image::edited_output_path(&source);
        let displayed = image_data.clone();
        self.spawn_save(source, output, Some(displayed));
    }

    pub(crate) fn batch_save_selected(&mut self) {
        let selected: Vec<PathBuf> = self
            .ui_state
            .selected_indices
            .iter()
            .filter_map(|&i| self.ui_state.files.get(i).map(|f| f.path.clone()))
            .collect();

        if selected.is_empty() {
            return;
        }

        let adjustments = self.ui_state.adjustments;
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        self.ui_state
            .set_status(format!("Batch saving {} images...", selected.len()));
        self.dirty = true;

        self.thread_pool().spawn(move || {
            let mut saved_count = 0;
            let mut lossless_count = 0;
            let mut failed_paths = Vec::new();
            for path in &selected {
                let output = crate::image::edited_output_path(path);
                match crate::image::save_edited(path, &output, &adjustments, None) {
                    Ok(outcome) => {
                        saved_count += 1;
                        if outcome.lossless {
                            lossless_count += 1;
                        }
                    }
                    Err(e) => failed_paths.push(format!("{}: {}", path.display(), e)),
                }
            }

            if let Some(ref proxy) = proxy {
                let lossless_note = if lossless_count > 0 {
                    format!(" ({lossless_count} lossless)")
                } else {
                    String::new()
                };
                let msg = if failed_paths.is_empty() {
                    format!("Batch save complete: {saved_count} images{lossless_note}")
                } else {
                    format!(
                        "Saved {} of {}{}. Failures:\n{}",
                        saved_count,
                        selected.len(),
                        lossless_note,
                        failed_paths.join("\n")
                    )
                };
                send_event(&tx, proxy, AppEvent::SetStatus(msg));
            }
        });
    }

    pub(crate) fn batch_delete_selected(&mut self) {
        let selected: Vec<PathBuf> = self
            .ui_state
            .selected_indices
            .iter()
            .filter_map(|&i| self.ui_state.files.get(i).map(|f| f.path.clone()))
            .collect();

        if selected.is_empty() {
            return;
        }

        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        self.thread_pool().spawn(move || {
            let dialog = rfd::AsyncMessageDialog::new()
                .set_title("Delete Images")
                .set_description(format!("Delete {} selected images?", selected.len()))
                .set_buttons(rfd::MessageButtons::YesNo);

            pollster::block_on(async move {
                if dialog.show().await == rfd::MessageDialogResult::Yes
                    && let Some(ref p) = proxy
                {
                    send_event(&tx, p, AppEvent::ConfirmBatchDelete(selected));
                }
            });
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

    pub(crate) fn rotate_image(&mut self) {
        self.ui_state.rotate_90();
        self.dirty = true;
    }

    pub(crate) fn toggle_crop(&mut self) {
        if !self.ui_state.is_cropping && self.is_comparing() {
            // The crop overlay maps the whole window onto the image; with two
            // images on screen that mapping would be wrong.
            self.ui_state
                .set_status("Close the compare view (K) before cropping");
            self.dirty = true;
            return;
        }
        if !self.ui_state.is_cropping {
            // Crop coordinates are texture-space: bake EXIF rotation first so
            // the on-screen rect maps 1:1 onto stored pixels.
            self.ensure_baked_current();
        }
        self.ui_state.is_cropping = !self.ui_state.is_cropping;
        self.crop_drag = None;
        if !self.ui_state.is_cropping {
            // Leaving crop mode clears the preview rect, but must not discard
            // a crop the user already committed via "Crop to View".
            let committed = self.ui_state.adjustments.crop_rect_actual.is_some();
            self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
            self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
            if committed {
                self.ui_state.adjustments.crop_rect_actual = None;
            }
        }
        self.dirty = true;
    }

    /// Bake deferred EXIF orientation into the current image buffer (rare path:
    /// crop mode / clipboard export). Display-only rotation stays free.
    pub(crate) fn ensure_baked_current(&mut self) {
        let Some(ref mut img) = self.current_image else {
            return;
        };
        if img.orientation_deg == 0 {
            return;
        }
        let deg = img.orientation_deg as i32;
        let (data, w, h) =
            crate::image::ImageProcessor::rotate_rgba(&img.rgba_data, img.width, img.height, deg);
        img.rgba_data = Arc::new(data);
        img.width = w;
        img.height = h;
        img.orientation_deg = 0;
        if let Some(ref mut r) = self.renderer
            && let Err(e) = r.load_image(img)
        {
            tracing::warn!("GPU upload failed after baking orientation: {e}");
        }
        self.dirty = true;
    }

    pub(crate) fn toggle_hdr_toning(&mut self) {
        self.ui_state.adjustments.hdr_toning = !self.ui_state.adjustments.hdr_toning;
        self.dirty = true;
    }

    pub(crate) fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.crop_drag = None;
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    pub(crate) fn reset_adjustments(&mut self) {
        self.ui_state.reset_adjustments();
        self.dirty = true;
    }

    pub(crate) fn toggle_sidebar(&mut self) {
        self.ui_state.show_sidebar = !self.ui_state.show_sidebar;
        self.config.show_sidebar = self.ui_state.show_sidebar;
        self.config.save();
        self.dirty = true;
    }

    pub(crate) fn toggle_thumbnail_strip(&mut self) {
        self.ui_state.show_thumbnail_strip = !self.ui_state.show_thumbnail_strip;
        self.config.show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
        self.config.save();
        self.dirty = true;
    }

    pub(crate) fn rename_current_image(&mut self) {
        if let Some(img) = &self.current_image {
            let old_path = img.path.clone();
            let filename = old_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            self.thread_pool().spawn(move || {
                if let Some(new_path) = rfd::FileDialog::new()
                    .set_title("Rename File")
                    .set_file_name(&filename)
                    .save_file()
                {
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        if let Some(ref p) = proxy {
                            send_event(&tx, p, AppEvent::SetStatus(format!("Rename failed: {e}")));
                        }
                    } else if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::FileRenamed(old_path, new_path));
                    }
                }
            });
        }
    }

    pub(crate) fn set_as_wallpaper(&mut self) {
        if let Some(img) = &self.current_image {
            let path = &img.path;
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(path)
            };

            if let Err(e) = wallpaper::set_from_path(&abs_path.to_string_lossy()) {
                self.ui_state
                    .set_status(format!("Failed to set wallpaper: {e}"));
            } else {
                self.ui_state.set_status("Desktop wallpaper set!");
            }
            self.dirty = true;
        }
    }

    pub(crate) fn toggle_fullscreen(&mut self) {
        if let Some(ref w) = self.window {
            let mode = if w.fullscreen().is_some() {
                None
            } else {
                Some(Fullscreen::Borderless(None))
            };
            w.set_fullscreen(mode);
            self.dirty = true;
        }
    }

    pub(crate) fn show_context_menu(&mut self) {
        #[cfg(windows)]
        self.show_context_menu_windows();
    }

    #[cfg(windows)]
    fn show_context_menu_windows(&mut self) {
        if let Some(ref w) = self.window {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, GetCursorPos, MF_STRING, TPM_NONOTIFY, TPM_RETURNCMD,
                TrackPopupMenu,
            };
            use windows::core::PCWSTR;
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

            unsafe {
                let hmenu = CreatePopupMenu().unwrap_or_default();
                if hmenu.is_invalid() {
                    return;
                }

                let items = [
                    "Open in Explorer",
                    "Copy Image (Ctrl+C)",
                    "Copy File Path (Ctrl+Shift+C)",
                    "Save As... (Ctrl+Shift+S)",
                    "Rename (F2)",
                    "Delete (Del)",
                    "Set as Wallpaper (Ctrl+W)",
                ];
                for (i, item) in items.iter().enumerate() {
                    let mut wide: Vec<u16> = std::ffi::OsStr::new(item).encode_wide().collect();
                    wide.push(0);
                    let _ = AppendMenuW(hmenu, MF_STRING, i + 1, PCWSTR(wide.as_ptr()));
                }

                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = GetCursorPos(&mut pt);

                let hwnd = if let Ok(handle) = w.window_handle() {
                    match handle.as_raw() {
                        RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut _),
                        _ => HWND::default(),
                    }
                } else {
                    HWND::default()
                };

                let cmd = TrackPopupMenu(
                    hmenu,
                    TPM_RETURNCMD | TPM_NONOTIFY,
                    pt.x,
                    pt.y,
                    Some(0),
                    hwnd,
                    None,
                );
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyMenu(hmenu);

                match cmd.0 {
                    1 => self.open_in_explorer(),
                    2 => self.copy_to_clipboard(),
                    3 => self.copy_path_to_clipboard(),
                    4 => self.save_image_as(),
                    5 => self.rename_current_image(),
                    6 => self.delete_current_image(),
                    7 => self.set_as_wallpaper(),
                    _ => {}
                }
            }
        }
    }

    #[cfg(windows)]
    pub(crate) fn open_in_explorer(&self) {
        if let Some(img) = &self.current_image {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::core::PCWSTR;
            let arg = format!("/select,\"{}\"", img.path.display());
            let verb: Vec<u16> = std::ffi::OsStr::new("open")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let file: Vec<u16> = std::ffi::OsStr::new("explorer.exe")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let params: Vec<u16> = std::ffi::OsStr::new(&arg)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                let _ = ShellExecuteW(
                    None,
                    PCWSTR(verb.as_ptr()),
                    PCWSTR(file.as_ptr()),
                    PCWSTR(params.as_ptr()),
                    None,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
            }
        }
    }

    pub(crate) fn print_image(&self) {
        #[cfg(windows)]
        if let Some(img) = &self.current_image {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::core::PCWSTR;
            let verb: Vec<u16> = std::ffi::OsStr::new("print")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let file: Vec<u16> = std::ffi::OsStr::new(&img.path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                ShellExecuteW(
                    None,
                    PCWSTR(verb.as_ptr()),
                    PCWSTR(file.as_ptr()),
                    None,
                    None,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
            }
        }
    }

    pub(crate) fn copy_to_clipboard(&mut self) {
        self.ensure_baked_current();
        if let Some(img) = &self.current_image {
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            let rgba = img.rgba_data.clone();
            let (w, h) = (img.width, img.height);
            let path = img.path.clone();

            self.thread_pool().spawn(move || {
                if let Err(e) = clipboard::set_image(&rgba, w, h) {
                    if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::SetStatus(e));
                    }
                    return;
                }

                // Add the source file so Explorer and Office get a real file to
                // paste, not just a bitmap. Best-effort: the image is already on
                // the clipboard and is what `Ctrl+V` reads back.
                #[cfg(windows)]
                if path.exists()
                    && let Err(e) = clipboard::set_file_drop(&[&path])
                {
                    tracing::debug!("Could not add the file drop to the clipboard: {e}");
                }

                if let Some(ref p) = proxy {
                    send_event(
                        &tx,
                        p,
                        AppEvent::SetStatus("Copied image to clipboard".to_string()),
                    );
                }
            });
        }
    }

    pub(crate) fn paste_from_clipboard(&mut self) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        self.thread_pool().spawn(move || {
            let (rgba_data, width, height) = match clipboard::get_image() {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::ImageError(e));
                    }
                    return;
                }
            };

            let image_data = crate::image::ImageData {
                path: PathBuf::from("Clipboard"),
                rgba_data: Arc::new(rgba_data),
                width,
                height,
                format: crate::image::ImageFormatType::Png,
                file_size_bytes: 0,
                frame_delay_ms: 0,
                exif_info: None,
                exif_loaded: true,
                histogram: None,
                is_downsampled: false,
                orientation_deg: 0,
                gps_coords: None,
                color_space: None,
            };

            if let Some(ref p) = proxy {
                send_event(&tx, p, AppEvent::ImageLoaded(vec![image_data]));
                send_event(
                    &tx,
                    p,
                    AppEvent::SetStatus("Pasted image from clipboard".to_string()),
                );
            }
        });
    }

    pub(crate) fn open_file_dialog(&mut self) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.thread_pool().spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", ImageBackend::supported_extensions())
                .pick_file()
                && let Some(ref proxy) = proxy
            {
                send_event(&tx, proxy, AppEvent::OpenPath(path));
            }
        });
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

    pub(crate) fn copy_path_to_clipboard(&mut self) {
        if let Some(ref img) = self.current_image {
            let path_str = img.path.to_string_lossy().into_owned();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            self.thread_pool()
                .spawn(move || match clipboard::set_text(&path_str) {
                    Ok(()) => {
                        if let Some(ref p) = proxy {
                            send_event(
                                &tx,
                                p,
                                AppEvent::SetStatus("Copied file path to clipboard".to_string()),
                            );
                        }
                    }
                    Err(e) => {
                        if let Some(ref p) = proxy {
                            send_event(&tx, p, AppEvent::SetStatus(e));
                        }
                    }
                });
        }
    }

    /// (Re)start the background GIF streamer for `path`, decoding from frame 1.
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

    pub(crate) fn save_config_on_exit(&self) {
        // Start from the in-memory settings: every preference change already
        // lands there, and re-reading the file would re-trigger the damaged-
        // file recovery (and its backup) on exit.
        let mut config = self.config.clone();
        self.store_window_position(&mut config);
        if let Some(ref w) = self.window {
            let size = w.inner_size();
            let scale_factor = w.scale_factor();
            let logical_size = size.to_logical::<f64>(scale_factor);
            config.window_width = logical_size.width.round() as u32;
            config.window_height = logical_size.height.round() as u32;
            config.window_maximized = w.is_maximized();
        }
        config.show_sidebar = self.ui_state.show_sidebar;
        config.show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
        config.show_info = self.ui_state.show_info;
        config.show_histogram = self.ui_state.show_histogram;
        config.show_osd = Some(self.ui_state.show_osd);
        config.save();
    }
}
