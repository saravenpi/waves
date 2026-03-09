mod spectrum;
mod waveform_pulse;
mod circle_spectrum;
mod agbe;
mod dots;

use crate::app::WavesApp;
use crate::config::AnimationType;
use eframe::egui;

impl WavesApp {
    pub fn render_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        if !self.config.animation {
            return;
        }

        match self.config.animation_type {
            AnimationType::Spectrum => self.render_spectrum_animation(ui, rect),
            AnimationType::WaveformPulse => self.render_waveform_pulse_animation(ui, rect),
            AnimationType::CircleSpectrum => self.render_circle_spectrum_animation(ui, rect),
            AnimationType::Agbe => self.render_agbe_animation(ui, rect),
            AnimationType::Dots => self.render_dots_animation(ui, rect),
        }
    }
}
