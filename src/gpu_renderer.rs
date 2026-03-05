//! GPU Renderer - WGPU-based image processing pipeline

use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BindingType, BlendState,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    FragmentState, FrontFace, Instance, LoadOp, Operations, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::image_backend::ImageData;

/// Height of the thumbnail strip in physical pixels.
pub const STRIP_HEIGHT_PX: u32 = 90;
/// Width of each thumbnail slot (including gap).
pub const THUMB_SLOT_W: u32 = 80;
/// Thumbnail image size (square, aspect-fit inside the slot).
pub const THUMB_SIZE: u32 = 74;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageAdjustments {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub rotation: f32,
    pub crop_rect_target: [f32; 4], // Where we want to be
    pub crop_rect: [f32; 4],        // Where we currently are (rendered)
    pub hdr_toning: bool,
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            rotation: 0.0,
            crop_rect_target: [0.0, 0.0, 1.0, 1.0],
            crop_rect: [0.0, 0.0, 1.0, 1.0],
            hdr_toning: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    rotation: f32,
    aspect_ratio: f32,
    window_aspect_ratio: f32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
    hdr_toning: f32,
    _padding: f32,
}

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    rotation: f32,
    aspect_ratio: f32,
    window_aspect_ratio: f32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
    hdr_toning: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    var tex = tex_coords * vec2<f32>(uniforms.crop_w, uniforms.crop_h) 
              + vec2<f32>(uniforms.crop_x, uniforms.crop_y);
    let center = vec2<f32>(0.5, 0.5);
    let rotated_tex = rotate(tex - center, uniforms.rotation) + center;
    var pos = position;

    let image_ar = uniforms.aspect_ratio;
    let window_ar = uniforms.window_aspect_ratio;
    let ratio = image_ar / window_ar;

    if (ratio > 1.0) {
        pos.y /= ratio;
    } else {
        pos.x *= ratio;
    }

    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = rotated_tex;
    return out;
}

fn rotate(coord: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(coord.x * c - coord.y * s, coord.x * s + coord.y * c);
}

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(1)
var image_sampler: sampler;

@group(0) @binding(2)
var image_texture: texture_2d<f32>;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(image_texture, image_sampler, input.tex_coords);
    var color = tex_color.rgb;
    color = color * uniforms.brightness;
    color = (color - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
    color = mix(vec3<f32>(gray), color, uniforms.saturation);

    if (uniforms.hdr_toning > 0.5) {
        let exposed = color * 1.6;
        color = exposed / (1.0 + exposed);
        color = color * color * (3.0 - 2.0 * color);
    }

    return vec4<f32>(color, tex_color.a);
}
"#;

const CROP_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct CropUniforms {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
};

@group(0) @binding(0)
var<uniform> crop: CropUniforms;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a full screen quad
    var out: VertexOutput;
    let x = f32(vertex_index & 1u) * 2.0 - 1.0;
    let y = f32((vertex_index >> 1u) & 1u) * 2.0 - 1.0;
    // We invert y for Vulkan/WGPU coordinates
    out.position = vec4<f32>(x, -y, 0.0, 1.0);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // In viewport coordinates (0 to width/height)
    // Actually, in.position.xy is in pixel coordinates.
    // We pass normalized crop rect (0.0 to 1.0) but we don't know window bounds in shader easily.
    // Instead we can generate the crop rect in vertex shader or just draw the overlay regions.
    return vec4<f32>(0.0, 0.0, 0.0, 0.5); // Just a generic darken, we will use scissors!
}
"#;

/// A fully-uploaded thumbnail ready for GPU rendering.
pub struct ThumbnailEntry {
    pub path: std::path::PathBuf,
    /// Bind group pointing at the thumbnail texture (same layout as image pipeline).
    pub bind_group: Arc<BindGroup>,
    pub width: u32,
    pub height: u32,
    /// The GPU texture (kept alive so bind group stays valid).
    _texture: Texture,
}

pub struct Renderer {
    _window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    pipeline: RenderPipeline,
    crop_pipeline: RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    /// Uniform buffer used with identity (no-op) uniforms for thumbnail rendering.
    thumb_uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    sampler: Sampler,
    image_texture: Option<Texture>,
    image_bind_group: Option<Arc<BindGroup>>,
    pub gif_textures: Vec<(Texture, Arc<BindGroup>)>, // cached GPU textures for GIF frames
    config: SurfaceConfiguration,
    image_size: Option<(u32, u32)>,
    pub scale_factor: f64, // DPI scale (12: DPI-aware rendering)

    // Text rendering
    text_brush: GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,

    // Thumbnails
    pub thumbnails: Vec<ThumbnailEntry>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::new(&wgpu::InstanceDescriptor::default());

        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create WGPU surface")?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("Failed to request WGPU adapter")?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("SpedImage Device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .context("Failed to request WGPU device")?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(SHADER.into()),
        });

        let vertex_data: [f32; 24] = [
            -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 0.0,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // A second uniform buffer pre-filled with "identity" values for thumbnail rendering.
        let thumb_uniforms = Uniforms {
            rotation: 0.0,
            aspect_ratio: 1.0,
            window_aspect_ratio: 1.0,
            crop_x: 0.0,
            crop_y: 0.0,
            crop_w: 1.0,
            crop_h: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            hdr_toning: 0.0,
            _padding: 0.0,
        };
        let thumb_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Thumbnail Uniform Buffer"),
            contents: bytemuck::bytes_of(&thumb_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Image Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: 16,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let crop_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Crop Shader"),
            source: ShaderSource::Wgsl(CROP_SHADER.into()),
        });

        let crop_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Crop Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let crop_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Crop Overlay Pipeline"),
            layout: Some(&crop_pipeline_layout),
            vertex: VertexState {
                module: &crop_shader_module,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[], // generating vertices directly
            },
            fragment: Some(FragmentState {
                module: &crop_shader_module,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Embed Inter-Regular as the guaranteed font fallback; try Segoe UI first on Windows.
        const EMBEDDED_FONT: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");
        let font_bytes = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf")
            .unwrap_or_else(|_| EMBEDDED_FONT.to_vec());

        let font = ab_glyph::FontArc::try_from_vec(font_bytes).unwrap_or_else(|_| {
            ab_glyph::FontArc::try_from_slice(EMBEDDED_FONT)
                .expect("Embedded Inter-Regular.ttf failed to parse — check asset integrity")
        });

        let text_brush = GlyphBrushBuilder::using_font(font).build(&device, format);

        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Ok(Self {
            _window: window.clone(),
            device,
            queue,
            surface,
            pipeline,
            crop_pipeline,
            uniform_buffer,
            thumb_uniform_buffer,
            vertex_buffer,
            sampler,
            image_texture: None,
            image_bind_group: None,
            gif_textures: Vec::new(),
            config,
            image_size: None,
            scale_factor: window.scale_factor(),
            text_brush,
            staging_belt,
            thumbnails: Vec::new(),
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        // wgpu_glyph text positions update automatically with new viewport dimensions
    }

    pub fn load_image(&mut self, image_data: &ImageData) -> Result<()> {
        // Explicitly destroy old GPU texture to free VRAM immediately
        if let Some(old_tex) = self.image_texture.take() {
            old_tex.destroy();
        }
        self.image_bind_group = None;

        let width = image_data.width;
        let height = image_data.height;

        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Image Texture"),
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
            image_data.as_rgba(),
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
            label: Some("Image Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(
                        self.uniform_buffer.as_entire_buffer_binding(),
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

        self.image_texture = Some(texture);
        self.image_bind_group = Some(bind_group);
        self.image_size = Some((width, height));

        tracing::debug!("Loaded image into GPU: {}x{}", width, height);
        Ok(())
    }

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

    /// Encode image draw commands into `encoder` targeting `view`.
    /// Does NOT submit or present — caller owns the frame lifetime.
    fn encode_image(
        &self,
        adjustments: &ImageAdjustments,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let window_aspect_ratio = if self.config.height > 0 {
            self.config.width as f32 / self.config.height as f32
        } else {
            1.0
        };

        let uniforms = Uniforms {
            rotation: adjustments.rotation,
            aspect_ratio: self
                .image_size
                .map(|(w, h)| w as f32 / h as f32)
                .unwrap_or(1.0),
            window_aspect_ratio,
            crop_x: adjustments.crop_rect[0],
            crop_y: adjustments.crop_rect[1],
            crop_w: adjustments.crop_rect[2],
            crop_h: adjustments.crop_rect[3],
            brightness: adjustments.brightness,
            contrast: adjustments.contrast,
            saturation: adjustments.saturation,
            hdr_toning: if adjustments.hdr_toning { 1.0 } else { 0.0 },
            _padding: 0.0,
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
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
        });

        if let Some(bind_group) = &self.image_bind_group {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
            render_pass.draw(0..6, 0..1);
        }
    }

    /// Encode the thumbnail strip at the bottom of the screen.
    /// Draws a dark background, then each uploaded thumbnail in its slot.
    /// `active_idx` is the index into `self.thumbnails` that is currently displayed.
    fn encode_thumbnail_strip(
        &mut self,
        active_idx: Option<usize>,
        selected_indices: &std::collections::HashSet<usize>,
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
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Thumbnail Strip Background"),
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
            pass.set_pipeline(&self.crop_pipeline);
            pass.set_scissor_rect(0, strip_y, win_w, STRIP_HEIGHT_PX);
            pass.draw(0..4, 0..1);
        }

        // --- Pass 2: draw each thumbnail using set_viewport --------------------
        let n = self.thumbnails.len();
        if n == 0 {
            return;
        }

        // Centre the strip horizontally; if too many thumbnails, they scroll left
        let total_w = n as u32 * THUMB_SLOT_W;
        let start_x: i64 = if total_w <= win_w {
            ((win_w - total_w) / 2) as i64
        } else if let Some(ai) = active_idx {
            // Keep active thumbnail in view
            let active_cx = ai as i64 * THUMB_SLOT_W as i64 + THUMB_SLOT_W as i64 / 2;
            let half_win = win_w as i64 / 2;
            let raw = half_win - active_cx;
            raw.clamp(win_w as i64 - total_w as i64, 0)
        } else {
            0
        };

        // Padding inside a slot so the thumbnail image is centred
        let pad = (THUMB_SLOT_W - THUMB_SIZE) / 2;

        // We need to iterate by index to capture bind groups - collect them first
        let bind_groups: Vec<(Arc<BindGroup>, u32, u32)> = self
            .thumbnails
            .iter()
            .map(|t| (Arc::clone(&t.bind_group), t.width, t.height))
            .collect();

        for (i, (bg, tw, th)) in bind_groups.iter().enumerate() {
            let slot_x = start_x + (i as i64) * THUMB_SLOT_W as i64;
            // Skip thumbnails fully outside the window
            if slot_x + THUMB_SLOT_W as i64 <= 0 || slot_x >= win_w as i64 {
                continue;
            }

            // Compute aspect-fit rect inside the slot's THUMB_SIZE square
            let thumb_ar = *tw as f32 / (*th).max(1) as f32;
            let (fit_w, fit_h) = if thumb_ar >= 1.0 {
                (THUMB_SIZE, (THUMB_SIZE as f32 / thumb_ar).round() as u32)
            } else {
                ((THUMB_SIZE as f32 * thumb_ar).round() as u32, THUMB_SIZE)
            };

            // Centre fit rect inside the THUMB_SIZE square
            let offset_x = (THUMB_SIZE - fit_w) / 2;
            let offset_y = (THUMB_SIZE - fit_h) / 2;

            let vp_x = (slot_x + pad as i64 + offset_x as i64).max(0) as u32;
            let vp_y = strip_y + pad + offset_y;
            let vp_w = fit_w.min(win_w.saturating_sub(vp_x));
            let vp_h = fit_h;

            if vp_w == 0 || vp_h == 0 {
                continue;
            }

            // Update thumb uniforms for this thumbnail's aspect ratio
            let ar = *tw as f32 / (*th).max(1) as f32;
            let thumb_uniforms = Uniforms {
                rotation: 0.0,
                aspect_ratio: ar,
                window_aspect_ratio: ar, // square viewport ≈ identity
                crop_x: 0.0,
                crop_y: 0.0,
                crop_w: 1.0,
                crop_h: 1.0,
                brightness: 1.0,
                contrast: 1.0,
                saturation: 1.0,
                hdr_toning: 0.0,
                _padding: 0.0,
            };
            self.queue.write_buffer(
                &self.thumb_uniform_buffer,
                0,
                bytemuck::bytes_of(&thumb_uniforms),
            );

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Thumbnail Draw"),
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

            pass.set_pipeline(&self.pipeline);
            pass.set_viewport(vp_x as f32, vp_y as f32, vp_w as f32, vp_h as f32, 0.0, 1.0);
            // Scissor to strip area so thumbnails cannot bleed outside
            pass.set_scissor_rect(0, strip_y, win_w, STRIP_HEIGHT_PX);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_bind_group(0, bg.as_ref(), &[]);
            pass.draw(0..6, 0..1);
        }

        // --- Pass 3: highlight borders (active and selected) ---------------------
        for (i, _) in bind_groups.iter().enumerate() {
            let is_active = active_idx.map_or(false, |ai| ai == i);
            let is_selected = selected_indices.contains(&i);

            if is_active || is_selected {
                let slot_x = start_x + i as i64 * THUMB_SLOT_W as i64;
                if slot_x + THUMB_SLOT_W as i64 > 0 && slot_x < win_w as i64 {
                    let bx = slot_x as i32 + 2;
                    let by = strip_y as i32 + 2;
                    let bw = THUMB_SLOT_W as i32 - 4;
                    let bh = STRIP_HEIGHT_PX as i32 - 4;
                    // Draw a subtle border for selected, prominent for active
                    let bsize = if is_active { 2 } else { 1 };
                    
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Thumbnail Border Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: Operations { load: LoadOp::Load, store: StoreOp::Store },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&self.crop_pipeline);
                    
                    // Top border
                    let b_win_w = win_w as u32;
                    let b_win_h = win_h as u32;
                    
                    if by >= 0 {
                        pass.set_scissor_rect(bx.max(0) as u32, by.max(0) as u32, bw.max(0) as u32, bsize);
                        pass.draw(0..4, 0..1);
                    }
                    // Bottom border
                    let bot = by + bh - bsize as i32;
                    if bot < win_h as i32 {
                        pass.set_scissor_rect(bx.max(0) as u32, bot.max(0) as u32, bw.max(0) as u32, bsize);
                        pass.draw(0..4, 0..1);
                    }
                    // Left border
                    if bx >= 0 {
                        pass.set_scissor_rect(bx.max(0) as u32, by.max(0) as u32, bsize, bh.max(0) as u32);
                        pass.draw(0..4, 0..1);
                    }
                    // Right border
                    let rx = bx + bw - bsize as i32;
                    if rx < win_w as i32 {
                        pass.set_scissor_rect(rx.max(0) as u32, by.max(0) as u32, bsize, bh.max(0) as u32);
                        pass.draw(0..4, 0..1);
                    }
                }
            }
        }
    }

    /// Encode UI overlay commands into `encoder` targeting `view`.
    /// Does NOT submit or present — caller owns the frame lifetime.
    #[allow(clippy::too_many_arguments)]
    fn encode_ui_overlay(
        &mut self,
        is_cropping: bool,
        crop_rect: [f32; 4],
        status_text: Option<&str>,
        show_help: bool,
        sidebar_files: Option<&[String]>,
        show_thumbnail_strip: bool,
        exif_text: Option<&str>,
        show_histogram: bool,
        histogram_data: Option<&([u32; 256], [u32; 256], [u32; 256])>,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // --- this is the body of the old render_ui_overlay, minus frame acquire/present ---

        // 1. Text rendering — queue all sections
        let scale = self.scale_factor as f32;
        let has_text = true; // We queue navigation arrows at minimum

        #[allow(unused_variables)]
        let help_text = "Shortcuts:\nA/W: Prev Image\nD/S: Next Image\nR: Rotate\nC: Toggle Crop\nH: Toggle HDR\nCtrl+S: Save\nF: Toggle Sidebar\nT: Toggle Thumbnails\nEsc: Quit";
        let sidebar_list_text: String = sidebar_files
            .map(|files| {
                files
                    .iter()
                    .enumerate()
                    .map(|(i, name)| format!("{}. {}", i + 1, name))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        // Navigation elements
        let nav_y = if show_thumbnail_strip && !self.thumbnails.is_empty() {
            // Push nav arrows up above the strip
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

        // Status text — move it above the strip when strip is visible
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

        // --- Histogram Rendering ---
        if show_histogram {
            if let Some((r_hist, g_hist, b_hist)) = histogram_data {
                let h_w = 256.0 * scale;
                let h_h = 100.0 * scale;
                let h_x = self.config.width as f32 - h_w - 10.0 * scale;
                let h_y = 10.0 * scale; // top right

                // Background
                self.text_brush.queue(Section::default()
                    .add_text(Text::new("▇")
                        .with_scale(h_h)
                        .with_color([0.0, 0.0, 0.0, 0.4]))
                    .with_screen_position((h_x, h_y))
                    .with_bounds((h_w, h_h)));

                let max_val = r_hist.iter().chain(g_hist.iter()).chain(b_hist.iter()).max().copied().unwrap_or(1).max(1);
                
                // Draw R, G, B bars
                for (chan_idx, (hist, color)) in [
                    (r_hist, [1.0f32, 0.3, 0.3, 0.6]),
                    (g_hist, [0.3f32, 1.0, 0.3, 0.6]),
                    (b_hist, [0.3f32, 0.3, 1.0, 0.6]),
                ].into_iter().enumerate() {
                    let mut bars = String::new();
                    // Subsample to 64 bins for performance and readability in text
                    for i in (0..256).step_by(4) {
                        let val = hist[i..i+4].iter().sum::<u32>() / 4;
                        let bar_h = (val as f32 / max_val as f32 * 8.0).round() as u32;
                        let char = match bar_h {
                            0 => " ",
                            1 => " ",
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
                    
                    self.text_brush.queue(Section::default()
                        .add_text(Text::new(&bars)
                            .with_scale(h_h / 4.0)
                            .with_color(color))
                        .with_screen_position((h_x, h_y + (chan_idx as f32 * h_h / 4.0))));
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

        if sidebar_files.map(|f| !f.is_empty()).unwrap_or(false) {
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

                let cx = (crop_rect[0] * win_w as f32) as u32;
                let cy = (crop_rect[1] * win_h as f32) as u32;
                let cw = (crop_rect[2] * win_w as f32) as u32;
                let ch = (crop_rect[3] * win_h as f32) as u32;

                let cx = cx.min(win_w.saturating_sub(1));
                let cy = cy.min(win_h.saturating_sub(1));
                let cw = cw.min(win_w - cx);
                let ch = ch.min(win_h - cy);

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
        } // drop render_pass

        // Draw queued text
        if has_text {
            if let Err(e) = self.text_brush.draw_queued(
                &self.device,
                &mut self.staging_belt,
                encoder,
                view,
                self.config.width,
                self.config.height,
            ) {
                tracing::warn!("Text draw error: {}", e);
            }
            self.staging_belt.finish();
        }
    }

    /// Combined render: acquires surface texture once, draws image then UI, presents once.
    /// This is the correct path for still images — avoids the double-present black screen bug.
    #[allow(clippy::too_many_arguments)]
    pub fn render_frame(
        &mut self,
        adjustments: &ImageAdjustments,
        is_cropping: bool,
        crop_rect: [f32; 4],
        status_text: Option<&str>,
        show_help: bool,
        sidebar_files: Option<&[String]>,
        show_thumbnail_strip: bool,
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

        // 1. Draw image (clears to black first)
        self.encode_image(adjustments, &view, &mut encoder);

        // 2. Draw thumbnail strip (before text so text overlays on top)
        if show_thumbnail_strip && !self.thumbnails.is_empty() {
            self.encode_thumbnail_strip(active_thumb_idx, selected_indices, &view, &mut encoder);
        }

        // 3. Draw UI overlay on top (LoadOp::Load to preserve image pixels)
        self.encode_ui_overlay(
            is_cropping,
            crop_rect,
            status_text,
            show_help,
            sidebar_files,
            show_thumbnail_strip,
            exif_text,
            show_histogram,
            histogram_data,
            &view,
            &mut encoder,
        );

        // 4. Single submit + present
        self.queue.submit([encoder.finish()]);
        self.staging_belt.recall();
        frame.present();
        Ok(())
    }

    pub fn render_ui_overlay(
        &mut self,
        is_cropping: bool,
        crop_rect: [f32; 4],
        status_text: Option<&str>,
        show_help: bool,
        sidebar_files: Option<&[String]>,
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
                label: Some("UI Render Encoder"),
            });
        self.encode_ui_overlay(
            is_cropping,
            crop_rect,
            status_text,
            show_help,
            sidebar_files,
            false,
            None,
            false,
            None,
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
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Loading Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        // Dark gray color for loading screen
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

    pub fn has_image(&self) -> bool {
        self.image_texture.is_some()
    }

    pub fn gif_frame_count(&self) -> usize {
        self.gif_textures.len()
    }

    /// Upload all GIF frames to GPU once. During playback, swap_gif_frame is
    /// called with an index — zero CPU→GPU copies per frame after initial upload.
    pub fn preload_gif_textures(&mut self, frames: &[ImageData]) -> Result<()> {
        // Explicitly destroy old GIF textures to free VRAM
        for (tex, _) in self.gif_textures.drain(..) {
            tex.destroy();
        }

        // Cap VRAM usage for GIF frames (256 MB max)
        const MAX_GIF_VRAM_BYTES: u64 = 256 * 1024 * 1024;
        let total_vram: u64 = frames
            .iter()
            .map(|f| (f.width as u64) * (f.height as u64) * 4)
            .sum();
        let frames_to_load = if total_vram > MAX_GIF_VRAM_BYTES && !frames.is_empty() {
            let per_frame = total_vram / frames.len() as u64;
            let max_frames = (MAX_GIF_VRAM_BYTES / per_frame).max(1) as usize;
            tracing::warn!(
                "GIF VRAM budget exceeded ({:.1} MB), limiting to {} of {} frames",
                total_vram as f64 / 1_048_576.0,
                max_frames,
                frames.len()
            );
            &frames[..max_frames]
        } else {
            frames
        };

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        for frame in frames_to_load {
            let (width, height) = (frame.width, frame.height);
            let texture = self.device.create_texture(&TextureDescriptor {
                label: Some("GIF Frame Texture"),
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
                frame.as_rgba(),
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
            let bind_group = Arc::new(self.device.create_bind_group(&BindGroupDescriptor {
                label: Some("GIF Frame Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            self.uniform_buffer.as_entire_buffer_binding(),
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
            self.gif_textures.push((texture, bind_group));
        }
        tracing::debug!("Preloaded {} GIF frames to GPU", self.gif_textures.len());
        Ok(())
    }

    /// Swap the active bind group to a cached GIF frame (no GPU transfer).
    pub fn swap_gif_frame(&mut self, idx: usize) {
        if let Some((tex, bg)) = self.gif_textures.get(idx) {
            self.image_size = Some((tex.width(), tex.height()));
            self.image_bind_group = Some(Arc::clone(bg));
        }
    }

    pub fn update_scale_factor(&mut self, scale: f64) {
        self.scale_factor = scale;
    }

    /// Return the index into `self.thumbnails` for a given pixel click coordinate
    /// within the thumbnail strip. Returns None if the click is not in the strip.
    pub fn thumbnail_index_at(&self, x: f64, y: f64) -> Option<usize> {
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
            0.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_adjustments_default() {
        let adj = ImageAdjustments::default();

        assert_eq!(adj.brightness, 1.0);
        assert_eq!(adj.contrast, 1.0);
        assert_eq!(adj.saturation, 1.0);
        assert_eq!(adj.rotation, 0.0);
        assert_eq!(adj.crop_rect, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(adj.crop_rect_target, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(adj.hdr_toning, false);
    }
}
