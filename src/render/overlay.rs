use color_eyre::eyre::{Result, eyre};
use std::sync::Arc;
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp,
};

use super::renderer::Renderer;
use super::types::{RenderParams, STRIP_HEIGHT_PX, Uniforms};

impl Renderer {
    pub(crate) fn render_ui_static(
        params: &mut RenderParams,
        ctx: &egui::Context,
        has_thumbnails: bool,
        win_w: u32,
        win_h: u32,
    ) {
        if !params.has_image {
            if params.is_loading {
                egui::Area::new(egui::Id::new("loading_screen"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(egui::Spinner::new().size(50.0));
                            ui.add_space(15.0);
                            ui.label(
                                egui::RichText::new("Loading Image...")
                                    .size(24.0)
                                    .color(egui::Color32::WHITE),
                            );
                        });
                    });
            } else {
                egui::Window::new("Welcome to SpedImage")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(true)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("🖼️ SpedImage")
                                    .size(32.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Ultra-Lightweight GPU-Accelerated Image Viewer",
                                )
                                .size(14.0)
                                .italics()
                                .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.add_space(20.0);

                            if ui
                                .button(egui::RichText::new("📂 Open Image...").size(18.0))
                                .clicked()
                            {
                                crate::app::types::send_event(
                                    params.event_tx,
                                    params.event_proxy,
                                    crate::app::types::AppEvent::TriggerOpenFileDialog,
                                );
                            }

                            ui.add_space(15.0);
                            ui.label(
                                egui::RichText::new(
                                    "Drag & Drop an image here or press 'O' to open.",
                                )
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                            );
                            ui.add_space(20.0);

                            ui.separator();
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Common Keyboard Shortcuts:").strong());
                            ui.add_space(5.0);

                            egui::Grid::new("shortcuts_grid")
                                .striped(true)
                                .spacing(egui::vec2(40.0, 10.0))
                                .show(ui, |ui| {
                                    ui.label("O");
                                    ui.label("Open file dialog");
                                    ui.end_row();
                                    ui.label("F");
                                    ui.label("Toggle sidebar / edits");
                                    ui.end_row();
                                    ui.label("T");
                                    ui.label("Toggle thumbnails");
                                    ui.end_row();
                                    ui.label("H");
                                    ui.label("Toggle HDR Toning");
                                    ui.end_row();
                                    ui.label("Esc");
                                    ui.label("Quit application");
                                    ui.end_row();
                                });
                            ui.add_space(10.0);
                        });
                    });
            }
            return;
        }

        if params.is_loading {
            egui::Area::new(egui::Id::new("subtle_loading"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(20.0));
                        ui.label(
                            egui::RichText::new("Loading...")
                                .size(14.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    });
                });
        }
        let nav_y = if params.show_thumbnail_strip && has_thumbnails {
            (win_h as f32 - STRIP_HEIGHT_PX as f32) / 2.0
        } else {
            win_h as f32 / 2.0
        };

        egui::Area::new(egui::Id::new("nav_left"))
            .fixed_pos(egui::pos2(20.0, nav_y))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("◀")
                        .size(48.0)
                        .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)),
                );
            });

        egui::Area::new(egui::Id::new("nav_right"))
            .fixed_pos(egui::pos2(win_w as f32 - 60.0, nav_y))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("▶")
                        .size(48.0)
                        .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)),
                );
            });

        if let Some(status) = params.status_text
            && !status.is_empty()
        {
            egui::Window::new("Status")
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
                .title_bar(false)
                .auto_sized()
                .frame(
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(150))
                        .inner_margin(5.0),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(status)
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                    );
                });
        }

        if params.show_help {
            egui::Window::new("Shortcuts")
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 40.0))
                .title_bar(true)
                .show(ctx, |ui| {
                    ui.label("A/W: Prev Image");
                    ui.label("D/S: Next Image");
                    ui.label("R: Rotate");
                    ui.label("C: Toggle Crop");
                    ui.label("H: Toggle HDR");
                    ui.label("Ctrl+S: Save");
                    ui.label("F: Toggle Sidebar");
                    ui.label("T: Toggle Thumbnails");
                    ui.label("Esc: Quit");
                });
        }

        if let Some(exif) = params.exif_text {
            egui::Area::new(egui::Id::new("exif"))
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(exif)
                            .size(15.0)
                            .color(egui::Color32::from_rgb(210, 240, 255)),
                    );
                });
        }

        if params.config.show_sidebar {
            egui::Window::new("File Browser")
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
                .default_width(220.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
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
                });

            // Interactive Sliders for Adjustments
            egui::Window::new("Image Adjustments")
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 250.0))
                .default_width(220.0)
                .title_bar(true)
                .show(ctx, |ui| {
                    let mut changed = false;
                    if ui.add(egui::Slider::new(&mut params.adjustments.brightness, 0.1..=3.0).text("Brightness")).changed() {
                        changed = true;
                    }
                    if ui.add(egui::Slider::new(&mut params.adjustments.contrast, 0.1..=3.0).text("Contrast")).changed() {
                        changed = true;
                    }
                    if ui.add(egui::Slider::new(&mut params.adjustments.saturation, 0.0..=3.0).text("Saturation")).changed() {
                        changed = true;
                    }

                    let mut rot_deg = params.adjustments.rotation.to_degrees().round();
                    if ui.add(egui::Slider::new(&mut rot_deg, 0.0..=360.0).text("Rotation (°)")).changed() {
                        params.adjustments.rotation = rot_deg.to_radians();
                        changed = true;
                    }

                    ui.separator();
                    if ui.button("✂ Crop to Zoomed Area").clicked() {
                        params.adjustments.crop_rect_actual = Some(params.adjustments.crop_rect);
                        crate::app::types::send_event(
                            params.event_tx,
                            params.event_proxy,
                            crate::app::types::AppEvent::SetStatus("Crop applied! Save (Ctrl+S) to commit crop.".to_string()),
                        );
                    }
                    if params.adjustments.crop_rect_actual.is_some()
                        && ui.button("↩ Reset Crop").clicked()
                    {
                        params.adjustments.crop_rect_actual = None;
                        crate::app::types::send_event(
                            params.event_tx,
                            params.event_proxy,
                            crate::app::types::AppEvent::SetStatus("Crop reset".to_string()),
                        );
                    }
                    ui.separator();

                    if ui.button("Reset All").clicked() {
                        params.adjustments.brightness = 1.0;
                        params.adjustments.contrast = 1.0;
                        params.adjustments.saturation = 1.0;
                        params.adjustments.rotation = 0.0;
                        params.adjustments.crop_rect = [0.0, 0.0, 1.0, 1.0];
                        params.adjustments.crop_rect_target = [0.0, 0.0, 1.0, 1.0];
                        params.adjustments.crop_rect_actual = None;
                        changed = true;
                    }

                    if changed {
                        crate::app::types::send_event(
                            params.event_tx,
                            params.event_proxy,
                            crate::app::types::AppEvent::SetStatus(params.status_text.unwrap_or("").to_string()),
                        );
                    }

                    // Collapsible Slideshow Controls (Suggestion 7)
                    ui.add_space(10.0);
                    ui.separator();
                    ui.collapsing("Slideshow Controls", |ui| {
                        ui.horizontal(|ui| {
                            let play_label = if *params.slideshow_active { "⏸ Pause Slideshow" } else { "▶ Play Slideshow" };
                            if ui.button(play_label).clicked() {
                                *params.slideshow_active = !*params.slideshow_active;
                            }
                        });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("Interval:");
                            ui.add(egui::Slider::new(params.slideshow_interval_secs, 1..=15).suffix("s"));
                        });

                        if let Some(progress) = params.slideshow_progress {
                            ui.add_space(5.0);
                            ui.add(egui::ProgressBar::new(progress).text("Next slide"));
                        }
                    });

                    // Collapsible EXIF Inspector (Suggestion 9)
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
                                                if ui.button("📋").on_hover_text("Copy value to clipboard").clicked() {
                                                    ui.ctx().copy_text(val.to_string());
                                                }
                                                ui.end_row();
                                            } else {
                                                ui.label(line);
                                                ui.label("");
                                                if ui.button("📋").on_hover_text("Copy line to clipboard").clicked() {
                                                    ui.ctx().copy_text(line.to_string());
                                                }
                                                ui.end_row();
                                            }
                                        }
                                    });
                            });
                        });
                    }

                    // Collapsible Preferences Panel (Suggestion 10)
                    ui.add_space(5.0);
                    ui.collapsing("Preferences", |ui| {
                        let mut pref_changed = false;

                        if ui.checkbox(&mut params.config.show_sidebar, "Show Sidebar").changed() {
                            pref_changed = true;
                        }
                        if ui.checkbox(&mut params.config.show_thumbnail_strip, "Show Thumbnail Strip").changed() {
                            pref_changed = true;
                        }
                        if ui.checkbox(&mut params.config.show_info, "Show Info / EXIF").changed() {
                            pref_changed = true;
                        }
                        if ui.checkbox(&mut params.config.show_histogram, "Show RGB Histogram").changed() {
                            pref_changed = true;
                        }

                        ui.separator();

                        let mut scroll_to_zoom = params.config.scroll_to_zoom.unwrap_or(true);
                        if ui.checkbox(&mut scroll_to_zoom, "Scroll wheel zooms").on_hover_text("Uncheck to navigate next/prev using scroll wheel (Ctrl+Scroll will zoom)").changed() {
                            params.config.scroll_to_zoom = Some(scroll_to_zoom);
                            pref_changed = true;
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Default Width:");
                            if ui.add(egui::DragValue::new(&mut params.config.window_width).range(800..=3840)).changed() {
                                pref_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Default Height:");
                            if ui.add(egui::DragValue::new(&mut params.config.window_height).range(600..=2160)).changed() {
                                pref_changed = true;
                            }
                        });

                        ui.add_space(5.0);
                        let mut max_dim = params.config.max_preview_dimension.unwrap_or(0);
                        if ui.add(egui::Slider::new(&mut max_dim, 0..=4096).text("Max Preview Size (0=Auto)")).changed() {
                            params.config.max_preview_dimension = if max_dim > 0 { Some(max_dim) } else { None };
                            pref_changed = true;
                        }

                        if pref_changed {
                            params.config.save();
                            crate::app::types::send_event(
                                params.event_tx,
                                params.event_proxy,
                                crate::app::types::AppEvent::SetStatus(params.status_text.unwrap_or("").to_string()),
                            );
                        }
                    });
                });
        }

        if params.show_histogram
            && let Some((r_hist, g_hist, b_hist)) = params.histogram_data
        {
            egui::Window::new("RGB Histogram")
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
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
                                            crate::app::types::AppEvent::OpenPath(
                                                file.path.clone(),
                                            ),
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

    pub fn render_frame(&mut self, params: RenderParams) -> Result<()> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            e => return Err(eyre!("Failed to get current surface texture: {:?}", e)),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // Image pass
        {
            self.encode_image(
                params.adjustments,
                params.transition_factor,
                &view,
                &mut encoder,
            );
        }

        if params.show_thumbnail_strip && !self.thumbnails.is_empty() {
            self.encode_thumbnail_strip(
                params.active_thumb_idx,
                params.selected_indices,
                params.thumb_scroll,
                &view,
                &mut encoder,
            );
        }

        // egui pass
        let raw_input = self.egui_state.take_egui_input(&self._window);

        // Extract what we need for render_ui to avoid borrowing self in the closure
        let has_thumbnails = !self.thumbnails.is_empty();
        let win_w = self.config.width;
        let win_h = self.config.height;

        let mut params = params;
        let full_output = self.egui_state.egui_ctx().run_ui(raw_input, |ctx| {
            Self::render_ui_static(&mut params, ctx, has_thumbnails, win_w, win_h);
        });

        self.egui_state
            .handle_platform_output(&self._window, full_output.platform_output);

        let tris = self
            .egui_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.scale_factor as f32,
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );
        }

        for id in full_output.textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    pub fn render_loading(&mut self, path: Option<&std::path::Path>) -> Result<()> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            e => return Err(eyre!("Failed to get current surface texture: {:?}", e)),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Loading Encoder"),
            });

        // 1. Draw Placeholder if thumbnail exists
        if let Some(path) = path {
            if let Some(thumb) = self.thumbnails.iter().find(|t| t.path == path) {
                // Prepare uniforms for upscaled thumbnail
                let window_aspect_ratio = if self.config.height > 0 {
                    self.config.width as f32 / self.config.height as f32
                } else {
                    1.0
                };

                let uniforms = Uniforms {
                    rotation: 0.0,
                    aspect_ratio: thumb.width as f32 / thumb.height as f32,
                    window_aspect_ratio,
                    crop_x: 0.0,
                    crop_y: 0.0,
                    crop_w: 1.0,
                    crop_h: 1.0,
                    brightness: 0.8, // Dim it slightly
                    contrast: 1.0,
                    saturation: 0.5, // Desaturate to look like "loading"
                    hdr_toning: 0.0,
                    transition_factor: 1.0,
                    pos_offset: [0.0, 0.0],
                    pos_scale: [1.0, 1.0],
                };

                self.queue
                    .write_buffer(&thumb.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Placeholder Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(wgpu::Color::BLACK),
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_bind_group(0, thumb.bind_group.as_ref(), &[]);
                render_pass.draw(0..6, 0..1);
            }
        } else {
            // Just clear to dark gray
            let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Loading Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // 2. Draw egui spinner
        let raw_input = self.egui_state.take_egui_input(&self._window);
        let full_output = self.egui_state.egui_ctx().run_ui(raw_input, |ctx| {
            egui::Area::new(egui::Id::new("loader"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add(egui::Spinner::new().size(40.0));
                });
        });

        self.egui_state
            .handle_platform_output(&self._window, full_output.platform_output);
        let tris = self
            .egui_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.scale_factor as f32,
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui Loader Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );
        }

        for id in full_output.textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    pub fn preload_gif_textures(&mut self, frames: &[crate::image::ImageData]) -> Result<()> {
        for (tex, _, _) in self.gif_textures.drain(..) {
            tex.destroy();
        }
        let layout = self.pipeline.get_bind_group_layout(0);
        for frame in frames {
            let (width, height) = (frame.width, frame.height);
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("GIF Frame Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                frame.as_rgba(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GIF Frame Bind Group (Linear)"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            self.uniform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            }));

            let bind_group_nearest =
                Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("GIF Frame Bind Group (Nearest)"),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                self.uniform_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler_nearest),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                    ],
                }));

            self.gif_textures
                .push((texture, bind_group, bind_group_nearest));
        }
        Ok(())
    }
}
