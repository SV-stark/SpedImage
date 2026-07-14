pub fn extract_exif_lazy(path: &std::path::Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut bufreader).ok()?;

    let mut out = String::new();

    // Helper to append EXIF fields concisely
    let mut add_field = |tag: exif::Tag, label: &str| {
        if let Some(field) = exif_data.get_field(tag, exif::In::PRIMARY) {
            out.push_str(label);
            out.push_str(&field.display_value().with_unit(&exif_data).to_string());
            out.push('\n');
        }
    };

    add_field(exif::Tag::Make, "Make: ");
    add_field(exif::Tag::Model, "Model: ");
    add_field(exif::Tag::LensModel, "Lens: ");

    let mut exposure_line = String::new();
    if let Some(f) = exif_data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("s  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        exposure_line.push_str("ISO ");
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
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

pub fn extract_orientation(path: &std::path::Path) -> Option<u32> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut bufreader).ok()?;

    if let Some(field) = exif_data.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
        match &field.value {
            exif::Value::Short(v) => v.first().map(|&x| x as u32),
            exif::Value::Byte(v) => v.first().map(|&x| x as u32),
            exif::Value::Long(v) => v.first().copied(),
            _ => None,
        }
    } else {
        None
    }
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
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None, None, None),
    };
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = match exifreader.read_from_container(&mut bufreader) {
        Ok(data) => data,
        Err(_) => return (None, None, None, None),
    };

    let mut out = String::new();

    // Helper to append EXIF fields concisely
    let mut add_field = |tag: exif::Tag, label: &str| {
        if let Some(field) = exif_data.get_field(tag, exif::In::PRIMARY) {
            out.push_str(label);
            out.push_str(&field.display_value().with_unit(&exif_data).to_string());
            out.push('\n');
        }
    };

    add_field(exif::Tag::Make, "Make: ");
    add_field(exif::Tag::Model, "Model: ");
    add_field(exif::Tag::LensModel, "Lens: ");

    let mut exposure_line = String::new();
    if let Some(f) = exif_data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
        exposure_line.push_str("s  ");
    }
    if let Some(f) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        exposure_line.push_str("ISO ");
        exposure_line.push_str(&f.display_value().with_unit(&exif_data).to_string());
    }

    add_field(exif::Tag::DateTimeOriginal, "Date: ");

    if !exposure_line.is_empty() {
        out.push_str("Exposure: ");
        out.push_str(&exposure_line);
        out.push('\n');
    }

    let exif_info = if out.is_empty() {
        None
    } else {
        Some(out.trim_end().to_string())
    };

    let orientation =
        if let Some(field) = exif_data.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
            match &field.value {
                exif::Value::Short(v) => v.first().map(|&x| x as u32),
                exif::Value::Byte(v) => v.first().map(|&x| x as u32),
                exif::Value::Long(v) => v.first().copied(),
                _ => None,
            }
        } else {
            None
        };

    let gps_coords = extract_gps(&exif_data);

    let color_space =
        if let Some(field) = exif_data.get_field(exif::Tag::ColorSpace, exif::In::PRIMARY) {
            match &field.value {
                exif::Value::Short(v) => v.first().map(|&x| x as u32),
                exif::Value::Byte(v) => v.first().map(|&x| x as u32),
                exif::Value::Long(v) => v.first().copied(),
                _ => None,
            }
        } else {
            None
        };

    (exif_info, orientation, gps_coords, color_space)
}
