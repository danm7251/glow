#![warn(clippy::all)] // All standard lints
#![warn(clippy::pedantic)] // Extra strict
//#![warn(clippy::nursery)] // Experimental
//#![warn(clippy::cargo)] // Cargo/project layout checks

mod app;
mod audio;
mod library;
mod player;

use eframe::{
    NativeOptions as eframeNativeOptions, Result as eframeResult, run_native as eframe_run_native,
};

fn main() -> eframeResult {
    setup_tracing();

    let native_options = eframeNativeOptions::default();

    eframe_run_native(
        "glow",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::GlowApp::default()))),
    )
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("glow=debug")
        .with_thread_names(true)
        .pretty()
        .init();

    tracing::info!("Tracing initialised...");
}
