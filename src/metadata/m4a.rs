use std::path::Path;

pub fn save_m4a_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
    cover_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
