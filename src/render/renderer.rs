use color_eyre::eyre::{Context, ContextCompat, Result};
use std::sync::Arc;
use wgpu::{
    BindGroup, Device, Queue, RenderPipeline, Sampler, Surface, SurfaceConfiguration, Texture,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use super::types::{ImageAdjustments, ThumbnailEntry, Uniforms};
use crate::image::ImageData;

pub struct Renderer {
    pub(crate) _window: Arc<Window>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) surface: Surface<'static>,
    pub(crate) pipeline: RenderPipeline,
    pub(crate) uniform_buffer: wgpu::Buffer,
    pub(crate) thumb_uniform_buffer: wgpu::Buffer,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) sampler: Sampler,
    pub(crate) sampler_nearest: Sampler,
    pub(crate) image_texture: Option<Texture>,
    pub(crate) image_bind_group: Option<Arc<BindGroup>>,
    pub(crate) image_bind_group_nearest: Option<Arc<BindGroup>>,
    pub gif_textures: Vec<(Texture, Arc<BindGroup>)>,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) image_size: Option<(u32, u32)>,
    pub scale_factor: f64,

    // egui
    pub(crate) egui_state: egui_winit::State,
    pub(crate) egui_renderer: egui_wgpu::Renderer,

    pub thumbnails: Vec<ThumbnailEntry>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let (device, queue, surface, adapter) =
            Self::create_device_and_surface(window.clone()).await?;
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (pipeline, _crop_pipeline) = Self::create_pipelines(&device, format)?;
        let (vertex_buffer, uniform_buffer, thumb_uniform_buffer) = Self::create_buffers(&device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler (Nearest)"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        Ok(Self {
            _window: window.clone(),
            device,
            queue,
            surface,
            pipeline,
            uniform_buffer,
            thumb_uniform_buffer,
            vertex_buffer,
            sampler,
            sampler_nearest,
            image_texture: None,
            image_bind_group: None,
            image_bind_group_nearest: None,
            gif_textures: Vec::new(),
            config,
            image_size: None,
            scale_factor: window.scale_factor(),
            egui_state,
            egui_renderer,
            thumbnails: Vec::new(),
        })
    }

    async fn create_device_and_surface(
        window: Arc<Window>,
    ) -> Result<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::Surface<'static>,
        wgpu::Adapter,
    )> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create WGPU surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("Failed to request WGPU adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("SpedImage Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .context("Failed to request WGPU device")?;

        Ok((device, queue, surface, adapter))
    }

    fn create_pipelines(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<(wgpu::RenderPipeline, wgpu::RenderPipeline)> {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::render::shaders::SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let crop_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Crop Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::render::shaders::CROP_SHADER.into()),
        });

        let crop_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crop Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let crop_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crop Overlay Pipeline"),
            layout: Some(&crop_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &crop_shader_module,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &crop_shader_module,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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

        Ok((pipeline, crop_pipeline))
    }

    fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        use wgpu::util::DeviceExt;
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

        let thumb_uniforms = Uniforms::identity();
        let thumb_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Thumbnail Uniform Buffer"),
            contents: bytemuck::bytes_of(&thumb_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        (vertex_buffer, uniform_buffer, thumb_uniform_buffer)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn load_image(&mut self, image_data: &ImageData) -> Result<()> {
        if let Some(old_tex) = self.image_texture.take() {
            old_tex.destroy();
        }
        self.image_bind_group = None;
        self.image_bind_group_nearest = None;

        let width = image_data.width;
        let height = image_data.height;

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Texture"),
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
            image_data.as_rgba(),
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
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        let bind_group = Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group"),
            layout: &bind_group_layout,
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

        let bind_group_nearest =
            Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Image Bind Group (Nearest)"),
                layout: &bind_group_layout,
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
                ],
            }));

        self.image_texture = Some(texture);
        self.image_bind_group = Some(bind_group);
        self.image_bind_group_nearest = Some(bind_group_nearest);
        self.image_size = Some((width, height));

        Ok(())
    }

    pub(crate) fn encode_image(
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
            pos_offset: [0.0, 0.0],
            pos_scale: [1.0, 1.0],
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some(bind_group) = &self.image_bind_group {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            if adjustments.pixel_perfect {
                if let Some(bg) = &self.image_bind_group_nearest {
                    render_pass.set_bind_group(0, bg.as_ref(), &[]);
                }
            } else {
                render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
            }
            render_pass.draw(0..6, 0..1);
        }
    }

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
