use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::super::{LoadOptions, apply_orientation, decode_box, downsample_to_fit, orient_box};
use crate::image::types::{ImageData, ImageFormatType};

/// Decode a HEIC/HEIF still image.
///
/// AVIF is a HEIF container holding AV1, which `heic-rs` recognises and
/// refuses by name rather than mis-decoding. `ImageFormatType::is_supported`
/// already excludes AVIF, so this only runs on an explicit open.
pub(crate) fn load_heic(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
    format_type: ImageFormatType,
    exif: &crate::image::metadata::ExifMeta,
    opts: LoadOptions,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let data = &mmap[..];

    let options = heic_rs::DecodeOptions::default().with_layout(heic_rs::PixelLayout::Rgba8);
    let image =
        heic_rs::decode(data, &options).map_err(|e| eyre!("HEIC/HEIF decode failed: {}", e))?;

    let rgba = image.data;
    let (width, height) = (image.width, image.height);
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(eyre!(
            "HEIC/HEIF decoder returned {} bytes for {width}x{height}",
            rgba.len()
        ));
    }

    let file_size = std::fs::metadata(path)?.len();
    let target_box = orient_box(decode_box(max_w, max_h), exif.orientation);
    let (rgba, width, height, is_downsampled) = downsample_to_fit(rgba, width, height, target_box)?;

    let (rgba, width, height, orientation_deg) = match exif.orientation {
        Some(o) => apply_orientation(rgba, width, height, o, opts.bake_orientation),
        None => (rgba, width, height, 0),
    };

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width,
            height,
            format: format_type,
            file_size_bytes: file_size,
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled,
            orientation_deg,
            gps_coords: exif.gps_coords,
            color_space: exif.color_space,
        }],
        format_type,
    ))
}
