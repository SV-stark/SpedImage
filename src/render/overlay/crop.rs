use super::super::types::{ImageAdjustments, RenderParams};

/// Largest centered crop rect with the requested *pixel* aspect ratio.
///
/// `crop_rect` is normalized against the image's own width/height, so a preset
/// ratio `want` has to be divided by the image aspect to get a rect whose
/// resulting pixels actually have ratio `want`.
pub(crate) fn apply_aspect_preset(adjustments: &mut ImageAdjustments, img_aspect: f32, want: f32) {
    let frac = if img_aspect > 0.0 {
        want / img_aspect
    } else {
        1.0
    };
    let (w, h) = if frac <= 1.0 {
        (frac.clamp(0.01, 1.0), 1.0)
    } else {
        (1.0, (1.0 / frac).clamp(0.01, 1.0))
    };

    let rect = [(1.0 - w) * 0.5, (1.0 - h) * 0.5, w, h];
    adjustments.crop_rect = rect;
    adjustments.crop_rect_target = rect;
    adjustments.crop_rect_actual = Some(rect);
}

/// Screen-space rect of the crop selection, or `None` when there is nothing to
/// draw (no image yet, or a degenerate window/image size).
pub(crate) fn crop_overlay_rect(
    img_w: u32,
    img_h: u32,
    window_size: (u32, u32),
    crop_rect: [f32; 4],
    total_rotation: f32,
) -> Option<egui::Rect> {
    let (img_w, img_h) = (img_w as f32, img_h as f32);
    let (win_w, win_h) = (window_size.0 as f32, window_size.1 as f32);
    if win_w <= 0.0 || win_h <= 0.0 || img_w <= 0.0 || img_h <= 0.0 {
        return None;
    }

    let mut disp_w = img_w;
    let mut disp_h = img_h;
    let rot_deg = ((total_rotation.to_degrees() % 360.0).round()).abs();
    if rot_deg == 90.0 || rot_deg == 270.0 {
        std::mem::swap(&mut disp_w, &mut disp_h);
    }

    // Letterbox the (possibly rotated) image into the window.
    let (fit_w, fit_h) = if disp_w / disp_h > win_w / win_h {
        (win_w, win_w * (disp_h / disp_w))
    } else {
        (win_h * (disp_w / disp_h), win_h)
    };
    let ox = (win_w - fit_w) * 0.5;
    let oy = (win_h - fit_h) * 0.5;

    let [cx, cy, cw, ch] = crop_rect;
    Some(egui::Rect::from_min_max(
        egui::pos2(ox + cx * fit_w, oy + cy * fit_h),
        egui::pos2(ox + (cx + cw) * fit_w, oy + (cy + ch) * fit_h),
    ))
}

/// Dim everything outside the crop selection, outline the selection with rule
/// of thirds guides and 8 interactive corner/edge grips, and update the hover
/// cursor icon.
pub(super) fn draw_crop_overlay(params: &RenderParams, ui: &egui::Ui) {
    let Some((_, img_w, img_h, _, _)) = params.current_image_info else {
        return;
    };
    let adj: &ImageAdjustments = params.adjustments;
    let Some(rect_px) = crop_overlay_rect(
        img_w,
        img_h,
        params.window_size,
        adj.crop_rect,
        adj.rotation + adj.pre_rotation,
    ) else {
        return;
    };

    let ppp = ui.ctx().pixels_per_point();
    let rect = egui::Rect::from_min_max(rect_px.min / ppp, rect_px.max / ppp);
    let screen = egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(
            params.window_size.0 as f32 / ppp,
            params.window_size.1 as f32 / ppp,
        ),
    );
    let dim = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150);
    let accent = egui::Color32::from_rgb(0, 180, 216);
    let guide = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 75);

    let painter = ui.painter().clone();
    // Dim exactly the four bands around the selection without overlap or missed areas.
    let bands = [
        egui::Rect::from_min_max(screen.min, egui::pos2(screen.max.x, rect.min.y)),
        egui::Rect::from_min_max(egui::pos2(screen.min.x, rect.max.y), screen.max),
        egui::Rect::from_min_max(
            egui::pos2(screen.min.x, rect.min.y),
            egui::pos2(rect.min.x, rect.max.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(rect.max.x, rect.min.y),
            egui::pos2(screen.max.x, rect.max.y),
        ),
    ];
    for band in bands {
        painter.rect_filled(band, 0.0, dim);
    }

    // Rule-of-thirds guides inside the crop rect.
    let third_w = rect.width() / 3.0;
    let third_h = rect.height() / 3.0;
    for i in 1..=2 {
        let x = rect.left() + third_w * i as f32;
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(0.75_f32, guide),
        );
        let y = rect.top() + third_h * i as f32;
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.75_f32, guide),
        );
    }

    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5_f32, accent),
        egui::StrokeKind::Outside,
    );

    // 4 corner grips + 4 edge midpoint grips.
    let corner_grip = 7.0_f32;
    for corner in [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ] {
        painter.rect_filled(
            egui::Rect::from_center_size(corner, egui::vec2(corner_grip, corner_grip)),
            1.5,
            accent,
        );
    }
    for (center, size) in [
        (
            egui::pos2(rect.center().x, rect.top()),
            egui::vec2(16.0, 4.0),
        ),
        (
            egui::pos2(rect.center().x, rect.bottom()),
            egui::vec2(16.0, 4.0),
        ),
        (
            egui::pos2(rect.left(), rect.center().y),
            egui::vec2(4.0, 16.0),
        ),
        (
            egui::pos2(rect.right(), rect.center().y),
            egui::vec2(4.0, 16.0),
        ),
    ] {
        painter.rect_filled(egui::Rect::from_center_size(center, size), 1.0, accent);
    }

    // Set appropriate cursor icon when hovering handles or selection interior.
    if let Some(pos) = ui.ctx().pointer_hover_pos() {
        let r = 10.0_f32;
        let near_l = (pos.x - rect.left()).abs() <= r;
        let near_r = (pos.x - rect.right()).abs() <= r;
        let near_t = (pos.y - rect.top()).abs() <= r;
        let near_b = (pos.y - rect.bottom()).abs() <= r;
        let in_x = pos.x >= rect.left() - r && pos.x <= rect.right() + r;
        let in_y = pos.y >= rect.top() - r && pos.y <= rect.bottom() + r;

        let icon = if (near_l && near_t) || (near_r && near_b) {
            Some(egui::CursorIcon::ResizeNwSe)
        } else if (near_r && near_t) || (near_l && near_b) {
            Some(egui::CursorIcon::ResizeNeSw)
        } else if (near_t || near_b) && in_x {
            Some(egui::CursorIcon::ResizeVertical)
        } else if (near_l || near_r) && in_y {
            Some(egui::CursorIcon::ResizeHorizontal)
        } else if rect.contains(pos) {
            Some(egui::CursorIcon::Grab)
        } else {
            None
        };
        if let Some(cursor) = icon {
            ui.ctx().set_cursor_icon(cursor);
        }
    }

    let label = format!(
        "Crop  {:.0}×{:.0} px   ({:.0}% × {:.0}%)  —  Drag corners/edges to resize",
        img_w as f32 * adj.crop_rect[2],
        img_h as f32 * adj.crop_rect[3],
        adj.crop_rect[2] * 100.0,
        adj.crop_rect[3] * 100.0,
    );
    painter.text(
        rect.left_top() + egui::vec2(0.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        label,
        egui::FontId::monospace(12.0),
        egui::Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::types::ImageAdjustments;

    fn pixel_aspect(rect: [f32; 4], img_w: u32, img_h: u32) -> f32 {
        (rect[2] * img_w as f32) / (rect[3] * img_h as f32)
    }

    #[test]
    fn aspect_presets_produce_the_requested_pixel_ratio() {
        for &(img_w, img_h) in &[(3000u32, 2000u32), (2000, 3000), (1920, 1080), (1024, 1024)] {
            let img_aspect = img_w as f32 / img_h as f32;
            for &(label, want) in &[
                ("1:1", 1.0f32),
                ("16:9", 16.0 / 9.0),
                ("4:3", 4.0 / 3.0),
                ("3:2", 3.0 / 2.0),
                ("9:16", 9.0 / 16.0),
            ] {
                let mut adj = ImageAdjustments::default();
                apply_aspect_preset(&mut adj, img_aspect, want);

                let rect = adj.crop_rect;
                assert!(
                    (rect[2] - adj.crop_rect_target[2]).abs() < f32::EPSILON
                        && (rect[3] - adj.crop_rect_target[3]).abs() < f32::EPSILON,
                    "{label}: rect and target diverged"
                );
                assert_eq!(adj.crop_rect_actual, Some(rect), "{label}: not committed");
                assert!(
                    rect[0] >= -f32::EPSILON && rect[1] >= -f32::EPSILON,
                    "{label}"
                );
                assert!(
                    rect[0] + rect[2] <= 1.0 + f32::EPSILON
                        && rect[1] + rect[3] <= 1.0 + f32::EPSILON,
                    "{label}: {rect:?} overflows"
                );
                assert!(
                    (rect[0] - (1.0 - rect[2]) * 0.5).abs() < 1e-6
                        && (rect[1] - (1.0 - rect[3]) * 0.5).abs() < 1e-6,
                    "{label}: not centered"
                );

                let got = pixel_aspect(rect, img_w, img_h);
                if want > img_aspect {
                    assert!(rect[2] >= 0.999, "{label}: should fill width");
                } else {
                    assert!(
                        (got - want).abs() < 0.01,
                        "{label} on {img_w}x{img_h}: got {got}, want {want} (rect {rect:?})"
                    );
                }
            }
        }
    }

    #[test]
    fn crop_overlay_rect_letterboxes_and_maps_the_selection() {
        let rect =
            crop_overlay_rect(2000, 1000, (1000, 1000), [0.0, 0.0, 1.0, 1.0], 0.0).expect("rect");
        assert!((rect.width() - 1000.0).abs() < 1e-3);
        assert!((rect.height() - 500.0).abs() < 1e-3);
        assert!((rect.center().y - 500.0).abs() < 1e-3);

        let rect =
            crop_overlay_rect(2000, 1000, (1000, 1000), [0.0, 0.0, 0.5, 0.5], 0.0).expect("rect");
        assert!((rect.width() - 500.0).abs() < 1e-3);
        assert!((rect.height() - 250.0).abs() < 1e-3);

        let upright =
            crop_overlay_rect(2000, 1000, (1000, 1000), [0.0, 0.0, 1.0, 1.0], 0.0).expect("rect");
        let sideways = crop_overlay_rect(
            2000,
            1000,
            (1000, 1000),
            [0.0, 0.0, 1.0, 1.0],
            std::f32::consts::FRAC_PI_2,
        )
        .expect("rect");
        assert!(sideways.width() < upright.width());
        assert!(sideways.height() > upright.height());
    }

    #[test]
    fn crop_overlay_rect_rejects_degenerate_input() {
        assert!(crop_overlay_rect(0, 1000, (1000, 1000), [0.0; 4], 0.0).is_none());
        assert!(crop_overlay_rect(2000, 1000, (0, 1000), [0.0; 4], 0.0).is_none());
        assert!(crop_overlay_rect(2000, 1000, (1000, 0), [0.0; 4], 0.0).is_none());
    }
}
