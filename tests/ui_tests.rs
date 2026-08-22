// UI state navigation and manipulation tests
use spedimage_lib::ui::{FileEntry, UiState};
use std::path::PathBuf;

#[test]
fn test_ui_navigation_and_selection() {
    let mut ui = UiState::default();
    let files = vec![
        FileEntry::new(PathBuf::from("a.png")),
        FileEntry::new(PathBuf::from("b.jpg")),
        FileEntry::new(PathBuf::from("c.webp")),
    ];
    ui.files = files;
    ui.current_file_index = Some(0);

    assert_eq!(ui.current_file().unwrap().to_str().unwrap(), "a.png");

    ui.current_file_index = Some(1);
    assert_eq!(ui.current_file().unwrap().to_str().unwrap(), "b.jpg");

    ui.selected_indices.insert(0);
    ui.selected_indices.insert(2);
    assert!(ui.selected_indices.contains(&0));
    assert!(ui.selected_indices.contains(&2));
    assert!(!ui.selected_indices.contains(&1));
}

#[test]
fn test_ui_adjustments_and_toggles() {
    let mut ui = UiState::default();

    assert!(!ui.show_info);
    ui.toggle_info();
    assert!(ui.show_info);
    ui.toggle_info();
    assert!(!ui.show_info);

    assert!(!ui.show_help);
    ui.toggle_help();
    assert!(ui.show_help);

    ui.rotate_90();
    assert!((ui.adjustments.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-5);

    ui.adjustments.brightness = 1.5;
    ui.adjustments.contrast = 2.0;
    ui.reset_adjustments();
    assert_eq!(ui.adjustments.brightness, 1.0);
    assert_eq!(ui.adjustments.contrast, 1.0);
    assert_eq!(ui.adjustments.rotation, 0.0);
}
