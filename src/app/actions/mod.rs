mod editing;
mod file_ops;
mod navigation;

use crate::app::constants;
use crate::app::state::SpedImageApp;
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
                } else if self.ui_state.show_color_picker {
                    self.ui_state.show_color_picker = false;
                    self.dirty = true;
                } else if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                    self.dirty = true;
                } else if self.is_comparing() {
                    self.toggle_compare();
                } else if self.ui_state.show_sidebar || self.config.show_sidebar {
                    self.ui_state.show_sidebar = false;
                    self.config.show_sidebar = false;
                    self.config.save();
                    self.dirty = true;
                } else if self.ui_state.show_adjustments || self.config.show_adjustments {
                    self.ui_state.show_adjustments = false;
                    self.config.show_adjustments = false;
                    self.config.save();
                    self.dirty = true;
                } else if self.ui_state.show_info || self.config.show_info {
                    self.ui_state.toggle_info();
                    self.config.show_info = self.ui_state.show_info;
                    self.config.save();
                    self.dirty = true;
                } else if self.ui_state.show_histogram || self.config.show_histogram {
                    self.ui_state.show_histogram = false;
                    self.config.show_histogram = false;
                    self.config.save();
                    self.dirty = true;
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
                "r" | "R" if self.modifiers.shift => self.rotate_image_ccw(),
                "r" | "R" => self.rotate_image(),
                "x" | "X" if !ctrl => self.flip_horizontal(),
                "y" | "Y" if ctrl => self.redo_edit(),
                "y" | "Y" => self.flip_vertical(),
                "u" | "U" if !ctrl => self.auto_enhance(),
                "o" | "O" => self.open_file_dialog(),
                "f" | "F" if !ctrl => self.toggle_sidebar(),
                "e" | "E" if !ctrl => self.toggle_adjustments(),
                "?" if !ctrl => {
                    self.ui_state.show_help = !self.ui_state.show_help;
                    self.dirty = true;
                }
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
                "z" | "Z" if ctrl && self.modifiers.shift => self.redo_edit(),
                "z" | "Z" if ctrl => self.undo_edit(),
                "z" | "Z" => {
                    self.ui_state.adjustments.pixel_perfect =
                        !self.ui_state.adjustments.pixel_perfect;
                    self.dirty = true;
                }
                "c" | "C" if ctrl && self.modifiers.alt => self.copy_adjustments(),
                "v" | "V" if ctrl && self.modifiers.alt => self.paste_adjustments(),
                "c" | "C" if ctrl && self.modifiers.shift => self.copy_path_to_clipboard(),
                "c" | "C" if ctrl => self.copy_to_clipboard(),
                "v" | "V" if ctrl => self.paste_from_clipboard(),
                "c" | "C" if self.modifiers.shift => self.cycle_inspect_mode(),
                "c" | "C" => self.toggle_crop(),
                "h" | "H" => {
                    if self.modifiers.shift {
                        self.ui_state.show_histogram = !self.ui_state.show_histogram;
                        self.config.show_histogram = self.ui_state.show_histogram;
                        self.config.save();
                        if self.ui_state.show_histogram {
                            self.ensure_histogram_async();
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
                    if self.ui_state.show_info {
                        self.ensure_exif_async();
                    }
                    self.dirty = true;
                }
                "p" | "P" if ctrl => self.print_image(),
                "p" | "P" => self.toggle_color_picker(),
                "+" | "=" => self.zoom_in(None),
                "-" => self.zoom_out(None),
                "0" => self.zoom_fit(),
                " " if self.modifiers.shift
                    && self.animation.is_animated
                    && self.animation.frame_delays.len() > 1 =>
                {
                    self.toggle_animation_pause();
                }
                " " => self.toggle_slideshow(),
                "," if self.animation.frame_delays.len() > 1 => self.step_animation_frame_prev(),
                "." if self.animation.frame_delays.len() > 1 => self.step_animation_frame(),
                "[" => self.adjust_slideshow_interval(-1),
                "]" => self.adjust_slideshow_interval(1),
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

    pub(crate) fn toggle_sidebar(&mut self) {
        self.ui_state.show_sidebar = !self.ui_state.show_sidebar;
        self.config.show_sidebar = self.ui_state.show_sidebar;
        self.config.save();
        self.dirty = true;
    }

    pub(crate) fn toggle_adjustments(&mut self) {
        self.ui_state.show_adjustments = !self.ui_state.show_adjustments;
        self.config.show_adjustments = self.ui_state.show_adjustments;
        self.config.save();
        self.dirty = true;
    }

    pub(crate) fn toggle_thumbnail_strip(&mut self) {
        self.ui_state.show_thumbnail_strip = !self.ui_state.show_thumbnail_strip;
        self.config.show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
        self.config.save();
        self.dirty = true;
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
        config.show_adjustments = self.ui_state.show_adjustments;
        config.show_thumbnail_strip = self.ui_state.show_thumbnail_strip;
        config.show_info = self.ui_state.show_info;
        config.show_histogram = self.ui_state.show_histogram;
        config.show_osd = Some(self.ui_state.show_osd);
        config.save();
    }
}
