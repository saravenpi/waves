mod duration;
mod extraction;
mod wav;
mod mp3;
mod m4a;
mod flac;
mod ogg;
mod saving;

pub use extraction::extract_metadata;
pub use saving::save_audio_metadata;
