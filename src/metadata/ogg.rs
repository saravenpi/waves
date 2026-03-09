use std::fs::File;
use std::io::Write;
use std::path::Path;
use oggvorbismeta::VorbisComments;

pub fn save_ogg_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    date: &str,
    cover_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut writer = std::io::BufWriter::new(output_file);
    writer.write_all(file_out.get_ref())?;
    writer.flush()?;

    eprintln!("OGG/Opus metadata written successfully");

    if cover_path.is_some() {
        eprintln!("Note: Cover art is not yet supported for OGG/Opus files");
    }

    Ok(())
}
