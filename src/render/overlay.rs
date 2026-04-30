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
        params: &RenderParams,
        ctx: &egui::Context,
        has_thumbnails: bool,
        win_w: u32,
        win_h: u32,
    ) {
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

        let full_output = self.egui_state.egui_ctx().run_ui(raw_input, |ctx| {
            Self::render_ui_static(&params, ctx, has_thumbnails, win_w, win_h);
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
