use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{CentralPanel, Context as eguiContext, Label, Sense, TextEdit, TopBottomPanel, Window},
};
use id3::{Tag, TagLike};
use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::read_dir;
use std::{collections::VecDeque, time::Duration};

use crate::{audio::AudioEngine, song::Song};

// Temporary state for song metadata editing
pub struct EditWindowBuffer {
    song_id: usize,
    title: String,
    artist: String,
}

impl EditWindowBuffer {
    // Later allow input in order to show current title etc.
    pub fn new(song: &Song) -> Self {
        Self {
            song_id: song.song_id,
            title: song.display_title.clone(),
            artist: song.display_artist.clone(),
        }
    }
}

pub struct GlowApp {
    songs: Vec<Song>,
    audio_engine: AudioEngine,
    error_queue: VecDeque<String>, // VecDeque for FIFO
    edit_window: Option<EditWindowBuffer>,
}

impl Default for GlowApp {
    fn default() -> Self {
        let mut error_queue = VecDeque::new();
        let songs = match load_songs("songs") {
            Ok(list) => list,
            Err(error) => {
                error_queue.push_back(format!("Failed to load songs: {}", error));
                Vec::new()
            }
        };

        Self {
            songs,
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
    pub fn get_song_mut(&mut self, song_id: usize) -> Option<&mut Song> {
        self.songs.iter_mut().find(|s| s.song_id == song_id)
    }

    fn render_ui(&mut self, ctx: &eguiContext) {
        TopBottomPanel::bottom("playback_control").show(ctx, |ui| {
            match self.audio_engine.is_playing {
                false => {
                    if ui.button("Play").clicked() {
                        self.audio_engine.resume();
                    }
                }
                true => {
                    if ui.button("Pause").clicked() {
                        self.audio_engine.pause();
                    }
                }
            }

            if ui.button("Stop").clicked() {
                self.audio_engine.stop();
            }
        });

        // Show takes the closure and creates a UI object to pass to it
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Songs");

            // self.songs is automatically borrowed by is_empty so no need to reference
            if self.songs.is_empty() {
                ui.label("No songs found...");
            } else {
                for song in &self.songs {
                    ui.horizontal(|ui| {
                        let title_label = ui.add(Label::new(&song.display_title).sense(Sense::click()));
                        let artist_label = ui.add(Label::new(&song.display_artist).sense(Sense::click()));
                                        // Using the add method allows the use of sense to make the label interactive
                        if title_label.clicked() {
                            // The contents of the if statement only runs if there is an error
                            if let Err(error) = self.audio_engine.play_song(&song.path) {
                                self.error_queue
                                    .push_back(format!("Playback failed: {}", error));
                            };
                        }

                        if artist_label.clicked() {
                            // TODO: filter songs by artist
                        }

                        // Right click menu for each song
                        title_label.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                self.edit_window = Some(EditWindowBuffer::new(song));
                            }
                        });
                    });
                }
            }
        });

        // If last error is Some, moves value into error, clearing last error
        if let Some(error) = self.error_queue.pop_front() {
            // Apparently move is better, rust still does it automatically though
            std::thread::spawn(move || {
                let _ = DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Error!")
                    .set_text(error)
                    .alert()
                    .show();
            });
        }

        // Cannot mutate self.edit_window if it has been moved to be used in the TextEdit.
        // To satisfy borrow checker use Option::take() which sets self.edit_window to none
        if let Some(buffer) = self.edit_window.take() {
            match self.render_edit_window(ctx, buffer) {
                Ok(Some(buffer)) => self.edit_window = Some(buffer),
                Ok(None) => (),
                Err(e) => self.error_queue.push_back(e),
            }
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn render_edit_window(
        &mut self,
        ctx: &eguiContext,
        mut buffer: EditWindowBuffer,
    ) -> Result<Option<EditWindowBuffer>, String> {
        let mut closed = false;
        let mut error: Option<String> = None;
        // Store textbox inputs in a buffer until saved, if closed early drop buffer, if saved only drop buffer once values have been passed to saving functions
        Window::new("Edit metadata").show(ctx, |ui| {
            ui.add(TextEdit::singleline(&mut buffer.title));
            ui.add(TextEdit::singleline(&mut buffer.artist));

            if ui.button("Close").clicked() {
                closed = true;
            }
            if ui.button("Save").clicked() {
                if let Some(song) = self.get_song_mut(buffer.song_id) {
                    song.display_title = buffer.title.clone();
                    song.display_artist = buffer.artist.clone();
                    if let Err(e) = save_metadata(song) {
                        error = Some(format!("Failed to write song to disk: {}", e));
                    }
                } else {
                    error = Some("Failed to get &mut song by id".to_string());
                }
                closed = true;
            }
        });

        if let Some(e) = error {
            return Err(e);
        }

        // If user has not closed window put the buffer back into self.edit_window to keep it alive
        if !closed { Ok(Some(buffer)) } else { Ok(None) }
    }
}

fn save_metadata(song: &Song) -> id3::Result<()> {
    let mut tag = Tag::read_from_path(&song.path)?;
    tag.set_title(&song.display_title);
    tag.set_artist(&song.display_artist);
    tag.write_to_path(&song.path, id3::Version::Id3v24)?;

    Ok(())
}

fn load_songs(target_folder: &str) -> std::io::Result<Vec<Song>> {
    let mut songs = Vec::new();

    let entries = read_dir(target_folder)?;
    // ? provides an unwrapped ReadDir or returns an error

    // Don't need to provide error handling for id as hitting usize max is impossible
    for (id, entry) in entries.flatten().enumerate() {
        // Flatten discards any failed files
        let path = entry.path();

        // Handles None case from extension() if the path is to a folder
        if let Some(ext) = path.extension() {
            // Windows allows capitals in extensions so ignore case
            if ext.eq_ignore_ascii_case("mp3") {
                // Only appends cleanly initialised songs
                if let Some(song) = Song::new(id, &path) {
                    songs.push(song);
                }
            }
        }
    }

    Ok(songs)
}
