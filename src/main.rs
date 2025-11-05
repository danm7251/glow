mod app;
mod audio;
mod song;

use eframe::{
    NativeOptions as eframeNativeOptions, Result as eframeResult, run_native as eframe_run_native,
};

fn main() -> eframeResult {
    let native_options = eframeNativeOptions {
        ..Default::default()
    };

    eframe_run_native(
        "glow",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::GlowApp::default()))),
    )
}
