use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::thread;

const DELETE_SOUND: &[u8] = include_bytes!("../assets/delete.mp3");

pub fn play_delete_sound() {
    thread::spawn(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let cursor = Cursor::new(DELETE_SOUND);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    });
}
