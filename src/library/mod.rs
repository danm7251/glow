use anyhow::{Context, Result, anyhow};
use id3::{Tag, TagLike};
use std::{
    fs::{copy, read_dir},
    path::{Path, PathBuf},
};

pub mod song;
pub use song::Song;

pub struct Library {
    songs: Vec<Song>,
    folder: PathBuf,
    allow_non_mp3: bool,
}

impl Library {
    /// Returns a Result containing a Library with a tracklist loaded from the target folder
    pub fn new(folder: impl Into<PathBuf>, allow_non_mp3: bool) -> Result<Self> {
        let folder = folder.into();
        let songs = load_songs(&folder, allow_non_mp3)?;

        Ok(Self {
            songs,
            folder,
            allow_non_mp3,
        })
    }

    /// Returns a Library with an empty tracklist. This function cannot fail
    pub fn empty(folder: impl Into<PathBuf>, allow_non_mp3: bool) -> Self {
        let folder = folder.into();

        Self {
            songs: Vec::new(),
            folder,
            allow_non_mp3,
        }
    }

    /// Returns the tracklist as a vector of Songs
    pub fn songs(&self) -> &Vec<Song> {
        &self.songs
    }

    /// Returns Some(&mut Song) if ``song_id`` exists
    pub fn song_mut(&mut self, song_id: usize) -> Option<&mut Song> {
        self.songs.iter_mut().find(|s| s.id() == song_id)
    }

    /// Returns the Result of attempting to permanently import a new song to the library
    pub fn import_from_path(&mut self, path: &Path) -> Result<()> {
        // TODO: Accept folders
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3") || self.allow_non_mp3)
        {
            return Err(anyhow!(
                "Path was not a valid mp3 file: {}\nError occurred in import_path()",
                path.display()
            ));
        }

        // Only occurs if the path ends in ".."
        let filename = path.file_name().ok_or_else(|| {
            anyhow!("Failed to get filename: {}\nError occurred in import_from_path() because path.file_name() returned None", path.display())
        })?;

        let target = self.folder.join(filename);

        copy(path, &target).with_context(|| {
            format!("Failed to copy {} to {}", path.display(), target.display())
        })?;

        // TODO: Refactor once Song -> Result<Song>,
        //       Consider what to do with the file copy
        match Song::new(self.songs.len(), &target) {
            Some(song) => self.songs.push(song),
            None => {
                return Err(anyhow!(
                    "Failed to instantiate Song from file: {}\nError occurred in import_path()",
                    target.display()
                ));
            }
        }

        Ok(())
    }
}

/// Returns the Result of writing an id3v23 tag to a Song's path, assumes file has id3v23 tag
pub fn write_song_metadata(song: &Song) -> Result<()> {
    let path = song.path();

    let mut tag = Tag::read_from_path(path).with_context(|| {
        format!(
            "Failed to read tag from path: {}\nError occurred in write_song_metadata()",
            path.display()
        )
    })?;

    tag.set_title(song.title());
    tag.set_artist(song.artist());

    tag.write_to_path(path, id3::Version::Id3v23)
        .with_context(|| {
            format!(
                "Failed to write tag to path: {}\nError occurred in write_song_metadata()",
                path.display()
            )
        })?;

    Ok(())
}

/// Returns a Result containing a vector of Songs constructed by scanning the target folder
fn load_songs(folder: &Path, allow_non_mp3: bool) -> Result<Vec<Song>> {
    let mut songs = Vec::new();
    let entries = read_dir(folder).with_context(|| {
        format!(
            "Failed to read directory: {}\nError occured in library::load_songs() at read_dir()",
            folder.display()
        )
    })?;

    // Uses flatten to discard failed entries
    for entry in entries.flatten() {
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|p| p.eq_ignore_ascii_case("mp3") || allow_non_mp3)
        {
            // TODO: Refactor once Song -> Result<Song>
            if let Some(song) = Song::new(songs.len(), &path) {
                songs.push(song);
            }
        }
    }

    Ok(songs)
}
