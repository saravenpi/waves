use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::io::Cursor;
use std::thread;

const DELETE_SOUND: &[u8] = include_bytes!("../../assets/delete.mp3");

pub fn play_delete_sound() {
    thread::spawn(move || {
        if let Ok(stream) = OutputStreamBuilder::open_default_stream() {
            let sink = Sink::connect_new(stream.mixer());
            let cursor = Cursor::new(DELETE_SOUND);
            if let Ok(source) = Decoder::new(cursor) {
                sink.append(source);
                sink.sleep_until_end();
            }
        }
    });
}
