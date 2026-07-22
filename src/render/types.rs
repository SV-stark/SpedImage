use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::{BindGroup, Texture};

/// Height of the thumbnail strip in physical pixels.
pub const STRIP_HEIGHT_PX: u32 = 90;
/// Width of each thumbnail slot (including gap).
pub const THUMB_SLOT_W: u32 = 80;
/// Size of the thumbnail texture.
pub const THUMB_SIZE: u32 = 80;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub rotation: f32,
    pub aspect_ratio: f32,
    pub window_aspect_ratio: f32,
    pub crop_x: f32,
    pub crop_y: f32,
    pub crop_w: f32,
    pub crop_h: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hdr_toning: f32,
    pub transition_factor: f32, // 0.0 (old) -> 1.0 (new)
    pub pos_offset: [f32; 2],
    pub pos_scale: [f32; 2],
    pub flip_horizontal: f32,
    pub flip_vertical: f32,
    pub _padding1: f32,
    pub _padding2: f32,
    pub color_matrix_col0: [f32; 4],
    pub color_matrix_col1: [f32; 4],
    pub color_matrix_col2: [f32; 4],
    pub has_color_matrix: f32,
    pub _padding_cm1: f32,
    pub _padding_cm2: f32,
    pub _padding_cm3: f32,
}

impl Uniforms {
    pub fn identity() -> Self {
        Self {
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
            transition_factor: 1.0,
            pos_offset: [0.0, 0.0],
            pos_scale: [1.0, 1.0],
            flip_horizontal: 0.0,
            flip_vertical: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
            color_matrix_col0: [1.0, 0.0, 0.0, 0.0],
            color_matrix_col1: [0.0, 1.0, 0.0, 0.0],
            color_matrix_col2: [0.0, 0.0, 1.0, 0.0],
            has_color_matrix: 0.0,
            _padding_cm1: 0.0,
            _padding_cm2: 0.0,
            _padding_cm3: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageAdjustments {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub rotation: f32,
    pub crop_rect: [f32; 4],
    pub crop_rect_target: [f32; 4],
    pub crop_rect_actual: Option<[f32; 4]>,
    pub hdr_toning: bool,
    pub pixel_perfect: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub color_space: Option<u32>,
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            rotation: 0.0,
            crop_rect: [0.0, 0.0, 1.0, 1.0],
            crop_rect_target: [0.0, 0.0, 1.0, 1.0],
            crop_rect_actual: None,
            hdr_toning: false,
            pixel_perfect: false,
            flip_horizontal: false,
            flip_vertical: false,
            color_space: None,
        }
    }
}

pub struct ThumbnailEntry {
    pub path: std::path::PathBuf,
    pub texture: Texture,
    pub bind_group: Arc<BindGroup>,
    pub uniform_buffer: wgpu::Buffer,
    pub width: u32,
    pub height: u32,
}

pub struct RenderParams<'a> {
    pub adjustments: &'a mut ImageAdjustments,
    pub is_cropping: bool,
    pub crop_rect: [f32; 4],
    pub status_text: Option<&'a str>,
    pub show_help: bool,
    pub sidebar_text: Option<&'a str>,
    pub show_thumbnail_strip: bool,
    pub thumb_scroll: f32,
    pub active_thumb_idx: Option<usize>,
    pub selected_indices: &'a rustc_hash::FxHashSet<usize>,
    pub exif_text: Option<&'a str>,
    pub show_histogram: bool,
    pub histogram_data: Option<&'a ([u32; 256], [u32; 256], [u32; 256])>,
    pub transition_factor: f32,
    pub files: &'a [crate::ui::FileEntry],
    pub event_tx: &'a crossbeam_channel::Sender<crate::app::types::AppEvent>,
    pub event_proxy: &'a winit::event_loop::EventLoopProxy<crate::app::types::WakeUp>,
    pub is_loading: bool,
    pub has_image: bool,
    pub config: &'a mut crate::config::AppConfig,
    pub slideshow_active: &'a mut bool,
    pub slideshow_interval_secs: &'a mut u64,
    pub slideshow_progress: Option<f32>,
    pub show_search: &'a mut bool,
    pub search_query: &'a mut String,
    pub gps_coords: Option<(f64, f64)>,
}
