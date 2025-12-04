#![warn(clippy::all)] // All standard lints
#![warn(clippy::pedantic)] // Extra strict
//#![warn(clippy::nursery)] // Experimental
//#![warn(clippy::cargo)] // Cargo/project layout checks

// TODO: [URGENT] A high level architecture review. Consider the benefits of separating playback state from GUI state. Consider how the AudioEngine, Library and GlowApp interact. It is acceptable for the GUI to poll the internals thats just the design paradigm that egui provides developers with.

mod app;
mod audio;
mod library;
mod player;

use eframe::{
    NativeOptions as eframeNativeOptions, Result as eframeResult, run_native as eframe_run_native,
};

fn main() -> eframeResult {
    let native_options = eframeNativeOptions::default();

    eframe_run_native(
        "glow",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::GlowApp::default()))),
    )
}
