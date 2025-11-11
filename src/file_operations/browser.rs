use crate::types::{FileEntry, Column};
use std::fs;
use std::path::{Path, PathBuf};

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
