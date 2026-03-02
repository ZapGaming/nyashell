use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub show_git: bool,
    pub font_size: u16,
    pub accent_color: String,
    pub background: String,
    pub cursor_style: String,
    pub animation_speed: String,
}

impl Settings {
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_path() {
            if let Ok(contents) = fs::read_to_string(config_path) {
                if let Ok(settings) = toml::from_str(&contents) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = Self::config_dir() {
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("nyashell.toml");
            let contents = toml::to_string_pretty(self)?;
            fs::write(config_path, contents)?;
        }
        Ok(())
    }

    fn config_dir() -> Option<PathBuf> {
        config_dir().map(|p| p.join("NyaShell"))
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("nyashell.toml"))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "catboy".to_string(),
            show_git: true,
            font_size: 14,
            accent_color: "#ff6b9d".to_string(),
            background: "#0d0d0d".to_string(),
            cursor_style: "block".to_string(),
            animation_speed: "normal".to_string(),
        }
    }
}
