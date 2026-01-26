//! High-level control over audio playback.
//!
//! This module defines [`Player`], which manages playback state and volume,
//! coordinating with the lower-level [`AudioEngine`] to manage audio output.

use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::{
    audio::{AudioBackend, AudioEngine},
    library::Library,
};

/// Represents the current state of audio playback.
///
/// `Playing` and `Paused` store the ID of the active song.
/// `Stopped` indicates no song is active.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackState {
    Playing { id: usize },
    Paused { id: usize },
    Stopped,
}

/// Controls audio playback state.
///
/// The `Player` stores its own playback state and other playback details.
/// It connects the UI and Audio layers.
pub struct Player<A: AudioBackend> {
    audio: A,
    state: PlaybackState,
    // The volume should never change without user input so store here
    // AudioEngine::set_volume does not fail so no need to check
    volume: u8,
}

impl Player<AudioEngine> {
    // TODO: [URGENT] Add documentation
    pub fn with_audio_engine() -> Result<Self> {
        let audio = AudioEngine::new()?;

        Ok(Self {
            audio,
            state: PlaybackState::Stopped,
            volume: 100,
        })
    }
}

impl<A: AudioBackend> Player<A> {
    // TODO: [URGENT] Update documentation
    /// Creates a new `Player`.
    ///
    /// The initial state is `Stopped` and the default volume is 100.
    #[allow(unused)]
    pub fn new(audio: A) -> Self {
        Self {
            audio,
            state: PlaybackState::Stopped,
            volume: 100,
        }
    }

    #[allow(dead_code)]
    /// Returns a reference to the current playback state.
    pub fn state(&self) -> PlaybackState {
        // REVIEW: [LATER] Return type
        self.state
    }

    /// Returns true if a song is currently playing.
    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing { .. })
    }

    /// Returns the ID of the current song, if it exists.
    pub fn current_id(&self) -> Option<usize> {
        match self.state {
            PlaybackState::Playing { id } | PlaybackState::Paused { id } => Some(id),
            PlaybackState::Stopped => None,
        }
    }

    /// Returns the current playback volume (0-100).
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Returns the players position in the current song.
    pub fn position(&self) -> Option<f64> {
        if self.state == PlaybackState::Stopped {
            return None;
        }
        Some(self.audio.time_elapsed().as_secs_f64())
    }

    /// Attempts to play a song.
    ///
    /// Tries getting the file path and playing the song.
    /// Finally it sets the playback state to `Playing`.
    pub fn play(&mut self, library: &Library, id: usize) -> Result<()> {
        let path = library.get_song_path(id)?;
        self.audio.play_song(path, id)?;
        Ok(())
    }

    /// Resumes the sink and sets the playback state.
    pub fn resume(&mut self) {
        if let PlaybackState::Paused { id } = self.state {
            self.audio.resume();
            self.state = PlaybackState::Playing { id };
            info!("Player state was resumed");
        } else {
            warn!("Resume was called while playing or stopped");
        }
    }

    /// Pauses the sink and sets the playback state.
    pub fn pause(&mut self) {
        if let PlaybackState::Playing { id } = self.state {
            self.audio.pause();
            self.state = PlaybackState::Paused { id };
            info!("Player state was paused");
        } else {
            warn!("Pause was called while paused or stopped");
        }
    }

    /// Stops the sink and sets the playback state.
    pub fn stop(&mut self) {
        if self.state != PlaybackState::Stopped {
            self.audio.stop();
        }
    }

    /// Checks for any changes in the state of the `AudioEngine`.
    ///
    /// Checks if the `AudioEngine` is idle and if it doesn't match `Player`'s internal state updates `Player`
    pub fn update(&mut self) {
        if self.audio.is_idle() {
            if self.state != PlaybackState::Stopped {
                self.state = PlaybackState::Stopped;
                info!("Player state was set to Stopped");
            }
        } else {
            let active_id = self.audio.active_id();

            match self.state {
                PlaybackState::Playing { id } if id != active_id => {
                    self.state = PlaybackState::Playing { id: active_id };
                    info!("Player state ID changed to active ID");
                }
                PlaybackState::Paused { id } if id != active_id => {
                    self.state = PlaybackState::Paused { id: active_id };
                    info!("Player state ID = {id} changed to active ID = {active_id}");
                    warn!("ID changed while Player state was paused");
                }
                PlaybackState::Stopped => {
                    // This happens when starting a song while stopped.
                    self.state = PlaybackState::Playing { id: active_id };
                    info!("Player state was set to Playing with ID {active_id}");
                }

                _ => {
                    // ID matches no state update needed.
                }
            }
        }
    }

    /// Sets the playback volume to `value` (0-100).
    ///
    /// The provided value is stored internally and passed to `AudioEngine`.
    pub fn set_volume(&mut self, value: u8) {
        let clamped = value.min(100);
        self.volume = clamped;
        self.audio.set_volume(clamped);
    }

    /// Attempts to set the time position of the song.
    pub fn set_position(&mut self, value: f64) -> Result<()> {
        tracing::info!("Attempting seek");
        let seeking_is_allowed = !matches!(self.state, PlaybackState::Stopped);

        if !seeking_is_allowed {
            tracing::warn!("Seek attempted while Player was stopped");
            return Ok(());
        }

        let time = Duration::from_secs_f64(value);
        self.audio.seek(time)
    }
}

#[cfg(test)]
mod tests {
    // TODO: [SOON] Automate tests with a commit hook and CI using github actions
    use super::*;

    struct MockAudioEngine;

    impl MockAudioEngine {
        fn new() -> Self {
            Self
        }
    }

    impl AudioBackend for MockAudioEngine {
        fn play_song(&mut self, path: &std::path::Path, id: usize) -> Result<()> {
            todo!()
        }

        fn pause(&mut self) {}

        fn resume(&mut self) {}

        fn stop(&mut self) {}

        fn seek(&mut self, time: Duration) -> Result<()> {
            todo!()
        }

        fn set_volume(&self, value: u8) {
            todo!()
        }

        fn is_idle(&self) -> bool {
            todo!()
        }

        fn active_id(&self) -> usize {
            todo!()
        }

        fn time_elapsed(&self) -> Duration {
            todo!()
        }
    }

    #[test]
    fn new_player_starts_stopped_with_full_volume() {
        let audio = MockAudioEngine;
        let player = Player::new(audio);
        assert!(matches!(player.state(), PlaybackState::Stopped));
        assert_eq!(player.volume(), 100);
    }

    #[test]
    fn playback_states_rotate_with_id() {
        let audio = MockAudioEngine;
        let mut player = Player::new(audio);

        // Player::play() is not used as it depends on Library
        // Consider using an AudioEngine trait to also apply to a MockAudioEngine in test
        player.state = PlaybackState::Playing { id: 7 };
        assert!(matches!(player.state(), PlaybackState::Playing { id: 7 }));

        player.pause();
        assert!(matches!(player.state(), PlaybackState::Paused { id: 7 }));

        player.resume();
        assert!(matches!(player.state(), PlaybackState::Playing { id: 7 }));

        player.stop();
        assert!(matches!(player.state(), PlaybackState::Stopped));
    }
}
