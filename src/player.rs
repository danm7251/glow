use crate::audio::AudioEngine;

pub enum PlaybackState {
    Playing { id: usize },
    Paused { id: usize },
    Stopped,
}

pub struct Player {
    audio: AudioEngine,
    state: PlaybackState,
    volume: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            audio: AudioEngine::new(),
            state: PlaybackState::Stopped,
            volume: 100,
        }
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }
}
