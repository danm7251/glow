use id3::{Tag, TagLike};
use std::path::{Path, PathBuf};

// TODO: Write minimal documentation
// TODO: Change all fields to private
// TODO: Consider renaming fields
// TODOL Prototype an enum and field to track the status of the file
pub struct Song {
    pub id: usize,
    pub path: PathBuf,
    pub display_title: String,
    pub display_artist: String,
}

impl Song {
    pub fn new(id: usize, path: &PathBuf) -> Option<Self> {
        // TODO: Convert return type to Result
        // TODO: Review and improve function, for example the type of parameter path
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
            id,
            path: path.clone(),
            display_title,
            display_artist,
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.display_title
    }

    pub fn artist(&self) -> &str {
        &self.display_artist
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.display_title = title.into();
    }

    pub fn set_artist(&mut self, artist: impl Into<String>) {
        self.display_artist = artist.into();
    }
}
