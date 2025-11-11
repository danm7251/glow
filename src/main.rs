#![warn(clippy::all)] // All standard lints
#![warn(clippy::pedantic)] // Extra strict
//#![warn(clippy::nursery)] // Experimental
//#![warn(clippy::cargo)] // Cargo/project layout checks
//#![warn(clippy::restriction)]

mod app;
mod audio;
mod song;

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
