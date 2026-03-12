use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp,
};
use wgpu_glyph::{Section, Text};

use super::renderer::Renderer;
use super::types::{RenderParams, STRIP_HEIGHT_PX};

impl Renderer {
    pub(crate) fn encode_ui_overlay(
        &mut self,
        params: &RenderParams,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let scale = self.scale_factor as f32;

        let nav_y = if params.show_thumbnail_strip && !self.thumbnails.is_empty() {
            (self.config.height as f32 - STRIP_HEIGHT_PX as f32) / 2.0
        } else {
            self.config.height as f32 / 2.0
        };

        self.text_brush.queue(
            Section::default()
                .add_text(
                    Text::new("◀")
                        .with_scale(48.0 * scale)
                        .with_color([0.8, 0.8, 0.8, 0.6]),
                )
                .with_screen_position((20.0 * scale, nav_y)),
        );
        self.text_brush.queue(
            Section::default()
                .add_text(
                    Text::new("▶")
                        .with_scale(48.0 * scale)
                        .with_color([0.8, 0.8, 0.8, 0.6]),
                )
                .with_screen_position((self.config.width as f32 - 60.0 * scale, nav_y)),
        );

        if let Some(status) = params.status_text {
            if !status.is_empty() {
                self.text_brush.queue(
                    Section::default()
                        .add_text(
                            Text::new(status)
                                .with_scale(18.0 * scale)
                                .with_color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .with_screen_position((10.0 * scale, 10.0 * scale)),
                );
            }
        }

        if params.show_help {
            let help_text = "Shortcuts:\nA/W: Prev Image\nD/S: Next Image\nR: Rotate\nC: Toggle Crop\nH: Toggle HDR\nCtrl+S: Save\nF: Toggle Sidebar\nT: Toggle Thumbnails\nEsc: Quit";
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(help_text)
                            .with_scale(16.0 * scale)
                            .with_color([0.9, 0.9, 0.9, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, 40.0 * scale)),
            );
        }

        if let Some(exif) = params.exif_text {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(exif)
                            .with_scale(15.0 * scale)
                            .with_color([0.85, 0.95, 1.0, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, self.config.height as f32 - 30.0 * scale)),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("UI Overlay Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
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
            });

            if params.is_cropping {
                render_pass.set_pipeline(&self.crop_pipeline);
                let win_w = self.config.width;
                let win_h = self.config.height;

                let cx = ((params.crop_rect[0] * win_w as f32) as u32).min(win_w.saturating_sub(1));
                let cy = ((params.crop_rect[1] * win_h as f32) as u32).min(win_h.saturating_sub(1));
                let cw = ((params.crop_rect[2] * win_w as f32) as u32).min(win_w - cx);
                let ch = ((params.crop_rect[3] * win_h as f32) as u32).min(win_h - cy);

                if cy > 0 {
                    render_pass.set_scissor_rect(0, 0, win_w, cy);
                    render_pass.draw(0..4, 0..1);
                }
                if cy + ch < win_h {
                    render_pass.set_scissor_rect(0, cy + ch, win_w, win_h - (cy + ch));
                    render_pass.draw(0..4, 0..1);
                }
                if cx > 0 {
                    render_pass.set_scissor_rect(0, cy, cx, ch);
                    render_pass.draw(0..4, 0..1);
                }
                if cx + cw < win_w {
                    render_pass.set_scissor_rect(cx + cw, cy, win_w - (cx + cw), ch);
                    render_pass.draw(0..4, 0..1);
                }
            }
        }

        let _ = self.text_brush.draw_queued(
            &self.device,
            &mut self.staging_belt,
            encoder,
            view,
            self.config.width,
            self.config.height,
        );
    }

    pub fn render_frame(&mut self, params: RenderParams) -> Result<()> {
        let frame = self
            .surface
            .get_current_texture()
            .context("Failed to get current surface texture")?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        self.encode_image(params.adjustments, &view, &mut encoder);

        if params.show_thumbnail_strip && !self.thumbnails.is_empty() {
            self.encode_thumbnail_strip(
                params.active_thumb_idx,
                params.selected_indices,
                params.thumb_scroll,
                &view,
                &mut encoder,
            );
        }
        self.encode_ui_overlay(&params, &view, &mut encoder);

        self.queue.submit([encoder.finish()]);
        self.staging_belt.recall();
        frame.present();
        Ok(())
    }

    pub fn render_loading(&self) -> Result<()> {
        let frame = self
            .surface
            .get_current_texture()
            .context("Failed to get current surface texture")?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Loading Encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Loading Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    pub fn preload_gif_textures(&mut self, frames: &[crate::image::ImageData]) -> Result<()> {
        for (tex, _) in self.gif_textures.drain(..) {
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
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            }));
            self.gif_textures.push((texture, bind_group));
        }
        Ok(())
    }
}
