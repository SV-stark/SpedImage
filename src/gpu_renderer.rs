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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageAdjustments {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub rotation: f32,
    pub crop_rect: [f32; 4],
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            rotation: 0.0,
            crop_rect: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    rotation: f32,
    aspect_ratio: f32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
}

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    rotation: f32,
    aspect_ratio: f32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
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
    pos.x *= uniforms.aspect_ratio;
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
    let gray = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    color = mix(vec3<f32>(gray), color, uniforms.saturation);
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

pub struct Renderer {
    _window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    pipeline: RenderPipeline,
    crop_pipeline: RenderPipeline,
    uniform_buffer: wgpu::Buffer,
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
                    visibility: wgpu::ShaderStages::VERTEX,
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

    pub fn render(&self, adjustments: &ImageAdjustments) -> Result<()> {
        let frame = self
            .surface
            .get_current_texture()
            .context("Failed to get current surface texture")?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let uniforms = Uniforms {
            rotation: adjustments.rotation,
            aspect_ratio: self
                .image_size
                .map(|(w, h)| w as f32 / h as f32)
                .unwrap_or(1.0),
            crop_x: adjustments.crop_rect[0],
            crop_y: adjustments.crop_rect[1],
            crop_w: adjustments.crop_rect[2],
            crop_h: adjustments.crop_rect[3],
            brightness: adjustments.brightness,
            contrast: adjustments.contrast,
            saturation: adjustments.saturation,
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
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
            });

            if let Some(bind_group) = &self.image_bind_group {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
                render_pass.draw(0..6, 0..1);
            }
        }

        self.queue.submit([encoder.finish()]);
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

        // 1. Text rendering — queue all sections
        let scale = self.scale_factor as f32;

        // Hoist all owned strings so they outlive the borrowed Section slices below
        let help_text = "Shortcuts:\nA/W: Prev Image\nD/S: Next Image\nR: Rotate\nC: Toggle Crop\nCtrl+S: Save\nF: Toggle Sidebar\nEsc: Quit";
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

        let mut has_text = false;

        if let Some(status) = status_text {
            self.text_brush.queue(
                Section::default()
                    .add_text(
                        Text::new(status)
                            .with_scale(18.0 * scale)
                            .with_color([1.0f32, 1.0, 1.0, 1.0]),
                    )
                    .with_screen_position((10.0 * scale, self.config.height as f32 - 30.0 * scale)),
            );
            has_text = true;
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
            has_text = true;
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
            has_text = true;
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("UI Overlay Pass"),
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
                &mut encoder,
                &view,
                self.config.width,
                self.config.height,
            ) {
                tracing::warn!("Text draw error: {}", e);
            }
            self.staging_belt.finish();
        }

        self.queue.submit([encoder.finish()]);
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
        self.gif_textures.clear();
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        for frame in frames {
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
}
