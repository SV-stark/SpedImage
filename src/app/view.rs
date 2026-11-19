//! Display-side state: monitor colour profile, side-by-side compare, and
//! window placement.

use crate::app::state::SpedImageApp;
use winit::dpi::{PhysicalPosition, PhysicalSize};

/// Where to reopen the window, or `None` to let the OS choose.
///
/// A saved position is only reused when a reasonable part of the title bar
/// lands on a connected monitor; otherwise a window last used on an unplugged
/// screen would reopen somewhere it cannot be seen or grabbed.
pub fn restore_position(
    saved: (Option<i32>, Option<i32>),
    window_size: PhysicalSize<u32>,
    monitors: &[(PhysicalPosition<i32>, PhysicalSize<u32>)],
) -> Option<PhysicalPosition<i32>> {
    let (Some(x), Some(y)) = saved else {
        return None;
    };
    // The strip along the top of the window that the user drags it by.
    const GRAB_H: i64 = 32;
    const MIN_VISIBLE_W: i64 = 100;
    let (x, y) = (x as i64, y as i64);
    let w = window_size.width.max(1) as i64;
    let on_screen = monitors.iter().any(|(pos, size)| {
        let (mx, my) = (pos.x as i64, pos.y as i64);
        let (mr, mb) = (mx + size.width as i64, my + size.height as i64);
        let overlap_w = (x + w).min(mr) - x.max(mx);
        let overlap_h = (y + GRAB_H).min(mb) - y.max(my);
        overlap_w >= MIN_VISIBLE_W.min(w) && overlap_h >= GRAB_H / 2
    });
    on_screen.then(|| PhysicalPosition::new(x as i32, y as i32))
}

/// Map a cursor position inside the image viewport (physical pixels) back to
/// the exact `(x, y)` pixel in `image_size`, inverting the vertex shader's
/// aspect fit, rotation, flip, and zoom/crop transforms.
///
/// Returns `None` when the cursor lies outside the rendered image bounds.
pub fn screen_to_image_pixel(
    cursor_view_px: (f32, f32),
    view_size_px: (f32, f32),
    image_size: (u32, u32),
    adjustments: &crate::render::ImageAdjustments,
) -> Option<(u32, u32)> {
    let (vw, vh) = view_size_px;
    let (iw, ih) = image_size;
    if vw <= 0.0 || vh <= 0.0 || iw == 0 || ih == 0 {
        return None;
    }

    let total_rotation = adjustments.rotation + adjustments.pre_rotation;
    let rot_deg = (total_rotation.to_degrees() % 360.0).round().abs();
    let is_sideways = (rot_deg - 90.0).abs() < 1.0 || (rot_deg - 270.0).abs() < 1.0;
    let raw_aspect = iw as f32 / ih as f32;
    let aspect_ratio = if is_sideways {
        1.0 / raw_aspect
    } else {
        raw_aspect
    };
    let window_aspect = vw / vh;
    let ratio = aspect_ratio / window_aspect;

    // Physical viewport pixel -> NDC [-1, 1] (Y points up in NDC).
    let ndc_x = (cursor_view_px.0 / vw) * 2.0 - 1.0;
    let ndc_y = 1.0 - (cursor_view_px.1 / vh) * 2.0;

    // Invert aspect-ratio fit from `vertex_main`.
    let (rot_x, rot_y) = if ratio > 1.0 {
        (ndc_x, ndc_y * ratio)
    } else {
        (ndc_x / ratio, ndc_y)
    };

    // Invert 2D rotation by `-total_rotation`.
    let c = total_rotation.cos();
    let s = total_rotation.sin();
    let px = rot_x * c + rot_y * s;
    let py = -rot_x * s + rot_y * c;

    if !(-1.0..=1.0).contains(&px) || !(-1.0..=1.0).contains(&py) {
        return None;
    }

    // Quad NDC -> base tex_coords [0, 1].
    let mut tc_x = (px + 1.0) * 0.5;
    let mut tc_y = (1.0 - py) * 0.5;
    if adjustments.flip_horizontal {
        tc_x = 1.0 - tc_x;
    }
    if adjustments.flip_vertical {
        tc_y = 1.0 - tc_y;
    }

    // Apply zoom/crop rect.
    let u = adjustments.crop_rect[0] + tc_x * adjustments.crop_rect[2];
    let v = adjustments.crop_rect[1] + tc_y * adjustments.crop_rect[3];
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        return None;
    }

    let x = ((u * iw as f32).floor() as u32).min(iw - 1);
    let y = ((v * ih as f32).floor() as u32).min(ih - 1);
    Some((x, y))
}

impl SpedImageApp {
    /// Sample the RGBA pixel currently under the cursor, if the eyedropper is
    /// enabled and the cursor is over the image.
    pub(crate) fn sample_pixel_at_cursor(&self) -> Option<crate::render::ColorSample> {
        if !self.ui_state.show_color_picker {
            return None;
        }
        let img = self.current_image.as_ref()?;
        let view_pos = self.to_image_view(self.last_cursor_pos);
        let view_size = self.image_view_size();
        let (x, y) = screen_to_image_pixel(
            (view_pos.x as f32, view_pos.y as f32),
            view_size,
            (img.width, img.height),
            &self.ui_state.adjustments,
        )?;
        let idx = ((y as usize) * (img.width as usize) + (x as usize)) * 4;
        if idx + 4 > img.rgba_data.len() {
            return None;
        }
        Some(crate::render::ColorSample {
            x,
            y,
            r: img.rgba_data[idx],
            g: img.rgba_data[idx + 1],
            b: img.rgba_data[idx + 2],
            a: img.rgba_data[idx + 3],
            cursor_px: (self.last_cursor_pos.x as f32, self.last_cursor_pos.y as f32),
        })
    }

    /// Re-read the monitor's ICC profile and push the resulting matrix to the
    /// renderer (or clear it when the preference is off or there is none).
    pub(crate) fn refresh_display_profile(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        self.profile_monitor = window.current_monitor().and_then(|m| m.name());
        let matrix = if self.config.display_profile_enabled() {
            crate::render::color::window_display_profile(&window)
                .and_then(|icc| crate::render::color::srgb_to_display_matrix(&icc))
        } else {
            None
        };
        if let Some(r) = &mut self.renderer {
            r.set_display_matrix(matrix);
        }
        self.dirty = true;
    }

    /// Called when the window moves or its DPI changes: only re-read the
    /// profile when it is now on a different monitor.
    pub(crate) fn refresh_display_profile_if_monitor_changed(&mut self) {
        let Some(ref window) = self.window else {
            return;
        };
        let monitor = window.current_monitor().and_then(|m| m.name());
        if monitor != self.profile_monitor {
            self.refresh_display_profile();
        }
    }

    /// Pin the current image to the left half for comparison, or leave
    /// compare mode if something is already pinned.
    pub(crate) fn toggle_compare(&mut self) {
        self.dirty = true;
        if self.compare_pinned.take().is_some() {
            if let Some(r) = &mut self.renderer {
                let _ = r.set_compare_image(None);
            }
            self.ui_state.set_status("Compare view closed");
            return;
        }
        let (Some(img), Some(r)) = (self.current_image.as_ref(), self.renderer.as_mut()) else {
            return;
        };
        if self.ui_state.is_cropping {
            self.ui_state.set_status("Finish cropping before comparing");
            return;
        }
        match r.set_compare_image(Some(img)) {
            Ok(()) => {
                let name = img
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.compare_pinned = Some(img.path.clone());
                self.ui_state.set_status(format!(
                    "Pinned {name}. Browse to another image to compare (K to close)"
                ));
            }
            Err(e) => self.ui_state.set_status(format!("Cannot compare: {e}")),
        }
    }

    pub(crate) fn is_comparing(&self) -> bool {
        self.compare_pinned.is_some()
    }

    /// Width of the left (pinned) half in compare mode, else 0.
    fn compare_left_width(&self) -> f64 {
        match (&self.window, self.is_comparing()) {
            (Some(w), true) => (w.inner_size().width / 2) as f64,
            _ => 0.0,
        }
    }

    /// Size of the area the current image is drawn into, in physical pixels:
    /// the whole window normally, the right half while comparing.
    pub(crate) fn image_view_size(&self) -> (f32, f32) {
        let Some(ref w) = self.window else {
            return (1.0, 1.0);
        };
        let size = w.inner_size();
        let left = self.compare_left_width();
        (
            (size.width as f64 - left).max(1.0) as f32,
            size.height.max(1) as f32,
        )
    }

    /// Map a window position into the image view's own coordinates. In
    /// compare mode both halves share zoom and pan, so a cursor over the
    /// pinned half anchors the zoom at the same relative point.
    pub(crate) fn to_image_view(&self, pos: PhysicalPosition<f64>) -> PhysicalPosition<f64> {
        let left = self.compare_left_width();
        if left > 0.0 && pos.x >= left {
            PhysicalPosition::new(pos.x - left, pos.y)
        } else {
            pos
        }
    }

    /// Remember where the window is, for the next launch.
    pub(crate) fn store_window_position(&self, config: &mut crate::config::AppConfig) {
        let Some(ref w) = self.window else {
            return;
        };
        if w.fullscreen().is_some() {
            return;
        }
        if w.is_maximized() {
            // A maximized window's position is the monitor's corner. Keep the
            // restored position, unless it was on another monitor, in which
            // case remember this one so the window maximizes here next time.
            if let Some(m) = w.current_monitor() {
                let (mp, ms) = (m.position(), m.size());
                let saved_here = matches!((config.window_x, config.window_y), (Some(x), Some(y))
                    if x >= mp.x && y >= mp.y
                        && (x as i64) < mp.x as i64 + ms.width as i64
                        && (y as i64) < mp.y as i64 + ms.height as i64);
                if !saved_here {
                    config.window_x = Some(mp.x + 64);
                    config.window_y = Some(mp.y + 64);
                }
            }
            return;
        }
        if let Ok(pos) = w.outer_position() {
            config.window_x = Some(pos.x);
            config.window_y = Some(pos.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitors() -> Vec<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
        vec![
            (PhysicalPosition::new(0, 0), PhysicalSize::new(1920, 1080)),
            // A second screen to the left of the primary.
            (
                PhysicalPosition::new(-2560, 0),
                PhysicalSize::new(2560, 1440),
            ),
        ]
    }

    #[test]
    fn a_position_on_a_connected_monitor_is_restored() {
        let size = PhysicalSize::new(1200, 800);
        assert_eq!(
            restore_position((Some(100), Some(50)), size, &monitors()),
            Some(PhysicalPosition::new(100, 50))
        );
        assert_eq!(
            restore_position((Some(-2000), Some(200)), size, &monitors()),
            Some(PhysicalPosition::new(-2000, 200))
        );
    }

    #[test]
    fn a_position_on_an_unplugged_monitor_is_dropped() {
        let size = PhysicalSize::new(1200, 800);
        // Was on a screen to the right that is gone now.
        assert_eq!(
            restore_position((Some(2500), Some(100)), size, &monitors()),
            None
        );
        // Title bar above every screen: could not be dragged back.
        assert_eq!(
            restore_position((Some(100), Some(-500)), size, &monitors()),
            None
        );
    }

    #[test]
    fn a_window_hanging_off_an_edge_is_kept_if_it_can_still_be_grabbed() {
        let size = PhysicalSize::new(1200, 800);
        assert!(restore_position((Some(1700), Some(100)), size, &monitors()).is_some());
        assert_eq!(
            restore_position((Some(1900), Some(100)), size, &monitors()),
            None
        );
    }

    #[test]
    fn nothing_saved_means_nothing_restored() {
        let size = PhysicalSize::new(1200, 800);
        assert_eq!(restore_position((None, Some(10)), size, &monitors()), None);
        assert_eq!(restore_position((Some(10), Some(10)), size, &[]), None);
    }

    #[test]
    fn screen_to_image_pixel_maps_fit_flip_zoom_and_rotation() {
        use crate::render::{ColorSample, ImageAdjustments};

        let mut adj = ImageAdjustments::default();
        // 100x100 image inside 200x100 viewport -> pillarboxed to x in [50, 150].
        assert_eq!(
            screen_to_image_pixel((10.0, 50.0), (200.0, 100.0), (100, 100), &adj),
            None,
            "left pillarbox margin is outside image"
        );
        assert_eq!(
            screen_to_image_pixel((100.0, 50.0), (200.0, 100.0), (100, 100), &adj),
            Some((50, 50))
        );
        assert_eq!(
            screen_to_image_pixel((50.5, 0.5), (200.0, 100.0), (100, 100), &adj),
            Some((0, 0))
        );

        // Horizontal flip mirrors X.
        adj.flip_horizontal = true;
        assert_eq!(
            screen_to_image_pixel((50.5, 0.5), (200.0, 100.0), (100, 100), &adj),
            Some((99, 0))
        );
        adj.flip_horizontal = false;

        // Zoomed to bottom-right quadrant [0.5, 0.5, 0.5, 0.5].
        adj.crop_rect = [0.5, 0.5, 0.5, 0.5];
        assert_eq!(
            screen_to_image_pixel((50.5, 0.5), (200.0, 100.0), (100, 100), &adj),
            Some((50, 50))
        );

        // ColorSample formatting and HSL conversion.
        let red = ColorSample {
            x: 1,
            y: 2,
            r: 255,
            g: 0,
            b: 0,
            a: 255,
            cursor_px: (0.0, 0.0),
        };
        assert_eq!(red.hex_string(), "#FF0000");
        assert_eq!(red.hsl(), (0, 100, 50));

        let semi = ColorSample { a: 128, ..red };
        assert_eq!(semi.hex_string(), "#FF000080");
    }
}
