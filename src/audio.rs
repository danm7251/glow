use rodio::decoder::DecoderError;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::path::PathBuf;
use std::fs::File;

const TEST: bool = false;

// May need stream_handle later
// When app is closed rodio prints to console about the outputstream being dropped
pub struct AudioEngine {
    pub is_playing: bool,
    stream_handle: OutputStream,
    sink: Sink,
}

impl AudioEngine {
    pub fn new() -> Self {
        let stream_handle = OutputStreamBuilder::open_default_stream().expect("Open default audio stream");
        let sink = Sink::connect_new(&stream_handle.mixer());

        Self {
            is_playing: false,
            stream_handle,
            sink,
        }
    }

    // Learn more about the Box type for error handling
    // Returns Box error type as DecoderError and FileError are incompatible
    pub fn play_song(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {

        // Opening and decoding songs each time could impact performance, try caching?

        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        self.sink.stop();
        self.sink.append(source);
        self.is_playing = true;

        if TEST {
            Err(Box::new(DecoderError::UnrecognizedFormat))
        } else {
            Ok(())
        }
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
}