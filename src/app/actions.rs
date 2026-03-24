use crate::app::constants;
use crate::app::state::SpedImageApp;
use crate::app::types::{send_event, AppEvent};
use crate::image::ImageBackend;
use std::path::PathBuf;
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
                    self.dirty = true;
                }
                "c" | "C" if ctrl => self.copy_to_clipboard(),
                "c" | "C" => self.toggle_crop(),
                "h" | "H" => {
                    if self.modifiers.shift {
                        self.ui_state.show_histogram = !self.ui_state.show_histogram;
                        if self.ui_state.show_histogram {
                            if let Some(ref mut img) = self.current_image {
                                if img.histogram.is_none() {
                                    let tx = self.event_tx.clone();
                                    let proxy = self.event_proxy.clone();
                                    let path = img.path.clone();
                                    let rgba = img.rgba_data.clone();
                                    self.thread_pool.spawn(move || {
                                        // Reuse ImageData::compute_histogram logic via a closure
                                        let mut r_hist = [0u32; 256];
                                        let mut g_hist = [0u32; 256];
                                        let mut b_hist = [0u32; 256];
                                        crate::image::compute_rgb_histogram(
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
                            }
                        }
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

    pub(crate) fn load_image(&mut self, path: &std::path::Path) {
        let generation = self
            .navigation
            .load_generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        // Compute prefetch targets
        let mut prefetch_targets = Vec::new();
        if self.ui_state.files.len() >= 2 {
            if let Some(idx) = self.ui_state.files.iter().position(|f| f.path == path) {
                let next_idx = (idx + 1) % self.ui_state.files.len();
                let prev_idx = if idx == 0 {
                    self.ui_state.files.len() - 1
                } else {
                    idx - 1
                };
                prefetch_targets.push(self.ui_state.files[next_idx].path.clone());
                prefetch_targets.push(self.ui_state.files[prev_idx].path.clone());
            }
        }

        let (max_w, max_h) = match &self.window {
            Some(w) => {
                let size = w.inner_size();
                (size.width, size.height)
            }
            None => (constants::DEFAULT_MAX_WIDTH, constants::DEFAULT_MAX_HEIGHT),
        };

        let pool = self.thread_pool.clone();
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        let current_gen = self.navigation.load_generation.clone();

        if let Some(cached_frames) = self.navigation.prefetch_cache.get(&path) {
            if let Some(ref proxy) = self.event_proxy {
                send_event(
                    &self.event_tx,
                    proxy,
                    AppEvent::ImageLoaded((*cached_frames).clone()),
                );
            }

            for target_path in prefetch_targets {
                let tx_p = tx.clone();
                let proxy_p = proxy.clone();
                let gen_p = current_gen.clone();
                pool.spawn(move || {
                    if let Ok(frames) =
                        ImageBackend::load_and_downsample(&target_path, max_w, max_h)
                    {
                        if gen_p.load(Ordering::Relaxed) == generation {
                            if let Some(ref p) = proxy_p {
                                send_event(&tx_p, p, AppEvent::Prefetched(target_path, frames));
                            }
                        }
                    }
                });
            }
            return;
        }

        self.ui_state.set_status("Loading...");
        self.loading = true;
        self.dirty = true;

        if let Some(ref w) = self.window {
            w.set_title(&format!("SpedImage — {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("SpedImage")));
        }

        let path_owned = path.to_path_buf();
        let pool_inner = pool.clone();

        pool.spawn(move || {
            let result = ImageBackend::load_and_downsample(&path_owned, max_w, max_h);

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
                    let tx_p = tx.clone();
                    let proxy_p = proxy.clone();
                    let gen_p = current_gen.clone();
                    pool_inner.spawn(move || {
                        if let Ok(frames) =
                            ImageBackend::load_and_downsample(&target_path, max_w, max_h)
                        {
                            if gen_p.load(Ordering::Relaxed) == generation {
                                if let Some(ref p) = proxy_p {
                                    send_event(&tx_p, p, AppEvent::Prefetched(target_path, frames));
                                }
                            }
                        }
                    });
                }
            }
        });
    }

    pub(crate) fn delete_current_image(&mut self) {
        if let Some(ref image) = self.current_image {
            let path = image.path.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();

            self.thread_pool.spawn(move || {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_title("Delete Image")
                    .set_description(format!(
                        "Delete {}?",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .set_buttons(rfd::MessageButtons::YesNo);

                pollster::block_on(async move {
                    if dialog.show().await == rfd::MessageDialogResult::Yes {
                        if let Some(ref p) = proxy {
                            send_event(&tx, p, AppEvent::ConfirmDelete(path));
                        }
                    }
                });
            });
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

            let rgba_data = image_data.rgba_data.clone();
            let (width, height) = (image_data.width, image_data.height);
            let is_downsampled = image_data.is_downsampled;

            self.thread_pool.spawn(move || {
                let result = (|| -> color_eyre::eyre::Result<()> {
                    use zune_image::image::Image;
                    use zune_image::traits::OperationsTrait;
                    use zune_imageprocs::brighten::Brighten;
                    use zune_imageprocs::contrast::Contrast;
                    use zune_imageprocs::crop::Crop;
                    use zune_imageprocs::rotate::Rotate;

                    let mut img = if is_downsampled {
                        Image::open(&path_clone)
                            .map_err(|e| color_eyre::eyre::eyre!("Failed to open image: {e:?}"))?
                    } else {
                        Image::from_u8(
                            &rgba_data,
                            width as usize,
                            height as usize,
                            zune_core::colorspace::ColorSpace::RGBA,
                        )
                    };

                    // Initial crop and rotate
                    if adjustments.crop_rect != [0.0, 0.0, 1.0, 1.0] {
                        let (w, h) = img.dimensions();
                        let crop_x = (adjustments.crop_rect[0] * w as f32) as usize;
                        let crop_y = (adjustments.crop_rect[1] * h as f32) as usize;
                        let crop_w = (adjustments.crop_rect[2] * w as f32) as usize;
                        let crop_h = (adjustments.crop_rect[3] * h as f32) as usize;

                        Crop::new(crop_w, crop_h, crop_x, crop_y)
                            .execute(&mut img)
                            .map_err(|e| color_eyre::eyre::eyre!("Crop failed: {e:?}"))?;
                    }

                    let rot_deg = (adjustments.rotation.to_degrees() % 360.0).round();
                    if rot_deg.abs() > 0.1 {
                        Rotate::new(rot_deg)
                            .execute(&mut img)
                            .map_err(|e| color_eyre::eyre::eyre!("Rotate failed: {e:?}"))?;
                    }

                    // Use zune-imageprocs for color/exposure adjustments
                    if (adjustments.brightness - 1.0).abs() > 0.01 {
                        Brighten::new(adjustments.brightness)
                            .execute(&mut img)
                            .map_err(|e| color_eyre::eyre::eyre!("Brighten failed: {e:?}"))?;
                    }
                    if (adjustments.contrast - 1.0).abs() > 0.01 {
                        Contrast::new(adjustments.contrast)
                            .execute(&mut img)
                            .map_err(|e| color_eyre::eyre::eyre!("Contrast failed: {e:?}"))?;
                    }

                    let (final_w, final_h) = img.dimensions();
                    let final_rgba = img.flatten_to_u8()[0].clone();
                    ImageBackend::save(
                        &save_path_clone,
                        &final_rgba,
                        final_w as u32,
                        final_h as u32,
                    )?;
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
            return;
        }

        let _adjustments = self.ui_state.adjustments;
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();

        self.ui_state
            .set_status(format!("Batch saving {} images...", selected.len()));
        self.dirty = true;

        self.thread_pool.spawn(move || {
            let mut saved_count = 0;
            for path in &selected {
                let _ = (|| -> color_eyre::eyre::Result<()> {
                    use zune_image::image::Image;
                    let img = Image::open(path)
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to open image: {e:?}"))?;

                    if let Some(stem) = path.file_stem() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let mut save_path = path.clone();
                        save_path.set_file_name(format!(
                            "{}_edited.{}",
                            stem.to_string_lossy(),
                            ext
                        ));

                        let (w, h) = img.dimensions();
                        let rgba = img.flatten_to_u8()[0].clone();
                        ImageBackend::save(&save_path, &rgba, w as u32, h as u32)?;
                    }
                    Ok(())
                })();
                saved_count += 1;
            }
            if let Some(ref proxy) = proxy {
                send_event(
                    &tx,
                    proxy,
                    AppEvent::SetStatus(format!("Batch save complete: {} images", saved_count)),
                );
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

        self.thread_pool.spawn(move || {
            let dialog = rfd::AsyncMessageDialog::new()
                .set_title("Delete Images")
                .set_description(format!("Delete {} selected images?", selected.len()))
                .set_buttons(rfd::MessageButtons::YesNo);

            pollster::block_on(async move {
                if dialog.show().await == rfd::MessageDialogResult::Yes {
                    if let Some(ref p) = proxy {
                        send_event(&tx, p, AppEvent::ConfirmBatchDelete(selected));
                    }
                }
            });
        });
    }

    pub(crate) fn next_image(&mut self) {
        self.ui_state.next_file();
        if let Some(path) = self.ui_state.current_file() {
            self.load_image(path);
        }
    }

    pub(crate) fn prev_image(&mut self) {
        self.ui_state.prev_file();
        if let Some(path) = self.ui_state.current_file() {
            self.load_image(path);
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

            self.thread_pool.spawn(move || {
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
            use windows::Win32::UI::WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, GetCursorPos, TrackPopupMenu, MF_STRING,
                TPM_NONOTIFY, TPM_RETURNCMD,
            };
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

            unsafe {
                let hmenu = CreatePopupMenu().unwrap_or_default();
                if hmenu.is_invalid() {
                    return;
                }

                let items = [
                    "Open in Explorer",
                    "Copy (Ctrl+C)",
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
                    5 => self.set_as_wallpaper(),
                    _ => {}
                }
            }
        }
    }

    #[cfg(windows)]
    pub(crate) fn open_in_explorer(&self) {
        if let Some(img) = &self.current_image {
            use std::os::windows::ffi::OsStrExt;
            use windows::core::PCWSTR;
            use windows::Win32::UI::Shell::ShellExecuteW;
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
            use windows::core::PCWSTR;
            use windows::Win32::UI::Shell::ShellExecuteW;
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
        if let Some(img) = &self.current_image {
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
            let rgba = img.rgba_data.clone();
            let (w, h) = (img.width, img.height);

            self.thread_pool.spawn(move || {
                let mut clipboard = arboard::Clipboard::new().unwrap();
                let image_data = arboard::ImageData {
                    width: w as usize,
                    height: h as usize,
                    bytes: std::borrow::Cow::from(rgba),
                };
                if clipboard.set_image(image_data).is_ok() {
                    if let Some(ref p) = proxy {
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

    pub(crate) fn open_file_dialog(&mut self) {
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.thread_pool.spawn(move || {
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
        self.zoom_by(0.8, cursor);
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
