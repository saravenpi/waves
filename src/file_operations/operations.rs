use crate::types::{ClipboardOperation, Column};
use std::path::PathBuf;

#[allow(dead_code)]
pub fn paste_clipboard(
    clipboard: &mut Option<(PathBuf, ClipboardOperation)>,
    current_dir: &PathBuf,
    columns: &[Column],
    update_columns_fn: &mut dyn FnMut(),
) {
    if let Some((source_path, operation)) = clipboard {
        let source_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let dest_dir = if !columns.is_empty() && !columns[0].entries.is_empty() {
            if let Some(selected) = columns[0].entries.get(columns[0].selected) {
                if selected.is_dir {
                    selected.path.clone()
                } else {
                    current_dir.clone()
                }
            } else {
                current_dir.clone()
            }
        } else {
            current_dir.clone()
        };

        let dest_path = dest_dir.join(source_name);

        if !source_path.exists() {
            eprintln!("Source path no longer exists: {:?}", source_path);
            *clipboard = None;
            return;
        }

        if dest_path.exists() {
            eprintln!("Destination already exists: {:?}", dest_path);
            return;
        }

        match operation {
            ClipboardOperation::Copy => {
                let result = if source_path.is_dir() {
                    std::process::Command::new("cp")
                        .arg("-r")
                        .arg(source_path)
                        .arg(&dest_path)
                        .status()
                } else {
                    std::process::Command::new("cp")
                        .arg(source_path)
                        .arg(&dest_path)
                        .status()
                };

                if let Err(e) = result {
                    eprintln!("Failed to copy: {}", e);
                } else {
                    update_columns_fn();
                }
            }
            ClipboardOperation::Cut => {
                let result = std::process::Command::new("mv")
                    .arg(source_path)
                    .arg(&dest_path)
                    .status();

                if let Err(e) = result {
                    eprintln!("Failed to move: {}", e);
                } else {
                    update_columns_fn();
                    *clipboard = None;
                }
            }
        }
    }
}
