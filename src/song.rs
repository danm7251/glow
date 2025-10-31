use std::{path::PathBuf};

use id3::{Tag, TagLike};

pub struct Song {
    pub path: PathBuf,
    pub display_title: String,
}

// Encapsulates the extraction of metadata from song files
// If there are any errors returns None avoiding the GUI having to handle them
// Allowing the GUI to skip them

impl Song {
    pub fn new(path: &PathBuf) -> Option<Self> {
        // Prepares filename for display returns None if file_name returns None
        let filename = path.file_name()?.to_string_lossy().into_owned();

        // Attempts to read tag, if it fails or title is None falls back to filename
        // Consider only retrieving filename on fail or None
        let display_title = match Tag::read_from_path(path) {
            Ok(tag) => tag.title().unwrap_or_else(|| &filename).to_string(),
            Err(_) => filename,
        };

        Some(
            Self {
                path: path.clone(),
                display_title,
            }
        )
    }
}