use crate::app::state::SpedImageApp;
use crate::app::types::{send_event, AppEvent};
use crate::image::ImageBackend;
use crate::render::STRIP_HEIGHT_PX;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use winit::dpi::PhysicalPosition;
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Fullscreen;

impl SpedImageApp {
    pub(crate) fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        // Named keys (non-text)
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                if self.ui_state.is_cropping {
                    self.cancel_crop();
                } else if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                    self.dirty = true;
                } else {
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
                if let Some(ref w) = self.window {
                    let mode = if w.fullscreen().is_some() {
                        None
                    } else {
                        Some(Fullscreen::Borderless(None))
                    };
                    w.set_fullscreen(mode);
                }
                return;
            }
            Key::Named(NamedKey::Delete) => {
                if self.modifiers.shift && !self.ui_state.selected_indices.is_empty() {
                    // Shift+Delete: batch delete selected
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
                        // Ctrl+Shift+S: batch save selected
                        self.batch_save_selected();
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
                "f" | "F" => self.toggle_sidebar(),
                "t" | "T" => self.toggle_thumbnail_strip(),
                "1" => self.reset_adjustments(),
                "z" | "Z" => {
                    self.ui_state.adjustments.pixel_perfect =
                        !self.ui_state.adjustments.pixel_perfect;
                    let state = if self.ui_state.adjustments.pixel_perfect {
                        "ON"
                    } else {
                        "OFF"
                    };
                    self.ui_state
                        .set_status(format!("Pixel-Perfect Zoom: {state}"));
                    self.dirty = true;
                }
                "c" | "C" if ctrl => self.copy_to_clipboard(),
                "c" | "C" => self.toggle_crop(),
                "h" | "H" => {
                    if self.modifiers.alt {
                        // do nothing; alt+h is reserved
                    } else if self.modifiers.shift {
                        self.ui_state.show_histogram = !self.ui_state.show_histogram;
                        let state = if self.ui_state.show_histogram {
                            "ON"
                        } else {
                            "OFF"
                        };
                        // Lazy compute histogram when turning on
                        if self.ui_state.show_histogram {
                            if let Some(ref mut img) = self.current_image {
                                img.compute_histogram();
                            }
                        }
                        self.ui_state.set_status(format!("Histogram: {state}"));
                        self.dirty = true;
                    } else {
                        self.toggle_hdr_toning();
                    }
                }
                "i" | "I" => {
                    self.ui_state.toggle_info();
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
            self.ui_state.set_status(format!(
                "Slideshow started ({}s per image)",
                self.slideshow.interval.as_secs()
            ));
        } else {
            self.slideshow.next_time = None;
            self.ui_state.set_status("Slideshow paused");
        }
        self.dirty = true;
    }

    pub(crate) fn adjust_slideshow_interval(&mut self, change: i32) {
        let current_secs = self.slideshow.interval.as_secs() as i32;
        let new_secs = (current_secs + change).clamp(1, 120) as u64;
        self.slideshow.interval = std::time::Duration::from_secs(new_secs);
        if self.slideshow.active {
            self.slideshow.next_time = Some(std::time::Instant::now() + self.slideshow.interval);
            self.ui_state
                .set_status(format!("Slideshow interval: {new_secs}s"));
        } else {
            self.ui_state
                .set_status(format!("Slideshow interval configured to {new_secs}s"));
        }
        self.dirty = true;
    }

    pub(crate) fn handle_mouse_wheel(
        &mut self,
        delta: MouseScrollDelta,
        cursor_pos: PhysicalPosition<f64>,
    ) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.zoom_in(Some(cursor_pos));
                } else if y < 0.0 {
                    self.zoom_out(Some(cursor_pos));
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                if pos.y > 0.0 {
                    self.zoom_in(Some(cursor_pos));
                } else if pos.y < 0.0 {
                    self.zoom_out(Some(cursor_pos));
                }
            }
        }
    }

    pub(crate) fn load_image(&mut self, path: PathBuf) {
        // (17) Load generation to prevent race conditions
        let generation = self
            .navigation
            .load_generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        // Check prefetch cache first (LRU cache handles eviction automatically)
        if let Some(cached_frames) = self.navigation.prefetch_cache.pop(&path) {
            tracing::info!("Cache hit for {:?}", path);
            if let Some(ref proxy) = self.event_proxy {
                send_event(&self.event_tx, proxy, AppEvent::ImageLoaded(cached_frames));
            }
            self.prefetch_adjacent(&path);
            return;
        }

        tracing::info!("Loading image: {:?}", path);
        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        if let Some(ref w) = self.window {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("SpedImage");
            w.set_title(&format!("SpedImage — {name}"));
        }

        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let size = w.inner_size();
                (size.width, size.height)
            }
            None => (3840, 2160),
        };

        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let path2 = path.clone();
        let current_gen = self.navigation.load_generation.clone();

        std::thread::spawn(move || {
            let result = ImageBackend::load_and_downsample(&path2, max_w, max_h);

            // Only send if this is still the current generation
            if current_gen.load(Ordering::SeqCst) == generation {
                let event = match result {
                    Ok(data) => AppEvent::ImageLoaded(data),
                    Err(e) => AppEvent::ImageError(e.to_string()),
                };
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, event);
                }
            }
        });

        self.prefetch_adjacent(&path);
    }

    pub(crate) fn delete_current_image(&mut self) {
        if let Some(ref image) = self.current_image {
            let path = image.path.clone();
            let confirmed = rfd::MessageDialog::new()
                .set_title("Delete Image")
                .set_description(format!(
                    "Delete {}?",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes;

            if confirmed {
                if let Err(e) = std::fs::remove_file(&path) {
                    self.ui_state.set_status(format!("Delete failed: {}", e));
                } else {
                    self.ui_state.set_status(format!(
                        "Deleted: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    self.current_image = None;
                    self.animation.frame_delays.clear();

                    let dir = path.parent().unwrap_or(&path).to_path_buf();
                    self.load_directory_async(dir);
                    self.dirty = true;
                    self.next_image();
                }
            }
        }
    }

    pub(crate) fn save_image(&mut self) {
        if let Some(ref image_data) = self.current_image {
            let path = image_data.path.clone();
            let mut save_path = path.clone();

            if let Some(stem) = path.file_stem() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                let stem_lossy = stem.to_string_lossy();
                save_path.set_file_name(format!("{stem_lossy}_edited.{ext}"));
            }

            self.ui_state.set_status("Saving...");
            self.dirty = true;

            let path_clone = path.clone();
            let save_path_clone = save_path.clone();
            let adjustments = self.ui_state.adjustments;
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            // (13) If we have high-res data already in memory, use it instead of reloading
            let in_memory_img = if image_data.width > 2000 || image_data.height > 2000 {
                None // It was downsampled, better reload
            } else {
                image::DynamicImage::ImageRgba8(
                    image::RgbaImage::from_raw(
                        image_data.width,
                        image_data.height,
                        image_data.rgba_data.clone(),
                    )
                    .unwrap(),
                )
                .into()
            };

            std::thread::spawn(move || {
                let result = (|| -> anyhow::Result<()> {
                    let mut img = if let Some(i) = in_memory_img {
                        i
                    } else {
                        image::open(&path_clone)?
                    };

                    if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
                        let (w, h) = (img.width() as f32, img.height() as f32);
                        let crop_x = (adjustments.crop_rect[0] * w) as u32;
                        let crop_y = (adjustments.crop_rect[1] * h) as u32;
                        let crop_w = (adjustments.crop_rect[2] * w) as u32;
                        let crop_h = (adjustments.crop_rect[3] * h) as u32;
                        img = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
                    }

                    let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round() as i32;
                    let rot_normalized = if rot_deg < 0 { rot_deg + 360 } else { rot_deg };
                    match rot_normalized {
                        90 => img = img.rotate90(),
                        180 => img = img.rotate180(),
                        270 => img = img.rotate270(),
                        _ => {}
                    }

                    if (adjustments.brightness - 1.0).abs() > 0.01
                        || (adjustments.contrast - 1.0).abs() > 0.01
                    {
                        let b = (adjustments.brightness - 1.0) * 255.0;
                        let c = adjustments.contrast;
                        img = img.adjust_contrast(c);
                        if b != 0.0 {
                            img = img.brighten(b as i32);
                        }
                    }

                    if (adjustments.saturation - 1.0).abs() > 0.01 {
                        let sat = adjustments.saturation;
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            let r = px[0] as f32 / 255.0;
                            let g = px[1] as f32 / 255.0;
                            let b = px[2] as f32 / 255.0;
                            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                            px[0] = ((gray + (r - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[1] = ((gray + (g - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[2] = ((gray + (b - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                        }
                        img = image::DynamicImage::ImageRgba8(rgba);
                    }

                    if adjustments.hdr_toning {
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            for c in 0..3 {
                                let mut color = (px[c] as f32 / 255.0) * 1.6;
                                color = color / (1.0 + color);
                                color = color * color * (3.0 - 2.0 * color);
                                px[c] = (color.clamp(0.0, 1.0) * 255.0) as u8;
                            }
                        }
                        img = image::DynamicImage::ImageRgba8(rgba);
                    }

                    ImageBackend::save(&save_path_clone, &img, 90)?;
                    Ok(())
                })();

                let event = match result {
                    Ok(()) => AppEvent::SaveComplete(save_path_clone),
                    Err(e) => AppEvent::SaveError(e.to_string()),
                };
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, event);
                }
            });

            self.ui_state.set_status(format!(
                "Saving to {}",
                save_path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    pub(crate) fn batch_save_selected(&mut self) {
        let selected: Vec<PathBuf> = self
            .ui_state
            .selected_indices
            .iter()
            .filter_map(|&i| self.ui_state.files.get(i).map(|f| f.path.clone()))
            .collect();

        if selected.is_empty() {
            self.ui_state
                .set_status("No images selected for batch save");
            self.dirty = true;
            return;
        }

        let adjustments = self.ui_state.adjustments;
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        self.ui_state
            .set_status(format!("Batch saving {} images...", selected.len()));
        self.dirty = true;

        std::thread::spawn(move || {
            let mut saved_count = 0;
            let mut errors = 0;

            for path in &selected {
                let result = (|| -> anyhow::Result<()> {
                    let mut img = image::open(path)?;

                    if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
                        let (w, h) = (img.width() as f32, img.height() as f32);
                        let crop_x = (adjustments.crop_rect[0] * w) as u32;
                        let crop_y = (adjustments.crop_rect[1] * h) as u32;
                        let crop_w = (adjustments.crop_rect[2] * w) as u32;
                        let crop_h = (adjustments.crop_rect[3] * h) as u32;
                        img = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
                    }

                    let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round() as i32;
                    let rot_normalized = if rot_deg < 0 { rot_deg + 360 } else { rot_deg };
                    match rot_normalized {
                        90 => img = img.rotate90(),
                        180 => img = img.rotate180(),
                        270 => img = img.rotate270(),
                        _ => {}
                    }

                    if (adjustments.brightness - 1.0).abs() > 0.01
                        || (adjustments.contrast - 1.0).abs() > 0.01
                    {
                        let b = (adjustments.brightness - 1.0) * 255.0;
                        let c = adjustments.contrast;
                        img = img.adjust_contrast(c);
                        if b != 0.0 {
                            img = img.brighten(b as i32);
                        }
                    }

                    if (adjustments.saturation - 1.0).abs() > 0.01 {
                        let sat = adjustments.saturation;
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            let r = px[0] as f32 / 255.0;
                            let g = px[1] as f32 / 255.0;
                            let b = px[2] as f32 / 255.0;
                            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                            px[0] = ((gray + (r - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[1] = ((gray + (g - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                            px[2] = ((gray + (b - gray) * sat).clamp(0.0, 1.0) * 255.0) as u8;
                        }
                        img = image::DynamicImage::ImageRgba8(rgba);
                    }

                    if adjustments.hdr_toning {
                        let mut rgba = img.to_rgba8();
                        for px in rgba.pixels_mut() {
                            for c in 0..3 {
                                let mut color = (px[c] as f32 / 255.0) * 1.6;
                                color = color / (1.0 + color);
                                color = color * color * (3.0 - 2.0 * color);
                                px[c] = (color.clamp(0.0, 1.0) * 255.0) as u8;
                            }
                        }
                        img = image::DynamicImage::ImageRgba8(rgba);
                    }

                    if let Some(stem) = path.file_stem() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let stem_lossy = stem.to_string_lossy();
                        let mut save_path = path.clone();
                        save_path.set_file_name(format!("{stem_lossy}_edited.{ext}"));
                        ImageBackend::save(&save_path, &img, 90)?;
                    }
                    Ok(())
                })();

                match result {
                    Ok(()) => saved_count += 1,
                    Err(e) => {
                        tracing::warn!("Failed to batch save {:?}: {}", path, e);
                        errors += 1;
                    }
                }
            }

            let msg = if errors > 0 {
                format!("Batch save: {} saved, {} errors", saved_count, errors)
            } else {
                format!("Batch save complete: {} images saved", saved_count)
            };

            if let Some(ref proxy) = proxy {
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
            self.ui_state
                .set_status("No images selected for batch delete");
            self.dirty = true;
            return;
        }

        let count = selected.len();
        let confirmed = rfd::MessageDialog::new()
            .set_title("Delete Images")
            .set_description(format!("Delete {} selected images?", count))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes;

        if !confirmed {
            return;
        }

        let mut deleted = 0;
        let mut errors = 0;

        for path in &selected {
            match std::fs::remove_file(path) {
                Ok(()) => deleted += 1,
                Err(e) => {
                    tracing::warn!("Failed to delete {:?}: {}", path, e);
                    errors += 1;
                }
            }
        }

        let msg = if errors > 0 {
            format!("Deleted {}, {} errors", deleted, errors)
        } else {
            format!("Deleted {} images", deleted)
        };

        self.ui_state.set_status(msg);
        self.ui_state.selected_indices.clear();

        if let Some(current) = self.ui_state.current_file() {
            if let Some(parent) = current.parent() {
                self.load_directory_async(parent.to_path_buf());
            }
        }

        self.dirty = true;
    }

    pub(crate) fn next_image(&mut self) {
        self.ui_state.next_file();
        if let Some(file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    pub(crate) fn prev_image(&mut self) {
        self.ui_state.prev_file();
        if let Some(file) = self.ui_state.current_file() {
            self.load_image(file.clone().to_path_buf());
        }
    }

    pub(crate) fn rotate_image(&mut self) {
        self.ui_state.rotate_90();
        self.dirty = true;
    }

    pub(crate) fn toggle_crop(&mut self) {
        self.ui_state.is_cropping = !self.ui_state.is_cropping;
        if !self.ui_state.is_cropping {
            self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
            self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        }
        self.dirty = true;
    }

    pub(crate) fn toggle_hdr_toning(&mut self) {
        self.ui_state.adjustments.hdr_toning = !self.ui_state.adjustments.hdr_toning;
        let label = if self.ui_state.adjustments.hdr_toning {
            "ON"
        } else {
            "OFF"
        };
        self.ui_state.set_status(format!("HDR Toning: {label}"));
        self.dirty = true;
    }

    pub(crate) fn cancel_crop(&mut self) {
        self.ui_state.is_cropping = false;
        self.ui_state.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
        self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    pub(crate) fn reset_adjustments(&mut self) {
        self.ui_state.reset_adjustments();
        self.ui_state.set_status("Adjustments reset");
        self.dirty = true;
    }

    pub(crate) fn toggle_sidebar(&mut self) {
        self.ui_state.show_sidebar = !self.ui_state.show_sidebar;
        self.dirty = true;
    }

    pub(crate) fn toggle_thumbnail_strip(&mut self) {
        self.ui_state.show_thumbnail_strip = !self.ui_state.show_thumbnail_strip;
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

            std::thread::spawn(move || {
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
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                let p = img.path.display().to_string();
                tracing::info!("Setting wallpaper: {p}");
                use std::os::windows::ffi::OsStrExt;
                use windows::Win32::UI::WindowsAndMessaging::{
                    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE,
                    SPI_SETDESKWALLPAPER,
                };

                let path = &img.path;
                let abs_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(path)
                };

                let mut path_wide: Vec<u16> = abs_path.as_os_str().encode_wide().collect();
                path_wide.push(0);

                unsafe {
                    let _ = SystemParametersInfoW(
                        SPI_SETDESKWALLPAPER,
                        0,
                        Some(path_wide.as_mut_ptr() as *mut _),
                        SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
                    );
                }
                self.ui_state.set_status("Desktop wallpaper set!");
                self.dirty = true;
            }
        }
        #[cfg(not(windows))]
        {
            self.ui_state
                .set_status("Wallpaper setting not supported on this OS");
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
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
            use windows::Win32::UI::WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, TrackPopupMenu, MF_STRING, TPM_NONOTIFY,
                TPM_RETURNCMD,
            };

            unsafe {
                let hmenu = CreatePopupMenu().unwrap_or_default();
                if hmenu.is_invalid() {
                    return;
                }

                let mut id = 1;
                let items = [
                    "Open in Explorer",
                    "Copy (Ctrl+C)",
                    "Rename (F2)",
                    "Delete (Del)",
                    "Set as Wallpaper (Ctrl+W)",
                ];

                for item in &items {
                    let mut wide: Vec<u16> = std::ffi::OsStr::new(item).encode_wide().collect();
                    wide.push(0);
                    let _ = AppendMenuW(hmenu, MF_STRING, id, PCWSTR(wide.as_ptr()));
                    id += 1;
                }

                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = GetCursorPos(&mut pt);

                let hwnd = if let Ok(handle) = w.window_handle() {
                    let raw: RawWindowHandle = handle.as_raw();
                    match raw {
                        RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut std::ffi::c_void),
                        _ => HWND::default(),
                    }
                } else {
                    HWND::default()
                };

                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);

                let cmd = TrackPopupMenu(
                    hmenu,
                    TPM_RETURNCMD | TPM_NONOTIFY,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    None,
                );

                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyMenu(hmenu);

                match cmd.0 {
                    1 => self.open_in_explorer(),
                    2 => self.copy_to_clipboard(),
                    3 => self.rename_current_image(),
                    4 => self.delete_current_image(),
                    5 => {
                        self.set_as_wallpaper();
                    }
                    _ => {}
                }
            }
        }
    }

    #[cfg(windows)]
    pub(crate) fn open_in_explorer(&self) {
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                let path = std::path::Path::new(&img.path);
                let abs_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(path)
                };

                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
                let abs_path_disp = abs_path.display();
                let arg = format!("/select,\"{abs_path_disp}\"");

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
                        SW_SHOW,
                    );
                }
            }
        }
    }

    pub(crate) fn print_image(&self) {
        #[cfg(windows)]
        {
            if let Some(img) = &self.current_image {
                let p = &img.path;
                tracing::info!("Printing image: {}", p.display());
                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

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
                        SW_SHOW,
                    );
                }
            }
        }
        #[cfg(not(windows))]
        {
            tracing::warn!("Print not currently supported on this platform.");
        }
    }

    pub(crate) fn copy_to_clipboard(&mut self) {
        if let Some(img) = &self.current_image {
            let path = std::path::PathBuf::from(&img.path);
            self.ui_state.set_status("Copying to clipboard...");
            self.dirty = true;

            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            // (13) Use in-memory data if it's small enough (not downsampled much)
            let in_memory_data = if img.width > 2000 || img.height > 2000 {
                None
            } else {
                Some(img.rgba_data.clone())
            };
            let (w, h) = (img.width, img.height);

            std::thread::spawn(move || {
                let res = if let Some(rgba) = in_memory_data {
                    let img_obj = image::DynamicImage::ImageRgba8(
                        image::RgbaImage::from_raw(w, h, rgba).unwrap(),
                    );
                    #[cfg(target_os = "windows")]
                    {
                        Self::do_copy_to_clipboard_windows(&img_obj)
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        Self::do_copy_to_clipboard(&path)
                    } // Fallback to path for other OS
                } else {
                    Self::do_copy_to_clipboard(&path)
                };

                if let Some(ref p) = proxy {
                    if let Err(e) = res {
                        send_event(&tx, p, AppEvent::SetStatus(format!("Copy failed: {e}")));
                    } else {
                        send_event(
                            &tx,
                            p,
                            AppEvent::SetStatus("Copied to clipboard".to_string()),
                        );
                    }
                }
            });
        }
    }

    pub(crate) fn do_copy_to_clipboard(path: &Path) -> Result<()> {
        let img = image::open(path)?;

        #[cfg(target_os = "linux")]
        {
            let mut png_data = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_data),
                image::ImageFormat::Png,
            )?;

            let child = std::process::Command::new("wl-copy")
                .arg("-t")
                .arg("image/png")
                .stdin(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut c) = child {
                if let Some(mut stdin) = c.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(&png_data);
                }
                let _ = c.wait();
                return Ok(());
            }

            let child = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-t", "image/png"])
                .stdin(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut c) = child {
                if let Some(mut stdin) = c.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(&png_data);
                }
                let _ = c.wait();
                return Ok(());
            }

            anyhow::bail!("Neither wl-copy nor xclip found");
        }

        #[cfg(target_os = "windows")]
        {
            return Self::do_copy_to_clipboard_windows(&img);
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            anyhow::bail!("Native clipboard not implemented on this OS");
        }
    }

    #[cfg(target_os = "windows")]
    fn do_copy_to_clipboard_windows(img: &image::DynamicImage) -> Result<()> {
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
        use windows::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut bgra = rgba.into_raw();
        for chunk in bgra.chunks_exact_mut(4) {
            chunk.swap(0, 2); // RGBA -> BGRA
        }

        let stride = (width * 4) as usize;
        let mut flipped = vec![0u8; bgra.len()];
        for (y, row) in bgra.chunks_exact(stride).enumerate() {
            let flipped_y = (height as usize - 1) - y;
            flipped[flipped_y * stride..(flipped_y + 1) * stride].copy_from_slice(row);
        }

        let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
        let size = header_size + flipped.len();

        unsafe {
            let hmem = GlobalAlloc(GHND, size)?;
            let ptr = GlobalLock(hmem) as *mut u8;

            let header = BITMAPINFOHEADER {
                biSize: header_size as u32,
                biWidth: width as i32,
                biHeight: height as i32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: flipped.len() as u32,
                ..Default::default()
            };

            std::ptr::copy_nonoverlapping(&header as *const _ as *const u8, ptr, header_size);
            std::ptr::copy_nonoverlapping(flipped.as_ptr(), ptr.add(header_size), flipped.len());
            let _ = GlobalUnlock(hmem);

            if OpenClipboard(None).is_ok() {
                let _ = EmptyClipboard();
                let _ = SetClipboardData(8, HANDLE(hmem.0 as *mut _)); // 8 is CF_DIB
                let _ = CloseClipboard();
            } else {
                anyhow::bail!("Failed to open clipboard");
            }
        }
        Ok(())
    }

    pub(crate) fn open_file_dialog(&mut self) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &ImageBackend::supported_extensions())
                .pick_file()
            {
                if let Some(ref proxy) = proxy {
                    send_event(&tx, proxy, AppEvent::OpenPath(path));
                }
            }
        });
    }

    pub(crate) fn zoom_in(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(0.8, cursor); // Switched to 0.8 for faster zoom
    }

    pub(crate) fn zoom_out(&mut self, cursor: Option<PhysicalPosition<f64>>) {
        self.zoom_by(1.25, cursor);
    }

    pub(crate) fn zoom_by(&mut self, factor: f32, cursor: Option<PhysicalPosition<f64>>) {
        let old_w = self.ui_state.adjustments.crop_rect_target[2];
        let old_h = self.ui_state.adjustments.crop_rect_target[3];
        let new_w = (old_w * factor).clamp(0.01, 1.0);
        let new_h = (old_h * factor).clamp(0.01, 1.0);

        if let (Some(pos), Some(ref w)) = (cursor, &self.window) {
            let win_size = w.inner_size();
            if win_size.width > 0 && win_size.height > 0 {
                let cx = (pos.x as f32 / win_size.width as f32)
                    .mul_add(old_w, self.ui_state.adjustments.crop_rect_target[0]);
                let cy = (pos.y as f32 / win_size.height as f32)
                    .mul_add(old_h, self.ui_state.adjustments.crop_rect_target[1]);

                self.ui_state.adjustments.crop_rect_target[0] =
                    (cx - new_w * (pos.x as f32 / win_size.width as f32)).clamp(0.0, 1.0 - new_w);
                self.ui_state.adjustments.crop_rect_target[1] =
                    (cy - new_h * (pos.y as f32 / win_size.height as f32)).clamp(0.0, 1.0 - new_h);
            }
        }

        self.ui_state.adjustments.crop_rect_target[2] = new_w;
        self.ui_state.adjustments.crop_rect_target[3] = new_h;
        self.dirty = true;
    }

    pub(crate) fn zoom_fit(&mut self) {
        self.ui_state.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
        self.dirty = true;
    }

    pub(crate) fn active_thumb_index(&self) -> Option<usize> {
        let current = self.ui_state.current_file()?;
        self.thumbnails.paths.iter().position(|p| p == current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slideshow_toggle() {
        let mut app = SpedImageApp::new();
        assert!(!app.slideshow.active);

        app.toggle_slideshow();
        assert!(app.slideshow.active);
        assert!(app.slideshow.next_time.is_some());

        app.toggle_slideshow();
        assert!(!app.slideshow.active);
        assert!(app.slideshow.next_time.is_none());
    }

    #[test]
    fn test_slideshow_interval() {
        let mut app = SpedImageApp::new();
        let initial = app.slideshow.interval;

        app.adjust_slideshow_interval(1);
        assert_eq!(
            app.slideshow.interval,
            initial + std::time::Duration::from_secs(1)
        );

        app.adjust_slideshow_interval(-2);
        assert_eq!(
            app.slideshow.interval,
            initial - std::time::Duration::from_secs(1)
        );
    }

    #[test]
    fn test_zoom_by_clamping() {
        let mut app = SpedImageApp::new();
        // Initial crop rect is [0.0, 0.0, 1.0, 1.0] by default in ImageAdjustments
        assert_eq!(app.ui_state.adjustments.crop_rect_target[2], 1.0);

        // Zooming out further should clamp at 1.0
        app.zoom_by(1.5, None);
        assert_eq!(app.ui_state.adjustments.crop_rect_target[2], 1.0);

        // Zooming in heavily should clamp at 0.01
        app.zoom_by(0.001, None);
        assert_eq!(app.ui_state.adjustments.crop_rect_target[2], 0.01);
    }

    #[test]
    fn test_active_thumb_index() {
        let mut app = SpedImageApp::new();
        // No current file -> None
        assert_eq!(app.active_thumb_index(), None);

        let path = std::path::PathBuf::from("test.jpg");
        app.ui_state.files.push(crate::ui::FileEntry {
            path: path.clone(),
            name: "test.jpg".to_string(),
            is_image: true,
        });
        app.ui_state.current_file_index = Some(0);

        // thumb_paths doesn't have it -> None
        assert_eq!(app.active_thumb_index(), None);

        app.thumbnails.paths.push(path);
        assert_eq!(app.active_thumb_index(), Some(0));
    }
}
