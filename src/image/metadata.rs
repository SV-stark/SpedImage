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
