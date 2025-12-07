//! Low-level control over audio playback.
//!
//! This module defines [`AudioEngine`], a thin wrapper around rodio's
//! [`OutputStream`] and [`Sink`]. It owns the output stream so audio continues
//! playing while the engine is alive. It exposes simple methods for controlling
//! output and getting the time elapsed.

use anyhow::{Context, Result, anyhow};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

/// Handles low-level audio output using rodio.
///
/// An `AudioEngine` owns both the output stream and a single sink. It is
/// responsible for keeping the output stream alive and issuing playback
/// commands for the current audio source.
pub struct AudioEngine {
    // OutputStream must be kept alive
    _stream: OutputStream,
    sink: Sink,
}

impl AudioEngine {
    /// Creates a new `AudioEngine`.
    ///
    /// This opens the default output device and constructs a sink that can be
    /// controlled via the other methods. Returns an error if no suitable
    /// output device or stream can be created.
    pub fn new() -> Result<Self> {
        let mut stream = OutputStreamBuilder::open_default_stream()
            .context("Failed to construct OutputStream in AufioEngine::new()")?;
        stream.log_on_drop(false);

        let sink = Sink::connect_new(stream.mixer());

        Ok(Self {
            _stream: stream,
            sink,
        })
    }

    /// Returns the current playback position of the sink.
    ///
    /// The returned [`Duration`] is measured from the start of the currently
    /// queued source. If nothing has been queued yet, the position is zero.
    pub fn time_elapsed(&self) -> Duration {
        self.sink.get_pos()
    }

    /// Returns a boolean indicating whether the sink has finished playing.
    pub fn is_idle(&self) -> bool {
        self.sink.empty()
    }

    /// Attempts to play a song from the file at `path`.
    ///
    /// The file is opened and decoded; on success the current sink is stopped
    /// and the new source is appended. Any existing playback is interrupted.
    pub fn play_song(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        self.sink.stop();
        self.sink.append(source);

        if self.sink.is_paused() {
            self.sink.play();
        }

        Ok(())
    }

    /// Resumes playback on the sink if it is paused.
    pub fn resume(&mut self) {
        self.sink.play();
    }

    /// Pauses playback on the sink if it is playing.
    pub fn pause(&mut self) {
        self.sink.pause();
    }

    /// Stops playback on the sink and clears any queued audio.
    pub fn stop(&mut self) {
        self.sink.stop();
    }

    /// Sets the playback volume of the sink.
    ///
    /// `value` is expected to be in the range `0..=100` and is normalized to
    /// `0.0..=1.0` before being passed to rodio.
    pub fn set_volume(&self, value: u8) {
        let normed_value = f32::from(value) * 0.01;
        self.sink.set_volume(normed_value);
    }

    /// Attempts to seek to `time` in the current source.
    ///
    /// Returns an error if the underlying rodio sink does not support seeking
    /// or if the position is invalid for the current source.
    pub fn seek(&mut self, time: Duration) -> Result<()> {
        self.sink
            .try_seek(time)
            .map_err(|e| anyhow!("Failed to seek! {e}"))?;

        Ok(())
    }
}
