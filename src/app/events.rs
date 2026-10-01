use crate::app::constants;
use crate::app::state::{CropDrag, SpedImageApp};
use crate::app::types::{APP_ICON, AppEvent, WakeUp};
use crate::render::{RenderParams, Renderer, STRIP_HEIGHT_PX};
use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

impl SpedImageApp {
    pub fn run(initial_path: Option<PathBuf>) -> Result<()> {
        use winit::event_loop::EventLoop;
        crate::startup::log("before EventLoop::new");
        let event_loop = EventLoop::<WakeUp>::with_user_event().build()?;
        crate::startup::log("after EventLoop::new");
        let mut app = SpedImageApp::new(event_loop.create_proxy());
        app.initial_path = initial_path;

        crate::startup::log("before run_app");
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    pub(crate) fn process_events(&mut self) {
        // Drain arrived GIF frames without over-buffering: leaving messages
        // in the bounded channel keeps the streamer back-pressured.
        if let Some(ref rx) = self.gif_rx {
            use crossbeam_channel::TryRecvError;
            loop {
                if self.animation.pending.len() >= constants::GIF_PENDING_MAX {
                    break;
                }
                match rx.try_recv() {
                    Ok(crate::image::GifStreamMsg::Frame(msg)) => {
                        let n = self.animation.frame_delays.len();
                        if msg.index >= n {
                            self.animation
                                .frame_delays
                                .resize(msg.index + 1, constants::GIF_DEFAULT_FRAME_MS);
                        }
                        self.animation.frame_delays[msg.index] =
                            msg.delay_ms.max(constants::GIF_DEFAULT_FRAME_MS);
                        self.animation.pending.push_back(msg);
                    }
                    Ok(crate::image::GifStreamMsg::Restarted) => {
                        // Next pass begins: park on the last known index so the
                        // tick advances to 0 once its frame arrives.
                        let n = self.animation.frame_delays.len();
                        if n > 0 {
                            self.animation.frame_idx = n - 1;
                        }
                        self.animation.pending.clear();
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.gif_rx = None;
                        break;
                    }
                }
            }
        }

        let mut count = 0;
        let mut file_list_or_selection_changed = false;
        let mut thumbs_loaded = false;

        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::RendererReady(res) => {
                    crate::startup::log("RendererReady event");
                    match res {
                        Ok(renderer) => {
                            let mut r = *renderer;
                            // Catch up to any resize that happened while we were
                            // initializing in the background.
                            if let Some(ref w) = self.window {
                                r.resize(w.inner_size());
                                let scale = w.scale_factor();
                                r.update_scale_factor(scale);
                            }
                            self.renderer = Some(r);
                            // If an image arrived before the GPU, upload it now.
                            if let Some(img) = self.current_image.clone()
                                && let Some(rr) = &mut self.renderer
                                && let Err(e) = rr.load_image(&img)
                            {
                                tracing::warn!("GPU upload failed: {e}");
                                self.ui_state.set_status(format!("Display error: {e}"));
                            }
                            // The window has been visible since `resumed`; now we
                            // can actually paint into it.
                            self.dirty = true;
                            if let Some(ref w) = self.window {
                                w.request_redraw();
                            }
                            crate::startup::log("RendererReady done");
                        }
                        Err(e) => {
                            tracing::error!("GPU init failed: {e}");
                            self.ui_state.set_status(format!("GPU Init Error: {e}"));
                            self.dirty = true;
                        }
                    }
                }
                AppEvent::ImageLoaded(frames) => {
                    crate::startup::log("ImageLoaded event");
                    self.loading = false;
                    self.animation.transition_start = Some(std::time::Instant::now());
                    self.animation.transition_factor = 0.0;
                    self.highres_in_flight = false;

                    if let Some(first) = frames.first() {
                        let path = first.path.clone();
                        let dir = path.parent().unwrap_or(&path).to_path_buf();
                        let dir_changed = self.ui_state.current_dir.as_ref() != Some(&dir);
                        if dir_changed
                            || self.ui_state.files.is_empty()
                            || self.file_watcher.is_none()
                        {
                            self.ui_state.current_dir = Some(dir.clone());
                            self.load_directory_async(dir.clone());
                            self.setup_file_watcher(&dir);
                        } else if let Some(idx) =
                            self.ui_state.files.iter().position(|f| f.path == path)
                            && self.ui_state.current_file_index != Some(idx)
                        {
                            self.ui_state.current_file_index = Some(idx);
                            file_list_or_selection_changed = true;
                        }

                        if let Some(renderer) = &mut self.renderer
                            // Silently ignoring this left a black window with no
                            // explanation for oversized/corrupt images.
                            && let Err(e) = renderer.load_image(first)
                        {
                            tracing::warn!("GPU upload failed: {e}");
                            self.ui_state.set_status(format!("Display error: {e}"));
                        }
                        let path0 = first.path.clone();
                        if first.format == crate::image::ImageFormatType::Gif {
                            // Streaming playback: show frame 0, decode the rest
                            // in the background with bounded look-ahead.
                            self.animation.frame_delays =
                                vec![first.frame_delay_ms.max(constants::GIF_DEFAULT_FRAME_MS)];
                            self.animation.frame_idx = 0;
                            self.animation.next_frame_time = Some(
                                std::time::Instant::now()
                                    + std::time::Duration::from_millis(
                                        self.animation.frame_delays[0] as u64,
                                    ),
                            );
                            self.start_gif_stream(path0);
                        } else {
                            self.animation.frame_delays.clear();
                            self.animation.next_frame_time = None;
                            self.animation.pending.clear();
                            if let Some(ref mut renderer) = self.renderer {
                                renderer.clear_gif_ring();
                            }
                        }
                    }
                    self.current_image = frames.into_iter().next();
                    if let Some(ref mut img) = self.current_image
                        && self.ui_state.show_info
                    {
                        img.load_exif();
                    }
                    self.dirty = true;
                }

                AppEvent::DirectoryLoaded(_dir, files) => {
                    // Selected indices are positions in the old listing, so a
                    // listing that shifted or shrank leaves them pointing at the
                    // wrong files (or out of bounds).
                    let listing_changed = self.ui_state.files.len() != files.len()
                        || self
                            .ui_state
                            .files
                            .iter()
                            .zip(files.iter())
                            .any(|(a, b)| a.path != b.path);
                    self.ui_state.files = files;
                    if listing_changed {
                        self.ui_state.selected_indices = self
                            .ui_state
                            .selected_indices
                            .iter()
                            .copied()
                            .filter(|&i| i < self.ui_state.files.len())
                            .collect();
                    }
                    if let Some(ref img) = self.current_image {
                        if let Some(idx) =
                            self.ui_state.files.iter().position(|f| f.path == img.path)
                        {
                            self.ui_state.current_file_index = Some(idx);
                        }
                    } else if !self.ui_state.files.is_empty() {
                        self.ui_state.current_file_index = Some(0);
                    }
                    file_list_or_selection_changed = true;
                    if self.ui_state.thumbnail_paths_differ() {
                        self.load_thumbnails_for_dir();
                    }
                    self.dirty = true;
                }
                AppEvent::DirectoryError(err) => {
                    self.ui_state.set_status(format!("Error: {}", err));
                    self.dirty = true;
                }
                AppEvent::ImageError(err) => {
                    self.loading = false;
                    self.ui_state.set_status(format!("Error: {}", err));
                    self.dirty = true;
                }
                AppEvent::OpenPath(path) => {
                    if let Some(ref w) = self.window {
                        w.focus_window();
                        #[cfg(windows)]
                        {
                            use windows::Win32::Foundation::HWND;
                            use windows::Win32::UI::WindowsAndMessaging::{
                                BringWindowToTop, IsIconic, SW_RESTORE, SetForegroundWindow,
                                ShowWindow,
                            };
                            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                            if let Ok(handle) = w.window_handle()
                                && let RawWindowHandle::Win32(h) = handle.as_raw()
                            {
                                let hwnd = HWND(h.hwnd.get() as *mut _);
                                unsafe {
                                    if IsIconic(hwnd).as_bool() {
                                        let _ = ShowWindow(hwnd, SW_RESTORE);
                                    }
                                    let _ = BringWindowToTop(hwnd);
                                    let _ = SetForegroundWindow(hwnd);
                                }
                            }
                        }
                    }
                    self.load_image(&path);
                }
                AppEvent::Prefetched(path, frames) => {
                    self.prefetch_cache().insert(path, Arc::new(frames));
                }
                AppEvent::ThumbnailLoaded(path, rgba, w, h, order) => {
                    if let Some(renderer) = &mut self.renderer {
                        if let Err(e) = renderer.upload_thumbnail(path, &rgba, w, h, order) {
                            tracing::debug!("thumbnail upload skipped: {e}");
                        }
                        thumbs_loaded = true;
                    }
                }
                AppEvent::SaveComplete(path) => {
                    self.ui_state.set_status(format!(
                        "Saved: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    self.dirty = true;
                }
                AppEvent::SaveError(err) => {
                    self.ui_state.set_status(format!("Save failed: {}", err));
                    self.dirty = true;
                }
                AppEvent::SetStatus(msg) => {
                    self.ui_state.set_status(msg);
                    self.dirty = true;
                }
                AppEvent::FileRenamed(old, new) => {
                    if let Some(ref mut img) = self.current_image
                        && img.path == old
                    {
                        img.path = new.clone();
                        if let Some(ref w) = self.window {
                            let name = new
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("SpedImage");
                            w.set_title(&format!("SpedImage — {name}"));
                        }
                    }
                    let dir = new.parent().unwrap_or(&new);
                    self.load_directory_async(dir.to_path_buf());
                    self.ui_state.set_status("File renamed");
                    self.dirty = true;
                }
                AppEvent::ConfirmDelete(path) => {
                    let tx = self.event_tx.clone();
                    let proxy = self.event_proxy.clone();
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "file".to_string());
                    let path_for_delete = path.clone();
                    self.thread_pool().spawn(move || {
                        if let Err(e) = trash::delete(&path_for_delete) {
                            if let Some(ref p) = proxy {
                                crate::app::types::send_event(
                                    &tx,
                                    p,
                                    AppEvent::SetStatus(format!("Delete failed: {e}")),
                                );
                            }
                        } else if let Some(ref p) = proxy {
                            crate::app::types::send_event(
                                &tx,
                                p,
                                AppEvent::SetStatus(format!("Moved to Recycle Bin: {file_name}")),
                            );
                        }
                    });
                    if let Some(pos) = self.ui_state.files.iter().position(|f| f.path == path) {
                        self.ui_state.files.remove(pos);
                        // Selections are stored as indices, so every index
                        // after the removed one shifts down by one.
                        self.ui_state.selected_indices = self
                            .ui_state
                            .selected_indices
                            .iter()
                            .filter_map(|&i| match i.cmp(&pos) {
                                std::cmp::Ordering::Less => Some(i),
                                std::cmp::Ordering::Equal => None,
                                std::cmp::Ordering::Greater => Some(i - 1),
                            })
                            .collect();
                        // The thumbnail strip is keyed by the old listing, so a
                        // removal invalidates it; refresh it from the new one.
                        if self.ui_state.thumbnail_paths_differ() {
                            self.load_thumbnails_for_dir();
                        }
                        if self.ui_state.files.is_empty() {
                            self.ui_state.current_file_index = None;
                            self.current_image = None;
                        } else {
                            let next_idx = pos.min(self.ui_state.files.len() - 1);
                            self.ui_state.current_file_index = Some(next_idx);
                            let next_path = self.ui_state.files[next_idx].path.clone();
                            self.load_image(&next_path);
                        }
                        file_list_or_selection_changed = true;
                    } else {
                        self.current_image = None;
                        self.next_image();
                    }
                    self.dirty = true;
                }
                AppEvent::ConfirmBatchDelete(selected) => {
                    let tx = self.event_tx.clone();
                    let proxy = self.event_proxy.clone();
                    self.thread_pool().spawn(move || {
                        let mut failed = Vec::new();
                        for path in &selected {
                            if let Err(e) = trash::delete(path) {
                                failed.push(format!("{}: {}", path.display(), e));
                            }
                        }
                        if let Some(ref p) = proxy {
                            if !failed.is_empty() {
                                crate::app::types::send_event(
                                    &tx,
                                    p,
                                    AppEvent::SetStatus(format!(
                                        "Failed to delete some: {}",
                                        failed.join(", ")
                                    )),
                                );
                            }
                            if let Some(first) = selected.first()
                                && let Some(parent) = first.parent()
                            {
                                crate::app::types::send_event(
                                    &tx,
                                    p,
                                    AppEvent::DirectoryChanged(parent.to_path_buf()),
                                );
                            }
                        }
                    });
                    self.ui_state.selected_indices.clear();
                    self.dirty = true;
                }
                AppEvent::HighResReady(path, frame) => {
                    self.highres_in_flight = false;
                    let is_current = self
                        .current_image
                        .as_ref()
                        .is_some_and(|img| img.path == path);
                    if is_current {
                        let mut new_img = *frame;
                        if let Some(ref old) = self.current_image {
                            // Carry over lazily-computed metadata.
                            new_img.exif_info.clone_from(&old.exif_info);
                            new_img.exif_loaded = old.exif_loaded;
                            new_img.histogram = old.histogram;
                            new_img.gps_coords = old.gps_coords;
                        }
                        if let Some(r) = &mut self.renderer
                            && let Err(e) = r.load_image(&new_img)
                        {
                            tracing::warn!("GPU upload failed: {e}");
                            self.ui_state.set_status(format!("Display error: {e}"));
                        }
                        self.current_image = Some(new_img);
                        self.dirty = true;
                    }
                }
                AppEvent::HistogramComputed(path, histogram) => {
                    if let Some(ref mut img) = self.current_image
                        && img.path == path
                    {
                        img.histogram = Some(*histogram);
                        self.dirty = true;
                    }
                }
                AppEvent::DirectoryChanged(dir) => {
                    self.load_directory_async(dir);
                }
                AppEvent::TriggerOpenFileDialog => {
                    self.open_file_dialog();
                }
            }
            count += 1;
            if count >= constants::MAX_EVENTS_PER_FRAME {
                break;
            }
        }

        if thumbs_loaded {
            self.dirty = true;
        }

        if file_list_or_selection_changed {
            self.recompute_sidebar_text();
        }
    }

    pub(crate) fn recompute_sidebar_text(&mut self) {
        self.ui_state.sidebar_text = Some(
            self.ui_state
                .files
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let prefix = if Some(i) == self.ui_state.current_file_index {
                        "> "
                    } else {
                        "  "
                    };
                    format!("{}{}\n", prefix, f.name)
                })
                .collect(),
        );
    }

    pub(crate) fn handle_left_click(&mut self, pos: winit::dpi::PhysicalPosition<f64>) {
        if let Some(ref r) = self.renderer
            && let Some(slot) = r.thumbnail_index_at(pos.x, pos.y, self.navigation.thumb_scroll)
            && let Some(thumb) = r.thumbnails.get(slot)
        {
            let path = thumb.path.clone();
            if let Some(idx) = self.ui_state.files.iter().position(|f| f.path == path) {
                self.handle_thumbnail_click(idx);
            }
            return;
        }

        if self.ui_state.is_cropping {
            self.begin_crop_drag(pos);
        } else {
            self.mouse_drag_start = Some(pos);
        }
    }

    /// Decide what a press inside crop mode means: grab the resize handle when
    /// the cursor is near the bottom-right corner, otherwise move the rect.
    fn begin_crop_drag(&mut self, pos: winit::dpi::PhysicalPosition<f64>) {
        let Some(rect) = self.crop_screen_rect() else {
            return;
        };
        // egui works in f32 points and winit reports f64 physical pixels; the
        // rect is built from physical pixels, so compare in f64.
        let (px, py) = (pos.x, pos.y);
        let (rx, ry) = (rect.max.x as f64, rect.max.y as f64);
        let handle_radius = 12.0f64;
        let near_handle = (px - rx).abs() <= handle_radius && (py - ry).abs() <= handle_radius;
        let inside = px >= rect.min.x as f64 && px <= rx && py >= rect.min.y as f64 && py <= ry;
        // Pressing inside the selection moves it; the handle and the area
        // outside resize it from the current bottom-right corner.
        self.crop_drag = Some(if near_handle || !inside {
            CropDrag::Resize
        } else {
            CropDrag::Move
        });
        self.mouse_drag_start = Some(pos);
        // A crop press also resets the double-click timer, otherwise the first
        // crop drag can be swallowed as a double-click zoom/fullscreen toggle.
        self.navigation.last_left_click_time = None;
    }

    /// Screen-space rectangle of the current crop, or `None` when there is no
    /// image or the window size is unknown.
    ///
    /// Shares its geometry with the drawing pass so the rect the user drags
    /// and the rect that gets shaded cannot drift apart.
    fn crop_screen_rect(&self) -> Option<egui::Rect> {
        let img = self.current_image.as_ref()?;
        let (win_w, win_h) = self.window.as_ref().map(|w| {
            let s = w.inner_size();
            (s.width, s.height)
        })?;
        crate::render::crop_overlay_rect(
            img.width,
            img.height,
            (win_w, win_h),
            self.ui_state.adjustments.crop_rect,
            self.ui_state.adjustments.rotation + self.ui_state.adjustments.pre_rotation,
        )
    }

    /// Convert a cursor move into a crop-rect update while a crop drag is live.
    fn update_crop_drag(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        let Some(mode) = self.crop_drag else {
            return;
        };
        let Some(start) = self.mouse_drag_start else {
            return;
        };
        let Some(rect) = self.crop_screen_rect() else {
            return;
        };
        // Texture-space delta: divide the pixel delta by the displayed size of
        // the whole image, which is what one unit of crop_rect covers.
        let (disp_w, disp_h) = (rect.width(), rect.height());
        if disp_w <= 0.0 || disp_h <= 0.0 {
            return;
        }
        let dx = ((position.x - start.x) / disp_w as f64) as f32;
        let dy = ((position.y - start.y) / disp_h as f64) as f32;

        let adj = &mut self.ui_state.adjustments;
        const MIN_SIZE: f32 = 0.02;
        match mode {
            CropDrag::Resize => {
                let w = (adj.crop_rect_target[2] + dx).clamp(MIN_SIZE, 1.0);
                let h = (adj.crop_rect_target[3] + dy).clamp(MIN_SIZE, 1.0);
                // Keep the rect inside the image as it grows.
                adj.crop_rect_target[0] = adj.crop_rect_target[0].clamp(0.0, 1.0 - w);
                adj.crop_rect_target[1] = adj.crop_rect_target[1].clamp(0.0, 1.0 - h);
                adj.crop_rect_target[2] = w;
                adj.crop_rect_target[3] = h;
            }
            CropDrag::Move => {
                let (x, y) = (adj.crop_rect_target[0] + dx, adj.crop_rect_target[1] + dy);
                let (w, h) = (adj.crop_rect_target[2], adj.crop_rect_target[3]);
                adj.crop_rect_target[0] = x.clamp(0.0, (1.0 - w).max(0.0));
                adj.crop_rect_target[1] = y.clamp(0.0, (1.0 - h).max(0.0));
            }
        }
        // Commit directly: crop should track the cursor, not lerp behind it.
        adj.crop_rect = adj.crop_rect_target;
        self.mouse_drag_start = Some(position);
        self.dirty = true;
    }

    pub(crate) fn handle_thumbnail_click(&mut self, idx: usize) {
        if let Some(file) = self.ui_state.files.get(idx) {
            let path = file.path.clone();
            self.ui_state.current_file_index = Some(idx);
            self.load_image(&path);
        }
    }
}

impl ApplicationHandler<WakeUp> for SpedImageApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        crate::startup::log("resumed enter");
        if self.window.is_some() {
            return;
        }

        let config = &self.config;

        // Fast path: title already contains the file name so the OS window
        // is discoverable (and benchmarkable) the instant it is created.
        // The heavy GPU init and icon decode run after the window is visible.
        let title = if let Some(ref p) = self.initial_path {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| format!("SpedImage — {n}"))
                .unwrap_or_else(|| "SpedImage".to_string())
        } else {
            "SpedImage".to_string()
        };
        let title_for_log = title.clone();

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(title)
                        .with_visible(true)
                        .with_maximized(config.window_maximized)
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            if config.window_width > 0 {
                                config.window_width as f64
                            } else {
                                1200.0
                            },
                            if config.window_height > 0 {
                                config.window_height as f64
                            } else {
                                800.0
                            },
                        )),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());
        crate::startup::log(&format!("after window creation title={:?}", title_for_log));

        window.set_visible(true);
        crate::startup::log("after set_visible true");

        // Single-instance and IPC setup off the critical path. The window is
        // already visible at ~85 ms — single_instance (~20 ms) and TcpListener
        // (~10 ms) run in the background so the benchmark (and the user) see
        // the window immediately.
        {
            let initial_for_ipc = self.initial_path.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
                crate::startup::log("bg single_instance start");
                let lock = match single_instance::SingleInstance::new("spedimage_app_instance_lock")
                {
                    Ok(l) => l,
                    Err(e) => {
                        crate::startup::log(&format!("single_instance error: {e:?}"));
                        return;
                    }
                };
                if !lock.is_single() {
                    if let Some(p) = initial_for_ipc.as_ref() {
                        use std::io::Write as _;
                        if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:49512") {
                            let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
                            let _ = stream.write_all(abs.to_string_lossy().as_bytes());
                        }
                    }
                    crate::startup::log("bg secondary, exiting");
                    std::process::exit(0);
                }
                crate::startup::log("bg primary, holding lock");
                std::mem::forget(lock);
                // Listener for future secondaries.
                let listener = match std::net::TcpListener::bind("127.0.0.1:49512")
                    .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"))
                {
                    Ok(l) => l,
                    Err(_) => return,
                };
                crate::startup::log("bg TcpListener bound");
                while let Ok((mut stream, _)) = listener.accept() {
                    use std::io::Read;
                    let mut buf = String::new();
                    if stream.read_to_string(&mut buf).is_ok() {
                        let path = PathBuf::from(buf.trim());
                        if path.exists()
                            && let Some(p) = proxy.as_ref()
                        {
                            crate::app::types::send_event(
                                &tx,
                                p,
                                crate::app::types::AppEvent::OpenPath(path),
                            );
                        }
                    }
                }
            });
        }
        // Diagnostic visibility poll — only when startup tracing is enabled.
        if std::env::var_os("SPEDIMAGE_STARTUP_LOG").is_some() {
            let w2 = window.clone();
            std::thread::spawn(move || {
                use winit::raw_window_handle::HasWindowHandle;
                use winit::raw_window_handle::RawWindowHandle;
                let hwnd = match w2.window_handle().map(|h| h.as_raw()) {
                    Ok(RawWindowHandle::Win32(h)) => {
                        windows::Win32::Foundation::HWND(h.hwnd.get() as *mut _)
                    }
                    _ => return,
                };
                let start = std::time::Instant::now();
                loop {
                    let vis = unsafe {
                        windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd).as_bool()
                    };
                    if vis {
                        crate::startup::log(&format!(
                            "IsWindowVisible true after {}ms",
                            start.elapsed().as_millis()
                        ));
                        break;
                    }
                    if start.elapsed().as_millis() > 3000 {
                        crate::startup::log("IsWindowVisible still false after 3000ms");
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            });
        }

        // Apply config to UI state immediately so the first frame is correct
        // even before the GPU is ready.
        self.ui_state.show_sidebar = config.show_sidebar;
        self.ui_state.show_thumbnail_strip = config.show_thumbnail_strip;
        self.ui_state.show_info = config.show_info;
        self.ui_state.show_histogram = config.show_histogram;

        // Make the window visible right away; the benchmark (and the user)
        // see it long before the GPU is warm.
        window.request_redraw();
        crate::startup::log("after window request_redraw");

        // Decode the window icon off the critical path and apply it when ready.
        {
            let w = window.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
                let icon = (|| -> Option<winit::window::Icon> {
                    use std::io::Cursor;
                    use zune_core::options::DecoderOptions;
                    use zune_image::image::Image;
                    let mut img =
                        Image::read(Cursor::new(APP_ICON), DecoderOptions::default()).ok()?;
                    img.convert_color(zune_core::colorspace::ColorSpace::RGBA)
                        .ok()?;
                    let (wi, hi) = img.dimensions();
                    let rgba = img.flatten_to_u8().swap_remove(0);
                    winit::window::Icon::from_rgba(rgba, wi as u32, hi as u32).ok()
                })();
                if let Some(icon) = icon {
                    w.set_window_icon(Some(icon));
                    // Wake the event loop so the icon is presented promptly.
                    if let Some(p) = proxy.as_ref() {
                        let _ = p.send_event(WakeUp);
                    }
                    let _ = tx.send(AppEvent::SetStatus(String::new()));
                }
            });
        }

        // Bring the GPU up in the background so the window stays responsive.
        // Image decode is kicked off in parallel (below) and does not wait
        // for the GPU — whichever finishes first, the other consumes it.
        {
            let w = window.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            std::thread::spawn(move || {
                let res = pollster::block_on(Renderer::new(w));
                let evt = match res {
                    Ok(r) => AppEvent::RendererReady(Ok(Box::new(r))),
                    Err(e) => AppEvent::RendererReady(Err(e.to_string())),
                };
                let _ = tx.send(evt);
                if let Some(p) = proxy {
                    let _ = p.send_event(WakeUp);
                }
            });
        }

        // Kick off the first image decode *now* in parallel with the GPU.
        // If the GPU is not yet ready, `ImageLoaded` will stash the image and
        // `RendererReady` will upload it.
        if let Some(path) = self.initial_path.take() {
            self.load_image(&path);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Special case: if Ctrl is held during a scroll, we want to zoom the image,
        // not scroll egui UI elements.
        let is_ctrl_scroll = matches!(event, WindowEvent::MouseWheel { .. }) && self.modifiers.ctrl;

        if let Some(ref mut r) = self.renderer
            && !is_ctrl_scroll
        {
            let response = r
                .egui_state
                .on_window_event(self.window.as_ref().unwrap(), &event);
            if response.consumed {
                self.dirty = true;
                return;
            }
        }
        match event {
            WindowEvent::CloseRequested => {
                self.save_config_on_exit();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut r) = self.renderer {
                    // winit emits this on every move/minimize transition even
                    // when the size is unchanged; reconfiguring the swapchain
                    // each time forces a full surface recreate.
                    if size.width != r.config.width || size.height != r.config.height {
                        r.resize(size);
                        self.dirty = true;
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(ref mut r) = self.renderer {
                    r.update_scale_factor(scale_factor);
                    self.dirty = true;
                }
            }
            WindowEvent::DroppedFile(path) => {
                self.load_image(&path);
            }
            WindowEvent::RedrawRequested => {
                {
                    static FIRST: std::sync::Once = std::sync::Once::new();
                    FIRST.call_once(|| crate::startup::log("first RedrawRequested"));
                }
                self.process_events();
                let active_thumb = self.active_thumb_index();

                if let Some(ref img) = self.current_image {
                    self.ui_state.adjustments.color_space = img.color_space;
                    self.ui_state.adjustments.pre_rotation =
                        (img.orientation_deg as f32).to_radians();
                } else {
                    self.ui_state.adjustments.color_space = None;
                    self.ui_state.adjustments.pre_rotation = 0.0;
                }

                if let Some(r) = &mut self.renderer {
                    // Lazy mipmaps: when the image is displayed minified well
                    // below 50%, build a mip chain once so trilinear sampling
                    // replaces shimmering point-minification.
                    if !r.image_mipmapped
                        && self.animation.frame_delays.is_empty()
                        && self.ui_state.adjustments.crop_rect[2] < 0.5
                        && let Some(ref img) = self.current_image
                    {
                        let tex_w = r.image_size.map(|(w, _)| w as f32).unwrap_or(1.0);
                        let shown_frac = self.ui_state.adjustments.crop_rect[2];
                        let win_w = r.config.width as f32;
                        if tex_w > 1.0 && (shown_frac * win_w / tex_w) < 0.5 {
                            r.ensure_mipmapped(img).ok();
                            self.dirty = true;
                        }
                    }

                    let sidebar_text = if self.ui_state.show_sidebar {
                        self.ui_state.sidebar_text.as_deref()
                    } else {
                        None
                    };

                    let exif_text = if self.ui_state.show_info
                        && let Some(ref img) = self.current_image
                    {
                        img.exif_info.as_deref()
                    } else {
                        None
                    };

                    let histogram_data = self
                        .current_image
                        .as_ref()
                        .and_then(|img| img.histogram.as_ref());
                    let transition_factor = self.animation.transition_factor;
                    let has_image = self.current_image.is_some();
                    let is_loading = self.loading;
                    let gps_coords = self.current_image.as_ref().and_then(|img| img.gps_coords);

                    let current_image_info = self.current_image.as_ref().map(|img| {
                        let name = img
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Image".to_string());
                        let zoom_pct = if self.ui_state.adjustments.crop_rect[2] > 0.0 {
                            (1.0 / self.ui_state.adjustments.crop_rect[2]) * 100.0
                        } else {
                            100.0
                        };
                        (name, img.width, img.height, img.file_size_bytes, zoom_pct)
                    });

                    // Borrow just the status field, and skip the per-frame
                    // `String` allocation the old `to_string()` did.
                    let status_text = crate::ui::UiState::status_of(&self.ui_state.status_message);
                    let is_cropping = self.ui_state.is_cropping;
                    let crop_rect = self.ui_state.adjustments.crop_rect;
                    let show_help = self.ui_state.show_help;
                    let show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
                    let show_histogram = self.ui_state.show_histogram;
                    let show_osd = self.ui_state.show_osd && self.config.osd_enabled();
                    let mut slideshow_interval_secs = self.slideshow.interval.as_secs();
                    let old_slideshow_active = self.slideshow.active;
                    let slideshow_progress = if self.slideshow.active
                        && let Some(next) = self.slideshow.next_time
                    {
                        let now = std::time::Instant::now();
                        if next > now {
                            let remaining = next.duration_since(now).as_secs_f32();
                            let total = self.slideshow.interval.as_secs_f32();
                            // A zero interval would divide by zero and hand NaN
                            // to the progress bar.
                            (total > 0.0).then(|| ((total - remaining) / total).clamp(0.0, 1.0))
                        } else {
                            Some(1.0)
                        }
                    } else {
                        None
                    };

                    if let Some(ref proxy) = self.event_proxy
                        && let Err(e) = r.render_frame(RenderParams {
                            adjustments: &mut self.ui_state.adjustments,
                            is_cropping,
                            crop_rect,
                            status_text,
                            window_size: (r.config.width, r.config.height),
                            show_help,
                            sidebar_text,
                            show_thumbnail_strip,
                            thumb_scroll: self.navigation.thumb_scroll,
                            active_thumb_idx: active_thumb,
                            selected_indices: &self.ui_state.selected_indices,
                            exif_text,
                            show_histogram,
                            histogram_data,
                            transition_factor,
                            files: &self.ui_state.files,
                            event_tx: &self.event_tx,
                            event_proxy: proxy,
                            is_loading,
                            has_image,
                            config: &mut self.config,
                            slideshow_active: &mut self.slideshow.active,
                            slideshow_interval_secs: &mut slideshow_interval_secs,
                            slideshow_progress,
                            show_search: &mut self.ui_state.show_search,
                            search_query: &mut self.ui_state.search_query,
                            gps_coords,
                            current_image_info,
                            show_osd,
                        })
                    {
                        tracing::warn!("render frame error: {e}");
                        if let Some(ref win) = self.window {
                            r.resize(win.inner_size());
                        }
                    }

                    if self.slideshow.active != old_slideshow_active {
                        if self.slideshow.active {
                            self.slideshow.next_time =
                                Some(std::time::Instant::now() + self.slideshow.interval);
                        } else {
                            self.slideshow.next_time = None;
                        }
                        self.dirty = true;
                    }
                    if slideshow_interval_secs != self.slideshow.interval.as_secs() {
                        self.slideshow.interval =
                            std::time::Duration::from_secs(slideshow_interval_secs.max(1));
                        if self.slideshow.active {
                            self.slideshow.next_time =
                                Some(std::time::Instant::now() + self.slideshow.interval);
                        }
                        self.dirty = true;
                    }
                }

                // Clear outside the `renderer.is_some()` block: when GPU init
                // failed, `dirty` stayed set forever and `about_to_wait`
                // re-requested a redraw every loop — a 100% CPU spin.
                self.dirty = false;
            }
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers.ctrl = m.state().control_key();
                self.modifiers.shift = m.state().shift_key();
                self.modifiers.alt = m.state().alt_key();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.handle_keyboard(event, event_loop);
                } else {
                    self.navigation.held_key = None;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor_pos = position;
                if self.crop_drag.is_some() {
                    self.update_crop_drag(position);
                } else if let Some(start) = self.mouse_drag_start
                    && let Some(ref r) = self.renderer
                {
                    let dx = (position.x - start.x) as f32 / r.config.width as f32;
                    let dy = (position.y - start.y) as f32 / r.config.height as f32;

                    // Rotate delta by total rotation and flipping so panning tracks cursor intuitively
                    let rot =
                        self.ui_state.adjustments.rotation + self.ui_state.adjustments.pre_rotation;
                    let cos_a = rot.cos();
                    let sin_a = rot.sin();

                    let mut tex_dx = dx;
                    let mut tex_dy = dy;
                    if self.ui_state.adjustments.flip_horizontal {
                        tex_dx = -tex_dx;
                    }
                    if self.ui_state.adjustments.flip_vertical {
                        tex_dy = -tex_dy;
                    }

                    let r_dx = tex_dx * cos_a + tex_dy * sin_a;
                    let r_dy = -tex_dx * sin_a + tex_dy * cos_a;

                    let new_x = self.ui_state.adjustments.crop_rect[0] - r_dx;
                    let new_y = self.ui_state.adjustments.crop_rect[1] - r_dy;

                    let min_x = 0.0f32.min(1.0 - self.ui_state.adjustments.crop_rect[2]);
                    let max_x = 0.0f32.max(1.0 - self.ui_state.adjustments.crop_rect[2]);
                    let min_y = 0.0f32.min(1.0 - self.ui_state.adjustments.crop_rect[3]);
                    let max_y = 0.0f32.max(1.0 - self.ui_state.adjustments.crop_rect[3]);

                    self.ui_state.adjustments.crop_rect[0] = new_x.clamp(min_x, max_x);
                    self.ui_state.adjustments.crop_rect[1] = new_y.clamp(min_y, max_y);
                    self.ui_state.adjustments.crop_rect_target =
                        self.ui_state.adjustments.crop_rect;
                    self.mouse_drag_start = Some(position);
                    self.dirty = true;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => {
                    if state == ElementState::Pressed {
                        let now = std::time::Instant::now();
                        let is_double_click =
                            if let Some(last) = self.navigation.last_left_click_time {
                                now.duration_since(last).as_millis() < 300
                            } else {
                                false
                            };
                        self.navigation.last_left_click_time = Some(now);

                        if is_double_click {
                            if self.config.double_click_zoom_enabled() {
                                self.toggle_zoom_100();
                            } else {
                                self.toggle_fullscreen();
                            }
                        } else {
                            self.handle_left_click(self.last_cursor_pos);
                        }
                    } else {
                        self.mouse_drag_start = None;
                        self.crop_drag = None;
                    }
                }
                MouseButton::Right if state == ElementState::Pressed => {
                    self.show_context_menu();
                }
                MouseButton::Back if state == ElementState::Pressed => {
                    self.prev_image();
                }
                MouseButton::Forward if state == ElementState::Pressed => {
                    self.next_image();
                }
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let win_h = self
                    .renderer
                    .as_ref()
                    .map(|r| r.config.height as f64)
                    .unwrap_or(0.0);
                let strip_y = win_h - STRIP_HEIGHT_PX as f64;

                if self.ui_state.show_thumbnail_strip && self.last_cursor_pos.y >= strip_y {
                    let d = match delta {
                        MouseScrollDelta::LineDelta(x, y) => (if x != 0.0 { x } else { -y }) * 50.0,
                        MouseScrollDelta::PixelDelta(p) => {
                            if p.x != 0.0 {
                                p.x as f32
                            } else {
                                -p.y as f32
                            }
                        }
                    };
                    let _max_scroll = if let Some(ref r) = self.renderer {
                        (self.thumbnails.paths.len() as f32 * crate::render::THUMB_SLOT_W as f32
                            - r.config.width as f32)
                            .max(0.0)
                    } else {
                        0.0
                    };
                    self.navigation.thumb_velocity += d * 0.5;
                    self.dirty = true;
                } else {
                    self.handle_mouse_wheel(delta, self.last_cursor_pos);
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: WakeUp) {
        self.process_events();
        if self.dirty
            && let Some(ref w) = self.window
        {
            w.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut needs_redraw = false;
        let mut next_wakeup: Option<std::time::Instant> = None;
        let now = std::time::Instant::now();

        let mut update_wakeup = |time: std::time::Instant| {
            next_wakeup = Some(next_wakeup.map_or(time, |t| t.min(time)));
        };

        // Handle Image Transition
        if let Some(start) = self.animation.transition_start {
            let elapsed = now.duration_since(start).as_millis() as f32;
            let duration = constants::TRANSITION_DURATION_MS;
            if elapsed < duration {
                self.animation.transition_factor = elapsed / duration;
                needs_redraw = true;
            } else {
                self.animation.transition_factor = 1.0;
                self.animation.transition_start = None;
            }
        }

        // Handle Momentum Scrolling
        if self.navigation.thumb_velocity.abs() > 0.1 {
            self.navigation.thumb_scroll += self.navigation.thumb_velocity;
            self.navigation.thumb_velocity *= constants::SCROLL_FRICTION;

            let max_scroll = if let Some(ref r) = self.renderer {
                (self.thumbnails.paths.len() as f32 * crate::render::THUMB_SLOT_W as f32
                    - r.config.width as f32)
                    .max(0.0)
            } else {
                0.0
            };
            self.navigation.thumb_scroll = self.navigation.thumb_scroll.clamp(0.0, max_scroll);
            needs_redraw = true;
        }

        if !self.animation.frame_delays.is_empty()
            && let Some(next) = self.animation.next_frame_time
        {
            if std::time::Instant::now() >= next {
                let n = self.animation.frame_delays.len();
                let candidate = (self.animation.frame_idx + 1) % n;
                // Advance only when the frame has been decoded and uploaded;
                // otherwise hold the current frame and retry shortly.
                if let Some(pos) = self
                    .animation
                    .pending
                    .iter()
                    .position(|m| m.index == candidate)
                {
                    let msg = self.animation.pending.remove(pos).expect("checked");
                    if let Some(ref mut r) = self.renderer
                        && r.upload_gif_frame(msg.index, msg.width, msg.height, &msg.data)
                            .is_ok()
                        && !r.swap_gif_frame(msg.index)
                    {
                        tracing::warn!("GIF ring swap failed for frame {}", msg.index);
                    }
                    self.animation.frame_idx = candidate;
                    let next_time = std::time::Instant::now()
                        + std::time::Duration::from_millis(
                            self.animation.frame_delays[candidate] as u64,
                        );
                    self.animation.next_frame_time = Some(next_time);
                    needs_redraw = true;
                    update_wakeup(next_time);
                } else {
                    let retry = std::time::Instant::now() + std::time::Duration::from_millis(8);
                    self.animation.next_frame_time = Some(retry);
                    update_wakeup(retry);
                }
            } else {
                update_wakeup(next);
            }
        }

        if self.slideshow.active
            && let Some(next) = self.slideshow.next_time
        {
            if std::time::Instant::now() >= next {
                self.next_image();
                let next_time = std::time::Instant::now() + self.slideshow.interval;
                self.slideshow.next_time = Some(next_time);
                needs_redraw = true;
                update_wakeup(next_time);
            } else {
                update_wakeup(next);
            }
        }

        // Progressive high-res refinement after zoom settles
        if let Some(t) = self.zoom_settle_at
            && t.elapsed() >= std::time::Duration::from_millis(constants::HIGHRES_SETTLE_MS)
        {
            self.zoom_settle_at = None;
            self.maybe_request_highres();
        }

        if let Some(c) = self.navigation.held_key
            && let Some(last) = self.navigation.last_advance_time
        {
            let next = last + constants::KEY_REPEAT_DELAY;
            if std::time::Instant::now() >= next {
                match c {
                    'd' | 's' => self.next_image(),
                    'a' | 'w' => self.prev_image(),
                    _ => {}
                }
                self.navigation.last_advance_time = Some(std::time::Instant::now());
                needs_redraw = true;
                update_wakeup(std::time::Instant::now() + constants::KEY_REPEAT_DELAY);
            } else {
                update_wakeup(next);
            }
        }

        for i in 0..4 {
            let target = self.ui_state.adjustments.crop_rect_target[i];
            let current = &mut self.ui_state.adjustments.crop_rect[i];
            if (*current - target).abs() > 0.001 {
                *current = *current + (target - *current) * constants::LERP_FACTOR;
                needs_redraw = true;
            } else {
                *current = target;
            }
        }

        // While any animation is still interpolating we must keep getting
        // callbacks; `ControlFlow::Wait` would park the loop and freeze the
        // zoom/transition on its current frame.
        let animating = self.animation.transition_start.is_some()
            || self.ui_state.adjustments.crop_rect != self.ui_state.adjustments.crop_rect_target
            || self.navigation.thumb_velocity.abs() > 0.1;

        if animating {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        } else if let Some(wakeup) = next_wakeup {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wakeup));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }

        if (needs_redraw || self.dirty)
            && let Some(ref w) = self.window
        {
            w.request_redraw();
        }
    }
}
