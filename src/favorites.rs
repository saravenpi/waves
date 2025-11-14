use crate::types::Favorite;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Returns the path to the favorites file.
///
/// Creates the ~/.waves directory if it doesn't exist.
/// Defaults to .waves/favorites.yml in current directory as fallback.
pub fn file_path() -> PathBuf {
    let waves_dir = if let Some(home) = dirs::home_dir() {
        home.join(".waves")
    } else {
        PathBuf::from(".waves")
    };
    fs::create_dir_all(&waves_dir).ok();
    waves_dir.join("favorites.yml")
}

/// Loads favorites from the YAML file.
///
/// Returns empty vector if the file doesn't exist or cannot be parsed.
/// Filters out favorites that point to non-existent files/directories.
pub fn load() -> Vec<Favorite> {
    let path = file_path();
    if let Ok(contents) = fs::read_to_string(&path) {
        let all_favorites: Vec<Favorite> = serde_yaml::from_str(&contents).unwrap_or_default();
        let original_count = all_favorites.len();

        // Filter out favorites where the path no longer exists
        let valid_favorites: Vec<Favorite> = all_favorites
            .into_iter()
            .filter(|fav| fav.path.exists())
            .collect();

        // If we filtered any out, save the cleaned list
        if valid_favorites.len() != original_count {
            save(&valid_favorites);
        }

        valid_favorites
    } else {
        Vec::new()
    }
}

/// Saves favorites to the YAML file.
///
/// # Arguments
/// * `favorites` - Vector of favorite items to persist
pub fn save(favorites: &Vec<Favorite>) {
    let path = file_path();
    if let Ok(yaml) = serde_yaml::to_string(favorites) {
        if let Ok(mut file) = File::create(&path) {
            let _ = file.write_all(yaml.as_bytes());
        }
    }
}
