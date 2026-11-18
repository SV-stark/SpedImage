use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::sync::Arc;

use super::super::{LoadOptions, apply_orientation, decode_box, downsample_to_fit, orient_box};
use crate::image::types::{ImageData, ImageFormatType};

pub(crate) fn load_tiff(
    path: &Path,
    max_w: Option<u32>,
    max_h: Option<u32>,
    exif: &crate::image::metadata::ExifMeta,
    opts: LoadOptions,
) -> Result<(Vec<ImageData>, ImageFormatType)> {
    use tiff::decoder::{Decoder, DecodingResult};
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let cursor = std::io::Cursor::new(&mmap[..]);
    let mut decoder = Decoder::new(cursor).map_err(|e| eyre!("TIFF decoder init failed: {e:?}"))?;

    let (width, height) = decoder
        .dimensions()
        .map_err(|e| eyre!("Failed to get TIFF dimensions: {e:?}"))?;

    let img_res = decoder
        .read_image()
        .map_err(|e| eyre!("TIFF decode failed: {e:?}"))?;

    let rgba_data = match img_res {
        DecodingResult::U8(v) => {
            use zune_core::colorspace::ColorSpace;
            use zune_image::image::Image;

            let colortype = decoder
                .colortype()
                .map_err(|e| eyre!("Failed to get TIFF colortype: {e:?}"))?;

            let input_space = match colortype {
                tiff::ColorType::RGB(8) => ColorSpace::RGB,
                tiff::ColorType::RGBA(8) => ColorSpace::RGBA,
                tiff::ColorType::Gray(8) => ColorSpace::Luma,
                _ => return Err(eyre!("Unsupported TIFF color type: {:?}", colortype)),
            };

            let mut img = Image::from_u8(&v, width as usize, height as usize, input_space);
            img.convert_color(ColorSpace::RGBA)?;
            img.flatten_to_u8().swap_remove(0)
        }
        _ => return Err(eyre!("Unsupported TIFF bit depth")),
    };

    let file_size = std::fs::metadata(path)?.len();
    let target_box = orient_box(decode_box(max_w, max_h), exif.orientation);
    let (rgba_data, width, height, is_downsampled) =
        downsample_to_fit(rgba_data, width, height, target_box)?;

    let (rgba_data, width, height, orientation_deg) = match exif.orientation {
        Some(o) => apply_orientation(rgba_data, width, height, o, opts.bake_orientation),
        None => (rgba_data, width, height, 0),
    };

    Ok((
        vec![ImageData {
            path: path.to_path_buf(),
            rgba_data: Arc::new(rgba_data),
            width,
            height,
            format: ImageFormatType::Tiff,
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
        ImageFormatType::Tiff,
    ))
}
