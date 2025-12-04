pub enum PlaybackState {
    Playing { id: usize },
    Paused { id: usize },
    Stopped,
}

pub struct Player {
    state: PlaybackState,
}

impl Player {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
        }
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }
}
