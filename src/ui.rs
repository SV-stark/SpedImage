//! UI Layer - User interface components

use crate::render::ImageAdjustments;
use std::path::PathBuf;

/// File entry for the sidebar
#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_image: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string());
        Self {
            path,
            name,
            is_image: true,
        }
    }
}

pub struct UiState {
    pub files: Vec<FileEntry>,
    pub current_file_index: Option<usize>,
    pub adjustments: ImageAdjustments,
    pub is_cropping: bool,
    pub show_help: bool,
    pub show_sidebar: bool,
    pub show_thumbnail_strip: bool,
    pub show_info: bool,
    pub show_histogram: bool,
    pub selected_indices: std::collections::HashSet<usize>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub sidebar_text: Option<String>,
    pub show_search: bool,
    pub search_query: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            current_file_index: None,
            adjustments: ImageAdjustments::default(),
            is_cropping: false,
            show_help: false,
            show_sidebar: false,
            show_thumbnail_strip: true,
            show_info: false,
            show_histogram: false,
            selected_indices: std::collections::HashSet::new(),
            status_message: None,
            sidebar_text: None,
            show_search: false,
            search_query: String::new(),
        }
    }
}

impl UiState {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn get_status(&self) -> &str {
        if let Some((ref msg, time)) = self.status_message
            && time.elapsed().as_secs() < 3
        {
            return msg;
        }
        ""
    }

    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file_index
            .and_then(|i| self.files.get(i).map(|f| &f.path))
    }

    pub fn next_file(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = self.current_file_index.unwrap_or(0);
        self.current_file_index = Some((i + 1) % self.files.len());
        self.sidebar_text = None;
    }

    pub fn prev_file(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = self.current_file_index.unwrap_or(0);
        self.current_file_index = Some(if i == 0 { self.files.len() - 1 } else { i - 1 });
        self.sidebar_text = None;
    }

    pub fn rotate_90(&mut self) {
        self.adjustments.rotation += std::f32::consts::FRAC_PI_2;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }

    pub fn reset_adjustments(&mut self) {
        self.adjustments = ImageAdjustments::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            is_image: true,
        }
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry::new(PathBuf::from("/photos/image.jpg"));
        assert_eq!(entry.name, "image.jpg");
        assert_eq!(entry.path, PathBuf::from("/photos/image.jpg"));
        assert!(entry.is_image);
    }

    #[test]
    fn test_file_entry_unknown_name() {
        let entry = FileEntry {
            path: PathBuf::from(""),
            name: String::new(),
            is_image: true,
        };
        assert_eq!(entry.name, "");
    }

    #[test]
    fn test_ui_state_default() {
        let ui = UiState::default();
        assert!(ui.files.is_empty());
        assert_eq!(ui.current_file_index, None);
        assert!(!ui.is_cropping);
        assert!(!ui.show_help);
        assert!(!ui.show_sidebar);
        assert!(ui.show_thumbnail_strip);
        assert!(!ui.show_info);
        assert!(!ui.show_histogram);
        assert!(ui.selected_indices.is_empty());
        assert!(ui.status_message.is_none());
        assert!(ui.sidebar_text.is_none());
    }

    #[test]
    fn test_ui_state_next_file() {
        let mut ui = UiState::default();
        ui.files = vec![
            make_entry("a.jpg"),
            make_entry("b.jpg"),
            make_entry("c.jpg"),
        ];

        // First call: 0 -> 1
        ui.next_file();
        assert_eq!(ui.current_file_index, Some(1));

        // Second call: 1 -> 2
        ui.next_file();
        assert_eq!(ui.current_file_index, Some(2));

        // Third call: wraps around 2 -> 0
        ui.next_file();
        assert_eq!(ui.current_file_index, Some(0));
    }

    #[test]
    fn test_ui_state_next_file_empty() {
        let mut ui = UiState::default();
        ui.next_file();
        assert_eq!(ui.current_file_index, None);
    }

    #[test]
    fn test_ui_state_prev_file() {
        let mut ui = UiState::default();
        ui.files = vec![
            make_entry("a.jpg"),
            make_entry("b.jpg"),
            make_entry("c.jpg"),
        ];

        // First call: default(0) -> 2 (wraps)
        ui.prev_file();
        assert_eq!(ui.current_file_index, Some(2));

        // Second call: 2 -> 1
        ui.prev_file();
        assert_eq!(ui.current_file_index, Some(1));

        // Third call: 1 -> 0
        ui.prev_file();
        assert_eq!(ui.current_file_index, Some(0));
    }

    #[test]
    fn test_ui_state_prev_file_empty() {
        let mut ui = UiState::default();
        ui.prev_file();
        assert_eq!(ui.current_file_index, None);
    }

    #[test]
    fn test_ui_state_rotate_90() {
        let mut ui = UiState::default();
        assert_eq!(ui.adjustments.rotation, 0.0);
        ui.rotate_90();
        assert!((ui.adjustments.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        ui.rotate_90();
        assert!((ui.adjustments.rotation - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_ui_state_toggle_help() {
        let mut ui = UiState::default();
        assert!(!ui.show_help);
        ui.toggle_help();
        assert!(ui.show_help);
        ui.toggle_help();
        assert!(!ui.show_help);
    }

    #[test]
    fn test_ui_state_toggle_info() {
        let mut ui = UiState::default();
        assert!(!ui.show_info);
        ui.toggle_info();
        assert!(ui.show_info);
        ui.toggle_info();
        assert!(!ui.show_info);
    }

    #[test]
    fn test_ui_state_reset_adjustments() {
        let mut ui = UiState::default();
        ui.adjustments.brightness = 1.5;
        ui.adjustments.contrast = 1.3;
        ui.adjustments.rotation = std::f32::consts::FRAC_PI_2;
        ui.adjustments.crop_rect_actual = Some([0.1, 0.1, 0.8, 0.8]);
        ui.adjustments.flip_horizontal = true;
        ui.adjustments.flip_vertical = true;
        ui.reset_adjustments();
        assert_eq!(ui.adjustments.brightness, 1.0);
        assert_eq!(ui.adjustments.contrast, 1.0);
        assert_eq!(ui.adjustments.rotation, 0.0);
        assert_eq!(ui.adjustments.crop_rect_actual, None);
        assert!(!ui.adjustments.flip_horizontal);
        assert!(!ui.adjustments.flip_vertical);
    }

    #[test]
    fn test_ui_state_default_flips() {
        let ui = UiState::default();
        assert!(!ui.adjustments.flip_horizontal);
        assert!(!ui.adjustments.flip_vertical);
    }

    #[test]
    fn test_ui_state_crop_decoupling() {
        let mut ui = UiState::default();
        assert_eq!(ui.adjustments.crop_rect_actual, None);
        ui.adjustments.crop_rect_actual = Some([0.2, 0.2, 0.6, 0.6]);
        assert_eq!(ui.adjustments.crop_rect_actual, Some([0.2, 0.2, 0.6, 0.6]));
    }

    #[test]
    fn test_ui_state_current_file() {
        let mut ui = UiState::default();
        assert_eq!(ui.current_file(), None);

        ui.files = vec![make_entry("a.jpg"), make_entry("b.jpg")];
        ui.current_file_index = Some(0);
        assert_eq!(ui.current_file(), Some(&PathBuf::from("a.jpg")));

        ui.current_file_index = Some(1);
        assert_eq!(ui.current_file(), Some(&PathBuf::from("b.jpg")));
    }

    #[test]
    fn test_ui_state_set_status() {
        let mut ui = UiState::default();
        ui.set_status("Test message");
        assert!(ui.status_message.is_some());
        let (msg, _) = ui.status_message.as_ref().unwrap();
        assert_eq!(msg, "Test message");
    }

    #[test]
    fn test_ui_state_selected_indices() {
        let mut ui = UiState::default();
        ui.selected_indices.insert(0);
        ui.selected_indices.insert(2);
        assert!(ui.selected_indices.contains(&0));
        assert!(ui.selected_indices.contains(&2));
        assert!(!ui.selected_indices.contains(&1));
        assert_eq!(ui.selected_indices.len(), 2);
    }
}
