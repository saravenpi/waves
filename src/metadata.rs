use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;
use rodio::{Decoder, Source};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use id3::TagLike;

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

    if let Some(tags) = probed.metadata.get().and_then(|m| m.current()).map(|m| m.tags()) {
        for tag in tags {
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

pub fn extract_duration(path: &Path) -> Duration {
    let result = std::panic::catch_unwind(|| {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file for duration extraction {:?}: {}", path, e);
                return Duration::from_secs(1);
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
                    return Duration::from_secs(1);
                }
            },
            _ => match Decoder::new(buf_reader) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to decode file for duration extraction {:?}: {}", path, e);
                    return Duration::from_secs(1);
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
            Duration::from_secs(1)
        }
    });

    match result {
        Ok(duration) => duration,
        Err(_) => {
            eprintln!("Panic while extracting duration from {:?}", path);
            Duration::from_secs(1)
        }
    }
}

#[allow(dead_code)]
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

fn read_wav_metadata(path: &Path) -> Result<(Option<String>, Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 4];

    file.read_exact(&mut buffer)?;
    if &buffer != b"RIFF" {
        return Err("Not a RIFF file".into());
    }

    file.seek(SeekFrom::Current(4))?;

    file.read_exact(&mut buffer)?;
    if &buffer != b"WAVE" {
        return Err("Not a WAVE file".into());
    }

    let mut title = None;
    let mut artist = None;
    let mut date = None;

    loop {
        let mut chunk_id = [0u8; 4];
        if file.read_exact(&mut chunk_id).is_err() {
            break;
        }

        let mut size_bytes = [0u8; 4];
        file.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        if &chunk_id == b"LIST" {
            let mut list_type = [0u8; 4];
            file.read_exact(&mut list_type)?;

            if &list_type == b"INFO" {
                let mut remaining = size as i64 - 4;
                while remaining > 0 {
                    let mut info_id = [0u8; 4];
                    if file.read_exact(&mut info_id).is_err() {
                        break;
                    }
                    let mut info_size_bytes = [0u8; 4];
                    file.read_exact(&mut info_size_bytes)?;
                    let info_size = u32::from_le_bytes(info_size_bytes) as usize;

                    let mut data = vec![0u8; info_size];
                    file.read_exact(&mut data)?;

                    let text = String::from_utf8_lossy(&data)
                        .trim_end_matches('\0')
                        .to_string();

                    match &info_id {
                        b"INAM" => title = Some(text),
                        b"IART" => artist = Some(text),
                        b"ICRD" => date = Some(text),
                        _ => {}
                    }

                    if info_size % 2 == 1 {
                        file.seek(SeekFrom::Current(1))?;
                    }

                    remaining -= 8 + info_size as i64;
                    if info_size % 2 == 1 {
                        remaining -= 1;
                    }
                }
                break;
            } else {
                file.seek(SeekFrom::Current(size as i64 - 4))?;
            }
        } else {
            file.seek(SeekFrom::Current(size as i64))?;
            if size % 2 == 1 {
                file.seek(SeekFrom::Current(1))?;
            }
        }
    }

    Ok((title, artist, date))
}

fn write_wav_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let mut file = File::open(path)?;
    let mut original_data = Vec::new();
    file.read_to_end(&mut original_data)?;
    drop(file);

    if original_data.len() < 12 || &original_data[0..4] != b"RIFF" || &original_data[8..12] != b"WAVE" {
        return Err("Invalid WAV file".into());
    }

    let mut new_data = Vec::new();
    new_data.extend_from_slice(b"RIFF");
    new_data.extend_from_slice(&[0,0, 0, 0]);
    new_data.extend_from_slice(b"WAVE");

    let mut pos = 12;

    while pos < original_data.len() {
        if pos + 8 > original_data.len() {
            break;
        }

        let chunk_id = &original_data[pos..pos+4];
        let size = u32::from_le_bytes([
            original_data[pos+4],
            original_data[pos+5],
            original_data[pos+6],
            original_data[pos+7],
        ]) as usize;

        if chunk_id == b"LIST" && pos + 12 <= original_data.len() {
            let list_type = &original_data[pos+8..pos+12];
            if list_type == b"INFO" {
                pos += 8 + size;
                if size % 2 == 1 {
                    pos += 1;
                }
                continue;
            }
        }

        if pos + 8 + size > original_data.len() {
            break;
        }

        new_data.extend_from_slice(&original_data[pos..pos+8+size]);
        pos += 8 + size;
        if size % 2 == 1 && pos < original_data.len() {
            new_data.push(original_data[pos]);
            pos += 1;
        }
    }

    let mut info_chunk_data = Vec::new();
    info_chunk_data.extend_from_slice(b"INFO");

    if !title.is_empty() {
        info_chunk_data.extend_from_slice(b"INAM");
        let title_bytes = title.as_bytes();
        info_chunk_data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        info_chunk_data.extend_from_slice(title_bytes);
        if title_bytes.len() % 2 == 1 {
            info_chunk_data.push(0);
        }
    }

    if !artist.is_empty() {
        info_chunk_data.extend_from_slice(b"IART");
        let artist_bytes = artist.as_bytes();
        info_chunk_data.extend_from_slice(&(artist_bytes.len() as u32).to_le_bytes());
        info_chunk_data.extend_from_slice(artist_bytes);
        if artist_bytes.len() % 2 == 1 {
            info_chunk_data.push(0);
        }
    }

    if !date.is_empty() {
        info_chunk_data.extend_from_slice(b"ICRD");
        let date_bytes = date.as_bytes();
        info_chunk_data.extend_from_slice(&(date_bytes.len() as u32).to_le_bytes());
        info_chunk_data.extend_from_slice(date_bytes);
        if date_bytes.len() % 2 == 1 {
            info_chunk_data.push(0);
        }
    }

    if info_chunk_data.len() > 4 {
        let list_size = info_chunk_data.len() as u32;
        let mut list_chunk = Vec::new();
        list_chunk.extend_from_slice(b"LIST");
        list_chunk.extend_from_slice(&list_size.to_le_bytes());
        list_chunk.extend_from_slice(&info_chunk_data);

        new_data.splice(12..12, list_chunk);
    }

    let total_size = (new_data.len() - 8) as u32;
    new_data[4..8].copy_from_slice(&total_size.to_le_bytes());

    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;

    output_file.write_all(&new_data)?;

    Ok(())
}

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
            let mut tag = if path.exists() {
                id3::Tag::read_from_path(path).unwrap_or_else(|_| id3::Tag::new())
            } else {
                id3::Tag::new()
            };

            tag.set_title(title);
            tag.set_artist(artist);

            if !date.is_empty() {
                if let Ok(year) = date.parse::<i32>() {
                    tag.set_date_recorded(id3::Timestamp {
                        year,
                        month: None,
                        day: None,
                        hour: None,
                        minute: None,
                        second: None,
                    });
                }
            }

            tag.remove_all_pictures();

            if let Some(cover_file) = cover_path {
                eprintln!("Reading cover file: {}", cover_file);
                match std::fs::read(cover_file) {
                    Ok(cover_data) => {
                        eprintln!("Cover file read successfully, {} bytes", cover_data.len());
                        let mime_type = if cover_file.to_lowercase().ends_with(".png") {
                            "image/png"
                        } else {
                            "image/jpeg"
                        };
                        tag.add_frame(id3::frame::Picture {
                            mime_type: mime_type.to_string(),
                            picture_type: id3::frame::PictureType::CoverFront,
                            description: String::new(),
                            data: cover_data,
                        });
                        eprintln!("Cover added to tag");
                    }
                    Err(e) => {
                        eprintln!("Failed to read cover file: {}", e);
                    }
                }
            }

            eprintln!("Writing tag to file...");
            tag.write_to_path(path, id3::Version::Id3v24)?;
            eprintln!("Tag written successfully");
        }
        "m4a" | "mp4" => {
            let mut tag = mp4ameta::Tag::read_from_path(path)?;

            tag.set_title(title);
            tag.set_artist(artist);

            if !date.is_empty() {
                tag.set_year(date);
            }

            tag.remove_artworks();

            if let Some(cover_file) = cover_path {
                eprintln!("Reading cover file: {}", cover_file);
                match std::fs::read(cover_file) {
                    Ok(cover_data) => {
                        eprintln!("Cover file read successfully, {} bytes", cover_data.len());
                        let img_fmt = if cover_file.to_lowercase().ends_with(".png") {
                            mp4ameta::ImgFmt::Png
                        } else {
                            mp4ameta::ImgFmt::Jpeg
                        };
                        tag.set_artwork(mp4ameta::Img {
                            fmt: img_fmt,
                            data: cover_data,
                        });
                        eprintln!("Cover added to tag");
                    }
                    Err(e) => {
                        eprintln!("Failed to read cover file: {}", e);
                    }
                }
            }

            eprintln!("Writing tag to file...");
            tag.write_to_path(path)?;
            eprintln!("Tag written successfully");
        }
        "flac" => {
            let mut tag = metaflac::Tag::read_from_path(path)?;

            tag.set_vorbis("TITLE", vec![title]);
            tag.set_vorbis("ARTIST", vec![artist]);

            if !date.is_empty() {
                tag.set_vorbis("DATE", vec![date]);
            }

            tag.remove_picture_type(metaflac::block::PictureType::CoverFront);

            if let Some(cover_file) = cover_path {
                eprintln!("Reading cover file: {}", cover_file);
                match std::fs::read(cover_file) {
                    Ok(cover_data) => {
                        eprintln!("Cover file read successfully, {} bytes", cover_data.len());
                        let mime_type = if cover_file.to_lowercase().ends_with(".png") {
                            "image/png"
                        } else {
                            "image/jpeg"
                        };
                        tag.add_picture(mime_type, metaflac::block::PictureType::CoverFront, cover_data);
                        eprintln!("Cover added to tag");
                    }
                    Err(e) => {
                        eprintln!("Failed to read cover file: {}", e);
                    }
                }
            }

            eprintln!("Writing tag to file...");
            tag.write_to_path(path)?;
            eprintln!("Tag written successfully");
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
            use oggvorbismeta::VorbisComments;

            eprintln!("Writing OGG/Opus metadata...");

            let mut tags = Vec::new();
            if !title.is_empty() {
                tags.push(("TITLE".to_string(), title.to_string()));
            }
            if !artist.is_empty() {
                tags.push(("ARTIST".to_string(), artist.to_string()));
            }
            if !date.is_empty() {
                tags.push(("DATE".to_string(), date.to_string()));
            }

            let file_in = File::open(path)?;
            let comment_header = <oggvorbismeta::CommentHeader as VorbisComments>::from("".to_string(), tags);
            let file_out = oggvorbismeta::replace_comment_header(file_in, &comment_header)?;

            let output_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)?;
            use std::io::Write;
            let mut writer = std::io::BufWriter::new(output_file);
            writer.write_all(file_out.get_ref())?;
            writer.flush()?;

            eprintln!("OGG/Opus metadata written successfully");

            if cover_path.is_some() {
                eprintln!("Note: Cover art is not yet supported for OGG/Opus files");
            }
        }
        _ => {
            return Err(format!("Unsupported file format: {}. Supported formats: MP3, M4A, FLAC, WAV, OGG, Opus.", extension).into());
        }
    }

    Ok(())
}
