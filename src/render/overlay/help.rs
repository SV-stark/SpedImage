pub(super) fn draw_help_modal(ctx: &egui::Context, logical_h: f32) {
    egui::Window::new("Keyboard Shortcuts")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_width(480.0)
        .frame(
            egui::Frame::window(ctx.global_style().as_ref())
                .fill(egui::Color32::from_rgb(12, 14, 23))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25),
                ))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::same(16)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("KEYBOARD SHORTCUTS")
                        .size(12.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 180, 216)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Press ? / F1 / Esc to close")
                            .size(10.0)
                            .color(egui::Color32::GRAY),
                    );
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);

            let section = |ui: &mut egui::Ui, title: &str, rows: &[(&str, &str)]| {
                ui.label(
                    egui::RichText::new(title)
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(180, 200, 220)),
                );
                ui.add_space(2.0);
                egui::Grid::new(title)
                    .striped(true)
                    .num_columns(2)
                    .spacing(egui::vec2(24.0, 4.0))
                    .show(ui, |ui| {
                        for &(key, desc) in rows {
                            ui.label(
                                egui::RichText::new(key)
                                    .monospace()
                                    .strong()
                                    .color(egui::Color32::from_rgb(0, 200, 230)),
                            );
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
            };

            egui::ScrollArea::vertical()
                .max_height((logical_h - 140.0).max(240.0))
                .show(ui, |ui| {
                    section(
                        ui,
                        "Navigation & Viewing",
                        &[
                            ("A / W / ◀ / Backspace", "Previous image"),
                            ("D / S / ▶", "Next image"),
                            ("PgUp / PgDown", "Previous / Next image"),
                            ("Home / End", "First / Last image"),
                            ("Enter", "Toggle 100% zoom / Fit"),
                            ("Z", "Toggle pixel-perfect (nearest) sampling"),
                            ("F11 / Double-Click", "Toggle Fullscreen (or 100% zoom)"),
                            ("K", "Pin & compare side by side"),
                        ],
                    );
                    section(
                        ui,
                        "Panels & Overlays",
                        &[
                            ("F", "Toggle File Browser"),
                            ("E", "Toggle Image Adjustments"),
                            ("T", "Toggle Thumbnail Strip"),
                            ("I", "Toggle EXIF Metadata Info"),
                            ("Shift + H", "Toggle RGB Histogram"),
                            ("P", "Toggle Pixel Color Picker (click to copy hex)"),
                            ("Tab", "Toggle OSD Info Bar"),
                            ("Ctrl + F", "Find File in folder"),
                            ("? / F1", "Toggle this Cheat Sheet"),
                        ],
                    );
                    section(
                        ui,
                        "Editing, Animation & File Operations",
                        &[
                            ("R / Shift + R", "Rotate 90° clockwise / counter-clockwise"),
                            ("X / Y", "Flip horizontal / Flip vertical"),
                            ("U", "Auto-Enhance levels & white balance"),
                            (
                                "Ctrl + Z / Ctrl + Y",
                                "Undo / Redo edit (also Ctrl+Shift+Z)",
                            ),
                            (
                                "Ctrl + Alt + C / V",
                                "Copy / Paste adjustments across images",
                            ),
                            (
                                "Shift + C",
                                "Cycle Channel / Clipping Inspector (R/G/B/A/Zebra)",
                            ),
                            ("C", "Toggle Crop mode (drag corners/edges)"),
                            ("H", "Toggle HDR Toning"),
                            ("1", "Reset all adjustments"),
                            ("Shift + Space", "Pause / Resume animated GIF or WebP"),
                            (", / .", "Step previous / next animation frame or TIFF page"),
                            ("O", "Open file dialog"),
                            ("Ctrl + S", "Save edited copy"),
                            ("Ctrl + Shift + S", "Save As / Batch save selected"),
                            ("Ctrl + C / Ctrl + V", "Copy / Paste image"),
                            ("Ctrl + Shift + C", "Copy file path"),
                            ("F2 / Delete", "Rename / Move to Recycle Bin"),
                            ("Esc", "Close active overlay or Quit"),
                        ],
                    );
                });
        });
}
