use crate::types::{FileEntry, Column};
use crate::metadata::extract_metadata;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub fn read_directory(path: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    let dir_entries = match fs::read_dir(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read directory {:?}: {}", path, e);
            return entries;
        }
    };

    let items_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut items: Vec<_> = dir_entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    return None;
                }
                let entry_path = e.path();

                let is_dir = match entry_path.metadata() {
                    Ok(m) => m.is_dir(),
                    Err(_) => return None,
                };

                if !is_dir {
                    let ext = entry_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if !matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                        return None;
                    }
                }

                Some(FileEntry { path: entry_path, name, is_dir })
            })
            .collect();

        items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        items
    }));

    match items_result {
        Ok(items) => entries.extend(items),
        Err(_) => eprintln!("Panic while reading directory {:?}", path),
    }

    entries
}

#[allow(dead_code)]
pub fn update_columns(
    current_dir: &PathBuf,
    columns: &mut Vec<Column>,
) {
    update_columns_with_selection(current_dir, columns, None);
}

#[allow(dead_code)]
pub fn update_columns_with_selection(
    current_dir: &PathBuf,
    columns: &mut Vec<Column>,
    selection: Option<usize>,
) {
    let current_selection = if let Some(sel) = selection {
        sel
    } else if !columns.is_empty() {
        columns[0].selected
    } else {
        0
    };

    columns.clear();

    let current_entries = read_directory(current_dir);

    let selected = if current_entries.is_empty() {
        0
    } else {
        current_selection.min(current_entries.len().saturating_sub(1))
    };

    let current_column = Column {
        entries: current_entries,
        selected,
    };
    columns.push(current_column);
}

pub fn collect_all_audio_files(root_path: &Path) -> Vec<PathBuf> {
    let mut audio_files = Vec::new();
    collect_audio_files_recursive(root_path, &mut audio_files);
    audio_files.sort();
    audio_files
}

fn collect_audio_files_recursive(path: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();

            if let Some(name) = entry_path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            if entry_path.is_dir() {
                collect_audio_files_recursive(&entry_path, files);
            } else if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if matches!(ext_str, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                            files.push(entry_path);
                        }
                    }
                }
            }
        }
    }
}

pub fn group_by_artist(audio_files: &[PathBuf]) -> Vec<(String, Vec<PathBuf>)> {
    let mut artist_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for file in audio_files {
        let (_title, artist, _album, _date, _track, _duration) = extract_metadata(file);
        let artist_name = artist
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Unknown Artist".to_string());
        artist_map.entry(artist_name).or_insert_with(Vec::new).push(file.clone());
    }

    let mut artists: Vec<_> = artist_map.into_iter().collect();
    artists.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    artists
}

pub fn group_by_album(audio_files: &[PathBuf]) -> Vec<(String, Vec<PathBuf>)> {
    let mut album_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for file in audio_files {
        let (_title, _artist, album, _date, _track, _duration) = extract_metadata(file);
        let album_name = album
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Unknown Album".to_string());
        album_map.entry(album_name).or_insert_with(Vec::new).push(file.clone());
    }

    let mut albums: Vec<_> = album_map.into_iter().collect();
    albums.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    albums
}
