use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn create_placeholder_waveform() -> Vec<f32> {
    const SAMPLES: usize = 500;
    vec![0.5; SAMPLES]
}

pub fn extract_waveform(path: &Path) -> Vec<f32> {
    const SAMPLES: usize = 500;

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file for waveform extraction {:?}: {}", path, e);
            return create_placeholder_waveform();
        }
    };

    let buf_reader = BufReader::new(file);
    let source = match Decoder::new(buf_reader) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to decode file for waveform extraction {:?}: {}", path, e);
            return create_placeholder_waveform();
        }
    };

    let audio_samples: Vec<f32> = source
        .convert_samples::<f32>()
        .collect();

    if audio_samples.is_empty() {
        eprintln!("No audio samples found in {:?}", path);
        return create_placeholder_waveform();
    }

    let chunk_size = audio_samples.len() / SAMPLES;
    if chunk_size == 0 {
        eprintln!("Audio file too short for waveform extraction: {:?}", path);
        return create_placeholder_waveform();
    }

    let mut waveform = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(audio_samples.len());

        if start >= audio_samples.len() || end > audio_samples.len() {
            eprintln!("Invalid range in waveform extraction for {:?}", path);
            waveform.push(0.0);
            continue;
        }

        let mut max_amplitude = 0.0_f32;
        for sample in &audio_samples[start..end] {
            if sample.is_finite() {
                max_amplitude = max_amplitude.max(sample.abs());
            }
        }

        waveform.push(max_amplitude.min(1.0));
    }

    waveform
}
