use crate::config::Config;
use crate::types::{Column, ClipboardOperation, Favorite, SidebarView, BrowsingMode};
use crate::audio::PlayerState;
use crate::file_operations::SearchResult;
use crate::ui::input::MetadataEditor;

use rustfft::FftPlanner;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use eframe::egui;

pub struct WavesApp {
    pub current_dir: PathBuf,
    pub root_dir: PathBuf,
    pub columns: Vec<Column>,
    pub player: Arc<Mutex<Option<PlayerState>>>,
    pub pending_seek: Option<f32>,
    pub waveform_cache: HashMap<PathBuf, Vec<f32>>,
    pub waveform_receiver: Receiver<(PathBuf, Vec<f32>)>,
    pub waveform_sender: Sender<(PathBuf, Vec<f32>)>,
    pub album_cover_cache: HashMap<PathBuf, egui::TextureHandle>,
    pub album_cover_receiver: Receiver<(PathBuf, egui::ColorImage)>,
    pub album_cover_sender: Sender<(PathBuf, egui::ColorImage)>,
    pub last_selected_file: Option<PathBuf>,
    pub spectrum_bars: Vec<f32>,
    pub fft_planner: FftPlanner<f32>,
    pub volume: f32,
    pub new_folder_prompt: Option<String>,
    pub rename_prompt: Option<(PathBuf, String)>,
    pub clipboard: Option<(PathBuf, ClipboardOperation)>,
    pub delete_confirm_prompt: Option<PathBuf>,
    pub favorites: Vec<Favorite>,
    pub sidebar_view: SidebarView,
    pub favorites_selected: usize,
    pub config: Config,
    pub file_to_play_on_start: Option<PathBuf>,
    pub default_folder_input: String,
    pub search_open: bool,
    pub search_just_opened: bool,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub context_menu: Option<(PathBuf, egui::Pos2)>,
    pub loop_enabled: bool,
    pub metadata_editor: Option<MetadataEditor>,
    pub file_open_receiver: Receiver<PathBuf>,
    #[cfg(target_os = "macos")]
    pub menu_action_receiver: Receiver<crate::macos::MenuAction>,
    pub sidebar_collapsed: bool,
    pub last_folder_check: std::time::Instant,
    pub last_folder_file_count: usize,
    pub settings_focused_item: usize,
    pub playback_context: SidebarView,
    pub animation_fullscreen: bool,
    pub last_mouse_movement: std::time::Instant,
    pub last_animation_hover: std::time::Instant,
    pub browsing_mode: BrowsingMode,
}

impl WavesApp {
    /// Creates a new instance of the WAVES application with a file open receiver.
    ///
    /// Initializes the file browser, audio player, and all application state.
    /// Processes command-line arguments to determine starting directory and file to play.
    pub fn new_with_receiver(
        file_open_receiver: Receiver<PathBuf>,
        #[cfg(target_os = "macos")]
        menu_action_receiver: Receiver<crate::macos::MenuAction>,
    ) -> Self {
        let args: Vec<String> = std::env::args().collect();
        eprintln!("WAVES DEBUG: Received {} arguments:", args.len());
        for (i, arg) in args.iter().enumerate() {
            eprintln!("  arg[{}]: {}", i, arg);
        }
        let config = Config::load();

        let get_default_dir = || {
            if let Some(ref default_folder) = config.default_folder {
                let expanded = shellexpand::tilde(default_folder).to_string();
                let default_path = PathBuf::from(expanded);
                if default_path.exists() && default_path.is_dir() {
                    return default_path;
                }
            }

            if let Some(music_dir) = dirs::audio_dir() {
                if music_dir.exists() {
                    return music_dir;
                }
            }

            if let Some(home) = dirs::home_dir() {
                let music_dir = home.join("Music");
                if music_dir.exists() {
                    return music_dir;
                }
            }

            PathBuf::from(".")
        };

        let (start_dir, file_to_play) = if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            eprintln!("WAVES DEBUG: Processing path: {:?}", path);
            eprintln!("WAVES DEBUG: Path exists: {}", path.exists());
            eprintln!("WAVES DEBUG: Path is_dir: {}", path.is_dir());
            eprintln!("WAVES DEBUG: Path is_file: {}", path.is_file());
            if path.exists() {
                if path.is_dir() {
                    eprintln!("WAVES DEBUG: Opening directory: {:?}", path);
                    (path, None)
                } else if path.is_file() {
                    let parent = path.parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| get_default_dir());
                    eprintln!("WAVES DEBUG: Opening file: {:?}, parent: {:?}", path, parent);
                    (parent, Some(path))
                } else {
                    eprintln!("WAVES DEBUG: Path exists but is neither file nor directory");
                    (get_default_dir(), None)
                }
            } else {
                eprintln!("WAVES DEBUG: Path does not exist, using default");
                (get_default_dir(), None)
            }
        } else {
            eprintln!("WAVES DEBUG: No arguments, using default directory");
            (get_default_dir(), None)
        };

        let (waveform_sender, waveform_receiver) = channel();
        let (album_cover_sender, album_cover_receiver) = channel();

        let default_folder_input = config.default_folder.clone().unwrap_or_else(|| String::from("~/Music"));

        let mut app = Self {
            current_dir: start_dir.clone(),
            root_dir: start_dir.clone(),
            columns: vec![],
            player: Arc::new(Mutex::new(None)),
            pending_seek: None,
            waveform_cache: HashMap::new(),
            waveform_receiver,
            waveform_sender,
            album_cover_cache: HashMap::new(),
            album_cover_receiver,
            album_cover_sender,
            last_selected_file: None,
            spectrum_bars: vec![0.0; 64],
            fft_planner: FftPlanner::new(),
            volume: 1.0,
            new_folder_prompt: None,
            rename_prompt: None,
            clipboard: None,
            delete_confirm_prompt: None,
            favorites: crate::favorites::load(),
            sidebar_view: SidebarView::FileBrowser,
            favorites_selected: 0,
            config,
            file_to_play_on_start: file_to_play,
            default_folder_input,
            search_open: false,
            search_just_opened: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            context_menu: None,
            loop_enabled: false,
            metadata_editor: None,
            file_open_receiver,
            #[cfg(target_os = "macos")]
            menu_action_receiver,
            sidebar_collapsed: false,
            last_folder_check: std::time::Instant::now(),
            last_folder_file_count: 0,
            settings_focused_item: 0,
            playback_context: SidebarView::FileBrowser,
            animation_fullscreen: false,
            last_mouse_movement: std::time::Instant::now(),
            last_animation_hover: std::time::Instant::now(),
            browsing_mode: BrowsingMode::FileStructure,
        };

        app.update_columns();
        app
    }

    /// Retrieves the configured primary color for UI accent elements.
    ///
    /// Parses the hex color from configuration and returns an egui Color32.
    /// Falls back to default purple color if parsing fails.
    pub fn primary_color(&self) -> egui::Color32 {
        let hex = self.config.primary_color.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return egui::Color32::from_rgb(r, g, b);
            }
        }
        egui::Color32::from_rgb(150, 100, 255)
    }
}
