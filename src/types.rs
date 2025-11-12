use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct Column {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
}

pub enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub timestamp: SystemTime,
}

pub enum SidebarView {
    FileBrowser,
    Favorites,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrowsingMode {
    FileStructure,
    ByArtist,
    ByAlbum,
    AllSongs,
}

impl BrowsingMode {
    pub fn to_string(&self) -> &str {
        match self {
            BrowsingMode::FileStructure => "File Structure",
            BrowsingMode::ByArtist => "By Artist",
            BrowsingMode::ByAlbum => "By Album",
            BrowsingMode::AllSongs => "All Songs",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BrowsingMode::FileStructure => BrowsingMode::ByArtist,
            BrowsingMode::ByArtist => BrowsingMode::ByAlbum,
            BrowsingMode::ByAlbum => BrowsingMode::AllSongs,
            BrowsingMode::AllSongs => BrowsingMode::FileStructure,
        }
    }
}
