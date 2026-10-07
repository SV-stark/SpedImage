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
    pub sharpen: f32,
    pub clarity: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub split_compare: f32,
    pub split_position: f32,
    pub has_color_matrix: f32,
    pub _pad: f32,
    pub color_matrix_col0: [f32; 4],
    pub color_matrix_col1: [f32; 4],
    pub color_matrix_col2: [f32; 4],
    /// Linear sRGB -> linear display RGB, applied last. `display_col0[3]` is
    /// the enable flag (1.0 = on), which keeps the struct 16-byte aligned
    /// without a separate padded scalar.
    pub display_col0: [f32; 4],
    pub display_col1: [f32; 4],
    pub display_col2: [f32; 4],
}

/// Shader columns for a display matrix, or the disabled identity.
pub fn display_matrix_columns(m: Option<&crate::render::color::Mat3>) -> [[f32; 4]; 3] {
    match m {
        Some(m) => [
            [m[0][0] as f32, m[1][0] as f32, m[2][0] as f32, 1.0],
            [m[0][1] as f32, m[1][1] as f32, m[2][1] as f32, 0.0],
            [m[0][2] as f32, m[1][2] as f32, m[2][2] as f32, 0.0],
        ],
        None => [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ],
    }
}

impl Uniforms {
    /// Install the display matrix (or disable it).
    pub fn with_display_matrix(mut self, m: Option<&crate::render::color::Mat3>) -> Self {
        let [c0, c1, c2] = display_matrix_columns(m);
        self.display_col0 = c0;
        self.display_col1 = c1;
        self.display_col2 = c2;
        self
    }

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
            sharpen: 0.0,
            clarity: 0.0,
            temperature: 0.0,
            tint: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            split_compare: 0.0,
            split_position: 0.5,
            has_color_matrix: 0.0,
            _pad: 0.0,
            color_matrix_col0: [1.0, 0.0, 0.0, 0.0],
            color_matrix_col1: [0.0, 1.0, 0.0, 0.0],
            color_matrix_col2: [0.0, 0.0, 1.0, 0.0],
            display_col0: [1.0, 0.0, 0.0, 0.0],
            display_col1: [0.0, 1.0, 0.0, 0.0],
            display_col2: [0.0, 0.0, 1.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageAdjustments {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub rotation: f32,
    /// EXIF orientation applied before user rotation (radians).
    pub pre_rotation: f32,
    pub crop_rect: [f32; 4],
    pub crop_rect_target: [f32; 4],
    pub crop_rect_actual: Option<[f32; 4]>,
    pub hdr_toning: bool,
    pub pixel_perfect: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub color_space: Option<u32>,
    pub sharpen: f32,
    pub clarity: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub split_compare: bool,
    pub split_position: f32,
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            rotation: 0.0,
            pre_rotation: 0.0,
            crop_rect: [0.0, 0.0, 1.0, 1.0],
            crop_rect_target: [0.0, 0.0, 1.0, 1.0],
            crop_rect_actual: None,
            hdr_toning: false,
            pixel_perfect: false,
            flip_horizontal: false,
            flip_vertical: false,
            color_space: None,
            sharpen: 0.0,
            clarity: 0.0,
            temperature: 0.0,
            tint: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            split_compare: false,
            split_position: 0.5,
        }
    }
}

/// One slot of the streaming GIF texture ring.
pub struct GifSlot {
    /// Logical frame index this slot currently holds.
    pub index: usize,
    pub texture: Texture,
    pub bind_group: Arc<BindGroup>,
    pub bind_group_nearest: Arc<BindGroup>,
    pub width: u32,
    pub height: u32,
}

pub struct ThumbnailEntry {
    pub path: std::path::PathBuf,
    /// Position in the current directory listing (insert-order key).
    pub order: usize,
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
    /// Window size in physical pixels, used to lay out screen-space overlays.
    pub window_size: (u32, u32),
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
    pub current_image_info: Option<(String, u32, u32, u64, f32)>,
    pub show_osd: bool,
    /// A full-resolution re-decode is running for the zoomed-in image.
    pub is_refining: bool,
    /// File names of the pinned (left) and current (right) image while the
    /// side-by-side compare view is open.
    pub compare_labels: Option<(String, String)>,
    /// Persistent message with an OK button; cleared when dismissed.
    pub notice: &'a mut Option<String>,
}
