use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::cell::RefCell;

static CURSOR_SOUND: &[u8] = include_bytes!("../assets/cursor_move.wav");

thread_local! {
    static AUDIO_STREAM: RefCell<Option<(OutputStream, rodio::OutputStreamHandle)>> = RefCell::new(None);
}

/// Initializes the persistent audio system.
///
/// This function is a no-op. The audio system is now lazily initialized per-thread.
pub fn init_sound_system() {
}

/// Plays the navigation sound effect instantly.
///
/// Uses a thread-local cached audio stream for minimal latency.
/// Creates the stream on first use in each thread.
///
/// # Arguments
/// * `enabled` - Whether UI sounds are enabled
/// * `volume` - Volume level (0.0 to 1.0)
pub fn play_cursor_sound(enabled: bool, volume: f32) {
    if !enabled {
        return;
    }

    let sound_data = CURSOR_SOUND.to_vec();

    std::thread::spawn(move || {
        AUDIO_STREAM.with(|stream_cell| {
            let mut stream_ref = stream_cell.borrow_mut();

            if stream_ref.is_none() {
                if let Ok((_stream, handle)) = OutputStream::try_default() {
                    *stream_ref = Some((_stream, handle));
                }
            }

            if let Some((_stream, stream_handle)) = stream_ref.as_ref() {
                if let Ok(sink) = Sink::try_new(stream_handle) {
                    let cursor = Cursor::new(sound_data);
                    if let Ok(source) = Decoder::new(cursor) {
                        sink.append(source);
                        sink.set_volume(volume);
                        sink.sleep_until_end();
                    }
                }
            }
        });
    });
}
