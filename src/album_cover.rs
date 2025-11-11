use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;

/// Extracts embedded album cover art from an audio file.
///
/// Searches both container-level and format-level metadata for visual data.
/// Uses symphonia for parsing multiple audio formats.
/// # Arguments
/// * `path` - Path to the audio file
/// # Returns
/// Optional byte vector containing the cover image data (JPEG or PNG)
pub fn extract_album_cover(path: &Path) -> Option<Vec<u8>> {
    let result = std::panic::catch_unwind(|| {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
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
            Err(_) => return None,
        };

        if let Some(metadata) = probed.metadata.get() {
            if let Some(current_metadata) = metadata.current() {
                if let Some(visual) = current_metadata.visuals().iter().next() {
                    return Some(visual.data.to_vec());
                }
            }
        }

        if let Some(metadata_rev) = probed.format.metadata().current() {
            if let Some(visual) = metadata_rev.visuals().iter().next() {
                return Some(visual.data.to_vec());
            }
        }

        None
    });

    match result {
        Ok(cover) => cover,
        Err(_) => None,
    }
}
