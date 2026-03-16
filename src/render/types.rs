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
    pub _padding: f32,
    pub pos_offset: [f32; 2],
    pub pos_scale: [f32; 2],
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
            _padding: 0.0,
            pos_offset: [0.0, 0.0],
            pos_scale: [1.0, 1.0],
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
    pub hdr_toning: bool,
    pub pixel_perfect: bool,
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
            hdr_toning: false,
            pixel_perfect: false,
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
    pub adjustments: &'a ImageAdjustments,
    pub is_cropping: bool,
    pub crop_rect: [f32; 4],
    pub status_text: Option<&'a str>,
    pub show_help: bool,
    pub sidebar_text: Option<&'a str>,
    pub show_thumbnail_strip: bool,
    pub thumb_scroll: f32,
    pub active_thumb_idx: Option<usize>,
    pub selected_indices: &'a std::collections::HashSet<usize>,
    pub exif_text: Option<&'a str>,
    pub show_histogram: bool,
    pub histogram_data: Option<&'a ([u32; 256], [u32; 256], [u32; 256])>,
}
