use anyhow::{Context, Result};
use id3::{Tag, TagLike};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

// TODO: Prototype validity enum

pub struct Song {
    id: usize,
    path: PathBuf,
    title: String,
    artist: String,
    duration: Duration,
}

impl Song {
    // TODO: REVIEW AND DOC
    /// Returns a Result containing a Song
    pub fn new(id: usize, path: &Path, duration: Duration) -> Result<Self> {
        let filename_os = path
            .file_name()
            .with_context(|| format!("Failed to get filename at {}", path.display()))?;
        let tag_result = Tag::read_from_path(path);

        let filename = filename_os.to_string_lossy().into_owned();

        let title = match &tag_result {
            Ok(tag) => tag.title().unwrap_or_else(|| &filename).to_string(),
            Err(_) => filename,
        };

        let artist = match &tag_result {
            Ok(tag) => tag.artist().unwrap_or("Unknown Artist").to_string(),
            Err(_) => "Unknown Artist".to_string(),
        };

        Ok(Self {
            id,
            path: path.to_path_buf(),
            title,
            artist,
            duration,
        })
    }

    /// Returns the ID of a song
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the filepath of a song
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the title of a song
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the artist of a song
    pub fn artist(&self) -> &str {
        &self.artist
    }

    // TODO: REVIEW AND DOC
    pub fn duration_as_str(&self) -> String {
        let d = self.duration.as_secs();
        let m = d / 60;
        let s: String = match d % 60 {
            n @ 0..=9 => "0".to_owned() + &n.to_string(),
            n => n.to_string(),
        };
        format!("{m}:{s}")
    }

    /// Sets the title of a song
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Sets the artist of a song
    pub fn set_artist(&mut self, artist: impl Into<String>) {
        self.artist = artist.into();
    }
}
