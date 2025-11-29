use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::path::Path;

// When app is closed rodio prints to console about the outputstream being dropped
// TODO: Review code, add minimal Doc
pub struct AudioEngine {
    // OutputStream must be kept alive
    _stream: OutputStream,
    sink: Sink,

    pub is_playing: bool,
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
            volume: 100,
        }
    }

    // TODO: play_song() does not work if player is paused
    pub fn play_song(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        self.sink.stop();
        self.sink.append(source);
        self.is_playing = true;

        Ok(())
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.is_playing = false;
    }

    pub fn resume(&mut self) {
        self.sink.play();
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.is_playing = false;
    }

    pub fn update(&mut self) {
        if self.is_playing && self.sink.empty() {
            self.is_playing = false;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn mut_volume(&mut self) -> &mut u8 {
        &mut self.volume
    }

    pub fn set_volume(&self) {
        let value = f32::from(self.volume) * 0.01;
        self.sink.set_volume(value);
    }
}
