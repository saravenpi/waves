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
pub struct Liked {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub timestamp: SystemTime,
}

pub enum SidebarView {
    FileBrowser,
    Liked,
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
    pub fn next(&self) -> Self {
        match self {
            BrowsingMode::FileStructure => BrowsingMode::ByArtist,
            BrowsingMode::ByArtist => BrowsingMode::ByAlbum,
            BrowsingMode::ByAlbum => BrowsingMode::AllSongs,
            BrowsingMode::AllSongs => BrowsingMode::FileStructure,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum GroupedView {
    GroupList,
    TrackList(String),
}
