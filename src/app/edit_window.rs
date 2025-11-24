use anyhow::{Error, Result, anyhow};
use eframe::egui::{Button, Context as eguiContext, TextEdit, Window};

use crate::library::{Library, Song, write_song_metadata};

pub struct EditWindow {
    open: bool,
    song_id: usize,
    title: String,
    artist: String,
}

impl EditWindow {
    pub fn new(song: &Song) -> Self {
        Self {
            open: true,
            song_id: song.id(),
            title: song.title().to_owned(),
            artist: song.artist().to_owned(),
        }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, library: &mut Library, ctx: &eguiContext) -> Result<()> {
        let mut error: Option<Error> = None;

        Window::new("New Edit Window").show(ctx, |ui| {
            // --- Input fields ---
            ui.horizontal(|ui| {
                ui.label("Title");
                ui.add(TextEdit::singleline(&mut self.title));
            });

            ui.horizontal(|ui| {
                ui.label("Artist");
                ui.add(TextEdit::singleline(&mut self.artist));
            });

            // --- Action buttons ---
            ui.horizontal(|ui| {
                let mut enable_save = true;

                if self.title.is_empty() || self.artist.is_empty() {
                    enable_save = false;
                }

                if ui.add_enabled(enable_save, Button::new("Save")).clicked() {
                    if let Some(song) = library.song_mut(self.song_id) {
                        song.set_title(&self.title);
                        song.set_artist(&self.artist);

                        if let Err(e) = write_song_metadata(song) {
                            error = Some(e.context("Failed to write metadata"));
                        }
                    } else {
                        error = Some(anyhow!("Failed to get &mut song by id".to_string()));
                    }
                    self.open = false;
                }
                if ui.button("Close").clicked() {
                    self.open = false;
                }
            });
        });

        // Wait to return the error
        if let Some(e) = error {
            return Err(e);
        }

        Ok(())
    }
}
