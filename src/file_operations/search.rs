use crate::config::Config;
use crate::metadata::extract_metadata;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub filename: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub fn search_audio_files(directory: &Path, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    search_audio_files_recursive(directory, query, &query_lower, &mut results, 0, 10);

    results
}

pub fn search_audio_files_recursive(
    directory: &Path,
    query: &str,
    query_lower: &str,
    results: &mut Vec<SearchResult>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    if results.len() >= 500 {
        return;
    }

    let entries = match fs::read_dir(directory) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            search_audio_files_recursive(&path, query, query_lower, results, depth + 1, max_depth);
        } else {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                continue;
            }

            let filename_lower = name.to_lowercase();

            if filename_lower.contains(query_lower) {
                let path_clone = path.clone();
                let metadata_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    extract_metadata(&path_clone)
                }));

                let (title, artist, album) = match metadata_result {
                    Ok((title, artist, album, _, _, _)) => (title, artist, album),
                    Err(_) => {
                        eprintln!("Warning: Failed to extract metadata from {:?}", path);
                        (name.clone(), None, None)
                    }
                };

                results.push(SearchResult {
                    path: path.clone(),
                    filename: name.clone(),
                    title,
                    artist,
                    album,
                });
            }

            if results.len() >= 500 {
                return;
            }
        }
    }
}

#[allow(dead_code)]
pub fn perform_search(
    search_query: &str,
    config: &Config,
) -> Vec<SearchResult> {
    if search_query.trim().is_empty() {
        return Vec::new();
    }

    if search_query.trim().len() < 2 {
        return Vec::new();
    }

    let search_dir = if let Some(ref default_folder) = config.default_folder {
        let expanded = shellexpand::tilde(default_folder).to_string();
        let path = PathBuf::from(expanded);
        if path.exists() && path.is_dir() {
            path
        } else {
            eprintln!("Default folder does not exist: {:?}", path);
            return Vec::new();
        }
    } else {
        if let Some(music_dir) = dirs::audio_dir() {
            if music_dir.exists() && music_dir.is_dir() {
                music_dir
            } else if let Some(home) = dirs::home_dir() {
                let music_fallback = home.join("Music");
                if music_fallback.exists() && music_fallback.is_dir() {
                    music_fallback
                } else {
                    eprintln!("No music directory found");
                    return Vec::new();
                }
            } else {
                eprintln!("No music directory found");
                return Vec::new();
            }
        } else if let Some(home) = dirs::home_dir() {
            let music_dir = home.join("Music");
            if music_dir.exists() && music_dir.is_dir() {
                music_dir
            } else {
                eprintln!("No music directory found");
                return Vec::new();
            }
        } else {
            eprintln!("No music directory found");
            return Vec::new();
        }
    };

    search_audio_files(&search_dir, search_query)
}
