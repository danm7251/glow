use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{
        Button, CentralPanel, Context as eguiContext, Label, Sense, TextEdit, TopBottomPanel,
        Window,
    },
};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};

use crate::{
    audio::AudioEngine,
    library::{Library, Song, write_song_metadata},
};

// Temporary hardcoded filepath, will be upgraded once permanent config is implemented
const SONG_FOLDER: &str = "test";
// Test flags
const ALLOW_NON_MP3: bool = false;

// Temporary state for song metadata editing
struct EditWindowBuffer {
    // TODO: Seperate and export as module
    song_id: usize,
    title: String,
    artist: String,
}

impl EditWindowBuffer {
    fn new(song: &Song) -> Self {
        Self {
            song_id: song.id,
            title: song.display_title.clone(),
            artist: song.display_artist.clone(),
        }
    }
}

pub struct GlowApp {
    library: Library,
    audio_engine: AudioEngine,
    error_queue: VecDeque<String>, // A VecDeque is used for FIFO action
    edit_window: Option<EditWindowBuffer>,
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
                        let title_label =
                            ui.add(Label::new(&song.display_title).sense(Sense::click()));
                        ui.label("by");
                        let artist_label =
                            ui.add(Label::new(&song.display_artist).sense(Sense::click()));

                        // --- Actions ---
                        #[allow(clippy::collapsible_if, reason = "Readability")]
                        if title_label.clicked() {
                            if let Err(e) = self.audio_engine.play_song(&song.path) {
                                self.error_queue.push_back(format!("Playback failed: {e}"));
                            }
                        }
                        if artist_label.clicked() {
                            // TODO: filter songs by artist
                        }
                        title_label.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                self.edit_window = Some(EditWindowBuffer::new(song));
                            }
                        });
                    });
                }
            }
        });

        // --- Error messages ---
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
        // If a buffer exists it is taken and rendered before being returned if the window has not been closed
        if let Some(buffer) = self.edit_window.take() {
            match self.render_edit_window(ctx, buffer) {
                Ok(Some(buffer)) => self.edit_window = Some(buffer),
                Ok(None) => (),
                Err(e) => self.error_queue.push_back(e),
            }
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    /// Renders an edit window by taking an `EditWindowBuffer` and returns it for re-use if the window is open
    fn render_edit_window(
        &mut self,
        ctx: &eguiContext,
        mut buffer: EditWindowBuffer,
    ) -> Result<Option<EditWindowBuffer>, String> {
        // TODO: Refactor to use anyhow
        let mut closed = false;
        let mut error: Option<String> = None;

        Window::new("Edit metadata").show(ctx, |ui| {
            // --- Input fields ---
            ui.horizontal(|ui| {
                ui.label("Title");
                ui.add(TextEdit::singleline(&mut buffer.title));
            });

            ui.horizontal(|ui| {
                ui.label("Artist");
                ui.add(TextEdit::singleline(&mut buffer.artist));
            });

            // --- Action buttons ---
            ui.horizontal(|ui| {
                let mut enable_save = true;

                if buffer.title.is_empty() || buffer.artist.is_empty() {
                    enable_save = false;
                }

                if ui.add_enabled(enable_save, Button::new("Save")).clicked() {
                    if let Some(song) = self.library.song_mut(buffer.song_id) {
                        song.display_title.clone_from(&buffer.title);
                        song.display_artist.clone_from(&buffer.artist);

                        if let Err(e) = write_song_metadata(song) {
                            error = Some(format!("Failed to save metadata: {e}"));
                        }
                    } else {
                        error = Some("Failed to get &mut song by id".to_string());
                    }
                    closed = true;
                }
                if ui.button("Close").clicked() {
                    closed = true;
                }
            });
        });

        // Wait to return the error
        if let Some(e) = error {
            return Err(e);
        }
        // As calling code takes the buffer it needs to be returned if we want it to persist
        if closed { Ok(None) } else { Ok(Some(buffer)) }
    }
}
