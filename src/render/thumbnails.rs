use color_eyre::eyre::Result;
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

use super::renderer::Renderer;
use super::types::{ThumbnailEntry, Uniforms, STRIP_HEIGHT_PX, THUMB_SLOT_W};

impl Renderer {
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
            ],
        }));

        self.thumbnails.push(ThumbnailEntry {
            path,
            texture,
            bind_group,
            uniform_buffer,
            width,
            height,
        });

        Ok(())
    }

    pub(crate) fn encode_thumbnail_strip(
        &self,
        _active_idx: Option<usize>,
        _selected_indices: &std::collections::HashSet<usize>,
        thumb_scroll: f32,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let win_w = self.config.width;
        let win_h = self.config.height;
        let strip_h = STRIP_HEIGHT_PX;

        let start_x = -thumb_scroll;

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Thumbnail Strip Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
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

            let mut uniforms = Uniforms::identity();
            uniforms.pos_scale = pos_scale;
            uniforms.pos_offset = pos_offset;

            self.queue
                .write_buffer(&thumb.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            pass.set_bind_group(0, Some(thumb.bind_group.as_ref()), &[]);
            pass.set_scissor_rect(
                x.max(0.0) as u32,
                win_h - strip_h,
                (THUMB_SLOT_W as f32 - (if x < 0.0 { -x } else { 0.0 }))
                    .min(win_w as f32 - x.max(0.0)) as u32,
                strip_h,
            );
            pass.draw(0..6, 0..1);
        }
    }

    pub fn clear_thumbnails(&mut self) {
        self.thumbnails.clear();
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
        if slot < n {
            Some(slot)
        } else {
            None
        }
    }
}
