pub struct MetadataEditor {
    pub file_path: std::path::PathBuf,
    pub title: String,
    pub artist: String,
    pub date: String,
    pub cover_path: Option<String>,
    pub has_existing_cover: bool,
    pub existing_cover_data: Option<Vec<u8>>,
    pub cover_changed: bool,
    pub just_opened: bool,
    pub error_message: Option<String>,
}
