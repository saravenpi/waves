use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

pub const AUDIO_EXTS: [&str; 6] = ["mp3", "flac", "wav", "ogg", "m4a", "aac"];

#[derive(Serialize)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}

pub fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

pub fn read_meta(path: &Path) -> Entry {
    let fallback = stem(path);
    let path_str = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&fallback)
        .to_string();

    let tagged = match Probe::open(path).and_then(|p| p.read()) {
        Ok(t) => t,
        Err(_) => {
            return Entry {
                name,
                path: path_str,
                is_dir: false,
                title: fallback,
                artist: String::new(),
                album: String::new(),
                duration: 0.0,
            }
        }
    };

    let duration = tagged.properties().duration().as_secs_f64();
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let (title, artist, album) = match tag {
        Some(t) => (
            t.title().map(|c| c.to_string()).filter(|s| !s.is_empty()).unwrap_or(fallback),
            t.artist().map(|c| c.to_string()).unwrap_or_default(),
            t.album().map(|c| c.to_string()).unwrap_or_default(),
        ),
        None => (fallback, String::new(), String::new()),
    };

    Entry {
        name,
        path: path_str,
        is_dir: false,
        title,
        artist,
        album,
        duration,
    }
}

#[tauri::command]
pub fn default_music_dir(default_folder: Option<String>) -> String {
    if let Some(f) = default_folder {
        let expanded = if let Some(rest) = f.strip_prefix("~/") {
            dirs::home_dir().unwrap_or_default().join(rest)
        } else {
            std::path::PathBuf::from(f)
        };
        if expanded.is_dir() {
            return expanded.to_string_lossy().to_string();
        }
    }
    if let Some(d) = dirs::audio_dir() {
        if d.is_dir() {
            return d.to_string_lossy().to_string();
        }
    }
    let music = dirs::home_dir().unwrap_or_default().join("Music");
    if music.is_dir() {
        return music.to_string_lossy().to_string();
    }
    dirs::home_dir().unwrap_or_default().to_string_lossy().to_string()
}

#[tauri::command]
pub fn list_dir(path: String) -> Vec<Entry> {
    let mut dirs_out: Vec<Entry> = Vec::new();
    let mut files_out: Vec<Entry> = Vec::new();

    if let Ok(rd) = std::fs::read_dir(&path) {
        for e in rd.flatten() {
            let p = e.path();
            let fname = e.file_name().to_string_lossy().to_string();
            if fname.starts_with('.') {
                continue;
            }
            if p.is_dir() {
                dirs_out.push(Entry {
                    name: fname.clone(),
                    path: p.to_string_lossy().to_string(),
                    is_dir: true,
                    title: fname,
                    artist: String::new(),
                    album: String::new(),
                    duration: 0.0,
                });
            } else if is_audio(&p) {
                files_out.push(read_meta(&p));
            }
        }
    }

    dirs_out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files_out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs_out.extend(files_out);
    dirs_out
}

#[tauri::command]
pub fn scan_library(root: String) -> Vec<Entry> {
    let mut tracks: Vec<Entry> = WalkDir::new(&root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_audio(e.path()))
        .map(|e| read_meta(e.path()))
        .collect();

    tracks.sort_by(|a, b| {
        a.artist
            .to_lowercase()
            .cmp(&b.artist.to_lowercase())
            .then(a.album.to_lowercase().cmp(&b.album.to_lowercase()))
            .then(a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
    tracks
}
