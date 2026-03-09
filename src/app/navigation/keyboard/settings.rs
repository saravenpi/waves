use crate::app::WavesApp;
use eframe::egui;

impl WavesApp {
    pub(super) fn handle_settings_decrease(&mut self, ctx: &egui::Context) -> bool {
        match self.settings_focused_item {
            0 => self.cycle_primary_color_backward(),
            1 => {
                if self.config.show_status_bar {
                    self.config.show_status_bar = false;
                    let _ = self.config.save();
                    true
                } else {
                    false
                }
            }
            2 => {
                if self.config.decorations {
                    self.config.decorations = false;
                    let _ = self.config.save();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(self.config.decorations));
                    true
                } else {
                    false
                }
            }
            3 => {
                if self.config.animation {
                    self.config.animation = false;
                    let _ = self.config.save();
                    true
                } else {
                    false
                }
            }
            4 => {
                if self.config.animation {
                    self.cycle_animation_type_backward()
                } else {
                    false
                }
            }
            5 => self.toggle_sidebar_position(),
            6 => {
                if self.config.ui_sounds_enabled {
                    self.config.ui_sounds_enabled = false;
                    let _ = self.config.save();
                    true
                } else {
                    false
                }
            }
            7 => {
                if self.config.ui_sounds_enabled {
                    self.decrease_ui_volume()
                } else {
                    false
                }
            }
            8 => {
                if self.config.startup_sound_enabled {
                    self.config.startup_sound_enabled = false;
                    let _ = self.config.save();
                    true
                } else {
                    false
                }
            }
            _ => false
        }
    }

    pub(super) fn cycle_primary_color_forward(&mut self) -> bool {
        let preset_colors = vec![
            "#9664FF", "#4A90E2", "#50E3C2",
            "#FF6B9D", "#FF8A00", "#FF4444"
        ];
        if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
            let next_idx = (current_idx + 1) % preset_colors.len();
            self.config.primary_color = preset_colors[next_idx].to_string();
            let _ = self.config.save();
            true
        } else {
            false
        }
    }

    pub(super) fn cycle_primary_color_backward(&mut self) -> bool {
        let preset_colors = vec![
            "#9664FF", "#4A90E2", "#50E3C2",
            "#FF6B9D", "#FF8A00", "#FF4444"
        ];
        if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
            let prev_idx = if current_idx == 0 {
                preset_colors.len() - 1
            } else {
                current_idx - 1
            };
            self.config.primary_color = preset_colors[prev_idx].to_string();
            let _ = self.config.save();
            true
        } else {
            false
        }
    }

    pub(super) fn cycle_animation_type_forward(&mut self) -> bool {
        use crate::config::AnimationType;
        self.config.animation_type = match self.config.animation_type {
            AnimationType::Spectrum => AnimationType::WaveformPulse,
            AnimationType::WaveformPulse => AnimationType::CircleSpectrum,
            AnimationType::CircleSpectrum => AnimationType::Agbe,
            AnimationType::Agbe => AnimationType::Dots,
            AnimationType::Dots => AnimationType::Spectrum,
        };
        let _ = self.config.save();
        true
    }

    pub(super) fn cycle_animation_type_backward(&mut self) -> bool {
        use crate::config::AnimationType;
        self.config.animation_type = match self.config.animation_type {
            AnimationType::Spectrum => AnimationType::Dots,
            AnimationType::Dots => AnimationType::Agbe,
            AnimationType::Agbe => AnimationType::CircleSpectrum,
            AnimationType::CircleSpectrum => AnimationType::WaveformPulse,
            AnimationType::WaveformPulse => AnimationType::Spectrum,
        };
        let _ = self.config.save();
        true
    }

    pub(super) fn toggle_sidebar_position(&mut self) -> bool {
        use crate::config::SidebarPosition;
        self.config.sidebar_position = match self.config.sidebar_position {
            SidebarPosition::Left => SidebarPosition::Right,
            SidebarPosition::Right => SidebarPosition::Left,
        };
        let _ = self.config.save();
        true
    }

    pub(super) fn increase_ui_volume(&mut self) -> bool {
        let old_volume = self.config.ui_sounds_volume;
        self.config.ui_sounds_volume = (self.config.ui_sounds_volume + 0.05).min(1.0);
        if (self.config.ui_sounds_volume - old_volume).abs() > 0.001 {
            let _ = self.config.save();
            true
        } else {
            false
        }
    }

    pub(super) fn decrease_ui_volume(&mut self) -> bool {
        let old_volume = self.config.ui_sounds_volume;
        self.config.ui_sounds_volume = (self.config.ui_sounds_volume - 0.05).max(0.0);
        if (old_volume - self.config.ui_sounds_volume).abs() > 0.001 {
            let _ = self.config.save();
            true
        } else {
            false
        }
    }
}
