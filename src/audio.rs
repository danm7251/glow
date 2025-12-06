use anyhow::{Result, anyhow};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, source};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

// TODO: [SOON] Review and refine code
// TODO: [SOON] Document module
// TODO: [LATER] Stop rodio printing when the OutputStream is dropped in developer builds
pub struct AudioEngine {
    // OutputStream must be kept alive
    _stream: OutputStream,
    sink: Sink,

    // Deprecated
    pub is_playing: bool, // TODO: [SOON] Encapsulate field
    // Deprecated
    current_song_id: Option<usize>, // TODO: [SOON] Consider removing this field and giving it to a playback controller
    // Depracated
    volume: u8,
}

impl AudioEngine {
    pub fn new() -> Self {
        let stream = OutputStreamBuilder::open_default_stream().expect("Open default audio stream");
        let sink = Sink::connect_new(stream.mixer());

        Self {
            _stream: stream,
            sink,
            is_playing: false,
            current_song_id: None,
            volume: 100,
        }
    }

    // Deprecated
    pub fn old_play_song(&mut self, path: &Path, id: usize) -> Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        self.sink.stop();
        self.sink.append(source);
        self.current_song_id = Some(id);
        self.is_playing = true;

        Ok(())
    }

    /// Attempts to play a song.
    ///
    /// Opens a file and tries to decode it, if succsessful the file is queued in the sink.
    pub fn play_song(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        // FIXME: [LATER] Find a better solution to the issue of playing a new song while the engine is paused
        // TODO: [SOONEST] self.sink.stop()
        self.sink.append(source);

        Ok(())
    }

    // Deprecated
    pub fn old_resume(&mut self) {
        self.sink.play();
        self.is_playing = true;
    }

    /// Resumes the sink.
    pub fn resume(&mut self) {
        self.sink.play();
    }

    // Deprecated
    pub fn old_pause(&mut self) {
        self.sink.pause();
        self.is_playing = false;
    }

    /// Pauses the sink.
    pub fn pause(&mut self) {
        self.sink.pause();
    }

    // Deprecated
    pub fn old_stop(&mut self) {
        self.sink.stop();
        self.is_playing = false;
    }

    /// Stops the sink.
    pub fn stop(&mut self) {
        self.sink.stop();
    }

    // Deprecated
    pub fn update(&mut self) {
        if self.is_playing && self.sink.empty() {
            self.is_playing = false;
        }
    }

    pub fn seek(&mut self, time: Duration) -> Result<()> {
        self.sink
            .try_seek(time)
            .map_err(|e| anyhow!("Failed to seek! {e}"))?;

        Ok(())
    }

    // Deprecated
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn time_elapsed(&self) -> Duration {
        self.sink.get_pos()
    }

    // Deprecated
    pub fn mut_volume(&mut self) -> &mut u8 {
        &mut self.volume
    }

    // Deprecated
    pub fn old_set_volume(&self) {
        let value = f32::from(self.volume) * 0.01;
        self.sink.set_volume(value);
    }

    /// Sets the volume of the sink.
    pub fn set_volume(&self, value: u8) {
        let normed_value = f32::from(value) * 0.01;
        self.sink.set_volume(normed_value);
    }
}
