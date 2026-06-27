use std::path::{Path, PathBuf};

#[tauri::command]
pub fn rename_path(path: String, new_name: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let parent = p.parent().ok_or("no parent")?;
    let dest = parent.join(new_name);
    std::fs::rename(&p, &dest).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn new_folder(parent: String, name: String) -> Result<(), String> {
    let dest = PathBuf::from(parent).join(name);
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    trash::delete(&path).map_err(|e| e.to_string())
}

fn copy_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

#[tauri::command]
pub fn paste_path(src: String, dest_dir: String, cut: bool) -> Result<(), String> {
    let src_p = PathBuf::from(&src);
    let name = src_p.file_name().ok_or("bad source")?;
    let dest = PathBuf::from(&dest_dir).join(name);
    if dest == src_p {
        return Ok(());
    }
    if cut {
        if std::fs::rename(&src_p, &dest).is_ok() {
            return Ok(());
        }
    }
    copy_recursive(&src_p, &dest).map_err(|e| e.to_string())?;
    if cut {
        trash::delete(&src_p).map_err(|e| e.to_string())?;
    }
    Ok(())
}
