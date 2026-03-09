mod movement;
mod actions;
mod arrows;
mod volume;
mod file_ops;
mod favorites;
mod views;
mod settings;
mod misc;

use crate::app::WavesApp;
use eframe::egui;

impl WavesApp {
    pub fn handle_navigation(&mut self, key: egui::Key, ctx: &egui::Context) {
        if self.columns.is_empty() {
            return;
        }

        if self.new_folder_prompt.is_some() || self.rename_prompt.is_some() || self.delete_confirm_prompt.is_some() || self.search_open {
            return;
        }

        match key {
            egui::Key::G => self.handle_g_key(ctx),
            egui::Key::J => self.handle_j_key(),
            egui::Key::K => self.handle_k_key(),
            egui::Key::L | egui::Key::Enter => self.handle_enter_key(ctx),
            egui::Key::H => self.handle_h_key(ctx),
            egui::Key::ArrowLeft => self.handle_arrow_left(ctx),
            egui::Key::ArrowRight => self.handle_arrow_right(ctx),
            egui::Key::Space => self.toggle_pause(),
            egui::Key::ArrowUp => self.handle_volume_up(),
            egui::Key::ArrowDown => self.handle_volume_down(),
            egui::Key::N => self.handle_new_folder(),
            egui::Key::R => self.handle_rename(),
            egui::Key::Y => self.handle_yank(),
            egui::Key::X => self.handle_cut(),
            egui::Key::P => self.handle_paste(),
            egui::Key::D => self.handle_delete(),
            egui::Key::F => self.handle_favorite(),
            egui::Key::M => self.handle_metadata_editor(),
            egui::Key::Tab => self.handle_tab(ctx),
            egui::Key::B => self.handle_browsing_mode_toggle(),
            egui::Key::Escape => self.handle_escape(),
            _ => {}
        }
    }
}
