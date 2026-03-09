use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use crate::metadata::duration::extract_duration;
use crate::metadata::wav::read_wav_metadata;

pub fn extract_metadata(path: &Path) -> (String, Option<String>, Option<String>, Option<String>, Option<String>, Duration) {
    let default_result = || {
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let duration = extract_duration(path);
        (title, None, None, None, None, duration)
    };

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.to_lowercase() == "wav" {
            if let Ok((title_opt, artist_opt, date_opt)) = read_wav_metadata(path) {
                let final_title = title_opt.unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string()
                });
                let duration = extract_duration(path);
                return (final_title, artist_opt, None, date_opt, None, duration);
            }
        }
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file for metadata extraction {:?}: {}", path, e);
                return default_result();
            }
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut probed = match symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to probe file for metadata {:?}: {}", path, e);
                return default_result();
            }
        };

    let mut title = None;
    let mut artist = None;
    let mut album = None;
    let mut date = None;
    let mut track_number = None;
    let mut track_total = None;

    if let Some(metadata_rev) = probed.format.metadata().current() {
        for tag in metadata_rev.tags() {
            match tag.std_key {
                Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                    title = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Artist) => {
                    artist = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Album) => {
                    album = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Date) => {
                    date = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::TrackNumber) => {
                    track_number = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::TrackTotal) => {
                    track_total = Some(tag.value.to_string());
                }
                _ => {}
            }
        }
    }

    if let Some(metadata) = probed.metadata.get() {
        if let Some(metadata_rev) = metadata.current() {
            for tag in metadata_rev.tags() {
            match tag.std_key {
                Some(symphonia::core::meta::StandardTagKey::TrackTitle) if title.is_none() => {
                    title = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Artist) if artist.is_none() => {
                    artist = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Album) if album.is_none() => {
                    album = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::Date) if date.is_none() => {
                    date = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::TrackNumber) if track_number.is_none() => {
                    track_number = Some(tag.value.to_string());
                }
                Some(symphonia::core::meta::StandardTagKey::TrackTotal) if track_total.is_none() => {
                    track_total = Some(tag.value.to_string());
                }
                _ => {}
                }
            }
        }
    }

        let final_title = title.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

        let track_info = match (track_number, track_total) {
            (Some(num), Some(total)) => Some(format!("{}/{}", num, total)),
            (Some(num), None) => Some(num),
            _ => None,
        };

        let duration = extract_duration(path);

        (final_title, artist, album, date, track_info, duration)
    }));

    match result {
        Ok(metadata) => metadata,
        Err(_) => {
            eprintln!("Panic while extracting metadata from {:?}", path);
            default_result()
        }
    }
}
