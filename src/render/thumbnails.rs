use color_eyre::eyre::{Result, eyre};
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

use super::renderer::Renderer;
use super::types::{STRIP_HEIGHT_PX, THUMB_SLOT_W, ThumbnailEntry, Uniforms};

impl Renderer {
    /// Upload a decoded thumbnail and evict the entry furthest from the
    /// viewport once the resident set exceeds [`MAX_THUMBNAILS`].
    ///
    /// Eviction (rather than refusing to load) is what keeps memory bounded
    /// without permanently blanking thumbnails far from the current image.
    pub fn upload_thumbnail(
        &mut self,
        path: std::path::PathBuf,
        rgba: &[u8],
        width: u32,
        height: u32,
        order: usize,
    ) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(eyre!("thumbnail has a zero dimension"));
        }
        if width > self.max_texture_dim || height > self.max_texture_dim {
            return Err(eyre!(
                "thumbnail {width}x{height} exceeds the {}px GPU limit",
                self.max_texture_dim
            ));
        }
        if rgba.len() < (width as usize) * (height as usize) * 4 {
            return Err(eyre!("thumbnail buffer too small for {width}x{height}"));
        }

        // Replace an entry that re-decoded the same file (file changed on disk).
        if let Some(existing) = self.thumbnails.iter().position(|t| t.path == path) {
            let victim = self.thumbnails.remove(existing);
            victim.texture.destroy();
            victim.uniform_buffer.destroy();
        } else if self.thumbnails.len() >= crate::app::types::MAX_THUMBNAILS {
            let evict = self
                .thumbnails
                .iter()
                .enumerate()
                .max_by_key(|(_, t)| t.order.abs_diff(order))
                .map(|(i, _)| i)
                .unwrap_or(0);
            let victim = self.thumbnails.remove(evict);
            victim.texture.destroy();
            victim.uniform_buffer.destroy();
        }

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
            format: TextureFormat::Rgba8UnormSrgb,
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

        let thumb_uniforms = Uniforms::identity();
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Thumbnail Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&thumb_uniforms));

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Thumbnail Bind Group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        }));

        let entry = ThumbnailEntry {
            path,
            order,
            texture,
            bind_group,
            uniform_buffer,
            width,
            height,
        };
        // Keep the list ordered by directory position without a full sort.
        let pos = self.thumbnails.partition_point(|t| t.order < order);
        self.thumbnails.insert(pos, entry);
        self.last_thumb_state = None;

        Ok(())
    }

    pub(crate) fn encode_thumbnail_strip(
        &mut self,
        _active_idx: Option<usize>,
        _selected_indices: &rustc_hash::FxHashSet<usize>,
        thumb_scroll: f32,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let win_w = self.config.width;
        let win_h = self.config.height;
        let strip_h = STRIP_HEIGHT_PX;

        let start_x = -thumb_scroll;

        let current_state = (win_w, win_h, thumb_scroll);
        let update_uniforms = self.last_thumb_state != Some(current_state);
        if update_uniforms {
            for (_i, thumb) in self.thumbnails.iter().enumerate() {
                let x = start_x + (_i as f32 * THUMB_SLOT_W as f32);
                if x + (THUMB_SLOT_W as f32) < 0.0 || x > win_w as f32 {
                    continue;
                }

                let (tw, th) = if thumb.width > thumb.height {
                    (
                        THUMB_SLOT_W as f32 - 10.0,
                        ((THUMB_SLOT_W as f32 - 10.0) * (thumb.height as f32 / thumb.width as f32)),
                    )
                } else {
                    (
                        ((strip_h as f32 - 10.0) * (thumb.width as f32 / thumb.height as f32)),
                        strip_h as f32 - 10.0,
                    )
                };

                let pos_scale = [tw / win_w as f32, th / win_h as f32];
                let pos_offset = [
                    (x + THUMB_SLOT_W as f32 / 2.0) / win_w as f32 * 2.0 - 1.0,
                    -((win_h as f32 - strip_h as f32 / 2.0) / win_h as f32 * 2.0 - 1.0),
                ];

                let mut uniforms =
                    Uniforms::identity().with_display_matrix(self.display_matrix.as_ref());
                uniforms.pos_scale = pos_scale;
                uniforms.pos_offset = pos_offset;

                self.queue
                    .write_buffer(&thumb.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
            }
            self.last_thumb_state = Some(current_state);
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Thumbnail Strip Pass"),
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
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        for (_i, thumb) in self.thumbnails.iter().enumerate() {
            let x = start_x + (_i as f32 * THUMB_SLOT_W as f32);
            if x + (THUMB_SLOT_W as f32) < 0.0 {
                continue;
            }
            if x > win_w as f32 {
                break;
            }

            let scissor_x = x.max(0.0) as u32;
            let scissor_y = win_h.saturating_sub(strip_h);
            let raw_w = (THUMB_SLOT_W as f32 - (if x < 0.0 { -x } else { 0.0 }))
                .min(win_w as f32 - x.max(0.0));
            if raw_w <= 0.0 || scissor_x >= win_w || scissor_y >= win_h {
                continue;
            }
            let scissor_w = (raw_w as u32).min(win_w.saturating_sub(scissor_x)).max(1);
            let scissor_h = strip_h.min(win_h.saturating_sub(scissor_y)).max(1);

            pass.set_bind_group(0, Some(thumb.bind_group.as_ref()), &[]);
            pass.set_scissor_rect(scissor_x, scissor_y, scissor_w, scissor_h);
            pass.draw(0..6, 0..1);
        }
    }

    pub fn clear_thumbnails(&mut self) {
        for thumb in self.thumbnails.drain(..) {
            thumb.texture.destroy();
            thumb.uniform_buffer.destroy();
        }
        self.last_thumb_state = None;
    }

    pub fn thumbnail_index_at(&self, x: f64, y: f64, thumb_scroll: f32) -> Option<usize> {
        let win_h = self.config.height as f64;
        let strip_h = STRIP_HEIGHT_PX as f64;

        if y < win_h - strip_h {
            return None;
        }

        let start_x = -thumb_scroll as f64;
        let n = self.thumbnails.len();
        let total_w = n as f64 * THUMB_SLOT_W as f64;

        if x < start_x || x >= start_x + total_w {
            return None;
        }

        let slot = ((x - start_x) / THUMB_SLOT_W as f64) as usize;
        if slot < n { Some(slot) } else { None }
    }
}
