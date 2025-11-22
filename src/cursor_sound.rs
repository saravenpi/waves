use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::io::Cursor;
use std::cell::RefCell;

static CURSOR_SOUND: &[u8] = include_bytes!("../assets/cursor_move.wav");

thread_local! {
    static AUDIO_STREAM: RefCell<Option<OutputStream>> = RefCell::new(None);
}

pub fn init_sound_system() {
}

pub fn play_cursor_sound(enabled: bool, volume: f32) {
    if !enabled {
        return;
    }

    let sound_data = CURSOR_SOUND.to_vec();

    std::thread::spawn(move || {
        AUDIO_STREAM.with(|stream_cell| {
            let mut stream_ref = stream_cell.borrow_mut();

            if stream_ref.is_none() {
                if let Ok(stream) = OutputStreamBuilder::open_default_stream() {
                    *stream_ref = Some(stream);
                }
            }

            if let Some(stream) = stream_ref.as_ref() {
                let sink = Sink::connect_new(stream.mixer());
                let cursor = Cursor::new(sound_data);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.set_volume(volume);
                    sink.sleep_until_end();
                }
            }
        });
    });
}
