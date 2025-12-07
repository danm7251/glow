use anyhow::{Context, Result, anyhow};
use id3::{Tag, TagLike};
use rodio::{Decoder, Source};
use std::{
    fs::{File, copy, read_dir, remove_file},
    path::{Path, PathBuf},
    time::Duration,
};

pub mod song;
pub use song::Song;

pub struct Library {
    songs: Vec<Song>,
    folder: PathBuf,
    allow_non_mp3: bool,
}

impl Library {
    /// Returns a Result containing a Library with a tracklist loaded from the target folder.
    pub fn new(folder: impl Into<PathBuf>, allow_non_mp3: bool) -> Result<Self> {
        let folder = folder.into();
        let songs = load_songs(&folder, allow_non_mp3)?;

        Ok(Self {
            songs,
            folder,
            allow_non_mp3,
        })
    }

    /// Returns a Library with an empty tracklist. This function cannot fail.
    pub fn empty(folder: impl Into<PathBuf>, allow_non_mp3: bool) -> Self {
        let folder = folder.into();

        Self {
            songs: Vec::new(),
            folder,
            allow_non_mp3,
        }
    }

    /// Returns the tracklist as a vector of Songs.
    pub fn songs(&self) -> &Vec<Song> {
        &self.songs
    }

    /// Returns an immutable reference to a Song, if it exists.
    pub fn get_song(&self, song_id: usize) -> Option<&Song> {
        self.songs.iter().find(|s| s.id() == song_id)
    }

    /// Returns a view into the filepath associated with the id.
    pub fn get_song_path(&self, song_id: usize) -> Result<&Path> {
        match self.songs.iter().find(|s| s.id() == song_id) {
            Some(s) => Ok(s.path()),
            None => Err(anyhow!("Failed to access path for song id: {song_id}")),
        }
    }

    /// Returns a mutable reference to a Song, if it exists.
    pub fn get_song_mut(&mut self, song_id: usize) -> Option<&mut Song> {
        self.songs.iter_mut().find(|s| s.id() == song_id)
    }

    /// Returns the Result of attempting to permanently import a new song to the library.
    pub fn import_from_path(&mut self, path: &Path) -> Result<()> {
        // TODO: [LATER] Accept dropped folders
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

        match Song::new(self.songs.len(), &target, try_duration(&target)) {
            Ok(song) => self.songs.push(song),
            Err(e) => {
                // Attempts to remove the copied file if Song creation fails
                let undo_copy = match remove_file(path) {
                    Ok(()) => format!("Succsessfully removed copy: {}", path.display()),
                    Err(e) => format!(
                        "Failed to remove the copy at: {}\n This was due to {e}",
                        path.display()
                    ),
                };
                // Returns error information with extra details about the emergency file removal
                return Err(e)
                    .context(format!(
                        "Failed to create Song from path: {}\nError occurred in import_path()",
                        target.display()
                    ))
                    .context(undo_copy);
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
            match Song::new(songs.len(), &path, try_duration(&path)) {
                Ok(song) => songs.push(song),
                Err(e) => {
                    return Err(e).context(format!(
                        "Failed to create Song from path: {}\nError occurred in load_songs()",
                        path.display()
                    ));
                }
            }
        }
    }

    Ok(songs)
}

/// Returns a Duration calculated by Rodio, if it exists
fn try_duration(path: &Path) -> Option<Duration> {
    let file = File::open(path).ok()?;
    let source = Decoder::try_from(file).ok()?;
    source.total_duration()
}
