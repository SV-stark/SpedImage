use crate::app::clipboard;
use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, send_event};
use crate::image::ImageBackend;
use std::path::PathBuf;
use std::sync::Arc;

impl SpedImageApp {
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

    fn save_options(&self) -> crate::image::SaveOptions {
        crate::image::SaveOptions {
            jpeg_quality: self.config.jpeg_quality(),
            strip_metadata: self.config.strip_metadata_enabled(),
            max_dimension: self.config.export_max_dim(),
        }
    }

    /// Run [`crate::image::save_edited_with_options`] on the worker pool and report back.
    fn spawn_save(
        &mut self,
        source: PathBuf,
        output: PathBuf,
        displayed: Option<crate::image::ImageData>,
    ) {
        self.ui_state.set_status("Saving...");
        self.dirty = true;
        let adjustments = self.ui_state.adjustments;
        let options = self.save_options();
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.thread_pool().spawn(move || {
            let event = match crate::image::save_edited_with_options(
                &source,
                &output,
                &adjustments,
                displayed.as_ref(),
                options,
            ) {
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
        let options = self.save_options();
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
            let event = match crate::image::save_edited_with_options(
                &source,
                &output,
                &adjustments,
                Some(&displayed),
                options,
            ) {
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
        let options = self.save_options();
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
                match crate::image::save_edited_with_options(
                    path,
                    &output,
                    &adjustments,
                    None,
                    options,
                ) {
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
}
