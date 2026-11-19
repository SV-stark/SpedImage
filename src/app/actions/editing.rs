use crate::app::constants;
use crate::app::state::SpedImageApp;
use crate::app::types::{AppEvent, send_event};
use std::sync::Arc;

impl SpedImageApp {
    pub(crate) fn rotate_image(&mut self) {
        self.ui_state.history.record(self.ui_state.adjustments);
        self.ui_state.rotate_90();
        self.dirty = true;
    }

    pub(crate) fn rotate_image_ccw(&mut self) {
        self.ui_state.history.record(self.ui_state.adjustments);
        self.ui_state.adjustments.rotation -= std::f32::consts::FRAC_PI_2;
        self.dirty = true;
    }

    pub(crate) fn flip_horizontal(&mut self) {
        self.ui_state.history.record(self.ui_state.adjustments);
        self.ui_state.adjustments.flip_horizontal = !self.ui_state.adjustments.flip_horizontal;
        self.ui_state
            .set_status(if self.ui_state.adjustments.flip_horizontal {
                "Flipped horizontally"
            } else {
                "Horizontal flip cleared"
            });
        self.dirty = true;
    }

    pub(crate) fn flip_vertical(&mut self) {
        self.ui_state.history.record(self.ui_state.adjustments);
        self.ui_state.adjustments.flip_vertical = !self.ui_state.adjustments.flip_vertical;
        self.ui_state
            .set_status(if self.ui_state.adjustments.flip_vertical {
                "Flipped vertically"
            } else {
                "Vertical flip cleared"
            });
        self.dirty = true;
    }

    pub(crate) fn undo_edit(&mut self) {
        if self.ui_state.history.undo(&mut self.ui_state.adjustments) {
            self.ui_state.set_status("Undo");
            self.dirty = true;
        }
    }

    pub(crate) fn redo_edit(&mut self) {
        if self.ui_state.history.redo(&mut self.ui_state.adjustments) {
            self.ui_state.set_status("Redo");
            self.dirty = true;
        }
    }

    pub(crate) fn auto_enhance(&mut self) {
        let Some(ref img) = self.current_image else {
            return;
        };
        let enhanced = crate::image::ImageProcessor::compute_auto_enhance(&img.rgba_data);
        self.ui_state.history.record(self.ui_state.adjustments);
        enhanced.apply_to(&mut self.ui_state.adjustments);
        self.ui_state.set_status("Auto-enhanced (Ctrl+Z to undo)");
        self.dirty = true;
    }

    pub(crate) fn toggle_color_picker(&mut self) {
        self.ui_state.show_color_picker = !self.ui_state.show_color_picker;
        if self.ui_state.show_color_picker {
            self.ui_state
                .set_status("Color Picker active â€” click to copy hex (P / Esc to close)");
        } else {
            self.ui_state.set_status("Color Picker closed");
        }
        self.dirty = true;
    }

    pub(crate) fn toggle_animation_pause(&mut self) {
        if self.animation.frame_delays.len() <= 1 {
            return;
        }
        self.animation.paused = !self.animation.paused;
        if !self.animation.paused {
            let delay = self
                .animation
                .frame_delays
                .get(self.animation.frame_idx)
                .copied()
                .unwrap_or(constants::GIF_DEFAULT_FRAME_MS);
            self.animation.next_frame_time =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(delay as u64));
        }
        self.ui_state.set_status(if self.animation.paused {
            "Animation paused (. to step frame)"
        } else {
            "Animation playing"
        });
        self.dirty = true;
    }

    pub(crate) fn copy_adjustments(&mut self) {
        self.ui_state.copied_adjustments = Some(self.ui_state.adjustments);
        self.ui_state
            .set_status("Copied adjustments (Ctrl+Alt+V to paste)");
        self.dirty = true;
    }

    pub(crate) fn paste_adjustments(&mut self) {
        let Some(src) = self.ui_state.copied_adjustments else {
            self.ui_state.set_status("No copied adjustments to paste");
            self.dirty = true;
            return;
        };
        self.ui_state.history.record(self.ui_state.adjustments);
        let dst = &mut self.ui_state.adjustments;
        dst.brightness = src.brightness;
        dst.contrast = src.contrast;
        dst.saturation = src.saturation;
        dst.highlights = src.highlights;
        dst.shadows = src.shadows;
        dst.clarity = src.clarity;
        dst.temperature = src.temperature;
        dst.tint = src.tint;
        dst.sharpen = src.sharpen;
        dst.hdr_toning = src.hdr_toning;
        self.ui_state
            .set_status("Pasted adjustments (Ctrl+Z to undo)");
        self.dirty = true;
    }

    pub(crate) fn cycle_inspect_mode(&mut self) {
        let next = self.ui_state.adjustments.inspect_mode.next();
        self.ui_state.adjustments.inspect_mode = next;
        self.ui_state
            .set_status(format!("Inspector: {}", next.label()));
        self.dirty = true;
    }

    pub(crate) fn pan_to_normalized(&mut self, u: f32, v: f32) {
        let cw = self.ui_state.adjustments.crop_rect[2];
        let ch = self.ui_state.adjustments.crop_rect[3];
        let max_x = (1.0 - cw).max(0.0);
        let max_y = (1.0 - ch).max(0.0);
        let x = (u - cw * 0.5).clamp(0.0, max_x);
        let y = (v - ch * 0.5).clamp(0.0, max_y);
        self.ui_state.adjustments.crop_rect[0] = x;
        self.ui_state.adjustments.crop_rect[1] = y;
        self.ui_state.adjustments.crop_rect_target[0] = x;
        self.ui_state.adjustments.crop_rect_target[1] = y;
        self.dirty = true;
    }

    /// Load EXIF metadata on the worker thread so pressing `I` never stalls the
    /// UI thread on slow drives or large RAW/TIFF files.
    pub(crate) fn ensure_exif_async(&mut self) {
        let Some(ref img) = self.current_image else {
            return;
        };
        if img.exif_loaded {
            return;
        }
        let path = img.path.clone();
        let tx = self.event_tx.clone();
        let proxy = self.event_proxy.clone();
        self.thread_pool().spawn(move || {
            let meta = crate::image::read_exif_meta(&path);
            if let Some(ref p) = proxy {
                send_event(
                    &tx,
                    p,
                    AppEvent::ExifLoaded(path, meta.exif_info, meta.gps_coords),
                );
            }
        });
    }

    pub(crate) fn step_animation_frame(&mut self) {
        self.step_animation_by(1);
    }

    pub(crate) fn step_animation_frame_prev(&mut self) {
        self.step_animation_by(-1);
    }

    fn step_animation_by(&mut self, delta: isize) {
        let n = self.animation.frame_delays.len();
        if n <= 1 {
            return;
        }
        self.animation.paused = true;
        let candidate = (self.animation.frame_idx as isize + delta).rem_euclid(n as isize) as usize;

        // In-memory multi-frame path (animated WebP, multi-page TIFF).
        if let Some(frame) = self.animation.frames.get(candidate).cloned() {
            if let Some(ref mut r) = self.renderer
                && let Err(e) = r.load_image(&frame)
            {
                tracing::warn!("Frame upload failed: {e}");
            }
            self.current_image = Some(frame);
            self.animation.frame_idx = candidate;
            self.dirty = true;
            return;
        }

        // Streamed GIF ring-buffer path.
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
            self.dirty = true;
        } else if let Some(ref mut r) = self.renderer
            && r.swap_gif_frame(candidate)
        {
            self.animation.frame_idx = candidate;
            self.dirty = true;
        }
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
        self.ui_state.history.record(self.ui_state.adjustments);
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
        self.ui_state.history.record(self.ui_state.adjustments);
        self.ui_state.reset_adjustments();
        self.ui_state.set_status("Adjustments reset");
        self.dirty = true;
    }

    pub(crate) fn ensure_histogram_async(&self) {
        if let Some(ref img) = self.current_image
            && img.histogram.is_none()
        {
            let path = img.path.clone();
            let rgba = img.rgba_data.clone();
            let tx = self.event_tx.clone();
            let proxy = self.event_proxy.clone();
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
                        AppEvent::HistogramComputed(path, Box::new((r_hist, g_hist, b_hist))),
                    );
                }
            });
        }
    }
}
