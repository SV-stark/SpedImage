use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_maximized: bool,
    pub show_sidebar: bool,
    pub show_thumbnail_strip: bool,
    pub show_info: bool,
    pub show_histogram: bool,
    pub show_osd: Option<bool>,
    pub last_open_dir: Option<String>,
    pub max_preview_dimension: Option<u32>, // None = auto from window
    pub scroll_to_zoom: Option<bool>,
    pub double_click_zoom: Option<bool>,
    pub confirm_delete: Option<bool>,
    pub transparency_checkerboard: Option<bool>,
    /// Re-decode at full resolution when the user zooms past the preview's
    /// native resolution. Defaults to true.
    pub progressive_refinement: Option<bool>,
}

impl AppConfig {
    pub fn refinement_enabled(&self) -> bool {
        self.progressive_refinement.unwrap_or(true)
    }

    pub fn osd_enabled(&self) -> bool {
        self.show_osd.unwrap_or(true)
    }

    pub fn checkerboard_enabled(&self) -> bool {
        self.transparency_checkerboard.unwrap_or(true)
    }

    pub fn confirm_delete_enabled(&self) -> bool {
        self.confirm_delete.unwrap_or(false)
    }

    pub fn double_click_zoom_enabled(&self) -> bool {
        self.double_click_zoom.unwrap_or(false)
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("spedimage").join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        if let Ok(data) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&data).unwrap_or_default();
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let tmp_path = path.with_extension("tmp");
            if let Ok(json) = serde_json::to_string_pretty(self)
                && std::fs::write(&tmp_path, json).is_ok()
            {
                let _ = std::fs::rename(&tmp_path, &path);
            }
        }
    }
}
