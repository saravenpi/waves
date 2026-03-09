use crate::types::Liked;
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
    waves_dir.join("liked.yml")
}

pub fn load() -> Vec<Liked> {
    let path = file_path();
    if let Ok(contents) = fs::read_to_string(&path) {
        let all_liked: Vec<Liked> = serde_yaml::from_str(&contents).unwrap_or_default();
        let original_count = all_liked.len();


        let valid_liked: Vec<Liked> = all_liked
            .into_iter()
            .filter(|item| item.path.exists())
            .collect();


        if valid_liked.len() != original_count {
            save(&valid_liked);
        }

        valid_liked
    } else {
        Vec::new()
    }
}

pub fn save(liked: &Vec<Liked>) {
    let path = file_path();
    if let Ok(yaml) = serde_yaml::to_string(liked) {
        if let Ok(mut file) = File::create(&path) {
            let _ = file.write_all(yaml.as_bytes());
        }
    }
}
