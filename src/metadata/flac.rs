use std::path::Path;

pub fn save_flac_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
    cover_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
