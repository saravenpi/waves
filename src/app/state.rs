use crate::config::Config;
use crate::types::{Column, ClipboardOperation, Liked, SidebarView, BrowsingMode, GroupedView};
use crate::audio::PlayerState;
use crate::file_operations::SearchResult;
use crate::ui::input::MetadataEditor;

use rodio::OutputStream;
use rustfft::FftPlanner;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use eframe::egui;

#[derive(Clone)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub frequency_band: usize,
}

pub struct WavesApp {
    pub current_dir: PathBuf,
    pub root_dir: PathBuf,
    pub columns: Vec<Column>,
    pub player: Arc<Mutex<Option<PlayerState>>>,
    pub audio_stream: Option<OutputStream>,
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
    pub delete_confirm_selected: usize,
    pub liked: Vec<Liked>,
    pub sidebar_view: SidebarView,
    pub liked_selected: usize,
    pub config: Config,
    pub file_to_play_on_start: Option<PathBuf>,
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
    pub last_folder_check: std::time::Instant,
    pub last_folder_file_count: usize,
    pub settings_focused_item: usize,
    pub playback_context: SidebarView,
    pub animation_fullscreen: bool,
    pub last_mouse_movement: std::time::Instant,
    pub last_animation_hover: std::time::Instant,
    pub browsing_mode: BrowsingMode,
    pub audio_files_cache: Option<Vec<PathBuf>>,
    pub artist_groups_cache: Option<Vec<(String, Vec<PathBuf>)>>,
    pub album_groups_cache: Option<Vec<(String, Vec<PathBuf>)>>,
    pub cache_root_dir: Option<PathBuf>,
    pub cache_loading: bool,
    pub cache_receiver: Option<Receiver<CacheResult>>,
    pub startup_animation: bool,
    pub startup_time: std::time::Instant,
    pub grouped_view: GroupedView,
    pub current_group_tracks: Vec<PathBuf>,
    pub help_modal_open: bool,
    pub last_g_press: Option<std::time::Instant>,
    pub song_loading: bool,
    pub song_loading_started: Option<std::time::Instant>,
    pub song_data_receiver: Receiver<SongLoadData>,
    pub song_data_sender: Sender<SongLoadData>,
    pub scroll_to_selection: bool,
    pub dots: Vec<Dot>,
    pub dots_initialized: bool,
    pub hann_window: Vec<f32>,
    pub freq_bands: Vec<(f32, f32)>,
}

pub enum CacheResult {
    AudioFiles(Vec<PathBuf>),
    ArtistGroups(Vec<(String, Vec<PathBuf>)>),
    AlbumGroups(Vec<(String, Vec<PathBuf>)>),
}

pub struct SongLoadData {
    pub path: PathBuf,
    pub file_bytes: Vec<u8>,
    pub title: String,
    pub artist: Option<String>,
    pub duration: std::time::Duration,
    pub waveform: Vec<f32>,
}

impl WavesApp {
    pub fn new_with_receiver(
        file_open_receiver: Receiver<PathBuf>,
        #[cfg(target_os = "macos")]
        menu_action_receiver: Receiver<crate::macos::MenuAction>,
    ) -> Self {
        let args: Vec<String> = std::env::args().collect();
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
            if path.exists() {
                if path.is_dir() {
                    (path, None)
                } else if path.is_file() {
                    let parent = path.parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| get_default_dir());
                    (parent, Some(path))
                } else {
                    (get_default_dir(), None)
                }
            } else {
                (get_default_dir(), None)
            }
        } else {
            (get_default_dir(), None)
        };

        let (waveform_sender, waveform_receiver) = channel();
        let (album_cover_sender, album_cover_receiver) = channel();
        let (song_data_tx, song_data_rx) = channel();

        if config.startup_sound_enabled {
            crate::sound::play_startup_sound();
        }

        let audio_stream = match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Some(stream),
            Err(e) => {
                eprintln!("Failed to open audio stream: {}", e);
                None
            }
        };

        let mut app = Self {
            current_dir: start_dir.clone(),
            root_dir: start_dir.clone(),
            columns: vec![],
            player: Arc::new(Mutex::new(None)),
            audio_stream,
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
            delete_confirm_selected: 1,
            liked: crate::liked::load(),
            sidebar_view: SidebarView::FileBrowser,
            liked_selected: 0,
            config,
            file_to_play_on_start: file_to_play,
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
            last_folder_check: std::time::Instant::now(),
            last_folder_file_count: 0,
            settings_focused_item: 0,
            playback_context: SidebarView::FileBrowser,
            animation_fullscreen: false,
            last_mouse_movement: std::time::Instant::now(),
            last_animation_hover: std::time::Instant::now(),
            browsing_mode: BrowsingMode::FileStructure,
            audio_files_cache: None,
            artist_groups_cache: None,
            album_groups_cache: None,
            cache_root_dir: None,
            cache_loading: false,
            cache_receiver: None,
            startup_animation: true,
            startup_time: std::time::Instant::now(),
            grouped_view: GroupedView::GroupList,
            current_group_tracks: Vec::new(),
            help_modal_open: false,
            last_g_press: None,
            song_loading: false,
            song_loading_started: None,
            song_data_receiver: song_data_rx,
            song_data_sender: song_data_tx,
            scroll_to_selection: false,
            dots: Vec::new(),
            dots_initialized: false,
            hann_window: (0..4096).map(|i| {
                0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / 4096.0).cos()
            }).collect(),
            freq_bands: (0..64).map(|i| {
                let freq_min = 20.0 * (20000.0_f32 / 20.0).powf(i as f32 / 64.0);
                let freq_max = 20.0 * (20000.0_f32 / 20.0).powf((i + 1) as f32 / 64.0);
                (freq_min, freq_max)
            }).collect(),
        };

        app.update_columns();
        app
    }

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

    pub fn primary_color_with_alpha(&self, alpha: u8) -> egui::Color32 {
        let base_color = self.primary_color();
        egui::Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), alpha)
    }
}
