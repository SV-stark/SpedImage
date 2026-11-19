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
                        egui::RichText::new(*name)
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
                    if ui
                        .small_button(if params.show_color_picker {
                            egui::RichText::new("✕ Pick")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(0, 210, 240))
                        } else {
                            egui::RichText::new("💧 Pick").size(10.0)
                        })
                        .on_hover_text("Toggle pixel color picker (P)")
                        .clicked()
                    {
                        crate::app::types::send_event(
                            params.event_tx,
                            params.event_proxy,
                            crate::app::types::AppEvent::ToggleColorPicker,
                        );
                    }
                    if let Some((frame_idx, total_frames, paused, is_animated)) =
                        params.animation_info
                    {
                        if is_animated
                            && ui
                                .small_button(
                                    egui::RichText::new(if paused { "▶" } else { "⏸" }).size(10.0),
                                )
                                .on_hover_text("Pause / Resume animation (Shift+Space)")
                                .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::ToggleAnimationPause,
                            );
                        }
                        if ui
                            .small_button(egui::RichText::new("⏮").size(10.0))
                            .on_hover_text("Previous frame / page (,)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::StepAnimationFramePrev,
                            );
                        }
                        if ui
                            .small_button(egui::RichText::new("⏭").size(10.0))
                            .on_hover_text("Next frame / page (.)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::StepAnimationFrame,
                            );
                        }
                        let prefix = if is_animated { "F" } else { "P" };
                        ui.label(
                            egui::RichText::new(format!(
                                "{prefix} {}/{}",
                                frame_idx + 1,
                                total_frames
                            ))
                            .size(10.0)
                            .color(egui::Color32::from_rgb(200, 180, 255)),
                        );
                    }
                    if params.adjustments.inspect_mode != crate::render::InspectMode::Normal {
                        ui.label(
                            egui::RichText::new(format!(
                                "• [{}]",
                                params.adjustments.inspect_mode.label()
                            ))
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(255, 190, 80)),
                        );
                    }
                });
            });
    }

    // Zoom Navigator Mini-Map: shown in bottom-right whenever zoomed in past 100%
    if !params.is_cropping
        && (params.adjustments.crop_rect[2] < 0.99 || params.adjustments.crop_rect[3] < 0.99)
        && let Some((_, w, h, _, zoom)) = params.current_image_info
        && w > 0
        && h > 0
    {
        let aspect = w as f32 / h as f32;
        let max_side = 112.0_f32;
        let (box_w, box_h) = if aspect >= 1.0 {
            (max_side, (max_side / aspect).max(28.0))
        } else {
            ((max_side * aspect).max(28.0), max_side)
        };
        let cr = params.adjustments.crop_rect;

        egui::Area::new(egui::Id::new("zoom_minimap"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-14.0, bottom_y))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(12, 14, 23, 210))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28),
                    ))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("NAVIGATOR")
                                        .size(9.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(0, 180, 216)),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{zoom:.0}%"))
                                        .size(9.0)
                                        .color(egui::Color32::from_rgb(180, 200, 220)),
                                );
                            });
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(box_w, box_h),
                                egui::Sense::click_and_drag(),
                            );
                            let painter = ui.painter();
                            painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(26, 30, 44));
                            painter.rect_stroke(
                                rect,
                                3.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(80)),
                                egui::StrokeKind::Inside,
                            );
                            let vp = egui::Rect::from_min_size(
                                egui::pos2(
                                    rect.min.x + cr[0].clamp(0.0, 1.0) * rect.width(),
                                    rect.min.y + cr[1].clamp(0.0, 1.0) * rect.height(),
                                ),
                                egui::vec2(
                                    (cr[2].clamp(0.02, 1.0) * rect.width()).max(6.0),
                                    (cr[3].clamp(0.02, 1.0) * rect.height()).max(6.0),
                                ),
                            )
                            .intersect(rect);
                            painter.rect_filled(
                                vp,
                                2.0,
                                egui::Color32::from_rgba_unmultiplied(0, 180, 216, 55),
                            );
                            painter.rect_stroke(
                                vp,
                                2.0,
                                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 210, 240)),
                                egui::StrokeKind::Inside,
                            );

                            if (resp.clicked() || resp.dragged())
                                && let Some(pos) = resp.interact_pointer_pos()
                            {
                                let u = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                                let v = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                                crate::app::types::send_event(
                                    params.event_tx,
                                    params.event_proxy,
                                    crate::app::types::AppEvent::PanToNormalized(u, v),
                                );
                            }
                        });
                    });
            });
    }

    // Live Pixel Color Picker floating loupe
    if let Some(sample) = params.color_sample {
        let ppp = ctx.pixels_per_point();
        let logical_w = params.window_size.0 as f32 / ppp;
        let logical_h = params.window_size.1 as f32 / ppp;
        let cx = sample.cursor_px.0 / ppp;
        let cy = sample.cursor_px.1 / ppp;
        let card_w = 185.0;
        let card_h = 68.0;
        let pos_x = if cx + 18.0 + card_w > logical_w {
            (cx - 18.0 - card_w).max(8.0)
        } else {
            cx + 18.0
        };
        let pos_y = if cy + 18.0 + card_h > logical_h {
            (cy - 18.0 - card_h).max(8.0)
        } else {
            cy + 18.0
        };

        let hex = sample.hex_string();
        let (h, s, l) = sample.hsl();
        egui::Area::new(egui::Id::new("color_picker_loupe"))
            .fixed_pos(egui::pos2(pos_x, pos_y))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(12, 14, 23, 230))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(0, 180, 216),
                    ))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (swatch_rect, _) = ui
                                .allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                swatch_rect,
                                4.0,
                                egui::Color32::from_rgba_unmultiplied(
                                    sample.r, sample.g, sample.b, sample.a,
                                ),
                            );
                            ui.painter().rect_stroke(
                                swatch_rect,
                                4.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(200)),
                                egui::StrokeKind::Inside,
                            );
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&hex)
                                            .monospace()
                                            .strong()
                                            .size(12.0)
                                            .color(egui::Color32::WHITE),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "({}, {})",
                                            sample.x, sample.y
                                        ))
                                        .monospace()
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(0, 200, 230)),
                                    );
                                });
                                ui.label(
                                    egui::RichText::new(format!(
                                        "rgb({}, {}, {})  hsl({}°, {}%, {}%)",
                                        sample.r, sample.g, sample.b, h, s, l
                                    ))
                                    .monospace()
                                    .size(9.5)
                                    .color(egui::Color32::from_rgb(185, 200, 220)),
                                );
                            });
                        });
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
