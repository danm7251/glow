use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::path::PathBuf;
use std::fs::File;

// May need stream_handle later
// Error handling needs an upgrade
pub struct AudioEngine {
    stream_handle: OutputStream,
    sink: Sink,
}

impl AudioEngine {
    pub fn new() -> Self {
        let stream_handle = OutputStreamBuilder::open_default_stream().expect("Open default audio stream");
        let sink = Sink::connect_new(&stream_handle.mixer());

        Self {
            stream_handle,
            sink,
        }
    }

    pub fn play_song(&mut self, path: &PathBuf) {
        let file = File::open(path).unwrap();

        let source = Decoder::try_from(file).unwrap();

        self.sink.stop();
        self.sink.append(source);
    }
}