/// Read raw EXIF metadata bytes directly from image container chunks (JPEG / PNG / WebP) without decoding pixel data
#[allow(dead_code)]
pub fn extract_exif_bytes_lossless(path: &std::path::Path) -> Option<Vec<u8>> {
    use img_parts::ImageEXIF;
    let data = std::fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => {
            let jpeg = img_parts::jpeg::Jpeg::from_bytes(data.into()).ok()?;
            jpeg.exif().map(|b| b.to_vec())
        }
        "png" => {
            let png = img_parts::png::Png::from_bytes(data.into()).ok()?;
            png.exif().map(|b| b.to_vec())
        }
        "webp" => {
            let webp = img_parts::webp::WebP::from_bytes(data.into()).ok()?;
            webp.exif().map(|b| b.to_vec())
        }
        _ => None,
    }
}

pub fn format_exif_data(exif_data: &exif::Exif) -> Option<String> {
    let mut out = String::new();

    let mut add_field = |tag: exif::Tag, label: &str| {
        if let Some(field) = exif_data.get_field(tag, exif::In::PRIMARY) {
            out.push_str(label);
            out.push_str(&field.display_value().with_unit(exif_data).to_string());
            out.push('\n');
        }
    };

    add_field(exif::Tag::Make, "Make: ");
    add_field(exif::Tag::Model, "Model: ");
    add_field(exif::Tag::LensModel, "Lens: ");

    let mut exposure_line = String::new();
    if let Some(f) = exif_data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(exif_data).to_string());
        exposure_line.push_str("s  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        exposure_line.push_str("ISO ");
        exposure_line.push_str(&f.display_value().with_unit(exif_data).to_string());
    }

    add_field(exif::Tag::DateTimeOriginal, "Date: ");

    if !exposure_line.is_empty() {
        out.push_str("Exposure: ");
        out.push_str(&exposure_line);
        out.push('\n');
    }

    if out.is_empty() {
        None
    } else {
        Some(out.trim_end().to_string())
    }
}

/// Everything a single EXIF parse can yield, so the file is only opened and
/// scanned once per load instead of once per field.
#[derive(Debug, Default, Clone)]
pub struct ExifMeta {
    pub exif_info: Option<String>,
    pub orientation: Option<u32>,
    pub gps_coords: Option<(f64, f64)>,
    /// 1 = sRGB, 2 = Adobe RGB (only value with a GPU-side conversion matrix).
    pub color_space: Option<u32>,
}

/// Read a numeric EXIF field as `u32`, accepting the integer encodings the
/// spec allows for it.
fn numeric_field(exif_data: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    let field = exif_data.get_field(tag, exif::In::PRIMARY)?;
    match &field.value {
        exif::Value::Short(v) => v.first().map(|&x| x as u32),
        exif::Value::Byte(v) => v.first().map(|&x| x as u32),
        exif::Value::Long(v) => v.first().copied(),
        _ => None,
    }
}

impl ExifMeta {
    fn from_exif(exif_data: &exif::Exif) -> Self {
        Self {
            exif_info: format_exif_data(exif_data),
            orientation: numeric_field(exif_data, exif::Tag::Orientation),
            gps_coords: extract_gps(exif_data),
            color_space: numeric_field(exif_data, exif::Tag::ColorSpace),
        }
    }
}

/// Parse an EXIF block straight out of memory. Accepts either a raw APP1
/// payload or one that still carries the `Exif\0\0` identifier.
pub fn parse_exif_block(raw: &[u8]) -> Option<ExifMeta> {
    let tiff_block = raw.strip_prefix(b"Exif\0\0").unwrap_or(raw);
    if tiff_block.is_empty() {
        return None;
    }
    let exif_data = exif::Reader::new().read_raw(tiff_block.to_vec()).ok()?;
    Some(ExifMeta::from_exif(&exif_data))
}

/// Read EXIF (orientation, GPS, color space, display text) from a container on
/// disk. Never fails hard: a malformed or missing block yields defaults.
pub fn read_exif_meta(path: &std::path::Path) -> ExifMeta {
    let Ok(file) = std::fs::File::open(path) else {
        return ExifMeta::default();
    };
    let mut bufreader = std::io::BufReader::new(&file);
    exif::Reader::new()
        .read_from_container(&mut bufreader)
        .map(|e| ExifMeta::from_exif(&e))
        .unwrap_or_default()
}

pub fn extract_exif_lazy(path: &std::path::Path) -> Option<String> {
    read_exif_meta(path).exif_info
}

pub fn extract_orientation(path: &std::path::Path) -> Option<u32> {
    read_exif_meta(path).orientation
}

fn parse_gps_rational(field: &exif::Field) -> Option<f64> {
    if let exif::Value::Rational(ref values) = field.value
        && values.len() >= 3
    {
        let d = values[0].to_f64();
        let m = values[1].to_f64();
        let s = values[2].to_f64();
        return Some(d + m / 60.0 + s / 3600.0);
    }
    None
}

fn extract_gps(exif_data: &exif::Exif) -> Option<(f64, f64)> {
    let lat_val = exif_data.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)?;
    let lon_val = exif_data.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)?;

    let mut lat = parse_gps_rational(lat_val)?;
    let mut lon = parse_gps_rational(lon_val)?;

    if let Some(ref_field) = exif_data.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY) {
        let ref_val = ref_field.display_value().to_string();
        if ref_val.contains('S') || ref_val.contains('s') {
            lat = -lat;
        }
    }
    if let Some(ref_field) = exif_data.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY) {
        let ref_val = ref_field.display_value().to_string();
        if ref_val.contains('W') || ref_val.contains('w') {
            lon = -lon;
        }
    }

    Some((lat, lon))
}

#[allow(clippy::type_complexity)]
pub fn extract_exif_and_orientation(
    path: &std::path::Path,
) -> (Option<String>, Option<u32>, Option<(f64, f64)>, Option<u32>) {
    let meta = read_exif_meta(path);
    (
        meta.exif_info,
        meta.orientation,
        meta.gps_coords,
        meta.color_space,
    )
}
