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
        }
    }
}

impl UiState {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn get_status(&self) -> &str {
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed().as_secs() < 3 {
                return msg;
            }
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
