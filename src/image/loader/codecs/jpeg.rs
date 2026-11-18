use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::super::{
    LoadOptions, apply_color_profile, apply_orientation, decode_box, downsample_to_fit, orient_box,
    pick_dct_scale,
};
use crate::image::types::{ImageData, ImageFormatType};

/// Decode a JPEG, using libjpeg-turbo's scaled IDCT so thumbnails and
/// previews never pay for a full-resolution decode of a 24 MP photo.
pub(crate) fn load_jpeg(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
    opts: LoadOptions,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    let file =
        std::fs::File::open(path).map_err(|e| eyre!("Failed to open JPEG file {path:?}: {e:?}"))?;
    let mmap = unsafe {
        memmap2::Mmap::map(&file).map_err(|e| eyre!("Failed to memory map JPEG {path:?}: {e:?}"))?
    };

    // Header parse only: gives dimensions, EXIF orientation, ICC and EXIF
    // blocks with no pixel work.
    let mut decoder = libjpeg_turbo_rs::Decoder::new(&mmap[..])
        .map_err(|e| eyre!("Failed to parse JPEG {path:?}: {e:?}"))?;

    let header = decoder.header();
    let src_w = header.width() as u32;
    let src_h = header.height() as u32;
    if src_w == 0 || src_h == 0 {
        return Err(eyre!("JPEG {path:?} reports a zero dimension"));
    }

    let orientation = decoder.exif_orientation().map(u32::from);
    let target_box = orient_box(decode_box(max_w, max_h), orientation);

    // Largest DCT reduction that still leaves room for a downscale-only
    // CPU pass, so the resize never has to upscale. Also keeps the
    // intermediate buffer small: a 24 MP RGBA frame is 96 MB at 1/1 but
    // only 1.5 MB at 1/8.
    let scale = match target_box {
        Some((tw, th)) => pick_dct_scale(src_w, src_h, tw, th),
        None => libjpeg_turbo_rs::ScalingFactor::new(1, 1),
    };
    decoder.set_scale(scale);
    decoder.set_output_format(libjpeg_turbo_rs::PixelFormat::Rgba);

    let mut buf = vec![0u8; decoder.output_buffer_size()?];
    let info = decoder.decode_image_into(&mut buf)?;
    let dec_w = info.width as u32;
    let dec_h = info.height as u32;
    // `output_buffer_size` is an upper bound on some paths; trim to what
    // was actually written before the buffer is reused as an RGBA image.
    buf.truncate(info.bytes_written);

    let icc = info.icc_profile.clone();
    let meta = info
        .exif_data
        .as_deref()
        .and_then(crate::image::metadata::parse_exif_block)
        .unwrap_or_default();

    let mut rgba = buf;
    let (mut final_w, mut final_h) = (dec_w, dec_h);
    let mut is_downsampled = scale.denom > 1;

    if let Some(box_wh) = target_box {
        let (r, w, h, downsampled) = downsample_to_fit(rgba, dec_w, dec_h, Some(box_wh))?;
        rgba = r;
        final_w = w;
        final_h = h;
        is_downsampled |= downsampled;
    }

    // ICC transform runs on the reduced buffer, so it is proportionally cheaper.
    if let Some(ref icc_bytes) = icc
        && let Err(e) = apply_color_profile(&mut rgba, icc_bytes)
    {
        tracing::warn!("Failed to apply color profile for {path:?}: {e:?}");
    }

    let (rgba, final_w, final_h, orientation_deg) = match orientation {
        Some(o) => apply_orientation(rgba, final_w, final_h, o, opts.bake_orientation),
        None => (rgba, final_w, final_h, 0),
    };

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba),
            width: final_w,
            height: final_h,
            format: ImageFormatType::Jpeg,
            file_size_bytes: file.metadata()?.len(),
            frame_delay_ms: 0,
            exif_info: None,
            exif_loaded: false,
            histogram: None,
            is_downsampled,
            orientation_deg,
            gps_coords: meta.gps_coords,
            color_space: meta.color_space,
        }],
        ImageFormatType::Jpeg,
    ))
}
