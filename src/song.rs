use std::path::PathBuf;
use id3::{Tag, TagLike};

pub struct Song {
    pub song_id: usize,
    pub path: PathBuf,
    pub display_title: String,
    pub display_artist: String,
}

impl Song {
    pub fn new(song_id: usize, path: &PathBuf) -> Option<Self> {
        let filename = path.file_name()?.to_string_lossy().into_owned();
        let tag_result = Tag::read_from_path(path);

        let display_title = match &tag_result {
            Ok(tag) => tag.title().unwrap_or_else(|| &filename).to_string(),
            Err(_) => filename,
        };

        let display_artist = match &tag_result {
            Ok(tag) => tag.artist().unwrap_or("Unknown Artist").to_string(),
            Err(_) => "Unknown Artist".to_string(),
        };

        Some(Self {
            song_id,
            path: path.clone(),
            display_title,
            display_artist,
        })
    }
}
