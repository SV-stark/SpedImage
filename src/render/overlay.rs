use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp,
};
use wgpu_glyph::{Section, Text};

use super::renderer::Renderer;
use super::types::{ImageAdjustments, STRIP_HEIGHT_PX};

impl Renderer {
    /// Encode UI overlay commands into `encoder` targeting `view`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_ui_overlay(
        &mut self,
        is_cropping: bool,
        crop_rect: [f32; 4],
        status_text: Option<&str>,
        show_help: bool,
        sidebar_text: Option<&str>,
        show_thumbnail_strip: bool,
        _thumb_scroll: f32,
        exif_text: Option<&str>,
        show_histogram: bool,
        histogram_data: Option<&([u32; 256], [u32; 256], [u32; 256])>,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let scale = self.scale_factor as f32;

        let help_text = "Shortcuts:\nA/W: Prev Image\nD/S: Next Image\nR: Rotate\nC: Toggle Crop\nH: Toggle HDR\nCtrl+S: Save\nF: Toggle Sidebar\nT: Toggle Thumbnails\nEsc: Quit";
        let sidebar_list_text = if let Some(sidebar_text) = sidebar_text {
            sidebar_text
        } else {
            ""
        };

        let nav_y = if show_thumbnail_strip && !self.thumbnails.is_empty() {
            (self.config.height as f32 - STRIP_HEIGHT_PX as f32) / 2.0
        } else {
            self.config.height as f32 / 2.0
        };

        self.text_brush.queue(
            Section::default()
                .add_text(
                    Text::new("◀")
                        .with_scale(48.0 * scale)
                        .with_color([0.8f32, 0.8, 0.8, 0.6]),
                )
                .with_screen_position((20.0 * scale, nav_y)),
        );
        self.text_brush.queue(
            Section::default()
                .add_text(
                    Text::new("▶")
                        .with_scale(48.0 * scale)
                        .with_color([0.8f32, 0.8, 0.8, 0.6]),
                )
                .with_screen_position((self.config.width as f32 - 60.0 * scale, nav_y)),
        );

        let status_y = if show_thumbnail_strip && !self.thumbnails.is_empty() {
            self.config.height as f32 - STRIP_HEIGHT_PX as f32 - 28.0 * scale
        } else {
            self.config.height as f32 - 30.0 * scale
        };

        if let Some(status) = status_text {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(status)
                            .with_scale(18.0 * scale)
                            .with_color([1.0f32, 1.0, 1.0, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, status_y)),
            );
        }

        if show_histogram {
            if let Some((r_hist, g_hist, b_hist)) = histogram_data {
                let h_w = 256.0 * scale;
                let h_h = 100.0 * scale;
                let h_x = self.config.width as f32 - h_w - 10.0 * scale;
                let h_y = 10.0 * scale;

                self.text_brush.queue(
                    Section::default()
                        .add_text(
                            Text::new("▇")
                                .with_scale(h_h)
                                .with_color([0.0, 0.0, 0.0, 0.4]),
                        )
                        .with_screen_position((h_x, h_y))
                        .with_bounds((h_w, h_h)),
                );

                let max_val = r_hist
                    .iter()
                    .chain(g_hist.iter())
                    .chain(b_hist.iter())
                    .max()
                    .copied()
                    .unwrap_or(1)
                    .max(1);

                for (chan_idx, (hist, color)) in [
                    (r_hist, [1.0f32, 0.3, 0.3, 0.6]),
                    (g_hist, [0.3f32, 1.0, 0.3, 0.6]),
                    (b_hist, [0.3f32, 0.3, 1.0, 0.6]),
                ]
                .into_iter()
                .enumerate()
                {
                    let mut bars = String::with_capacity(64);
                    for i in (0..256).step_by(4) {
                        let val = hist[i..i + 4].iter().sum::<u32>() / 4;
                        let bar_h = (val as f32 / max_val as f32 * 8.0).round() as u32;
                        let char = match bar_h {
                            0 | 1 => " ",
                            2 => "▂",
                            3 => "▃",
                            4 => "▄",
                            5 => "▅",
                            6 => "▆",
                            7 => "▇",
                            _ => "█",
                        };
                        bars.push_str(char);
                    }

                    self.text_brush.queue(
                        Section::default()
                            .add_text(Text::new(&bars).with_scale(h_h / 4.0).with_color(color))
                            .with_screen_position((h_x, h_y + (chan_idx as f32 * h_h / 4.0))),
                    );
                }
            }
        }

        if show_help {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(help_text)
                            .with_scale(16.0 * scale)
                            .with_color([0.9f32, 0.9, 0.9, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, 10.0 * scale)),
            );
        }

        if let Some(exif) = exif_text {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(exif)
                            .with_scale(15.0 * scale)
                            .with_color([0.85f32, 0.95, 1.0, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, 10.0 * scale)),
            );
        }

        if sidebar_text.map(|f| !f.is_empty()).unwrap_or(false) {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(&sidebar_list_text)
                            .with_scale(14.0 * scale)
                            .with_color([0.85f32, 0.95, 1.0, 1.0]),
                    )
                    .with_screen_position((self.config.width as f32 - 280.0 * scale, 10.0 * scale))
                    .with_bounds((270.0 * scale, self.config.height as f32 - 20.0 * scale)),
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

            if is_cropping {
                render_pass.set_pipeline(&self.crop_pipeline);
                let win_w = self.config.width;
                let win_h = self.config.height;

                let cx = ((crop_rect[0] * win_w as f32) as u32).min(win_w.saturating_sub(1));
                let cy = ((crop_rect[1] * win_h as f32) as u32).min(win_h.saturating_sub(1));
                let cw = ((crop_rect[2] * win_w as f32) as u32).min(win_w - cx);
                let ch = ((crop_rect[3] * win_h as f32) as u32).min(win_h - cy);

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

        if let Err(e) = self.text_brush.draw_queued(
            &self.device,
            &mut self.staging_belt,
            encoder,
            view,
            self.config.width,
            self.config.height,
        ) {
            tracing::warn!("Text draw error: {e}");
        }
    }

    pub fn render_frame(
        &mut self,
        adjustments: &ImageAdjustments,
        is_cropping: bool,
        crop_rect: [f32; 4],
        status_text: Option<&str>,
        show_help: bool,
        sidebar_text: Option<&str>,
        show_thumbnail_strip: bool,
        thumb_scroll: f32,
        active_thumb_idx: Option<usize>,
        selected_indices: &std::collections::HashSet<usize>,
        exif_text: Option<&str>,
        show_histogram: bool,
        histogram_data: Option<&([u32; 256], [u32; 256], [u32; 256])>,
    ) -> Result<()> {
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

        self.encode_image(adjustments, &view, &mut encoder);

        if show_thumbnail_strip && !self.thumbnails.is_empty() {
            self.encode_thumbnail_strip(
                active_thumb_idx,
                selected_indices,
                thumb_scroll,
                &view,
                &mut encoder,
            );
        }
        self.encode_ui_overlay(
            is_cropping,
            crop_rect,
            status_text,
            show_help,
            sidebar_text,
            show_thumbnail_strip,
            thumb_scroll,
            exif_text,
            show_histogram,
            histogram_data,
            &view,
            &mut encoder,
        );

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
        const MAX_GIF_VRAM_BYTES: u64 = 256 * 1024 * 1024;
        let total_vram: u64 = frames
            .iter()
            .map(|f| (f.width as u64) * (f.height as u64) * 4)
            .sum();
        let frames_to_load = if total_vram > MAX_GIF_VRAM_BYTES && !frames.is_empty() {
            let per_frame = total_vram / frames.len() as u64;
            let max_frames = (MAX_GIF_VRAM_BYTES / per_frame).max(1) as usize;
            &frames[..max_frames]
        } else {
            frames
        };

        let layout = self.pipeline.get_bind_group_layout(0);
        for frame in frames_to_load {
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

    pub fn has_image(&self) -> bool {
        self.image_texture.is_some()
    }
    pub fn gif_frame_count(&self) -> usize {
        self.gif_textures.len()
    }
}
