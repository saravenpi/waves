use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
}

impl Default for SidebarPosition {
    fn default() -> Self {
        SidebarPosition::Left
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnimationType {
    Spectrum,
    CircleSpectrum,
    Agbe,
    Dots,
}

impl Default for AnimationType {
    fn default() -> Self {
        AnimationType::Spectrum
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_true")]
    pub animation: bool,
    #[serde(default)]
    pub animation_type: AnimationType,
    #[serde(default)]
    pub sidebar_position: SidebarPosition,
    #[serde(default = "default_true")]
    pub decorations: bool,
    #[serde(default)]
    pub window_corner_radius: f32,
    #[serde(default)]
    pub default_folder: Option<String>,
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    #[serde(default = "default_primary_color")]
    pub primary_color: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    #[serde(default = "default_true")]
    pub ui_sounds_enabled: bool,
    #[serde(default = "default_ui_sounds_volume")]
    pub ui_sounds_volume: f32,
    #[serde(default = "default_true")]
    pub startup_sound_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_primary_color() -> String {
    "#9664FF".to_string()
}

fn default_sidebar_width() -> f32 {
    500.0
}

fn default_ui_sounds_volume() -> f32 {
    0.04
}

impl Default for Config {
    fn default() -> Self {
        Config {
            animation: true,
            animation_type: AnimationType::default(),
            sidebar_position: SidebarPosition::Left,
            decorations: true,
            window_corner_radius: 0.0,
            default_folder: None,
            show_status_bar: true,
            primary_color: default_primary_color(),
            sidebar_width: default_sidebar_width(),
            ui_sounds_enabled: true,
            ui_sounds_volume: default_ui_sounds_volume(),
            startup_sound_enabled: true,
        }
    }
}

impl Config {
    pub fn file_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".waves.yml")
        } else {
            PathBuf::from(".waves.yml")
        }
    }

    pub fn load() -> Config {
        let path = Self::file_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            serde_yaml::from_str(&contents).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::file_path();
        let contents = serde_yaml::to_string(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }
}
