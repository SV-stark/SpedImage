use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub show_sidebar: bool,
    pub show_thumbnail_strip: bool,
    pub show_info: bool,
    pub show_histogram: bool,
    pub last_open_dir: Option<String>,
    pub max_preview_dimension: Option<u32>, // None = auto from window
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("spedimage").join("config.json"))
    }

    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) => {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    return serde_json::from_str(&data).unwrap_or_default();
                }
            }
            None => {}
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, serde_json::to_string_pretty(self).unwrap());
        }
    }
}
