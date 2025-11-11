use crate::types::Favorite;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn file_path() -> PathBuf {
    let waves_dir = if let Some(home) = dirs::home_dir() {
        home.join(".waves")
    } else {
        PathBuf::from(".waves")
    };
    fs::create_dir_all(&waves_dir).ok();
    waves_dir.join("favorites.yml")
}

pub fn load() -> Vec<Favorite> {
    let path = file_path();
    if let Ok(contents) = fs::read_to_string(&path) {
        serde_yaml::from_str(&contents).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save(favorites: &Vec<Favorite>) {
    let path = file_path();
    if let Ok(yaml) = serde_yaml::to_string(favorites) {
        if let Ok(mut file) = File::create(&path) {
            let _ = file.write_all(yaml.as_bytes());
        }
    }
}
