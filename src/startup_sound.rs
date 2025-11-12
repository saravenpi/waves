use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;

const STARTUP_SOUND: &[u8] = include_bytes!("../assets/startup.mp3");

pub fn play_startup_sound(finished_flag: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let cursor = Cursor::new(STARTUP_SOUND);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }

        if let Ok(mut finished) = finished_flag.lock() {
            *finished = true;
        }
    });
}
