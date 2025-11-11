use crate::app::WavesApp;
use crate::file_operations::search::search_audio_files;
use std::path::PathBuf;

impl WavesApp {
    pub fn perform_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            return;
        }

        if self.search_query.trim().len() < 2 {
            return;
        }

        let search_dir = if let Some(ref default_folder) = self.config.default_folder {
            let expanded = shellexpand::tilde(default_folder).to_string();
            let path = PathBuf::from(expanded);
            if path.exists() && path.is_dir() {
                path
            } else {
                eprintln!("Default folder does not exist: {:?}", path);
                return;
            }
        } else {
            if let Some(music_dir) = dirs::audio_dir() {
                if music_dir.exists() && music_dir.is_dir() {
                    music_dir
                } else if let Some(home) = dirs::home_dir() {
                    let music_fallback = home.join("Music");
                    if music_fallback.exists() && music_fallback.is_dir() {
                        music_fallback
                    } else {
                        eprintln!("No music directory found");
                        return;
                    }
                } else {
                    eprintln!("No music directory found");
                    return;
                }
            } else if let Some(home) = dirs::home_dir() {
                let music_dir = home.join("Music");
                if music_dir.exists() && music_dir.is_dir() {
                    music_dir
                } else {
                    eprintln!("No music directory found");
                    return;
                }
            } else {
                eprintln!("No music directory found");
                return;
            }
        };

        self.search_results = search_audio_files(&search_dir, &self.search_query);
        self.search_selected = 0;
    }
}
