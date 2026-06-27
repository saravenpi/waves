mod config;
mod fileops;
mod library;
mod meta;
mod watcher;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(watcher::WatcherState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config,
            config::get_liked,
            config::toggle_liked,
            library::default_music_dir,
            library::list_dir,
            library::scan_library,
            meta::read_cover,
            meta::write_metadata,
            fileops::rename_path,
            fileops::new_folder,
            fileops::delete_path,
            fileops::paste_path,
            watcher::watch_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
