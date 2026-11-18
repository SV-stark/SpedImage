mod adjustments;
mod crop;
mod help;
mod histogram;
mod osd;
mod sidebar;

pub(crate) use crop::crop_overlay_rect;

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
        // Persistent notice (e.g. a damaged settings file). Drawn before the
        // early return below so it also shows on the welcome screen.
        if let Some(text) = params.notice.clone() {
            egui::Window::new("⚠ Notice")
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0))
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.label(text);
                    ui.add_space(6.0);
                    if ui.button("OK").clicked() {
                        *params.notice = None;
                    }
                });
        }

        // Overlay UI rendering
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
                                    ui.label("F / E");
                                    ui.label("Toggle file browser / adjustments");
                                    ui.end_row();
                                    ui.label("T");
                                    ui.label("Toggle thumbnails");
                                    ui.end_row();
                                    ui.label("? / F1");
                                    ui.label("All keyboard shortcuts");
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

        let ppp = ctx.pixels_per_point();
        let logical_w = win_w as f32 / ppp;
        let logical_h = win_h as f32 / ppp;
        let strip_h = if params.show_thumbnail_strip && has_thumbnails {
            STRIP_HEIGHT_PX as f32 / ppp
        } else {
            0.0
        };
        let bottom_y = -12.0 - strip_h;

        if params.is_loading {
            egui::Area::new(egui::Id::new("subtle_loading"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, bottom_y))
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
        } else if params.is_refining {
            egui::Area::new(egui::Id::new("refining_hint"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, bottom_y))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(16.0));
                        ui.label(
                            egui::RichText::new("Loading full resolution…")
                                .size(13.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    });
                });
        }

        if let Some((pinned, current)) = &params.compare_labels {
            let half = logical_w / 2.0;
            ctx.layer_painter(egui::LayerId::new(
                egui::Order::Background,
                egui::Id::new("compare_divider"),
            ))
            .line_segment(
                [egui::pos2(half, 0.0), egui::pos2(half, logical_h)],
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(0, 180, 216)),
            );
            for (id, text, dx) in [
                ("compare_left", format!("📌 {pinned}"), -half / 2.0),
                ("compare_right", current.clone(), half / 2.0),
            ] {
                egui::Area::new(egui::Id::new(id))
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(dx, 84.0))
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(12, 14, 23, 200))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(text)
                                            .size(12.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .extend(),
                                );
                            });
                    });
            }
        }

        let nav_y = (logical_h - strip_h) / 2.0 - 24.0;
        egui::Area::new(egui::Id::new("nav_left"))
            .fixed_pos(egui::pos2(20.0, nav_y))
            .show(ctx, |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("◀")
                                .size(48.0)
                                .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Previous image (◀ / A)")
                    .clicked()
                {
                    crate::app::types::send_event(
                        params.event_tx,
                        params.event_proxy,
                        crate::app::types::AppEvent::PrevImage,
                    );
                }
            });

        egui::Area::new(egui::Id::new("nav_right"))
            .fixed_pos(egui::pos2((logical_w - 60.0).max(20.0), nav_y))
            .show(ctx, |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("▶")
                                .size(48.0)
                                .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Next image (▶ / D)")
                    .clicked()
                {
                    crate::app::types::send_event(
                        params.event_tx,
                        params.event_proxy,
                        crate::app::types::AppEvent::NextImage,
                    );
                }
            });

        if params.is_cropping {
            egui::Area::new(egui::Id::new("crop_overlay_area"))
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Middle)
                .show(ctx, |ui| crop::draw_crop_overlay(params, ui));
        }

        osd::draw_status_and_osd(params, ctx, bottom_y);

        if params.show_help {
            help::draw_help_modal(ctx, logical_h);
        }

        sidebar::draw_sidebar_and_search(params, ctx, logical_h, strip_h);
        adjustments::draw_adjustments(params, ctx, logical_h, strip_h);

        if !params.config.show_sidebar || !params.config.show_adjustments {
            egui::Area::new(egui::Id::new("open_panels_bar"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if !params.config.show_adjustments
                            && ui
                                .button(egui::RichText::new("🎛 Edit (E)").size(11.0))
                                .on_hover_text("Open adjustments & export settings (E)")
                                .clicked()
                        {
                            params.config.show_adjustments = true;
                            params.config.save();
                        }
                        if !params.config.show_sidebar
                            && ui
                                .button(egui::RichText::new("📁 Files (F)").size(11.0))
                                .on_hover_text("Open file browser (F)")
                                .clicked()
                        {
                            params.config.show_sidebar = true;
                            params.config.save();
                        }
                    });
                });
        }

        histogram::draw_histogram(params, ctx, bottom_y);
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

        // Image pass: while cropping, keep the underlying image uncropped so
        // the overlay handles map 1:1 onto the full frame without fighting zoom.
        {
            let display_adj = if params.is_cropping {
                let mut a = *params.adjustments;
                a.crop_rect = [0.0, 0.0, 1.0, 1.0];
                a
            } else {
                *params.adjustments
            };
            self.encode_image(
                &display_adj,
                params.transition_factor,
                params.config.checkerboard_enabled(),
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

    /// Draw the spinner-only frame used while no image has arrived yet.
    #[allow(dead_code)]
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

        if let Some(path) = path {
            if let Some(thumb) = self.thumbnails.iter().find(|t| t.path == path) {
                let window_aspect_ratio = if self.config.height > 0 {
                    self.config.width as f32 / self.config.height as f32
                } else {
                    1.0
                };

                let mut uniforms = Uniforms::identity();
                uniforms.aspect_ratio = thumb.width as f32 / thumb.height as f32;
                uniforms.window_aspect_ratio = window_aspect_ratio;
                uniforms.brightness = 0.8;
                uniforms.saturation = 0.5;

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

    /// Upload a streamed GIF frame into its ring slot, recycling the texture
    /// when the slot already holds one of matching size.
    pub fn upload_gif_frame(
        &mut self,
        index: usize,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<()> {
        use crate::app::constants::GIF_RING_SLOTS;
        let pos = index % GIF_RING_SLOTS;
        let layout = self.pipeline.get_bind_group_layout(0);

        match &mut self.gif_ring[pos] {
            Some(slot) if slot.index == index && slot.width == width && slot.height == height => {
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &slot.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    rgba,
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
            }
            slot => {
                if let Some(old) = slot.take() {
                    old.texture.destroy();
                }
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
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
                    rgba,
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
                let make_bg = |sampler: &wgpu::Sampler| {
                    Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("GIF Frame Bind Group"),
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
                                resource: wgpu::BindingResource::Sampler(sampler),
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
                    }))
                };
                let bg_linear = make_bg(&self.sampler);
                let bg_nearest = make_bg(&self.sampler_nearest);

                *slot = Some(super::types::GifSlot {
                    index,
                    texture,
                    bind_group: bg_linear,
                    bind_group_nearest: bg_nearest,
                    width,
                    height,
                });
            }
        }
        Ok(())
    }

    pub fn clear_gif_ring(&mut self) {
        for slot in self.gif_ring.iter_mut() {
            if let Some(s) = slot.take() {
                s.texture.destroy();
            }
        }
    }
}
