use eframe::egui;
use rodio::Sink;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct PlayerState {
    pub sink: Sink,
    pub current_file: PathBuf,
    pub waveform: Vec<f32>,
    pub audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Duration,
    pub start_time: Instant,
    pub pause_offset: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album_cover: Option<egui::TextureHandle>,
}

