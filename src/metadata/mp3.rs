use std::path::Path;
use id3::TagLike;

pub fn save_mp3_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
    cover_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
