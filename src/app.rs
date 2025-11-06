use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{CentralPanel, Context as eguiContext, Label, Sense, TextEdit, TopBottomPanel, Window},
};
use id3::{Tag, TagLike};
use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::{copy, read_dir};
use std::{collections::VecDeque, time::Duration};
use std::path::PathBuf;

use crate::{audio::AudioEngine, song::Song};

// Temporary state for song metadata editing
pub struct EditWindowBuffer {
    song_id: usize,
    title: String,
    artist: String,
}

impl EditWindowBuffer {
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
    // A VecDeque is used for FIFO action
    error_queue: VecDeque<String>,
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
    /// Returns a mutable reference to a Song struct from it's ID (may not need to be public)
    pub fn get_song_mut(&mut self, song_id: usize) -> Option<&mut Song> {
        self.songs.iter_mut().find(|s| s.song_id == song_id)
    }

    fn render_ui(&mut self, ctx: &eguiContext) {
        // --- WIP --- (Works but is missing a display reload)
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for file in dropped_files {
                println!("Dropped in! {:?}", file);
                if let Some(path) = file.path {
                    if let Some(filename) = path.file_name() {
                        let target: PathBuf = ["songs", &filename.to_string_lossy()].iter().collect();
                        if let Err(e) = copy(path, target) {
                            self.error_queue.push_back(format!("Failed to copy: {:?}", e));
                        }
                    } else {
                        self.error_queue.push_back("Failed to get filename".to_string());
                    }
                } else {
                    self.error_queue.push_back("Failed to get path".to_string());
                }
            }
        }
        // --- WIP ---

        TopBottomPanel::bottom("playback_control").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Songs");

            if self.songs.is_empty() {
                ui.label("No songs found...");
            } else {
                for song in &self.songs {
                    ui.horizontal(|ui| {
                        // --- Labels ---
                        let title_label =
                            ui.add(Label::new(&song.display_title).sense(Sense::click()));
                        ui.label("by");
                        let artist_label =
                            ui.add(Label::new(&song.display_artist).sense(Sense::click()));

                        // --- Actions ---
                        if title_label.clicked() {
                            if let Err(error) = self.audio_engine.play_song(&song.path) {
                                self.error_queue
                                    .push_back(format!("Playback failed: {}", error));
                            };
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

    // Renders an edit window for editing a Song's metadata
    fn render_edit_window(
        &mut self,
        ctx: &eguiContext,
        mut buffer: EditWindowBuffer,
    ) -> Result<Option<EditWindowBuffer>, String> {
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
                if ui.button("Save").clicked() {
                    if let Some(song) = self.get_song_mut(buffer.song_id) {
                        song.display_title = buffer.title.clone();
                        song.display_artist = buffer.artist.clone();

                        if let Err(e) = save_metadata(song) {
                            error = Some(format!("Failed to save metadata: {}", e));
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

        if let Some(e) = error {
            return Err(e);
        }
        // As calling code takes the buffer it needs to be returned if we want it to persist
        if !closed { Ok(Some(buffer)) } else { Ok(None) }
    }
}

// Writes a Song's id3 tags to storage
fn save_metadata(song: &Song) -> id3::Result<()> {
    let mut tag = Tag::read_from_path(&song.path)?;
    tag.set_title(&song.display_title);
    tag.set_artist(&song.display_artist);
    tag.write_to_path(&song.path, id3::Version::Id3v24)?;

    Ok(())
}

fn reload_songs(target_folder: &str) {
    // TODO
}

// Generates a Vector of Song structs from a target folder containing .mp3 files
fn load_songs(target_folder: &str) -> std::io::Result<Vec<Song>> {
    let mut songs = Vec::new();
    let entries = read_dir(target_folder)?;

    // Uses flatten to discard any failed entries and enumerate to allow the for loop
    for (id, entry) in entries.flatten().enumerate() {
        let path = entry.path();

        // Uses conditional pattern matching to discard folder paths with no extension
        if let Some(ext) = path.extension() {
            // Case is ignored as file extensions are not case sensitive in windows
            if ext.eq_ignore_ascii_case("mp3") {
                // Conditional pattern matching in case any malformed Song structs return as None
                if let Some(song) = Song::new(id, &path) {
                    songs.push(song);
                }
            }
        }
    }

    Ok(songs)
}
