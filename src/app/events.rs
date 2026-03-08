use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, WakeUp, APP_ICON};
use crate::render::{Renderer, STRIP_HEIGHT_PX};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Icon, Window, WindowId};

impl SpedImageApp {
    pub fn run(initial_path: Option<PathBuf>) -> Result<()> {
        let event_loop: EventLoop<WakeUp> = EventLoop::with_user_event().build()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = Self::new();
        app.event_proxy = Some(event_loop.create_proxy());
        app.initial_path = initial_path;
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    pub(crate) fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::SaveComplete(path) => {
                    self.dirty = true;
                    self.ui_state.set_status(format!(
                        "Saved: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
                AppEvent::SaveError(e) => {
                    self.dirty = true;
                    self.ui_state.set_status(format!("Save failed: {e}"));
                }
                AppEvent::SetStatus(msg) => {
                    self.dirty = true;
                    self.ui_state.set_status(msg);
                }
                AppEvent::Prefetched(path, frames) => {
                    self.navigation.prefetch_cache.push(path, frames);
                }
                AppEvent::ThumbnailLoaded(path, rgba, width, height) => {
                    if let Some(ref mut renderer) = self.renderer {
                        let already_have = renderer.thumbnails.iter().any(|t| t.path == path);
                        if !already_have {
                            if let Err(e) = renderer.upload_thumbnail(path, &rgba, width, height) {
                                tracing::warn!("Failed to upload thumbnail: {e}");
                            } else {
                                self.dirty = true;
                            }
                        }
                    }
                }
                AppEvent::ImageLoaded(mut frames) => {
                    self.ui_state.reset_adjustments();
                    self.dirty = true;
                    if frames.is_empty() {
                        continue;
                    }
                    let mut first_frame = frames.remove(0);
                    let path = first_frame.path.clone();

                    let frame_delays: Vec<u32> = frames.iter().map(|f| f.frame_delay_ms).collect();

                    let new_dir = path.parent().map(|p| p.to_path_buf());
                    let old_dir = self
                        .ui_state
                        .current_file()
                        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

                    if let Some(parent) = path.parent() {
                        self.ui_state.load_directory(parent.to_path_buf());
                    }
                    for (i, f) in self.ui_state.files.iter().enumerate() {
                        if f.path == path {
                            self.ui_state.current_file_index = Some(i);
                            break;
                        }
                    }

                    if new_dir != old_dir || self.thumbnails.paths.is_empty() {
                        self.load_thumbnails_for_dir();
                        if let Some(parent) = path.parent() {
                            self.setup_file_watcher(parent);
                        }
                    }

                    if let Some(ref mut renderer) = self.renderer {
                        if let Err(e) = renderer.load_image(&first_frame) {
                            tracing::error!("Failed to load image to GPU: {e}");
                            self.ui_state.set_status("Failed to load image");
                            self.loading = false;
                            return;
                        }
                        if !frame_delays.is_empty() {
                            if let Err(e) = renderer.preload_gif_textures(&frames) {
                                tracing::warn!("Failed to preload GIF textures: {e}");
                            }
                        } else {
                            for (tex, _) in renderer.gif_textures.drain(..) {
                                tex.destroy();
                            }
                        }
                    }

                    first_frame.rgba_data.clear();
                    first_frame.rgba_data.shrink_to_fit();
                    drop(frames);

                    if !frame_delays.is_empty() && first_frame.frame_delay_ms > 0 {
                        self.animation.next_frame_time = Some(
                            std::time::Instant::now()
                                + std::time::Duration::from_millis(
                                    first_frame.frame_delay_ms as u64,
                                ),
                        );
                    } else if frame_delays.is_empty() {
                        self.animation.next_frame_time = None;
                    }

                    let size_mb = first_frame.file_size_bytes as f64 / 1_048_576.0;
                    let frame_info = if frame_delays.is_empty() {
                        String::new()
                    } else {
                        let len = frame_delays.len() + 1;
                        format!("  |  {len} frames")
                    };

                    let image_count = self.ui_state.files.iter().filter(|f| f.is_image).count();
                    let current_idx = self.ui_state.current_file_index.unwrap_or(0) + 1;

                    self.ui_state.set_status(format!(
                        "{}/{}  |  {}  |  {}×{}  |  {size_mb:.1} MB{}",
                        current_idx,
                        image_count,
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        first_frame.width,
                        first_frame.height,
                        frame_info
                    ));

                    self.current_image = Some(first_frame);
                    self.animation.frame_delays = frame_delays;
                    self.animation.frame_idx = 0;
                    self.loading = false;
                }
                AppEvent::ImageError(e) => {
                    self.dirty = true;
                    tracing::error!("Failed to load image: {e}");
                    self.ui_state.set_status(format!("Error: {e}"));
                    self.loading = false;
                }
                AppEvent::OpenPath(path) => {
                    self.load_image(path);
                }
                AppEvent::FileRenamed(old_path, new_path) => {
                    if let Some(img) = &mut self.current_image {
                        if img.path == old_path {
                            img.path = new_path.clone();
                        }
                    }
                    for file in &mut self.ui_state.files {
                        if file.path == old_path {
                            file.path = new_path.clone();
                            file.name = new_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            break;
                        }
                    }
                    if let Some(frames) = self.navigation.prefetch_cache.pop(&old_path) {
                        self.navigation.prefetch_cache.push(new_path.clone(), frames);
                    }
                    self.ui_state.set_status(format!(
                        "Renamed to {}",
                        new_path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    self.dirty = true;
                }
            }
        }
    }
}

impl ApplicationHandler<WakeUp> for SpedImageApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let icon = image::load_from_memory(APP_ICON).ok().and_then(|img| {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                Icon::from_rgba(rgba.into_raw(), w, h).ok()
            });

            let mut attrs = Window::default_attributes()
                .with_title("SpedImage")
                .with_decorations(true);
            if let Some(icon) = icon {
                attrs = attrs.with_window_icon(Some(icon));
            }
            let window = match event_loop.create_window(attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    tracing::error!("Failed to create window: {e}");
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(window.clone());

            match pollster::block_on(Renderer::new(window.clone())) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                }
                Err(e) => {
                    tracing::error!("Failed to initialize GPU renderer: {e}");
                    let _ = rfd::MessageDialog::new()
                        .set_title("GPU Error")
                        .set_description(format!(
                            "Failed to initialize GPU: {}\n\nThe app will exit.",
                            e
                        ))
                        .show();
                    event_loop.exit();
                    return;
                }
            }
            self.dirty = true;
            if let Some(path) = self.initial_path.take() {
                self.load_image(path);
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: WakeUp) {
        self.dirty = true;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(size);
                    self.dirty = true;
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.update_scale_factor(scale_factor);
                    self.dirty = true;
                }
            }
            WindowEvent::DroppedFile(path) => {
                tracing::info!("File dropped: {:?}", path);
                self.load_image(path);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.handle_keyboard(event, event_loop);
                } else {
                    if let Some(c) = event.logical_key.to_text() {
                        let key = c.to_lowercase().chars().next().unwrap_or(' ');
                        if self.navigation.held_key == Some(key) {
                            self.navigation.held_key = None;
                            self.navigation.last_advance_time = None;
                        }
                    }
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers.ctrl = mods.state().control_key();
                self.modifiers.alt = mods.state().alt_key();
                self.modifiers.shift = mods.state().shift_key();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta, self.last_cursor_pos);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(start) = self.mouse_drag_start {
                    if let Some(ref w) = self.window {
                        let size = w.inner_size();
                        if size.width > 0 && size.height > 0 {
                            let dx = (position.x - start.x) as f32 / size.width as f32;
                            let dy = (position.y - start.y) as f32 / size.height as f32;
                            let rect = &mut self.ui_state.adjustments.crop_rect;
                            rect[0] = (rect[0] - dx * rect[2]).clamp(0.0, 1.0 - rect[2]);
                            rect[1] = (rect[1] - dy * rect[3]).clamp(0.0, 1.0 - rect[3]);
                            self.dirty = true;
                        }
                    }
                    self.mouse_drag_start = Some(position);
                }
                self.last_cursor_pos = position;
            }
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.handle_left_click();
                }
                (MouseButton::Back, ElementState::Released) => {
                    self.prev_image();
                }
                (MouseButton::Forward, ElementState::Released) => {
                    self.next_image();
                }
                (MouseButton::Left, ElementState::Released) => {
                    self.mouse_drag_start = None;
                }
                (MouseButton::Right, ElementState::Released) => {
                    self.mouse_drag_start = None;
                    self.show_context_menu();
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                self.process_events();
                if self.dirty {
                    let status_opt: Option<String> =
                        self.ui_state.status_message.clone().map(|msg| {
                            let mut final_msg = msg;
                            let zoom_pct = (1.0f32 / self.ui_state.adjustments.crop_rect[2] * 100.0)
                                .round() as u32;
                            if zoom_pct != 100 {
                                final_msg = format!("{final_msg}  |  {zoom_pct}%");
                            }
                            if self.slideshow.active {
                                let interval_secs = self.slideshow.interval.as_secs();
                                final_msg = format!("▶ {interval_secs}s  |  {final_msg}");
                            }
                            final_msg
                        });

                    let is_cropping = self.ui_state.is_cropping;
                    let crop_rect = self.ui_state.adjustments.crop_rect;
                    let show_help = self.ui_state.show_help;
                    let show_sidebar = self.ui_state.show_sidebar;
                    let show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
                    let show_info = self.ui_state.show_info;
                    let active_thumb_idx = self.active_thumb_index();

                    if show_info {
                        if let Some(ref mut img) = self.current_image {
                            img.load_exif();
                        }
                    }

                    let exif_text = if show_info {
                        self.current_image
                            .as_ref()
                            .and_then(|img| img.exif_info.as_deref())
                    } else {
                        None
                    };
                    let sidebar_files: Vec<String> = if show_sidebar {
                        self.ui_state.files.iter().map(|f| f.name.clone()).collect()
                    } else {
                        Vec::new()
                    };

                    if let Some(ref mut renderer) = self.renderer {
                        if self.loading {
                            let _ = renderer.render_loading();
                        } else {
                            let _ = renderer.render_frame(
                                &self.ui_state.adjustments,
                                is_cropping,
                                crop_rect,
                                status_opt.as_deref(),
                                show_help,
                                if show_sidebar {
                                    Some(&sidebar_files)
                                } else {
                                    None
                                },
                                show_thumbnail_strip,
                                active_thumb_idx,
                                &self.ui_state.selected_indices,
                                exif_text,
                                self.ui_state.show_histogram,
                                self.current_image
                                    .as_ref()
                                    .and_then(|img| img.histogram.as_ref()),
                            );
                        }
                    }
                    self.dirty = false;
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.process_events();

        let current = self.ui_state.adjustments.crop_rect;
        let target = self.ui_state.adjustments.crop_rect_target;
        let mut animating_zoom = false;

        let diff: f32 = current
            .iter()
            .zip(target.iter())
            .map(|(&c, &t): (&f32, &f32)| (c - t).abs())
            .sum();
        if diff > 0.001 {
            animating_zoom = true;
            for i in 0..4 {
                self.ui_state.adjustments.crop_rect[i] =
                    current[i] + (target[i] - current[i]) * 0.2;
            }
            self.dirty = true;
        } else if diff > 0.0 {
            self.ui_state.adjustments.crop_rect = target;
            self.dirty = true;
        }

        if let Some(next_time) = self.animation.next_frame_time {
            let now = std::time::Instant::now();
            if now >= next_time && !self.animation.frame_delays.is_empty() {
                let total = self.animation.frame_delays.len() + 1;
                self.animation.frame_idx = (self.animation.frame_idx + 1) % total;

                let delay = if self.animation.frame_idx == 0 {
                    if let Some(ref mut renderer) = self.renderer {
                        renderer.swap_gif_frame(0);
                    }
                    self.current_image
                        .as_ref()
                        .map(|f| f.frame_delay_ms)
                        .unwrap_or(100)
                } else {
                    let idx = self.animation.frame_idx;
                    if let Some(ref mut renderer) = self.renderer {
                        renderer.swap_gif_frame(idx);
                    }
                    self.animation.frame_delays
                        .get(idx - 1)
                        .copied()
                        .unwrap_or(100)
                };

                self.animation.next_frame_time =
                    Some(now + std::time::Duration::from_millis(delay.max(10) as u64));
                self.dirty = true;
            }
        }

        let now = std::time::Instant::now();
        if self.slideshow.active {
            if let Some(st) = self.slideshow.next_time {
                if now >= st {
                    self.next_image();
                    self.slideshow.next_time = Some(now + self.slideshow.interval);
                }
            }
        }

        let mut wait_until = None;
        if animating_zoom {
            wait_until = Some(now + std::time::Duration::from_millis(16));
        }
        if let Some(ft) = self.animation.next_frame_time {
            wait_until = Some(wait_until.map_or(ft, |w| w.min(ft)));
        }
        if self.slideshow.active {
            if let Some(st) = self.slideshow.next_time {
                wait_until = Some(wait_until.map_or(st, |w| w.min(st)));
            }
        }

        const HOLD_ADVANCE_INTERVAL_MS: u64 = 150;
        if let (Some(key), Some(last_time)) = (self.navigation.held_key, self.navigation.last_advance_time) {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_time);
            if elapsed.as_millis() >= HOLD_ADVANCE_INTERVAL_MS as u128 {
                match key {
                    'a' | 'w' => self.prev_image(),
                    'd' | 's' => self.next_image(),
                    _ => {}
                }
                self.navigation.last_advance_time = Some(now);
                wait_until = Some(now + std::time::Duration::from_millis(HOLD_ADVANCE_INTERVAL_MS));
            }
        }

        if let Some(wu) = wait_until {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wu));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }

        if self.dirty {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

impl SpedImageApp {
    fn handle_left_click(&mut self) {
        let pos = self.last_cursor_pos;

        if self.ui_state.show_thumbnail_strip {
            if let Some(renderer) = &self.renderer {
                if let Some(thumb_slot) = renderer.thumbnail_index_at(pos.x, pos.y) {
                    self.handle_thumbnail_click(thumb_slot);
                    return;
                }
            }
        }

        if self.modifiers.alt {
            self.pick_color_at(self.last_cursor_pos);
            return;
        }

        if let Some(ref w) = self.window {
            let win_h = w.inner_size().height as f64;
            if self.ui_state.show_thumbnail_strip && pos.y > win_h - STRIP_HEIGHT_PX as f64 {
                return;
            }

            let size = w.inner_size();
            if size.width > 0 {
                let mouse_x_ratio = self.last_cursor_pos.x / size.width as f64;
                if mouse_x_ratio < 0.1 {
                    self.prev_image();
                    return;
                } else if mouse_x_ratio > 0.9 {
                    self.next_image();
                    return;
                }
            }
        }

        if !self.ui_state.is_cropping {
            self.mouse_drag_start = Some(self.last_cursor_pos);
        }
    }

    fn handle_thumbnail_click(&mut self, thumb_slot: usize) {
        let path = match self.thumbnails.paths.get(thumb_slot) {
            Some(p) => p.clone(),
            None => return,
        };
        let file_idx = match self.ui_state.files.iter().position(|f| f.path == path) {
            Some(idx) => idx,
            None => return,
        };

        if self.modifiers.ctrl {
            if self.ui_state.selected_indices.contains(&file_idx) {
                self.ui_state.selected_indices.remove(&file_idx);
            } else {
                self.ui_state.selected_indices.insert(file_idx);
            }
            let sel_count = self.ui_state.selected_indices.len();
            self.ui_state
                .set_status(format!("{} item(s) selected", sel_count));
            self.dirty = true;
        } else {
            self.ui_state.selected_indices.clear();
            self.ui_state.current_file_index = Some(file_idx);
            self.load_image(path);
        }
    }
}
 
#[cfg(test)]
mod tests {
    use super::*;
    use winit::event_loop::EventLoopBuilder;
    use crate::app::state::SpedImageApp;
    use std::path::PathBuf;
 
    #[test]
    fn test_handle_thumbnail_click_navigation() {
        use winit::event_loop::EventLoop;
        let event_loop = EventLoop::<crate::app::types::WakeUp>::with_user_event().build().unwrap();
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut app = SpedImageApp::default_with_proxy(event_loop.create_proxy(), tx);
 
        let path = PathBuf::from("test.jpg");
        app.ui_state.files.push(crate::ui::FileEntry {
            path: path.clone(),
            name: "test.jpg".into(),
            is_image: true,
        });
        app.thumbnails.paths.push(path.clone());
 
        app.handle_thumbnail_click(0);
 
        assert_eq!(app.ui_state.current_file_index, Some(0));
    }
}
