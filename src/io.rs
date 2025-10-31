use id3::{Tag, TagLike};
use crate::song::Song;

pub struct TagWriter;

impl TagWriter {
    // Assumes that the in-memory song is already up to date
    // On Ok edit window should close, otherwise don't close and throw error
    pub fn save_metadata(song: &Song) -> id3::Result<()> {
        let mut tag = Tag::read_from_path(&song.path)?;
        tag.set_title(&song.display_title);
        tag.write_to_path(&song.path, id3::Version::Id3v24)?;

        Ok(())
    }
}