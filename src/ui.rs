//! UI Layer - User interface components
//!
//! Provides UI elements for the image viewer including file browser,
//! adjustment controls, and toolbar.

use crate::gpu_renderer::ImageAdjustments;
use std::path::PathBuf;

/// File entry for the sidebar
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_image: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let is_image = ImageBackend::is_supported(&path);

        Self {
            path,
            name,
            is_image,
        }
    }
}

/// Application state for UI
#[derive(Debug, Clone)]
pub struct UiState {
    pub files: Vec<FileEntry>,
    pub current_file_index: Option<usize>,
    pub adjustments: ImageAdjustments,
    pub is_cropping: bool,
    pub show_file_dialog: bool,
    pub show_help: bool,
    pub show_info: bool,
    pub show_sidebar: bool,
    pub show_thumbnail_strip: bool,
    pub show_histogram: bool,
    pub status_message: Option<String>,
    /// Set of file indices that are currently selected in the thumbnail strip.
    pub selected_indices: std::collections::HashSet<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            current_file_index: None,
            adjustments: ImageAdjustments::default(),
            is_cropping: false,
            show_file_dialog: false,
            show_help: false,
            show_info: false,
            show_sidebar: false,
            show_thumbnail_strip: true,
            show_histogram: false,
            status_message: None,
            selected_indices: std::collections::HashSet::new(),
        }
    }
}

impl UiState {
    // --- File Navigation --- //

    /// Get the current file path if any
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file_index
            .and_then(|idx| self.files.get(idx))
            .map(|f| &f.path)
    }

    /// Navigate to next image
    pub fn next_file(&mut self) {
        if let Some(current) = self.current_file_index {
            let image_count = self.files.iter().filter(|f| f.is_image).count();
            if image_count > 0 {
                // Find next image file
                let mut search_idx = (current + 1) % self.files.len();
                for _ in 0..self.files.len() {
                    if self.files[search_idx].is_image {
                        self.current_file_index = Some(search_idx);
                        return;
                    }
                    search_idx = (search_idx + 1) % self.files.len();
                }
            }
        } else if !self.files.is_empty() {
            // Select first image
            for (i, f) in self.files.iter().enumerate() {
                if f.is_image {
                    self.current_file_index = Some(i);
                    return;
                }
            }
        }
    }

    /// Navigate to previous image
    pub fn prev_file(&mut self) {
        if let Some(current) = self.current_file_index {
            if !self.files.is_empty() {
                // Find previous image file
                let mut search_idx = if current == 0 {
                    self.files.len() - 1
                } else {
                    current - 1
                };
                for _ in 0..self.files.len() {
                    if self.files[search_idx].is_image {
                        self.current_file_index = Some(search_idx);
                        return;
                    }
                    search_idx = if search_idx == 0 {
                        self.files.len() - 1
                    } else {
                        search_idx - 1
                    };
                }
            }
        }
    }

    /// Load files from a directory
    pub fn load_directory(&mut self, dir: PathBuf) {
        self.files.clear();
        self.current_file_index = None;

        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| FileEntry::new(e.path()))
                .collect();

            // Sort files by name
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Filter to only images and sort
            files.retain(|f| f.is_image);
            self.files = files;

            // Select first image
            if !self.files.is_empty() {
                self.current_file_index = Some(0);
            }
        }
    }

    // --- Image Adjustments --- //

    /// Reset all adjustments to default
    pub fn reset_adjustments(&mut self) {
        self.adjustments = ImageAdjustments::default();
    }

    /// Rotate by 90 degrees
    pub fn rotate_90(&mut self) {
        self.adjustments.rotation += std::f32::consts::FRAC_PI_2;
    }

    // --- Status Bar --- //

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Get current status message as str (empty if none)
    pub fn get_status(&self) -> &str {
        self.status_message.as_deref().unwrap_or("")
    }

    // --- UI Toggles --- //

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle info panel
    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }
}

// Re-export ImageBackend for file filtering
use crate::image_backend::ImageBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_file_entry_new() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("test_image.png");

        std::fs::File::create(&test_path).unwrap();

        let entry = FileEntry::new(test_path.clone());
        assert_eq!(entry.name, "test_image.png");
        assert!(entry.is_image);

        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_file_entry_unknown_extension() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("test.xyz");

        if std::fs::File::create(&test_path).is_err() {
            return;
        }

        let entry = FileEntry::new(test_path.clone());
        assert_eq!(entry.name, "test.xyz");
        assert!(!entry.is_image);

        let _ = std::fs::remove_file(&test_path);
    }

    #[test]
    fn test_ui_state_current_file() {
        let mut state = UiState::default();
        assert!(state.current_file().is_none());

        let temp_dir = env::temp_dir();
        let path = temp_dir.join("test_ui_state_current_file.png");
        std::fs::File::create(&path).unwrap();

        state.files.push(FileEntry::new(path.clone()));
        state.current_file_index = Some(0);

        assert_eq!(state.current_file(), Some(&path));

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_ui_state_next_file() {
        let mut state = UiState::default();

        let temp_dir = env::temp_dir();
        let path1 = temp_dir.join("a.png");
        let path2 = temp_dir.join("b.png");
        let path3 = temp_dir.join("c.png");

        std::fs::File::create(&path1).unwrap();
        std::fs::File::create(&path2).unwrap();
        std::fs::File::create(&path3).unwrap();

        state.files.push(FileEntry::new(path1));
        state.files.push(FileEntry::new(path2.clone()));
        state.files.push(FileEntry::new(path3));
        state.current_file_index = Some(0);

        state.next_file();
        assert_eq!(state.current_file_index, Some(1));

        state.next_file();
        assert_eq!(state.current_file_index, Some(2));

        state.next_file();
        assert_eq!(state.current_file_index, Some(0));

        std::fs::remove_file(&path2).unwrap();
    }

    #[test]
    fn test_ui_state_prev_file() {
        let mut state = UiState::default();

        let temp_dir = env::temp_dir();
        let path1 = temp_dir.join("a.png");
        let path2 = temp_dir.join("b.png");

        std::fs::File::create(&path1).unwrap();
        std::fs::File::create(path2.clone()).unwrap();

        state.files.push(FileEntry::new(path1.clone()));
        state.files.push(FileEntry::new(path2));
        state.current_file_index = Some(1);

        state.prev_file();
        assert_eq!(state.current_file_index, Some(0));

        state.prev_file();
        assert_eq!(state.current_file_index, Some(1));

        std::fs::remove_file(&path1).unwrap();
    }

    #[test]
    fn test_ui_state_load_directory() {
        let mut state = UiState::default();

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("spedimage_test_dir");
        std::fs::create_dir(&test_dir).unwrap();

        let path1 = test_dir.join("aaa.png");
        let path2 = test_dir.join("bbb.jpg");

        std::fs::File::create(&path1).unwrap();
        std::fs::File::create(&path2).unwrap();

        state.load_directory(test_dir.clone());

        assert!(!state.files.is_empty());
        assert!(state.current_file_index.is_some());

        std::fs::remove_file(&path1).unwrap();
        std::fs::remove_file(&path2).unwrap();
        std::fs::remove_dir(&test_dir).unwrap();
    }

    #[test]
    fn test_ui_state_reset_adjustments() {
        let mut state = UiState::default();

        state.adjustments.brightness = 2.0;
        state.adjustments.contrast = 1.5;
        state.adjustments.rotation = std::f32::consts::FRAC_PI_4;
        state.adjustments.hdr_toning = true;

        state.reset_adjustments();

        assert_eq!(state.adjustments.brightness, 1.0);
        assert_eq!(state.adjustments.contrast, 1.0);
        assert_eq!(state.adjustments.rotation, 0.0);
        assert!(!state.adjustments.hdr_toning);
    }

    #[test]
    fn test_ui_state_rotate_90() {
        let mut state = UiState::default();

        let initial_rotation = state.adjustments.rotation;
        state.rotate_90();

        assert_eq!(
            state.adjustments.rotation,
            initial_rotation + std::f32::consts::FRAC_PI_2
        );
    }

    #[test]
    fn test_ui_state_set_status() {
        let mut state = UiState::default();

        state.set_status("Test message");
        assert_eq!(state.get_status(), "Test message");

        state.set_status("Another message");
        assert_eq!(state.get_status(), "Another message");
    }

    #[test]
    fn test_ui_state_clear_status() {
        let mut state = UiState::default();

        state.set_status("Test message");
        state.clear_status();

        assert_eq!(state.get_status(), "");
    }

    #[test]
    fn test_ui_state_toggle_help() {
        let mut state = UiState::default();

        assert!(!state.show_help);

        state.toggle_help();
        assert!(state.show_help);

        state.toggle_help();
        assert!(!state.show_help);
    }

    #[test]
    fn test_ui_state_default_flags() {
        let state = UiState::default();
        assert!(state.show_thumbnail_strip);
        assert!(!state.show_sidebar);
        assert!(!state.show_histogram);
        assert!(!state.show_help);
        assert!(!state.show_info);
    }

    #[test]
    fn test_current_file_none() {
        let state = UiState::default();
        assert!(state.current_file().is_none());
    }

    #[test]
    fn test_navigation_wrapping() {
        let mut state = UiState::default();
        // Setup 3 dummy files
        state.files = vec![
            FileEntry { path: "1".into(), name: "1".into(), is_image: true },
            FileEntry { path: "2".into(), name: "2".into(), is_image: true },
            FileEntry { path: "3".into(), name: "3".into(), is_image: true },
        ];
        state.current_file_index = Some(0);

        // prev should wrap to last
        state.prev();
        assert_eq!(state.current_file_index, Some(2));
        
        // next should wrap to first
        state.next();
        assert_eq!(state.current_file_index, Some(0));
    }
}
