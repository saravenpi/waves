use std::path::Path;
use crate::metadata::mp3::save_mp3_metadata;
use crate::metadata::m4a::save_m4a_metadata;
use crate::metadata::flac::save_flac_metadata;
use crate::metadata::wav::write_wav_metadata;
use crate::metadata::ogg::save_ogg_metadata;

pub fn save_audio_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
    cover_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Saving metadata for: {:?}", path);
    eprintln!("Title: '{}', Artist: '{}', Date: '{}', Cover: {:?}", title, artist, date, cover_path);

    let extension = path.extension()
        .and_then(|e| e.to_str())
        .ok_or("No file extension")?
        .to_lowercase();

    match extension.as_str() {
        "mp3" => {
            save_mp3_metadata(path, title, artist, date, cover_path)?;
        }
        "m4a" | "mp4" => {
            save_m4a_metadata(path, title, artist, date, cover_path)?;
        }
        "flac" => {
            save_flac_metadata(path, title, artist, date, cover_path)?;
        }
        "wav" => {
            eprintln!("Writing WAV metadata...");
            write_wav_metadata(path, title, artist, date)?;
            eprintln!("WAV metadata written successfully");

            if cover_path.is_some() {
                eprintln!("Note: Cover art is not yet supported for WAV files");
            }
        }
        "ogg" | "opus" => {
            save_ogg_metadata(path, title, artist, date, cover_path)?;
        }
        _ => {
            return Err(format!("Unsupported file format: {}. Supported formats: MP3, M4A, FLAC, WAV, OGG, Opus.", extension).into());
        }
    }

    Ok(())
}
