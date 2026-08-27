use color_eyre::eyre::{Context, Result};
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
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) sampler: Sampler,
    pub(crate) sampler_nearest: Sampler,
    pub(crate) image_texture: Option<Texture>,
    pub(crate) image_texture_prev: Option<Texture>,
    pub(crate) image_bind_group: Option<Arc<BindGroup>>,
    pub(crate) image_bind_group_nearest: Option<Arc<BindGroup>>,
    pub(crate) image_bind_group_prev: Option<Arc<BindGroup>>,
    pub gif_ring: Vec<Option<super::types::GifSlot>>,
    pub(crate) texture_pool: Vec<Texture>,
    pub(crate) texture_pool_bytes: u64,
    /// True once the current image texture has a full mip chain.
    pub(crate) image_mipmapped: bool,
    pub(crate) mip_pipeline: RenderPipeline,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) image_size: Option<(u32, u32)>,
    pub scale_factor: f64,

    // egui
    pub(crate) egui_state: egui_winit::State,
    pub(crate) egui_renderer: egui_wgpu::Renderer,

    pub thumbnails: Vec<ThumbnailEntry>,
    pub(crate) last_thumb_state: Option<(u32, u32, f32)>,
}

/// VRAM budget for recycled image textures.
const TEXTURE_POOL_MAX_BYTES: u64 = 64 * 1024 * 1024;

fn texture_bytes(t: &Texture) -> u64 {
    let s = t.size();
    s.width as u64 * s.height as u64 * 4
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        crate::startup::log("Renderer::new enter");
        let (device, queue, surface, adapter) =
            Self::create_device_and_surface(window.clone()).await?;
        crate::startup::log("Renderer::new after device/surface");

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
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (pipeline, _crop_pipeline) = Self::create_pipelines(&device, format)?;
        let (vertex_buffer, uniform_buffer) = Self::create_buffers(&device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler (Nearest)"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let egui_ctx = egui::Context::default();
        let mut style: egui::Style = (*egui_ctx.global_style()).clone();
        style.visuals.window_corner_radius = 12.0.into();
        style.visuals.window_fill = egui::Color32::from_rgb(18, 25, 41); // Slate panel #121929
        style.visuals.panel_fill = egui::Color32::from_rgb(11, 15, 25); // Darker #0b0f19
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(11, 15, 25);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 35, 58);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 180, 216); // Cyan accent
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 150, 180);
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(200, 210, 230);
        style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(11, 15, 25);
        style.visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 180, 216, 120);
        egui_ctx.set_global_style(style);

        let egui_state = egui_winit::State::new(
            egui_ctx,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer =
            egui_wgpu::Renderer::new(&device, format, egui_wgpu::RendererOptions::default());

        let mip_pipeline = Self::create_mip_pipeline(&device);
        crate::startup::log("Renderer::new done");

        Ok(Self {
            _window: window.clone(),
            device,
            queue,
            surface,
            pipeline,
            uniform_buffer,
            vertex_buffer,
            sampler,
            sampler_nearest,
            image_texture: None,
            image_texture_prev: None,
            image_bind_group: None,
            image_bind_group_nearest: None,
            image_bind_group_prev: None,
            gif_ring: (0..crate::app::constants::GIF_RING_SLOTS)
                .map(|_| None)
                .collect(),
            texture_pool: Vec::new(),
            texture_pool_bytes: 0,
            image_mipmapped: false,
            mip_pipeline,
            config,
            image_size: None,
            scale_factor: window.scale_factor(),
            egui_state,
            egui_renderer,
            thumbnails: Vec::new(),
            last_thumb_state: None,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Destroy previous texture if it exists
        if let Some(tex) = self.image_texture_prev.take() {
            tex.destroy();
        }

        // Destroy all textures in the reuse pool
        for tex in self.texture_pool.drain(..) {
            tex.destroy();
        }
        self.texture_pool_bytes = 0;

        // Destroy all thumbnail textures and uniform buffers
        for thumb in self.thumbnails.drain(..) {
            thumb.texture.destroy();
            thumb.uniform_buffer.destroy();
        }

        // Destroy all GIF ring textures
        for slot in self.gif_ring.iter_mut() {
            if let Some(s) = slot.take() {
                s.texture.destroy();
            }
        }
    }
}

impl Renderer {
    async fn create_device_and_surface(
        window: Arc<Window>,
    ) -> Result<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::Surface<'static>,
        wgpu::Adapter,
    )> {
        // Fastest cold start on Windows: DX12 only (no Vulkan ICD enumeration,
        // no GL driver probing). Matches Photos' D3D path and is ~100-300 ms
        // faster than Vulkan|DX12 on Intel iGPUs.
        #[cfg(windows)]
        let backends = wgpu::Backends::DX12;
        #[cfg(not(windows))]
        let backends = wgpu::Backends::PRIMARY;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create WGPU surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // Prefer the integrated/low-power GPU — it is the one driving
                // the window on this laptop and avoids enumerating the (absent)
                // discrete adapter. Mirrors the OS compositor's choice.
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("Failed to request WGPU adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("SpedImage Device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::Off,
            })
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
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
                    // Decoded buffers use straight alpha; premultiplied blending
                    // darkened transparent edges (PNG/WebP halos).
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            multiview_mask: None,
            cache: None,
        });

        let crop_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Crop Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::render::shaders::CROP_SHADER.into()),
        });

        let crop_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crop Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            multiview_mask: None,
            cache: None,
        });

        Ok((pipeline, crop_pipeline))
    }

    /// Pipeline used for the lazy GPU mipmap blit chain.
    fn create_mip_pipeline(device: &wgpu::Device) -> RenderPipeline {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mipmap Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::render::shaders::MIP_SHADER.into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mipmap Blit Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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
            label: Some("Mipmap Blit Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mipmap Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    // Must match the mipped image texture format.
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }

    fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
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

        (vertex_buffer, uniform_buffer)
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
        let width = image_data.width;
        let height = image_data.height;

        // Move current to prev for transition
        if let Some(current_tex) = self.image_texture.take()
            && let Some(old_prev) = self.image_texture_prev.replace(current_tex)
        {
            if self.texture_pool.len() < 4 {
                self.texture_pool_bytes += texture_bytes(&old_prev);
                self.texture_pool.push(old_prev);
            } else {
                old_prev.destroy();
            }
        }
        self.image_bind_group_prev = self.image_bind_group.take();

        let mut recycled_texture = None;
        if let Some(pos) = self.texture_pool.iter().position(|t| {
            let size = t.size();
            size.width == width && size.height == height
        }) {
            let tex = self.texture_pool.remove(pos);
            self.texture_pool_bytes -= texture_bytes(&tex);
            recycled_texture = Some(tex);
        }
        self.enforce_pool_budget();

        self.image_mipmapped = false;

        let mip_level_count = 1;

        let texture = recycled_texture.unwrap_or_else(|| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let view_prev = self
            .image_texture_prev
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // If no prev view, create a tiny black texture so the shader doesn't crash
        let black_tex = if view_prev.is_none() {
            let t = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Black Texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            Some(t)
        } else {
            None
        };

        let black_view = black_tex
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        let actual_view_prev = view_prev.as_ref().or(black_view.as_ref()).unwrap();

        let (bind_group, bind_group_nearest) =
            self.create_image_bind_groups(&view, actual_view_prev);

        self.image_texture = Some(texture);
        self.image_bind_group = Some(bind_group);
        self.image_bind_group_nearest = Some(bind_group_nearest);
        self.image_size = Some((width, height));

        if let Some(texture) = &self.image_texture {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
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
        }

        Ok(())
    }

    /// Destroy oldest pooled textures until the pool fits its byte budget.
    fn enforce_pool_budget(&mut self) {
        while self.texture_pool_bytes > TEXTURE_POOL_MAX_BYTES {
            let Some(tex) = self.texture_pool.first() else {
                break;
            };
            self.texture_pool_bytes -= texture_bytes(tex);
            let tex = self.texture_pool.remove(0);
            tex.destroy();
        }
    }

    fn create_image_bind_groups(
        &self,
        view: &wgpu::TextureView,
        view_prev: &wgpu::TextureView,
    ) -> (Arc<BindGroup>, Arc<BindGroup>) {
        let layout = self.pipeline.get_bind_group_layout(0);
        let linear = Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group"),
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
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(view_prev),
                },
            ],
        }));
        let nearest = Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group (Nearest)"),
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
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(view_prev),
                },
            ],
        }));
        (linear, nearest)
    }

    /// Lazily build a full mip chain for the current image texture.
    ///
    /// The texture is recreated with `TEXTURE_BINDING | COPY_DST | RENDER_ATTACHMENT`
    /// usage, level 0 is re-uploaded from the CPU buffer, and each successive
    /// level is downsampled on the GPU with a blit pass. Zero cost until the
    /// image is actually displayed minified beyond ~50%.
    pub fn ensure_mipmapped(&mut self, image_data: &ImageData) -> Result<()> {
        if self.image_mipmapped {
            return Ok(());
        }
        let Some(old_tex) = self.image_texture.as_ref() else {
            return Ok(());
        };
        let (width, height) = (old_tex.width(), old_tex.height());
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Ok(());
        }

        let levels = width.max(height).ilog2() + 1;
        if levels <= 1 {
            self.image_mipmapped = true;
            return Ok(());
        }

        let new_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Texture (Mipped)"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &new_tex,
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

        // Blit chain: downsample level i-1 into level i.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mipmap Generation"),
            });
        for level in 1..levels {
            let src_view = new_tex.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_view = new_tex.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Mipmap Blit Bind Group"),
                layout: &self.mip_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mipmap Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.mip_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..6, 0..1);
        }
        self.queue.submit([encoder.finish()]);

        // Swap in the mipped texture and rebuild bind groups against it.
        let full_view = new_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let prev_view = self
            .image_texture_prev
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        let black_fallback;
        let actual_view_prev = match &prev_view {
            Some(v) => v,
            None => {
                let t = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Black Texture"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                black_fallback = t.create_view(&wgpu::TextureViewDescriptor::default());
                &black_fallback
            }
        };

        old_tex.destroy();
        let (bg_linear, bg_nearest) = self.create_image_bind_groups(&full_view, actual_view_prev);
        self.image_texture = Some(new_tex);
        self.image_bind_group = Some(bg_linear);
        self.image_bind_group_nearest = Some(bg_nearest);
        self.image_mipmapped = true;

        Ok(())
    }

    pub(crate) fn encode_image(
        &self,
        adjustments: &ImageAdjustments,
        transition_factor: f32,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let window_aspect_ratio = if self.config.height > 0 {
            self.config.width as f32 / self.config.height as f32
        } else {
            1.0
        };

        let total_rotation = adjustments.rotation + adjustments.pre_rotation;
        let rot_deg = (total_rotation.to_degrees() % 360.0).round().abs();
        let is_sideways = (rot_deg - 90.0).abs() < 1.0 || (rot_deg - 270.0).abs() < 1.0;
        let raw_aspect = self
            .image_size
            .map(|(w, h)| w as f32 / h as f32)
            .unwrap_or(1.0);
        let aspect_ratio = if is_sideways {
            1.0 / raw_aspect
        } else {
            raw_aspect
        };

        let (has_cm, col0, col1, col2) = if adjustments.color_space == Some(2) {
            (
                1.0f32,
                [1.3983, -0.0051, 0.0655, 0.0],
                [-0.3867, 1.0151, -0.1082, 0.0],
                [-0.0116, -0.0100, 1.1518, 0.0],
            )
        } else {
            (
                0.0f32,
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            )
        };

        let uniforms = Uniforms {
            rotation: total_rotation,
            aspect_ratio,
            window_aspect_ratio,
            crop_x: adjustments.crop_rect[0],
            crop_y: adjustments.crop_rect[1],
            crop_w: adjustments.crop_rect[2],
            crop_h: adjustments.crop_rect[3],
            brightness: adjustments.brightness,
            contrast: adjustments.contrast,
            saturation: adjustments.saturation,
            hdr_toning: if adjustments.hdr_toning { 1.0 } else { 0.0 },
            transition_factor,
            pos_offset: [0.0, 0.0],
            pos_scale: [1.0, 1.0],
            flip_horizontal: if adjustments.flip_horizontal {
                1.0
            } else {
                0.0
            },
            flip_vertical: if adjustments.flip_vertical { 1.0 } else { 0.0 },
            sharpen: adjustments.sharpen,
            clarity: adjustments.clarity,
            temperature: adjustments.temperature,
            tint: adjustments.tint,
            highlights: adjustments.highlights,
            shadows: adjustments.shadows,
            split_compare: if adjustments.split_compare { 1.0 } else { 0.0 },
            split_position: adjustments.split_position,
            has_color_matrix: has_cm,
            _pad: 0.0,
            color_matrix_col0: col0,
            color_matrix_col1: col1,
            color_matrix_col2: col2,
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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some(bind_group) = &self.image_bind_group {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            if adjustments.pixel_perfect || adjustments.crop_rect[2] < 0.2 {
                if let Some(bg) = &self.image_bind_group_nearest {
                    render_pass.set_bind_group(0, bg.as_ref(), &[]);
                }
            } else {
                render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
            }
            render_pass.draw(0..6, 0..1);
        }
    }

    /// Point the image bind groups at ring slot `index` if that frame has
    /// arrived. Returns false when playback must wait for the streamer.
    pub fn swap_gif_frame(&mut self, index: usize) -> bool {
        let pos = index % crate::app::constants::GIF_RING_SLOTS;
        match &self.gif_ring[pos] {
            Some(slot) if slot.index == index => {
                self.image_size = Some((slot.width, slot.height));
                self.image_bind_group = Some(Arc::clone(&slot.bind_group));
                self.image_bind_group_nearest = Some(Arc::clone(&slot.bind_group_nearest));
                true
            }
            _ => false,
        }
    }

    pub fn update_scale_factor(&mut self, scale: f64) {
        self.scale_factor = scale;
    }
}
