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
