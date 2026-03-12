use anyhow::Result;
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

use super::renderer::Renderer;
use super::types::{ThumbnailEntry, STRIP_HEIGHT_PX, THUMB_SLOT_W};

impl Renderer {
    /// Upload a single thumbnail RGBA buffer to the GPU and return its bind group.
    /// The bind group uses `thumb_uniform_buffer` so thumbnails render with identity settings.
    pub fn upload_thumbnail(
        &mut self,
        path: std::path::PathBuf,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<()> {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Thumbnail Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Thumbnail Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(
                        self.thumb_uniform_buffer.as_entire_buffer_binding(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        }));

        self.thumbnails.push(ThumbnailEntry {
            path,
            bind_group,
            width,
            height,
            _texture: texture,
        });

        Ok(())
    }

    /// Remove all thumbnails and free their GPU textures.
    pub fn clear_thumbnails(&mut self) {
        // ThumbnailEntry holds the texture so dropping the Vec frees VRAM.
        self.thumbnails.clear();
    }

    /// Encode the thumbnail strip at the bottom of the screen.
    pub(crate) fn encode_thumbnail_strip(
        &mut self,
        active_idx: Option<usize>,
        selected_indices: &std::collections::HashSet<usize>,
        thumb_scroll: f32,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let win_w = self.config.width;
        let win_h = self.config.height;

        if win_h < STRIP_HEIGHT_PX || win_w < THUMB_SLOT_W {
            return;
        }

        let strip_y = win_h - STRIP_HEIGHT_PX;

        // --- Pass 1: darken the strip background using the crop pipeline --------
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Thumbnail Strip Background"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.crop_pipeline);
            // We draw the full-screen quad but use a scissor rect for the strip itself.
            pass.set_scissor_rect(0, strip_y, win_w, STRIP_HEIGHT_PX);
            pass.draw(0..4, 0..1);
        }

        // --- Pass 2: draw thumbnails -----------------------------------------
        // We calculate horizontal centering
        let n = self.thumbnails.len();
        let total_w = n as f32 * THUMB_SLOT_W as f32;
        let start_x: f32 = if total_w <= win_w as f32 {
            ((win_w as f32 - total_w) / 2.0).floor()
        } else {
            -thumb_scroll
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Thumbnails pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            for (i, thumb) in self.thumbnails.iter().enumerate() {
                let x = start_x + (i as f32 * THUMB_SLOT_W as f32);
                if x + (THUMB_SLOT_W as f32) < 0.0 {
                    continue;
                }
                if x > win_w as f32 {
                    break;
                }

                // Aspect-fit thumbnail in the slot
                let (tw, th) = if thumb.width > thumb.height {
                    let h = (super::types::THUMB_SIZE as f32
                        * (thumb.height as f32 / thumb.width as f32))
                        as u32;
                    (super::types::THUMB_SIZE, h)
                } else {
                    let w = (super::types::THUMB_SIZE as f32
                        * (thumb.width as f32 / thumb.height as f32))
                        as u32;
                    (w, super::types::THUMB_SIZE)
                };

                let off_x = (THUMB_SLOT_W - tw) / 2;
                let off_y = (STRIP_HEIGHT_PX - th) / 2;

                let draw_x = x + off_x as f32;
                let draw_y = strip_y as f32 + off_y as f32;

                // Calculate NDC offset and scale
                // NDC x = (2*x + w)/win_w - 1.0
                // NDC y = 1.0 - (2*y + h)/win_h
                // NDC scale x = w / win_w
                // NDC scale y = h / win_h
                let pos_offset = [
                    (2.0 * draw_x + tw as f32) / win_w as f32 - 1.0,
                    1.0 - (2.0 * draw_y + th as f32) / win_h as f32,
                ];
                let pos_scale = [tw as f32 / win_w as f32, th as f32 / win_h as f32];

                let mut uniforms = super::types::Uniforms::identity();
                uniforms.pos_offset = pos_offset;
                uniforms.pos_scale = pos_scale;

                self.queue.write_buffer(
                    &self.thumb_uniform_buffer,
                    0,
                    bytemuck::bytes_of(&uniforms),
                );

                // Draw highlight for selected items
                let is_active = Some(i) == active_idx;
                let is_selected = selected_indices.contains(&i);

                if is_active || is_selected {
                    // Could draw a highlight here
                }

                pass.set_bind_group(0, thumb.bind_group.as_ref(), &[]);
                pass.draw(0..6, 0..1);
            }
        }
    }

    /// Return the index into `self.thumbnails` for a given pixel click coordinate
    pub fn thumbnail_index_at(&self, x: f64, y: f64, thumb_scroll: f32) -> Option<usize> {
        let win_h = self.config.height as f64;
        let win_w = self.config.width as f64;
        let strip_y = win_h - STRIP_HEIGHT_PX as f64;

        if y < strip_y || self.thumbnails.is_empty() {
            return None;
        }

        let n = self.thumbnails.len();
        let total_w = n as f64 * THUMB_SLOT_W as f64;
        let start_x: f64 = if total_w <= win_w {
            (win_w - total_w) / 2.0
        } else {
            -thumb_scroll as f64
        };

        if x < start_x || x >= start_x + total_w {
            return None;
        }

        let slot = ((x - start_x) / THUMB_SLOT_W as f64) as usize;
        if slot < n {
            Some(slot)
        } else {
            None
        }
    }
}
