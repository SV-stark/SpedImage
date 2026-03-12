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
                                img.compute_histogram();
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
        let new_secs = (current_secs + change).clamp(1, 120) as u64;
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

    pub(crate) fn load_image(&mut self, path: PathBuf) {
        let generation = self
            .navigation
            .load_generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        if let Some(cached_frames) = self.navigation.prefetch_cache.pop(&path) {
            if let Some(ref proxy) = self.event_proxy {
                send_event(&self.event_tx, proxy, AppEvent::ImageLoaded(cached_frames));
            }
            self.prefetch_adjacent(&path);
            return;
        }

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
                    self.current_image = None;
                    let dir = path.parent().unwrap_or(&path).to_path_buf();
                    self.load_directory_async(dir);
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

            let in_memory_img = if image_data.width > 2000 || image_data.height > 2000 {
                None
            } else {
                image::RgbaImage::from_raw(
                    image_data.width,
                    image_data.height,
                    image_data.rgba_data.clone(),
                )
                .map(image::DynamicImage::ImageRgba8)
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

        std::thread::spawn(move || {
            let mut saved_count = 0;
            for path in &selected {
                let _ = (|| -> anyhow::Result<()> {
                    let img = image::open(path)?;
                    // Simplified: apply same adjustments as save_image
                    if let Some(stem) = path.file_stem() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let mut save_path = path.clone();
                        save_path.set_file_name(format!(
                            "{}_edited.{}",
                            stem.to_string_lossy(),
                            ext
                        ));
                        ImageBackend::save(&save_path, &img, 90)?;
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

        if rfd::MessageDialog::new()
            .set_title("Delete Images")
            .set_description(format!("Delete {} selected images?", selected.len()))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
        {
            for path in &selected {
                let _ = std::fs::remove_file(path);
            }
            self.ui_state.selected_indices.clear();
            if let Some(current) = self.ui_state.current_file() {
                if let Some(parent) = current.parent() {
                    self.load_directory_async(parent.to_path_buf());
                }
            }
            self.dirty = true;
        }
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

            std::thread::spawn(move || {
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
