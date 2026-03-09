use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;

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

        if let Some(cover) = probed.metadata.get()
            .and_then(|m| m.current())
            .and_then(|c| c.visuals().iter().next())
            .map(|v| v.data.to_vec()) {
            return Some(cover);
        }

        if let Some(cover) = probed.format.metadata().current()
            .and_then(|m| m.visuals().iter().next())
            .map(|v| v.data.to_vec()) {
            return Some(cover);
        }

        None
    });

    match result {
        Ok(cover) => cover,
        Err(_) => None,
    }
}
