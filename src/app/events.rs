use crate::app::constants;
use crate::app::state::SpedImageApp;
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
        let event_loop = EventLoop::<WakeUp>::with_user_event().build()?;
        let mut app = SpedImageApp::new(event_loop.create_proxy());
        app.initial_path = initial_path;
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    pub(crate) fn process_events(&mut self) {
        let mut count = 0;
        let mut file_list_or_selection_changed = false;
        let mut thumbs_loaded = false;

        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::ImageLoaded(frames) => {
                    self.loading = false;
                    self.animation.transition_start = Some(std::time::Instant::now());
                    self.animation.transition_factor = 0.0;

                    if let Some(first) = frames.first() {
                        let path = first.path.clone();
                        let dir = path.parent().unwrap_or(&path).to_path_buf();
                        if self.ui_state.files.is_empty() || self.file_watcher.is_none() {
                            self.load_directory_async(dir.clone());
                            self.setup_file_watcher(&dir);
                        } else if let Some(idx) =
                            self.ui_state.files.iter().position(|f| f.path == path)
                            && self.ui_state.current_file_index != Some(idx)
                        {
                            self.ui_state.current_file_index = Some(idx);
                            file_list_or_selection_changed = true;
                        }

                        if let Some(ref mut renderer) = self.renderer {
                            renderer.load_image(first).ok();
                            if frames.len() > 1 {
                                renderer.preload_gif_textures(&frames).ok();
                                self.animation.frame_delays =
                                    frames.iter().map(|f| f.frame_delay_ms).collect();
                                self.animation.frame_idx = 0;
                                self.animation.next_frame_time = Some(
                                    std::time::Instant::now()
                                        + std::time::Duration::from_millis(
                                            self.animation.frame_delays[0] as u64,
                                        ),
                                );
                            } else {
                                self.animation.frame_delays.clear();
                                self.animation.next_frame_time = None;
                            }
                        }
                    }
                    self.current_image = frames.into_iter().next();
                    if let Some(ref mut img) = self.current_image {
                        if self.ui_state.show_info {
                            img.load_exif();
                        }
                    }
                    self.dirty = true;
                }
                AppEvent::DirectoryLoaded(_dir, files) => {
                    self.ui_state.files = files;
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
                    self.load_thumbnails_for_dir();
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
                    self.load_image(&path);
                }
                AppEvent::Prefetched(path, frames) => {
                    self.navigation
                        .prefetch_cache
                        .insert(path, Arc::new(frames));
                }
                AppEvent::ThumbnailLoaded(path, rgba, w, h) => {
                    if let Some(ref mut renderer) = self.renderer {
                        renderer.upload_thumbnail(path, &rgba, w, h).ok();
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
                    if let Err(e) = std::fs::remove_file(&path) {
                        self.ui_state.set_status(format!("Delete failed: {}", e));
                    } else {
                        self.current_image = None;
                        let dir = path.parent().unwrap_or(&path).to_path_buf();
                        self.load_directory_async(dir);
                        self.next_image();
                    }
                    self.dirty = true;
                }
                AppEvent::ConfirmBatchDelete(selected) => {
                    for path in &selected {
                        let _ = std::fs::remove_file(path);
                    }
                    self.ui_state.selected_indices.clear();
                    if let Some(current) = self.ui_state.current_file()
                        && let Some(parent) = current.parent()
                    {
                        self.load_directory_async(parent.to_path_buf());
                    } else if let Some(first) = selected.first()
                        && let Some(parent) = first.parent()
                    {
                        self.load_directory_async(parent.to_path_buf());
                    }
                    self.dirty = true;
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
        if let Some(idx) = self
            .renderer
            .as_ref()
            .and_then(|r| r.thumbnail_index_at(pos.x, pos.y, self.navigation.thumb_scroll))
        {
            self.handle_thumbnail_click(idx);
            return;
        }

        if self.ui_state.is_cropping {
            // Handle crop drag start
        } else {
            self.mouse_drag_start = Some(pos);
        }
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
        let icon = (|| -> Option<winit::window::Icon> {
            use std::io::Cursor;
            use zune_core::options::DecoderOptions;
            use zune_image::image::Image;
            let mut img = Image::read(Cursor::new(APP_ICON), DecoderOptions::default()).ok()?;
            img.convert_color(zune_core::colorspace::ColorSpace::RGBA)
                .ok()?;
            let (w, h) = img.dimensions();
            let rgba = img.flatten_to_u8()[0].clone();
            winit::window::Icon::from_rgba(rgba, w as u32, h as u32).ok()
        })();

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("SpedImage")
                        .with_window_icon(icon)
                        .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();
        self.renderer = Some(renderer);

        if let Some(path) = self.initial_path.take() {
            self.load_image(&path);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Special case: if Ctrl is held during a scroll, we want to zoom the image, 
        // not scroll egui UI elements.
        let is_ctrl_scroll = matches!(event, WindowEvent::MouseWheel { .. }) && self.modifiers.ctrl;

        if let Some(ref mut r) = self.renderer {
            if !is_ctrl_scroll {
                let response = r
                    .egui_state
                    .on_window_event(self.window.as_ref().unwrap(), &event);
                if response.consumed {
                    self.dirty = true;
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(ref mut r) = self.renderer {
                    r.resize(size);
                    self.dirty = true;
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
                self.process_events();
                let active_thumb = self.active_thumb_index();

                if let (Some(r), Some(img)) = (&mut self.renderer, &self.current_image) {
                    let sidebar_text = if self.ui_state.show_sidebar {
                        self.ui_state.sidebar_text.as_deref()
                    } else {
                        None
                    };

                    let exif_text = if self.ui_state.show_info {
                        img.exif_info.as_deref()
                    } else {
                        None
                    };

                    let status = self.ui_state.get_status();
                    r.render_frame(RenderParams {
                        adjustments: &self.ui_state.adjustments,
                        is_cropping: self.ui_state.is_cropping,
                        crop_rect: self.ui_state.adjustments.crop_rect,
                        status_text: Some(status),
                        show_help: self.ui_state.show_help,
                        sidebar_text,
                        show_thumbnail_strip: self.ui_state.show_thumbnail_strip,
                        thumb_scroll: self.navigation.thumb_scroll,
                        active_thumb_idx: active_thumb,
                        selected_indices: &self.ui_state.selected_indices,
                        exif_text,
                        show_histogram: self.ui_state.show_histogram,
                        histogram_data: img.histogram.as_ref(),
                        transition_factor: self.animation.transition_factor,
                    })
                    .ok();
                    self.dirty = false;
                } else if let Some(ref mut r) = self.renderer {
                    r.render_loading(self.ui_state.current_file().map(|p| p.as_ref()))
                        .ok();
                }
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
                if let Some(start) = self.mouse_drag_start
                    && let Some(ref r) = self.renderer
                {
                    let dx = (position.x - start.x) as f32 / r.config.width as f32;
                    let dy = (position.y - start.y) as f32 / r.config.height as f32;

                    let new_x = self.ui_state.adjustments.crop_rect[0] - dx;
                    let new_y = self.ui_state.adjustments.crop_rect[1] - dy;

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
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    if state == ElementState::Pressed {
                        self.handle_left_click(self.last_cursor_pos);
                    } else {
                        self.mouse_drag_start = None;
                    }
                } else if button == MouseButton::Right && state == ElementState::Pressed {
                    self.show_context_menu();
                }
            }
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
                update_wakeup(now + std::time::Duration::from_millis(8)); // ~120fps sync
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
            update_wakeup(now + std::time::Duration::from_millis(8));
        }

        if let Some(next) = self.animation.next_frame_time {
            if std::time::Instant::now() >= next {
                self.animation.frame_idx =
                    (self.animation.frame_idx + 1) % self.animation.frame_delays.len();
                if let Some(ref mut r) = self.renderer {
                    r.swap_gif_frame(self.animation.frame_idx);
                }
                let next_time = std::time::Instant::now()
                    + std::time::Duration::from_millis(
                        self.animation.frame_delays[self.animation.frame_idx] as u64,
                    );
                self.animation.next_frame_time = Some(next_time);
                needs_redraw = true;
                update_wakeup(next_time);
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

        let mut is_lerping = false;
        for i in 0..4 {
            let target = self.ui_state.adjustments.crop_rect_target[i];
            let current = &mut self.ui_state.adjustments.crop_rect[i];
            if (*current - target).abs() > 0.001 {
                *current = *current + (target - *current) * constants::LERP_FACTOR;
                needs_redraw = true;
                is_lerping = true;
            } else {
                *current = target;
            }
        }

        if is_lerping {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        } else if let Some(wakeup) = next_wakeup {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wakeup));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }

        if (needs_redraw || self.dirty)
            && let Some(ref w) = self.window
        {
            self.dirty = true;
            w.request_redraw();
        }
    }
}
