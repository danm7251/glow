use std::{ffi::OsString, path::PathBuf};

pub struct Song {
    pub path: PathBuf,
    pub title: OsString,
}

// Encapsulates the extraction of metadata from song files
// If there are any errors returns None avoiding the GUI having to handle them
// Allowing the GUI to skip them

impl Song {
    pub fn new(path: &PathBuf) -> Option<Self> {
        let title = path.file_name()?.to_os_string();

        Some(
            Self {
                path: path.clone(),
                title,
            }
        )
    }
}