use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;
use rodio::{Decoder, Source};

pub fn extract_duration(path: &Path) -> Duration {
    let result = std::panic::catch_unwind(|| {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file for duration extraction {:?}: {}", path, e);
                return Duration::from_secs(0);
            }
        };

        let buf_reader = BufReader::new(file);
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let source = match ext.as_str() {
            "m4a" | "mp4" | "aac" => match Decoder::new_mp4(buf_reader) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to decode M4A file for duration extraction {:?}: {}", path, e);
                    return Duration::from_secs(0);
                }
            },
            _ => match Decoder::new(buf_reader) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to decode file for duration extraction {:?}: {}", path, e);
                    return Duration::from_secs(0);
                }
            },
        };

        if let Some(duration) = source.total_duration() {
            return duration;
        }

        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let sample_count = source.count();

        if sample_rate > 0 && channels > 0 {
            let total_samples = sample_count as u64;
            let duration_secs = total_samples / (sample_rate as u64 * channels as u64);
            Duration::from_secs(duration_secs)
        } else {
            Duration::from_secs(0)
        }
    });

    match result {
        Ok(duration) => duration,
        Err(_) => {
            eprintln!("Panic while extracting duration from {:?}", path);
            Duration::from_secs(0)
        }
    }
}
