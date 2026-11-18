use super::super::types::RenderParams;

pub(super) fn draw_histogram(params: &RenderParams, ctx: &egui::Context, bottom_y: f32) {
    if !params.show_histogram {
        return;
    }
    let Some((r_hist, g_hist, b_hist)) = params.histogram_data else {
        return;
    };

    egui::Window::new("RGB Histogram")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, bottom_y))
        .title_bar(true)
        .resizable(false)
        .default_width(280.0)
        .default_height(140.0)
        .show(ctx, |ui| {
            let width = 256.0;
            let height = 100.0;
            let (rect, _response) =
                ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 4.0, egui::Color32::from_black_alpha(150));

            let mut max_val = 1u32;
            for i in 0..256 {
                max_val = max_val.max(r_hist[i]).max(g_hist[i]).max(b_hist[i]);
            }

            let draw_channel = |hist: &[u32; 256], color: egui::Color32| {
                let mut points = Vec::with_capacity(256);
                for (i, &val) in hist.iter().enumerate() {
                    let x = rect.left() + (i as f32 / 255.0) * width;
                    let y = rect.bottom() - (val as f32 / max_val as f32) * height;
                    points.push(egui::pos2(x, y));
                }
                for j in 0..255 {
                    painter.line_segment(
                        [points[j], points[j + 1]],
                        egui::Stroke::new(1.5_f32, color),
                    );
                }
            };

            draw_channel(
                r_hist,
                egui::Color32::from_rgba_unmultiplied(255, 100, 100, 180),
            );
            draw_channel(
                g_hist,
                egui::Color32::from_rgba_unmultiplied(100, 255, 100, 180),
            );
            draw_channel(
                b_hist,
                egui::Color32::from_rgba_unmultiplied(100, 100, 255, 180),
            );
        });
}
