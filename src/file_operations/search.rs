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
    pub relevance: i32,
}

fn calculate_relevance(title: &str, artist: &Option<String>, filename: &str, query_lower: &str) -> i32 {
    let title_lower = title.to_lowercase();
    let filename_lower = filename.to_lowercase();
    let artist_lower = artist.as_ref().map(|a| a.to_lowercase());

    // Higher score = better match
    let mut score = 0;

    // Exact title match: highest priority
    if title_lower == query_lower {
        score += 1000;
    } else if title_lower.starts_with(query_lower) {
        score += 500;
    } else if title_lower.contains(query_lower) {
        score += 100;
    }

    // Artist match
    if let Some(ref artist_l) = artist_lower {
        if artist_l == query_lower {
            score += 800;
        } else if artist_l.starts_with(query_lower) {
            score += 400;
        } else if artist_l.contains(query_lower) {
            score += 80;
        }
    }

    // Filename match (fallback)
    if filename_lower.starts_with(query_lower) {
        score += 300;
    } else if filename_lower.contains(query_lower) {
        score += 50;
    }

    score
}

pub fn search_audio_files(directory: &Path, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Two-pass strategy for speed:
    // Pass 1: Quick filename-only search (very fast)
    let mut filename_matches: Vec<PathBuf> = Vec::new();
    collect_filename_matches(directory, &query_lower, &mut filename_matches, 0, 4);

    // Pass 2: Extract metadata only for promising candidates
    for path in filename_matches.iter().take(30) {
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy().to_string();
            let path_clone = path.clone();
            let metadata_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                extract_metadata(&path_clone)
            }));

            let (title, artist, album) = match metadata_result {
                Ok((title, artist, album, _, _, _)) => (title, artist, album),
                Err(_) => (name.clone(), None, None)
            };

            let relevance = calculate_relevance(&title, &artist, &name, &query_lower);

            results.push(SearchResult {
                path: path.clone(),
                filename: name.clone(),
                title,
                artist,
                album,
                relevance,
            });
        }
    }

    // Sort by relevance (highest first)
    results.sort_by(|a, b| b.relevance.cmp(&a.relevance));

    results
}

fn collect_filename_matches(
    directory: &Path,
    query_lower: &str,
    matches: &mut Vec<PathBuf>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth || matches.len() >= 100 {
        return;
    }

    let entries = match fs::read_dir(directory) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if matches.len() >= 100 {
            return;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_filename_matches(&path, query_lower, matches, depth + 1, max_depth);
        } else {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                continue;
            }

            let filename_lower = name.to_lowercase();
            if filename_lower.contains(query_lower) {
                matches.push(path);
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
