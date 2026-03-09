use crate::metadata::extract_metadata;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub relevance: i32,
}

fn calculate_relevance(title: &str, artist: &Option<String>, filename: &str, query_lower: &str) -> i32 {
    let title_lower = title.to_lowercase();
    let filename_lower = filename.to_lowercase();
    let artist_lower = artist.as_ref().map(|a| a.to_lowercase());


    let mut score = 0;


    if title_lower == query_lower {
        score += 1000;
    } else if title_lower.starts_with(query_lower) {
        score += 500;
    } else if title_lower.contains(query_lower) {
        score += 100;
    }


    if let Some(ref artist_l) = artist_lower {
        if artist_l == query_lower {
            score += 800;
        } else if artist_l.starts_with(query_lower) {
            score += 400;
        } else if artist_l.contains(query_lower) {
            score += 80;
        }
    }


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


    let mut all_files: Vec<PathBuf> = Vec::new();
    collect_audio_files(directory, &mut all_files, 0, 4);


    let mut filename_matches = Vec::new();
    let mut other_files = Vec::new();

    for path in all_files {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.to_lowercase().contains(&query_lower) {
                filename_matches.push(path);
            } else {
                other_files.push(path);
            }
        }
    }


    for path in filename_matches.iter().take(20) {
        if let Some(result) = extract_and_check(&path, &query_lower) {
            results.push(result);
        }
    }


    let needed = 30_usize.saturating_sub(results.len());
    for path in other_files.iter().take(needed) {
        if let Some(result) = extract_and_check(&path, &query_lower) {
            results.push(result);
        }
    }


    results.sort_by(|a, b| b.relevance.cmp(&a.relevance));

    results
}

fn extract_and_check(path: &PathBuf, query_lower: &str) -> Option<SearchResult> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let path_clone = path.clone();

    let metadata_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        extract_metadata(&path_clone)
    }));

    let (title, artist, album) = match metadata_result {
        Ok((title, artist, album, _, _, _)) => (title, artist, album),
        Err(_) => (name.clone(), None, None)
    };


    let title_lower = title.to_lowercase();
    let artist_lower = artist.as_ref().map(|a| a.to_lowercase());
    let album_lower = album.as_ref().map(|a| a.to_lowercase());
    let filename_lower = name.to_lowercase();

    let matches = filename_lower.contains(query_lower)
        || title_lower.contains(query_lower)
        || artist_lower.as_ref().map_or(false, |a| a.contains(query_lower))
        || album_lower.as_ref().map_or(false, |a| a.contains(query_lower));

    if matches {
        let relevance = calculate_relevance(&title, &artist, &name, query_lower);
        Some(SearchResult {
            path: path.clone(),
            title,
            artist,
            album,
            relevance,
        })
    } else {
        None
    }
}

fn collect_audio_files(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth || files.len() >= 200 {
        return;
    }

    let entries = match fs::read_dir(directory) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if files.len() >= 200 {
            return;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_audio_files(&path, files, depth + 1, max_depth);
        } else {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                files.push(path);
            }
        }
    }
}
