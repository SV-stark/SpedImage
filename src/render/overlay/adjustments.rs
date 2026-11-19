use super::super::types::RenderParams;
use super::crop::apply_aspect_preset;

pub(super) fn draw_adjustments(
    params: &mut RenderParams,
    ctx: &egui::Context,
    logical_h: f32,
    strip_h: f32,
) {
    if !params.config.show_adjustments {
        return;
    }

    let adj_y = if params.show_osd && params.current_image_info.is_some() {
        if params.status_text.is_some_and(|s| !s.is_empty()) {
            88.0
        } else {
            50.0
        }
    } else if params.status_text.is_some_and(|s| !s.is_empty()) {
        50.0
    } else {
        12.0
    };

    egui::Window::new("Image Adjustments")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, adj_y))
        .default_width(220.0)
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
            let mut close_adjustments = false;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("ADJUSTMENTS")
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 180, 216)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(egui::RichText::new("✕").size(12.0).strong())
                        .on_hover_text("Close adjustments (E or Esc)")
                        .clicked()
                    {
                        close_adjustments = true;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let max_scroll_h = (logical_h - adj_y - strip_h - 40.0).max(180.0);
            egui::ScrollArea::vertical()
                .max_height(max_scroll_h)
                .show(ui, |ui| {
                    let before = *params.adjustments;
                    let mut changed = false;
                    let mut record_undo = false;

                    // Quick-action bar: Undo / Redo / Auto-Enhance
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(params.can_undo, egui::Button::new("↶ Undo"))
                            .on_hover_text("Undo last edit (Ctrl+Z)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::UndoEdit,
                            );
                        }
                        if ui
                            .add_enabled(params.can_redo, egui::Button::new("↷ Redo"))
                            .on_hover_text("Redo edit (Ctrl+Y / Ctrl+Shift+Z)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::RedoEdit,
                            );
                        }
                        if ui
                            .button("✨ Auto")
                            .on_hover_text("Auto-Enhance levels & white balance (U)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::AutoEnhance,
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .small_button("📋 Copy Edits")
                            .on_hover_text("Copy adjustments to apply on another image (Ctrl+Alt+C)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::CopyAdjustments,
                            );
                        }
                        if ui
                            .add_enabled(
                                params.has_copied_edits,
                                egui::Button::new("📥 Paste Edits").small(),
                            )
                            .on_hover_text("Paste copied adjustments (Ctrl+Alt+V)")
                            .clicked()
                        {
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::PasteAdjustments,
                            );
                        }
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Group: Channel & Clipping Inspector (Pure WGSL)
                    ui.label(
                        egui::RichText::new("Channel Inspector (Shift+C)")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for mode in crate::render::InspectMode::ALL {
                            let selected = params.adjustments.inspect_mode == mode;
                            if ui.selectable_label(selected, mode.label()).clicked() && !selected {
                                params.adjustments.inspect_mode = mode;
                                changed = true;
                            }
                        }
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let mut track_slider = |resp: egui::Response| {
                        if resp.drag_started() {
                            record_undo = true;
                        }
                        if resp.changed() {
                            changed = true;
                        }
                    };

                    // Group: Exposure & Toning
                    ui.label(
                        egui::RichText::new("Tone & Exposure")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.brightness, 0.1..=3.0)
                            .text("Brightness"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.contrast, 0.1..=3.0)
                            .text("Contrast"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.highlights, -1.0..=1.0)
                            .text("Highlights"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.shadows, -1.0..=1.0)
                            .text("Shadows"),
                    ));

                    ui.add_space(4.0);
                    // Group: Color & White Balance
                    ui.label(
                        egui::RichText::new("Color & White Balance")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.saturation, 0.0..=3.0)
                            .text("Saturation"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.temperature, -1.0..=1.0)
                            .text("Temperature"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.tint, -1.0..=1.0).text("Tint"),
                    ));

                    ui.add_space(4.0);
                    // Group: Detail & Clarity
                    ui.label(
                        egui::RichText::new("Detail & Clarity")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.sharpen, 0.0..=2.0)
                            .text("Sharpen"),
                    ));
                    track_slider(ui.add(
                        egui::Slider::new(&mut params.adjustments.clarity, -1.0..=1.0)
                            .text("Clarity"),
                    ));

                    ui.add_space(4.0);
                    // Group: Comparison
                    ui.label(
                        egui::RichText::new("A/B Comparison")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    if ui
                        .checkbox(
                            &mut params.adjustments.split_compare,
                            "Split View (Before/After)",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if params.adjustments.split_compare
                        && ui
                            .add(
                                egui::Slider::new(&mut params.adjustments.split_position, 0.0..=1.0)
                                    .text("Split Line"),
                            )
                            .changed()
                    {
                        changed = true;
                    }

                    ui.add_space(4.0);
                    // Group: Transform
                    ui.label(
                        egui::RichText::new("Transform")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    let mut rot_deg = params.adjustments.rotation.to_degrees().round();
                    let rot_resp = ui.add(
                        egui::Slider::new(&mut rot_deg, 0.0..=360.0).text("Rotation (°)"),
                    );
                    if rot_resp.drag_started() {
                        record_undo = true;
                    }
                    if rot_resp.changed() {
                        params.adjustments.rotation = rot_deg.to_radians();
                        changed = true;
                    }

                    ui.horizontal(|ui| {
                        if ui
                            .checkbox(&mut params.adjustments.flip_horizontal, "Flip H")
                            .changed()
                        {
                            record_undo = true;
                            changed = true;
                        }
                        if ui
                            .checkbox(&mut params.adjustments.flip_vertical, "Flip V")
                            .changed()
                        {
                            record_undo = true;
                            changed = true;
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Aspect Ratio Presets")
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 200, 220)),
                    );
                    ui.horizontal(|ui| {
                        let img_aspect = params
                            .current_image_info
                            .as_ref()
                            .map(|(_, w, h, _, _)| {
                                if *w > 0 && *h > 0 {
                                    *w as f32 / *h as f32
                                } else {
                                    1.0
                                }
                            })
                            .unwrap_or(1.0);
                        let presets: [(&str, f32); 5] = [
                            ("1:1", 1.0),
                            ("16:9", 16.0 / 9.0),
                            ("4:3", 4.0 / 3.0),
                            ("3:2", 3.0 / 2.0),
                            ("9:16", 9.0 / 16.0),
                        ];
                        for (label, want) in presets {
                            if ui.button(label).clicked() {
                                record_undo = true;
                                apply_aspect_preset(params.adjustments, img_aspect, want);
                                changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("✂ Crop to View").clicked() {
                            record_undo = true;
                            params.adjustments.crop_rect_actual = Some(params.adjustments.crop_rect);
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::SetStatus(
                                    "Crop applied! Save (Ctrl+S) to commit crop.".to_string(),
                                ),
                            );
                        }
                        if params.adjustments.crop_rect_actual.is_some()
                            && ui.button("↩ Reset Crop").clicked()
                        {
                            record_undo = true;
                            params.adjustments.crop_rect_actual = None;
                            params.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
                            params.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::SetStatus("Crop reset".to_string()),
                            );
                        }
                    });

                    ui.separator();
                    if ui.button("↺ Reset All Adjustments").clicked() {
                        record_undo = true;
                        params.adjustments.reset_user_edits();
                        changed = true;
                    }

                    if record_undo {
                        crate::app::types::send_event(
                            params.event_tx,
                            params.event_proxy,
                            crate::app::types::AppEvent::RecordEditSnapshot(Box::new(before)),
                        );
                    }
                    if changed {
                        let _ = params.event_proxy.send_event(crate::app::types::WakeUp);
                    }

                    // Collapsible Named Presets (Built-in + User Saved)
                    ui.add_space(6.0);
                    ui.separator();
                    ui.collapsing("Presets", |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for preset in crate::config::AdjustmentPreset::built_ins() {
                                if ui.small_button(&preset.name).clicked() {
                                    crate::app::types::send_event(
                                        params.event_tx,
                                        params.event_proxy,
                                        crate::app::types::AppEvent::RecordEditSnapshot(Box::new(
                                            *params.adjustments,
                                        )),
                                    );
                                    preset.apply_to(params.adjustments);
                                    let _ =
                                        params.event_proxy.send_event(crate::app::types::WakeUp);
                                }
                            }
                        });
                        if !params.config.custom_presets.is_empty() {
                            ui.add_space(4.0);
                            let mut remove_idx: Option<usize> = None;
                            for (i, preset) in params.config.custom_presets.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    if ui.small_button(&preset.name).clicked() {
                                        crate::app::types::send_event(
                                            params.event_tx,
                                            params.event_proxy,
                                            crate::app::types::AppEvent::RecordEditSnapshot(
                                                Box::new(*params.adjustments),
                                            ),
                                        );
                                        preset.apply_to(params.adjustments);
                                        let _ = params
                                            .event_proxy
                                            .send_event(crate::app::types::WakeUp);
                                    }
                                    if ui
                                        .small_button("✕")
                                        .on_hover_text("Delete preset")
                                        .clicked()
                                    {
                                        remove_idx = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = remove_idx {
                                params.config.custom_presets.remove(i);
                                params.config.save();
                            }
                        }
                        ui.add_space(4.0);
                        if ui
                            .small_button("＋ Save Current as Preset")
                            .on_hover_text("Save current sliders to config.json")
                            .clicked()
                        {
                            let idx = params.config.custom_presets.len() + 1;
                            let name = format!("Custom {idx}");
                            params.config.custom_presets.push(
                                crate::config::AdjustmentPreset::from_adjustments(
                                    &name,
                                    params.adjustments,
                                ),
                            );
                            params.config.save();
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::SetStatus(format!(
                                    "Saved preset '{name}'"
                                )),
                            );
                        }
                    });

                    // Collapsible Slideshow Controls
                    ui.add_space(6.0);
                    ui.separator();
                    ui.collapsing("Slideshow Controls", |ui| {
                        ui.horizontal(|ui| {
                            let play_label = if *params.slideshow_active {
                                "⏸ Pause Slideshow"
                            } else {
                                "▶ Play Slideshow"
                            };
                            if ui.button(play_label).clicked() {
                                *params.slideshow_active = !*params.slideshow_active;
                            }
                        });

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Interval:");
                            ui.add(
                                egui::Slider::new(params.slideshow_interval_secs, 1..=15)
                                    .suffix("s"),
                            );
                        });

                        if let Some(progress) = params.slideshow_progress {
                            ui.add_space(5.0);
                            ui.add(egui::ProgressBar::new(progress).text("Next slide"));
                        }
                    });

                    // Collapsible EXIF Inspector
                    if let Some(exif) = params.exif_text {
                        ui.add_space(5.0);
                        ui.collapsing("EXIF Metadata", |ui| {
                            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                                egui::Grid::new("exif_grid_sidebar")
                                    .striped(true)
                                    .spacing(egui::vec2(10.0, 5.0))
                                    .show(ui, |ui| {
                                        for line in exif.lines() {
                                            if let Some((key, val)) = line.split_once(": ") {
                                                ui.label(egui::RichText::new(key).strong());
                                                ui.label(val);
                                                if ui
                                                    .button("📋")
                                                    .on_hover_text("Copy value to clipboard")
                                                    .clicked()
                                                {
                                                    ui.ctx().copy_text(val.to_string());
                                                }
                                                ui.end_row();
                                            } else {
                                                ui.label(line);
                                                ui.label("");
                                                if ui
                                                    .button("📋")
                                                    .on_hover_text("Copy line to clipboard")
                                                    .clicked()
                                                {
                                                    ui.ctx().copy_text(line.to_string());
                                                }
                                                ui.end_row();
                                            }
                                        }
                                    });
                            });
                        });
                    }

                    // Collapsible Preferences & Export Settings
                    ui.add_space(5.0);
                    ui.collapsing("Preferences & Export", |ui| {
                        let mut pref_changed = false;

                        let mut jpeg_q = params.config.jpeg_quality();
                        if ui
                            .add(egui::Slider::new(&mut jpeg_q, 50..=100).text("JPEG Quality"))
                            .on_hover_text("Quality used when saving re-encoded JPEG images (Ctrl+S / Ctrl+Shift+S)")
                            .changed()
                        {
                            params.config.jpeg_quality = Some(jpeg_q);
                            pref_changed = true;
                        }

                        let mut strip_meta = params.config.strip_metadata_enabled();
                        if ui
                            .checkbox(&mut strip_meta, "Strip EXIF & GPS on Save (Privacy)")
                            .on_hover_text("Remove camera EXIF, GPS location, and ICC chunks when saving (works even with lossless JPEG rotation)")
                            .changed()
                        {
                            params.config.strip_metadata_on_save = Some(strip_meta);
                            pref_changed = true;
                        }

                        ui.horizontal(|ui| {
                            ui.label("Export Max Dim (px):");
                            let mut max_d = params.config.export_max_dimension.unwrap_or(0);
                            if ui
                                .add(
                                    egui::DragValue::new(&mut max_d)
                                        .range(0..=16384)
                                        .speed(64.0)
                                        .suffix(" px"),
                                )
                                .on_hover_text("Downscale longest edge on save via SIMD fast_image_resize (0 = keep original size)")
                                .changed()
                            {
                                params.config.export_max_dimension = (max_d > 0).then_some(max_d);
                                pref_changed = true;
                            }
                        });

                        ui.separator();

                        if ui
                            .checkbox(&mut params.config.show_sidebar, "Show File Browser (F)")
                            .changed()
                        {
                            pref_changed = true;
                        }
                        if ui
                            .checkbox(&mut params.config.show_adjustments, "Show Adjustments (E)")
                            .changed()
                        {
                            pref_changed = true;
                        }
                        if ui
                            .checkbox(&mut params.config.show_thumbnail_strip, "Show Thumbnail Strip")
                            .changed()
                        {
                            pref_changed = true;
                        }
                        if ui
                            .checkbox(&mut params.config.show_info, "Show Info / EXIF")
                            .changed()
                        {
                            pref_changed = true;
                        }
                        if ui
                            .checkbox(&mut params.config.show_histogram, "Show RGB Histogram")
                            .changed()
                        {
                            pref_changed = true;
                        }

                        let mut show_osd = params.config.osd_enabled();
                        if ui.checkbox(&mut show_osd, "Show OSD Info Bar (Tab)").changed() {
                            params.config.show_osd = Some(show_osd);
                            pref_changed = true;
                        }

                        let mut checkerboard = params.config.checkerboard_enabled();
                        if ui
                            .checkbox(&mut checkerboard, "Transparency Checkerboard")
                            .changed()
                        {
                            params.config.transparency_checkerboard = Some(checkerboard);
                            pref_changed = true;
                        }

                        let mut light_canvas = params.config.light_canvas_enabled();
                        if ui
                            .checkbox(&mut light_canvas, "Light Canvas Background")
                            .on_hover_text("Use a light neutral canvas behind images (useful for inspecting dark transparent icons/PNGs)")
                            .changed()
                        {
                            params.config.light_canvas = Some(light_canvas);
                            pref_changed = true;
                        }

                        let mut match_display = params.config.display_profile_enabled();
                        if ui
                            .checkbox(&mut match_display, "Match monitor colour profile")
                            .on_hover_text("Convert colours to the ICC profile Windows has assigned to this monitor. Matters on wide-gamut screens; does nothing on sRGB ones.")
                            .changed()
                        {
                            params.config.match_display_profile = Some(match_display);
                            pref_changed = true;
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::DisplayProfileToggled,
                            );
                        }

                        ui.separator();

                        let mut scroll_to_zoom = params.config.scroll_to_zoom.unwrap_or(true);
                        if ui
                            .checkbox(&mut scroll_to_zoom, "Scroll wheel zooms")
                            .on_hover_text("Uncheck to navigate next/prev using scroll wheel (Ctrl+Scroll will zoom)")
                            .changed()
                        {
                            params.config.scroll_to_zoom = Some(scroll_to_zoom);
                            pref_changed = true;
                        }

                        let mut double_click_zoom = params.config.double_click_zoom_enabled();
                        if ui
                            .checkbox(&mut double_click_zoom, "Double-click toggles 100% zoom")
                            .on_hover_text("Uncheck to toggle Fullscreen on double-click")
                            .changed()
                        {
                            params.config.double_click_zoom = Some(double_click_zoom);
                            pref_changed = true;
                        }

                        let mut confirm_del = params.config.confirm_delete_enabled();
                        if ui
                            .checkbox(&mut confirm_del, "Confirm before Recycle Bin")
                            .on_hover_text("When unchecked, pressing Delete instantly moves to Recycle Bin")
                            .changed()
                        {
                            params.config.confirm_delete = Some(confirm_del);
                            pref_changed = true;
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Default Width:");
                            if ui
                                .add(egui::DragValue::new(&mut params.config.window_width).range(800..=3840))
                                .changed()
                            {
                                pref_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Default Height:");
                            if ui
                                .add(egui::DragValue::new(&mut params.config.window_height).range(600..=2160))
                                .changed()
                            {
                                pref_changed = true;
                            }
                        });

                        ui.add_space(5.0);
                        let mut max_dim = params.config.max_preview_dimension.unwrap_or(0);
                        if ui
                            .add(egui::Slider::new(&mut max_dim, 0..=4096).text("Max Preview Size (0=Auto)"))
                            .changed()
                        {
                            params.config.max_preview_dimension =
                                if max_dim > 0 { Some(max_dim) } else { None };
                            pref_changed = true;
                        }

                        if pref_changed {
                            params.config.save();
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::SetStatus(
                                    params.status_text.unwrap_or("").to_string(),
                                ),
                            );
                        }
                    });
                });

            if close_adjustments {
                params.config.show_adjustments = false;
                params.config.save();
                crate::app::types::send_event(
                    params.event_tx,
                    params.event_proxy,
                    crate::app::types::AppEvent::SetStatus("Adjustments closed".to_string()),
                );
            }
        });
}
