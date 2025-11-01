use eframe::{egui::{CentralPanel, Context as eguiContext, Label, Sense, TextEdit, TopBottomPanel, Window}, App as eframeApp, Frame as eframeFrame};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};
use std::fs::read_dir;

use crate::{audio::AudioEngine, song::Song};

pub struct EditWindowBuffer {
    song_id: usize,
    title: String,
}

impl EditWindowBuffer {
    // Later allow input in order to show current title etc.
    pub fn new(song_id: usize) -> Self {
        Self {
            song_id,
            title: String::new(),
        }
    }
}

pub struct GlowApp {
    songs: Vec<Song>,
    audio_engine: AudioEngine,
    error_queue: VecDeque<String>, // VecDeque for FIFO
    edit_window: Option<EditWindowBuffer>
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
                // Changed to &mut self to set title --CHECK--
                for song in &mut self.songs {

                    // Using the add method allows the use of sense to make the label interactive
                    // Some depth to display() to explore
                    let label = ui.add(Label::new(&song.display_title).sense(Sense::click()));

                    if label.clicked() {
                        // The contents of the if statement only runs if there is an error
                        if let Err(error) = self.audio_engine.play_song(&song.path) {
                            self.error_queue.push_back(format!("Playback failed: {}", error));
                        };
                    }

                    // Right click menu for each song
                    label.context_menu(|ui| {
                        if ui.button("Edit").clicked() {
                            // Pass song_id as value as a reference would outlive &mut songs
                            self.edit_window = Some(EditWindowBuffer::new(song.song_id));
                        }
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
        if let Some(mut buffer) = self.edit_window.take() {
            let mut closed = false;
            // Store textbox inputs in a buffer until saved, if closed early drop buffer, if saved only drop buffer once values have been passed to saving functions
            Window::new("Edit metadata").show(ctx, |ui| {
                ui.add(TextEdit::singleline(&mut buffer.title));

                if ui.button("Close").clicked() {
                    closed = true;
                }
                if ui.button("Save").clicked() {
                    if let Some(song) = self.get_song_mut(buffer.song_id) {
                        println!("{:?}", song.display_title);
                    } else {
                        self.error_queue.push_back("(EditWindow) Song not found".to_string());
                    }
                    closed = true;
                }

                // If user has not closed window put the buffer back into self.edit_window to keep it alive
                if !closed {
                    self.edit_window = Some(buffer);
                }
            });
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }
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