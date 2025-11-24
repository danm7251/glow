use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{CentralPanel, Context as eguiContext, Label, Sense, TopBottomPanel},
};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};

pub mod edit_window;

use crate::{app::edit_window::EditWindow, audio::AudioEngine, library::Library};

// Temporary hardcoded filepath, will be upgraded once permanent config is implemented
const SONG_FOLDER: &str = "test";
// Test flags
const ALLOW_NON_MP3: bool = false;

pub struct GlowApp {
    library: Library,
    audio_engine: AudioEngine,
    error_queue: VecDeque<String>, // A VecDeque is used for FIFO action
    edit_window: Option<EditWindow>,
}

impl Default for GlowApp {
    fn default() -> Self {
        let mut error_queue = VecDeque::new();
        let library = match Library::new(SONG_FOLDER, ALLOW_NON_MP3) {
            Ok(lib) => lib,
            Err(e) => {
                error_queue.push_back(e.to_string());
                Library::empty(SONG_FOLDER, ALLOW_NON_MP3)
            }
        };

        Self {
            library,
            audio_engine: AudioEngine::new(),
            error_queue,
            edit_window: None,
        }
    }
}

impl eframeApp for GlowApp {
    fn update(&mut self, ctx: &eguiContext, _frame: &mut eframeFrame) {
        self.audio_engine.update();
        self.render_ui(ctx);
    }
}

impl GlowApp {
    /// Main GUI loop
    fn render_ui(&mut self, ctx: &eguiContext) {
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in dropped_files {
                #[allow(clippy::collapsible_if, reason = "Readability")]
                if let Some(path) = file.path {
                    if let Err(e) = self.library.import_from_path(path.as_path()) {
                        self.error_queue.push_back(e.to_string());
                    }
                }
            }
        }

        TopBottomPanel::bottom("playback_control").show(ctx, |ui| {
            // TODO: Fix bug when pausing and using label to play
            ui.horizontal(|ui| {
                #[allow(clippy::collapsible_else_if, reason = "Readability")]
                if self.audio_engine.is_playing {
                    if ui.button("Pause").clicked() {
                        self.audio_engine.pause();
                    }
                } else {
                    if ui.button("Play").clicked() {
                        self.audio_engine.resume();
                    }
                }

                if ui.button("Stop").clicked() {
                    self.audio_engine.stop();
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Songs");

            if self.library.songs().is_empty() {
                ui.label("No songs found...");
            } else {
                for song in self.library.songs() {
                    ui.horizontal(|ui| {
                        // --- Labels ---
                        let title_label = ui.add(Label::new(song.title()).sense(Sense::click()));
                        ui.label("by");
                        let artist_label = ui.add(Label::new(song.artist()).sense(Sense::click()));

                        // --- Actions ---
                        #[allow(clippy::collapsible_if, reason = "Readability")]
                        if title_label.clicked() {
                            if let Err(e) = self.audio_engine.play_song(song.path()) {
                                self.error_queue.push_back(format!("Playback failed: {e}"));
                            }
                        }
                        if artist_label.clicked() {
                            // TODO: filter songs by artist
                        }
                        title_label.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                self.edit_window = Some(EditWindow::new(song));
                            }
                        });
                    });
                }
            }
        });

        // --- Error messages ---
        // TODO: Centralise string conversion
        if let Some(error) = self.error_queue.pop_front() {
            std::thread::spawn(move || {
                let _ = DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Error!")
                    .set_text(error)
                    .alert()
                    .show();
            });
        }

        // --- Metadata editing window ---
        if let Some(edit_window) = &mut self.edit_window {
            match edit_window.show(&mut self.library, ctx) {
                Ok(()) => (),
                Err(e) => self.error_queue.push_back(e.to_string()),
            }

            if !edit_window.open() {
                self.edit_window = None;
            }
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
