use std::sync::Arc;
use wgpu::{BindGroup, Texture};

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
    pub pixel_perfect: bool, // Nearest-neighbor sampling for pixel art
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
            pixel_perfect: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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
/// A fully-uploaded thumbnail ready for GPU rendering.
pub struct ThumbnailEntry {
    pub path: std::path::PathBuf,
    /// Bind group pointing at the thumbnail texture (same layout as image pipeline).
    pub bind_group: Arc<BindGroup>,
    pub width: u32,
    pub height: u32,
    /// The GPU texture (kept alive so bind group stays valid).
    pub _texture: Texture,
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
        assert!(!adj.hdr_toning);
    }
}
