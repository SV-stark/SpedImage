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

impl SpedImageApp {
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
}
