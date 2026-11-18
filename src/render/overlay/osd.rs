use super::super::types::RenderParams;

pub(super) fn draw_status_and_osd(params: &mut RenderParams, ctx: &egui::Context, bottom_y: f32) {
    if let Some(status) = params.status_text
        && !status.is_empty()
    {
        egui::Window::new("Status")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 50.0))
            .title_bar(false)
            .auto_sized()
            .frame(
                egui::Frame::window(ctx.global_style().as_ref())
                    .fill(egui::Color32::from_rgba_unmultiplied(12, 14, 23, 210))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(0, 180, 216),
                    ))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(status)
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                );
            });
    }

    // On-Screen Display (OSD) HUD pill: filename, index, dimensions, file size, zoom %
    if params.show_osd
        && let Some((name, w, h, size_bytes, zoom)) = &params.current_image_info
    {
        let total = params.files.len();
        let idx_str = params
            .active_thumb_idx
            .map(|i| format!("{}/{}", i + 1, total))
            .unwrap_or_default();
        let size_str = if *size_bytes >= 1024 * 1024 {
            format!("{:.1} MB", *size_bytes as f64 / (1024.0 * 1024.0))
        } else if *size_bytes >= 1024 {
            format!("{} KB", *size_bytes / 1024)
        } else if *size_bytes > 0 {
            format!("{} B", *size_bytes)
        } else {
            String::new()
        };

        egui::Window::new("osd_overlay")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .frame(
                egui::Frame::window(ctx.global_style().as_ref())
                    .fill(egui::Color32::from_rgba_unmultiplied(12, 14, 23, 200))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                    ))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(10, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    if !idx_str.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("• [{}]", idx_str))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(0, 180, 216)),
                        );
                    }
                    ui.label(
                        egui::RichText::new(format!("• {}×{}", w, h))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(180, 195, 215)),
                    );
                    if !size_str.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("• {}", size_str))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(150, 165, 185)),
                        );
                    }
                    ui.label(
                        egui::RichText::new(format!("• {:.0}%", zoom))
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(100, 220, 150)),
                    );
                    if ui
                        .small_button(if params.config.show_adjustments {
                            egui::RichText::new("✕ Edit").size(10.0)
                        } else {
                            egui::RichText::new("🎛 Edit").size(10.0)
                        })
                        .on_hover_text("Toggle adjustments (E)")
                        .clicked()
                    {
                        params.config.show_adjustments = !params.config.show_adjustments;
                        params.config.save();
                    }
                    if ui
                        .small_button(if params.config.show_sidebar {
                            egui::RichText::new("✕ Files").size(10.0)
                        } else {
                            egui::RichText::new("📁 Files").size(10.0)
                        })
                        .on_hover_text("Toggle file browser (F)")
                        .clicked()
                    {
                        params.config.show_sidebar = !params.config.show_sidebar;
                        params.config.save();
                    }
                });
            });
    }

    if let Some(exif) = params.exif_text {
        egui::Window::new("Metadata Info")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, bottom_y))
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .frame(
                egui::Frame::window(ctx.global_style().as_ref())
                    .fill(egui::Color32::from_rgb(12, 14, 23))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15),
                    ))
                    .corner_radius(12.0)
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("IMAGE METADATA")
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 180, 216)),
                );
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                for line in exif.lines() {
                    ui.label(
                        egui::RichText::new(line).color(egui::Color32::from_rgb(220, 225, 235)),
                    );
                }

                if let Some((lat, lon)) = params.gps_coords {
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if ui.button("📍 Open in Maps").clicked() {
                        let url = format!(
                            "https://www.google.com/maps/search/?api=1&query={},{}",
                            lat, lon
                        );
                        let _ = webbrowser::open(&url);
                    }
                }
            });
    }
}
