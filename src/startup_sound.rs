use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn play_startup_sound(finished_flag: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(800));

        if let Ok(mut finished) = finished_flag.lock() {
            *finished = true;
        }
    });
}
