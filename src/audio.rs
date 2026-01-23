//! Low-level control over audio playback.
//!
//! This module defines the [`AudioBackend`] trait and a concrete implementation,
//! [`AudioEngine`], a thin wrapper around the `rodio` crate.
//!
//! [`AudioEngine`] owns the [`OutputStream`] and [`Sink`]. Audio playback in `rodio` stops if the [`OutputStream`]
//! is dropped, so it must live for the duration of playback. The [`Sink`] manages audio playback.

use anyhow::{Context, Result, anyhow};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source, cpal::FromSample};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::time::Duration;

const NO_TRACK: usize = usize::MAX;

// TODO: [SOON] Update documentation to reflect the difference in AudioBackend and Audioengines contracts.

/// Interface for audio playback
///
/// This trait defines the minimum capabilities required by the player.
pub trait AudioBackend {
    // Commands

    /// Start playback of the song at `path`, replacing any existing playback
    fn play_song(&mut self, path: &Path, id: usize) -> Result<()>;

    /// Pause playback
    fn pause(&mut self);

    /// Resume playback
    fn resume(&mut self);

    /// Stop playback and clear any queued audio
    fn stop(&mut self);

    /// Skip to a position in the current track
    fn seek(&mut self, time: Duration) -> Result<()>;

    // Configuration

    /// Set the playback volume.
    fn set_volume(&self, value: u8);

    // State queries

    /// Returns true if no audio is currently queued or playing.
    fn is_idle(&self) -> bool;

    /// Returns the ID of the last activated track.
    fn active_playback_id(&self) -> usize;

    /// Returns the elapsed playback time since the start of the current track.
    fn time_elapsed(&self) -> Duration;
}

/// Handles low-level audio output using rodio.
///
/// An `AudioEngine` owns both the output stream and a single sink. It is
/// responsible for keeping the output stream alive and issuing playback
/// commands for the current audio source.
pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
    active_playback_id: Arc<AtomicUsize>,
}

impl AudioBackend for AudioEngine {
    // TODO: [URGENT] Update/Review documentation
    /// Opens, decodes and starts playing the file at `path`.
    ///
    /// Any existing playback is stopped and replaced.
    /// Returns an error if the file cannot be opened or decoded.
    fn play_song(&mut self, path: &Path, id: usize) -> Result<()> {
        let file = File::open(path)?;
        let inner_source = Decoder::try_from(file)?;
        let source = SourceTracker::new(inner_source, id, self.active_playback_id.clone());

        self.sink.stop();
        self.sink.append(source);

        if self.sink.is_paused() {
            self.sink.play();
        }

        Ok(())
    }

    /// Pauses playback on the sink if it is playing.
    fn pause(&mut self) {
        self.sink.pause();
    }

    /// Resumes playback on the sink if it is paused.
    fn resume(&mut self) {
        self.sink.play();
    }

    /// Stops playback on the sink and clears any queued audio.
    fn stop(&mut self) {
        self.sink.stop();
    }

    /// Seeks audio at `time`
    ///
    /// Returns an error if the source does not support seeking or
    /// when seeking beyond the end of a source with an unknown duration.
    ///
    /// If the duration of the source is known,
    /// seeking beyond it will clamp to the total duration
    fn seek(&mut self, time: Duration) -> Result<()> {
        self.sink
            .try_seek(time)
            .map_err(|e| anyhow!("Failed to seek! {e}"))?;

        Ok(())
    }

    /// Sets the playback volume of the sink.
    ///
    /// `value` is expected to be in the range `0..=100` and is normalized to
    /// `0.0..=1.0` before being passed to rodio.
    ///
    /// This uses interior mutability, so exclusive ownership is not required.
    fn set_volume(&self, value: u8) {
        let normed_value = f32::from(value) * 0.01;
        self.sink.set_volume(normed_value);
    }

    /// Returns the ID of the most recently active audio track
    fn active_playback_id(&self) -> usize {
        self.active_playback_id.load(SeqCst)
    }

    /// Returns a boolean indicating whether the sink has finished playing.
    fn is_idle(&self) -> bool {
        self.sink.empty()
    }

    /// Returns the current playback position of the sink.
    ///
    /// The returned [`Duration`] is measured from the start of the currently
    /// queued source. If nothing has been queued yet, the position is zero.
    fn time_elapsed(&self) -> Duration {
        self.sink.get_pos()
    }
}

impl AudioEngine {
    /// Creates a new `AudioEngine`.
    ///
    /// This opens the default output device and constructs a sink that can be
    /// controlled via the other methods. Returns an error if no suitable
    /// output device or stream can be created.
    pub fn new() -> Result<Self> {
        let mut stream = OutputStreamBuilder::open_default_stream()
            .context("Failed to construct OutputStream in AudioEngine::new()")?;
        stream.log_on_drop(false);

        let sink = Sink::connect_new(stream.mixer());

        Ok(Self {
            _stream: stream,
            sink,
            active_playback_id: Arc::new(AtomicUsize::new(NO_TRACK)),
        })
    }
}

struct SourceTracker<S> {
    source: S,
    id: usize,
    id_sync: Arc<AtomicUsize>,
    has_started: bool,
}

impl<S> SourceTracker<S> {
    pub fn new(source: S, id: usize, id_sync: Arc<AtomicUsize>) -> Self {
        Self {
            source,
            id,
            id_sync,
            has_started: false,
        }
    }
}

impl<S: Source> Iterator for SourceTracker<S>
where
    S::Item: FromSample<S::Item>,
{
    type Item = S::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_started {
            self.id_sync.store(self.id, SeqCst);
            self.has_started = true;
        }

        self.source.next()
    }
}

impl<S: Source> Source for SourceTracker<S> {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }

    #[inline]
    fn channels(&self) -> rodio::ChannelCount {
        self.source.channels()
    }

    #[inline]
    fn sample_rate(&self) -> rodio::SampleRate {
        self.source.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> std::result::Result<(), rodio::source::SeekError> {
        self.source.try_seek(pos)
    }
}
