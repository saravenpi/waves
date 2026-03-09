use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub fn read_wav_metadata(path: &Path) -> Result<(Option<String>, Option<String>, Option<String>), Box<dyn std::error::Error>> {
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

pub fn write_wav_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
