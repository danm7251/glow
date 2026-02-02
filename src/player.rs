//! High-level control over audio playback.
//!
//! This module defines [`Player`], which manages playback state and volume,
//! coordinating with the lower-level [`AudioEngine`] to manage audio output.

use anyhow::Result;
use std::time::Duration;

use crate::{
    audio::{self, AudioBackend, AudioEngine},
    library::{self, Library},
};

/// Controls audio playback state.
///
/// The `Player` stores its own playback state and other playback details.
/// It connects the UI and Audio layers.
pub struct Player<A: AudioBackend> {
    audio: A,
    // The volume should never change without user input so store here
    // AudioEngine::set_volume does not fail so no need to check
    volume: u8,
}

impl Player<AudioEngine> {
    // TODO: [URGENT] Add documentation
    pub fn with_audio_engine() -> Result<Self> {
        tracing::info!("Creating new Player with AudioEngine");

        let audio = AudioEngine::new()?;

        // TODO: Derive volume from audio?
        Ok(Self { audio, volume: 100 })
    }
}

impl<A: AudioBackend> Player<A> {
    // TODO: [URGENT] Update documentation
    /// Creates a new `Player`.
    ///
    /// The initial state is `Stopped` and the default volume is 100.
    #[allow(unused)]
    pub fn new(audio: A) -> Self {
        Self { audio, volume: 100 }
    }

    /// Returns true if a song is currently playing.
    pub fn is_playing(&self) -> bool {
        !self.audio.is_paused()
    }

    /// Returns the ID of the current song, if it exists.
    pub fn active_id(&self) -> Option<usize> {
        match self.audio.active_id() {
            audio::NO_TRACK => None,
            id => Some(id),
        }
    }

    /// Returns the current playback volume (0-100).
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Returns the players position in the current song.
    pub fn position(&self) -> Option<f64> {
        if self.audio.is_empty() {
            return None;
        }

        Some(self.audio.time_elapsed().as_secs_f64())
    }

    /// Attempts to play a song.
    ///
    /// Tries getting the file path and playing the song.
    /// Finally it sets the playback state to `Playing`.
    pub fn play(&mut self, library: &Library, id: usize) -> Result<()> {
        tracing::info!("Attempting to play song={id} immediately.");

        let path = library.get_song_path(id)?;
        self.audio.play_song(path, id)?;
        Ok(())
    }

    /// Resumes the sink and sets the playback state.
    pub fn resume(&mut self) {
        self.audio.resume();

        // Observability
        if self.audio.is_paused() {
            tracing::info!("Resuming audio playback.");
        } else {
            tracing::warn!("Attempted resume() while audio was already playing.");
        }
    }

    /// Pauses the sink and sets the playback state.
    pub fn pause(&mut self) {
        self.audio.pause();

        // Observability
        #[allow(clippy::if_not_else, reason = "Else clause may be temporary.")]
        if !self.audio.is_paused() {
            tracing::info!("Pausing audio playback.");
        } else {
            tracing::warn!("Attempted pause() while audio was already paused.");
        }
    }

    /// Stops the sink and sets the playback state.
    pub fn stop(&mut self) {
        self.audio.stop();

        // Observability
        #[allow(clippy::if_not_else, reason = "Else clause may be temporary.")]
        if !self.audio.is_empty() {
            tracing::info!("Stopping audio playback.");
        } else {
            tracing::warn!("Attempted stop() while audio was already stopped.");
        }
    }

    pub fn queue(&mut self, library: &Library, id: usize) -> Result<()> {
        tracing::info!("Attempting to play song={id} immediately.");

        let path = library.get_song_path(id)?;
        self.audio.play_song(path, id)?;
        Ok(())
    }

    /// Sets the playback volume to `value` (0-100).
    ///
    /// The provided value is stored internally and passed to `AudioEngine`.
    pub fn set_volume(&mut self, value: u8) {
        tracing::info!("Setting volume to {value}.");

        let clamped = value.min(100);
        self.volume = clamped;
        self.audio.set_volume(clamped);
    }

    /// Attempts to set the time position of the song.
    pub fn set_position(&mut self, value: f64) -> Result<()> {
        // REVIEW: Active ID might be a more effective guard.
        if !self.audio.is_empty() {
            tracing::info!("Attempting seek to {value}.");

            let time = Duration::from_secs_f64(value);
            return self.audio.seek(time);
        }

        tracing::warn!("Attempted seek while audio was stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAudioEngine {
        is_paused: bool,
        sink_empty: bool,
        active_id: usize,
    }

    impl MockAudioEngine {
        fn new() -> Self {
            Self {
                is_paused: false,
                sink_empty: true,
                active_id: audio::NO_TRACK,
            }
        }
    }

    impl AudioBackend for MockAudioEngine {
        fn play_song(&mut self, _path: &std::path::Path, id: usize) -> Result<()> {
            self.is_paused = false;
            self.sink_empty = false;
            self.active_id = id;
            Ok(())
        }

        fn pause(&mut self) {
            self.is_paused = true;
        }

        fn resume(&mut self) {
            self.is_paused = false;
        }

        fn stop(&mut self) {
            self.sink_empty = true;
            self.active_id = audio::NO_TRACK;
        }

        fn queue(&mut self, _path: &std::path::Path, _id: usize) -> Result<()> {
            todo!()
        }

        fn seek(&mut self, _time: Duration) -> Result<()> {
            todo!()
        }

        fn set_volume(&self, _value: u8) {
            todo!()
        }

        fn is_empty(&self) -> bool {
            self.sink_empty
        }

        fn is_paused(&self) -> bool {
            self.is_paused
        }

        fn active_id(&self) -> usize {
            self.active_id
        }

        fn time_elapsed(&self) -> Duration {
            todo!()
        }
    }

    #[test]
    fn new_player_has_full_volume() {
        let audio = MockAudioEngine::new();
        let player = Player::new(audio);
        assert_eq!(player.volume(), 100);
    }
}
