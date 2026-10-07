use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// How the folder listing is ordered.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    /// Natural file-name order (`img2` before `img10`), like Explorer.
    #[default]
    Name,
    /// File modification time.
    Modified,
    /// EXIF `DateTimeOriginal`, falling back to the modification time for
    /// files that carry none.
    Taken,
    /// File size in bytes.
    Size,
    /// File extension, then name.
    Type,
}

impl SortKey {
    pub const ALL: [SortKey; 5] = [
        SortKey::Name,
        SortKey::Modified,
        SortKey::Taken,
        SortKey::Size,
        SortKey::Type,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SortKey::Name => "Name",
            SortKey::Modified => "Date modified",
            SortKey::Taken => "Date taken",
            SortKey::Size => "Size",
            SortKey::Type => "Type",
        }
    }
}

// `default` at the container level: a field missing from an older or
// hand-edited file falls back to its default instead of failing the whole
// parse (which used to reset every preference at once).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_maximized: bool,
    /// Outer window position in physical pixels, restored on the next launch
    /// when it still lies on a connected monitor.
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
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
    pub sort_key: Option<SortKey>,
    pub sort_descending: Option<bool>,
    /// Convert from sRGB to the monitor's ICC profile when one is installed.
    /// Defaults to true.
    pub match_display_profile: Option<bool>,
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

    pub fn sort_key(&self) -> SortKey {
        self.sort_key.unwrap_or_default()
    }

    pub fn sort_descending(&self) -> bool {
        self.sort_descending.unwrap_or(false)
    }

    pub fn display_profile_enabled(&self) -> bool {
        self.match_display_profile.unwrap_or(true)
    }
}

/// Result of reading a settings file that may be damaged.
#[derive(Debug, Default)]
pub struct ParsedConfig {
    pub config: AppConfig,
    /// Keys whose values could not be read and were reset to defaults.
    pub dropped_keys: Vec<String>,
    /// The file was not valid JSON at all, so nothing could be salvaged.
    pub unreadable: bool,
}

impl ParsedConfig {
    pub fn is_clean(&self) -> bool {
        self.dropped_keys.is_empty() && !self.unreadable
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("spedimage").join("config.json"))
    }

    /// Where a damaged settings file is copied before it gets overwritten.
    pub fn backup_path(config_path: &Path) -> PathBuf {
        config_path.with_file_name("config.invalid.json")
    }

    pub fn load() -> Self {
        Self::load_reporting().0
    }

    /// Load the settings, salvaging what can be salvaged from a damaged file.
    ///
    /// Returns a user-facing warning when anything had to be reset. A damaged
    /// file is copied to [`Self::backup_path`] first, so nothing the user
    /// wrote is lost when the repaired settings are saved over it.
    pub fn load_reporting() -> (Self, Option<String>) {
        let Some(path) = Self::config_path() else {
            return (Self::default(), None);
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            // No file yet (first run) is not an error.
            return (Self::default(), None);
        };
        let parsed = Self::parse_lenient(&text);
        if parsed.is_clean() {
            return (parsed.config, None);
        }

        let backup = Self::backup_path(&path);
        let backed_up = std::fs::write(&backup, &text).is_ok();
        let what = if parsed.unreadable {
            "Settings file was not valid JSON, so all settings were reset.".to_string()
        } else {
            format!(
                "Some settings could not be read and were reset: {}.",
                parsed.dropped_keys.join(", ")
            )
        };
        let warning = if backed_up {
            format!("{what} The original file was saved to {}", backup.display())
        } else {
            what
        };
        tracing::warn!("{warning}");
        (parsed.config, Some(warning))
    }

    /// Parse `text`, keeping every field that is individually valid.
    ///
    /// One malformed value (say `"window_width": "wide"`) used to fail the
    /// whole parse and silently reset every preference. Each top-level key is
    /// now tried on its own and only the ones that do not deserialize are
    /// dropped.
    pub fn parse_lenient(text: &str) -> ParsedConfig {
        if let Ok(config) = serde_json::from_str::<AppConfig>(text) {
            return ParsedConfig {
                config,
                ..Default::default()
            };
        }

        let Ok(serde_json::Value::Object(user)) = serde_json::from_str::<serde_json::Value>(text)
        else {
            return ParsedConfig {
                unreadable: true,
                ..Default::default()
            };
        };

        let mut merged = match serde_json::to_value(AppConfig::default()) {
            Ok(serde_json::Value::Object(m)) => m,
            _ => serde_json::Map::new(),
        };
        let mut dropped_keys = Vec::new();
        for (key, value) in user {
            let mut candidate = merged.clone();
            candidate.insert(key.clone(), value);
            if serde_json::from_value::<AppConfig>(serde_json::Value::Object(candidate.clone()))
                .is_ok()
            {
                merged = candidate;
            } else {
                dropped_keys.push(key);
            }
        }
        let config = serde_json::from_value(serde_json::Value::Object(merged)).unwrap_or_default();
        ParsedConfig {
            config,
            dropped_keys,
            unreadable: false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_file_parses_cleanly() {
        let cfg = AppConfig {
            window_width: 1000,
            show_sidebar: true,
            sort_key: Some(SortKey::Taken),
            ..Default::default()
        };
        let parsed = AppConfig::parse_lenient(&serde_json::to_string(&cfg).unwrap());
        assert!(parsed.is_clean());
        assert_eq!(parsed.config, cfg);
    }

    #[test]
    fn one_bad_field_does_not_reset_the_others() {
        let text = r#"{
            "window_width": "wide",
            "window_height": 700,
            "show_sidebar": true,
            "sort_key": "not_a_key",
            "confirm_delete": true
        }"#;
        let parsed = AppConfig::parse_lenient(text);
        assert!(!parsed.unreadable);
        let mut dropped = parsed.dropped_keys.clone();
        dropped.sort();
        assert_eq!(dropped, ["sort_key", "window_width"]);
        assert_eq!(parsed.config.window_width, 0, "bad value reset to default");
        assert_eq!(parsed.config.window_height, 700);
        assert!(parsed.config.show_sidebar);
        assert_eq!(parsed.config.confirm_delete, Some(true));
    }

    #[test]
    fn missing_fields_take_defaults_without_a_warning() {
        let parsed = AppConfig::parse_lenient(r#"{"show_info": true}"#);
        assert!(parsed.is_clean());
        assert!(parsed.config.show_info);
        assert_eq!(parsed.config.window_width, 0);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let parsed = AppConfig::parse_lenient(r#"{"from_the_future": 1, "show_info": true}"#);
        assert!(parsed.is_clean());
        assert!(parsed.config.show_info);
    }

    #[test]
    fn invalid_json_is_reported_as_unreadable() {
        let parsed = AppConfig::parse_lenient("{ this is not json");
        assert!(parsed.unreadable);
        assert_eq!(parsed.config, AppConfig::default());
    }

    #[test]
    fn sort_keys_round_trip_as_snake_case() {
        let json = serde_json::to_string(&SortKey::Modified).unwrap();
        assert_eq!(json, "\"modified\"");
        for key in SortKey::ALL {
            let back: SortKey =
                serde_json::from_str(&serde_json::to_string(&key).unwrap()).unwrap();
            assert_eq!(back, key);
        }
    }
}
