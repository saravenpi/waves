use crate::app::WavesApp;

impl WavesApp {
    pub(super) fn handle_volume_up(&mut self) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        self.volume = (self.volume + 0.05).min(1.0);
        if let Ok(player) = self.player.lock() {
            if let Some(state) = player.as_ref() {
                state.sink.set_volume(self.volume);
            }
        }
    }

    pub(super) fn handle_volume_down(&mut self) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        self.volume = (self.volume - 0.05).max(0.0);
        if let Ok(player) = self.player.lock() {
            if let Some(state) = player.as_ref() {
                state.sink.set_volume(self.volume);
            }
        }
    }
}
