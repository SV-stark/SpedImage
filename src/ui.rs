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
    pub crop_rect: [f32; 4],
    pub show_file_dialog: bool,
    pub show_help: bool,
    pub status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            current_file_index: None,
            adjustments: ImageAdjustments::default(),
            is_cropping: false,
            crop_rect: [0.0, 0.0, 1.0, 1.0],
            show_file_dialog: false,
            show_help: false,
            status_message: None,
        }
    }
}

impl UiState {
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

    /// Reset all adjustments to default
    pub fn reset_adjustments(&mut self) {
        self.adjustments = ImageAdjustments::default();
        self.crop_rect = [0.0, 0.0, 1.0, 1.0];
    }

    /// Rotate by 90 degrees
    pub fn rotate_90(&mut self) {
        self.adjustments.rotation += std::f32::consts::FRAC_PI_2;
    }

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

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

// Re-export ImageBackend for file filtering
use crate::image_backend::ImageBackend;
