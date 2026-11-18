use super::super::types::RenderParams;

pub(super) fn draw_sidebar_and_search(
    params: &mut RenderParams,
    ctx: &egui::Context,
    logical_h: f32,
    strip_h: f32,
) {
    if params.config.show_sidebar {
        egui::Window::new("File Browser")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
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
                let mut close_sidebar = false;
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("FILE BROWSER")
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 180, 216)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("✕").size(12.0).strong())
                            .on_hover_text("Close file browser (F or Esc)")
                            .clicked()
                        {
                            close_sidebar = true;
                        }
                    });
                });
                ui.add_space(6.0);

                let mut sort_changed = false;
                ui.horizontal(|ui| {
                    let mut key = params.config.sort_key();
                    egui::ComboBox::from_id_salt("sort_key")
                        .selected_text(format!("Sort: {}", key.label()))
                        .show_ui(ui, |ui| {
                            for k in crate::config::SortKey::ALL {
                                if ui.selectable_value(&mut key, k, k.label()).changed() {
                                    sort_changed = true;
                                }
                            }
                        });
                    if sort_changed {
                        params.config.sort_key = Some(key);
                    }
                    let descending = params.config.sort_descending();
                    let (arrow, hint) = if descending {
                        ("⬇", "Descending (click for ascending)")
                    } else {
                        ("⬆", "Ascending (click for descending)")
                    };
                    if ui.button(arrow).on_hover_text(hint).clicked() {
                        params.config.sort_descending = Some(!descending);
                        sort_changed = true;
                    }
                });
                if sort_changed {
                    params.config.save();
                    crate::app::types::send_event(
                        params.event_tx,
                        params.event_proxy,
                        crate::app::types::AppEvent::SortChanged,
                    );
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .max_height((logical_h - strip_h - 100.0).clamp(200.0, 450.0))
                    .show(ui, |ui| {
                        for (idx, file) in params.files.iter().enumerate() {
                            let is_selected = Some(idx) == params.active_thumb_idx;
                            let text = if is_selected {
                                format!("> {}", file.name)
                            } else {
                                format!("  {}", file.name)
                            };
                            if ui.selectable_label(is_selected, text).clicked() {
                                crate::app::types::send_event(
                                    params.event_tx,
                                    params.event_proxy,
                                    crate::app::types::AppEvent::OpenPath(file.path.clone()),
                                );
                            }
                        }
                    });

                if close_sidebar {
                    params.config.show_sidebar = false;
                    params.config.save();
                    crate::app::types::send_event(
                        params.event_tx,
                        params.event_proxy,
                        crate::app::types::AppEvent::SetStatus("File browser closed".to_string()),
                    );
                }
            });
    }

    if *params.show_search {
        egui::Window::new("🔍 Find File")
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
            .collapsible(false)
            .resizable(true)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let text_edit = ui.text_edit_singleline(params.search_query);
                    text_edit.request_focus();
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                let query = params.search_query.to_lowercase();
                let filtered: Vec<(usize, &crate::ui::FileEntry)> = params
                    .files
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.name.to_lowercase().contains(&query))
                    .collect();

                if filtered.is_empty() {
                    ui.label(
                        egui::RichText::new("No files found.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for (idx, file) in &filtered {
                                let is_selected = Some(*idx) == params.active_thumb_idx;
                                let label_text = if is_selected {
                                    format!("> {}", file.name)
                                } else {
                                    format!("  {}", file.name)
                                };

                                if ui.selectable_label(is_selected, label_text).clicked() {
                                    crate::app::types::send_event(
                                        params.event_tx,
                                        params.event_proxy,
                                        crate::app::types::AppEvent::OpenPath(file.path.clone()),
                                    );
                                    *params.show_search = false;
                                }
                            }
                        });
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *params.show_search = false;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !filtered.is_empty() {
                    let first_file = filtered[0].1;
                    crate::app::types::send_event(
                        params.event_tx,
                        params.event_proxy,
                        crate::app::types::AppEvent::OpenPath(first_file.path.clone()),
                    );
                    *params.show_search = false;
                }
            });
    }
}
