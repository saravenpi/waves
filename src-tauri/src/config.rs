use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

fn d_true() -> bool {
    true
}
fn d_spectrum() -> String {
    "spectrum".into()
}
fn d_left() -> String {
    "left".into()
}
fn d_width() -> f32 {
    500.0
}
fn d_zero() -> f32 {
    0.0
}
fn d_color() -> String {
    "#9664FF".into()
}
fn d_sounds_vol() -> f32 {
    0.04
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "d_true")]
    pub animation: bool,
    #[serde(default = "d_spectrum")]
    pub animation_type: String,
    #[serde(default = "d_left")]
    pub sidebar_position: String,
    #[serde(default = "d_width")]
    pub sidebar_width: f32,
    #[serde(default = "d_true")]
    pub decorations: bool,
    #[serde(default = "d_zero")]
    pub window_corner_radius: f32,
    #[serde(default)]
    pub default_folder: Option<String>,
    #[serde(default = "d_true")]
    pub show_status_bar: bool,
    #[serde(default = "d_color")]
    pub primary_color: String,
    #[serde(default = "d_true")]
    pub ui_sounds_enabled: bool,
    #[serde(default = "d_sounds_vol")]
    pub ui_sounds_volume: f32,
    #[serde(default = "d_true")]
    pub startup_sound_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            animation: true,
            animation_type: d_spectrum(),
            sidebar_position: d_left(),
            sidebar_width: d_width(),
            decorations: true,
            window_corner_radius: 0.0,
            default_folder: None,
            show_status_bar: true,
            primary_color: d_color(),
            ui_sounds_enabled: true,
            ui_sounds_volume: 0.04,
            startup_sound_enabled: true,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".waves.yml")
}

fn waves_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".waves")
}

#[tauri::command]
pub fn get_config() -> Config {
    match std::fs::read_to_string(config_path()) {
        Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[tauri::command]
pub fn set_config(config: Config) -> Result<(), String> {
    let s = serde_yaml::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), s).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Liked {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub timestamp: SystemTime,
}

fn read_liked_file(name: &str) -> Vec<Liked> {
    match std::fs::read_to_string(waves_dir().join(name)) {
        Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub fn get_liked() -> Vec<Liked> {
    let mut liked = read_liked_file("liked.yml");
    if liked.is_empty() {
        liked = read_liked_file("favorites.yml");
    }
    liked.retain(|l| PathBuf::from(&l.path).exists());
    liked.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    liked
}

fn write_liked(liked: &[Liked]) -> Result<(), String> {
    let dir = waves_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let s = serde_yaml::to_string(liked).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("liked.yml"), s).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_liked(path: String, name: String, is_dir: bool) -> Result<Vec<Liked>, String> {
    let mut liked = get_liked();
    if let Some(pos) = liked.iter().position(|l| l.path == path) {
        liked.remove(pos);
    } else {
        liked.insert(
            0,
            Liked {
                path,
                name,
                is_dir,
                timestamp: SystemTime::now(),
            },
        );
    }
    write_liked(&liked)?;
    Ok(liked)
}
