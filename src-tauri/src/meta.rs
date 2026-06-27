use base64::{engine::general_purpose::STANDARD, Engine as _};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::{Accessor, Tag};

#[tauri::command]
pub fn read_cover(path: String) -> Option<String> {
    let tagged = Probe::open(&path).ok()?.read().ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let pic = tag.pictures().first()?;
    let mime = pic.mime_type().map(|m| m.as_str()).unwrap_or("image/jpeg");
    Some(format!("data:{};base64,{}", mime, STANDARD.encode(pic.data())))
}

#[tauri::command]
pub fn write_metadata(
    path: String,
    title: String,
    artist: String,
    album: String,
    date: String,
) -> Result<(), String> {
    let mut tagged = Probe::open(&path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    if tagged.primary_tag_mut().is_none() {
        let tt = tagged.primary_tag_type();
        tagged.insert_tag(Tag::new(tt));
    }
    let tag = tagged.primary_tag_mut().ok_or("no tag")?;

    tag.set_title(title);
    tag.set_artist(artist);
    tag.set_album(album);
    if let Ok(year) = date.trim().parse::<u32>() {
        tag.set_year(year);
    }

    tagged
        .save_to_path(&path, WriteOptions::default())
        .map_err(|e| e.to_string())
}
