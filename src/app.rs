use eframe::{App as eframeApp, egui::{Context as eguiContext, CentralPanel, Sense, Label}, Frame as eframeFrame};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};
use std::path::PathBuf;
use std::fs::read_dir;

use crate::audio::AudioEngine;

pub struct GlowApp {
    songs: Vec<PathBuf>,
    audio_engine: AudioEngine,
    error_queue: VecDeque<String>, // VecDeque for FIFO
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
        }
    }
}

impl eframeApp for GlowApp {
    fn update(&mut self, ctx: &eguiContext, _frame: &mut eframeFrame) {
        self.render_ui(ctx);
    }
}

impl GlowApp {
    fn render_ui(&mut self, ctx: &eguiContext) {

        // Show takes the closure and creates a UI object to pass to it
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Songs");

            // self.songs is automatically borrowed by is_empty so no need to reference
            if self.songs.is_empty() {
                ui.label("No songs found...");
            } else {
                for song in &self.songs {
                    // Extract UI information from song
                    // Add error handling
                    let song_title = song.file_name().expect("Failed to get file name of song");
                    // Using the add method allows the use of sense to make the label interactive
                    // Some depth to display() to explore
                    let label = ui.add(Label::new(song_title.display().to_string()).sense(Sense::click()));

                    if label.clicked() {
                        // The contents of the if statement only runs if there is an error
                        if let Err(error) = self.audio_engine.play_song(song) {
                            self.error_queue.push_back(format!("Playback failed: {}", error));
                        };
                    }

                    // Right click menu for each song
                    label.context_menu(|ui| {
                        if ui.button("Edit").clicked() {
                            println!("Edit!");
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

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}


fn load_songs(target_folder: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut songs = Vec::new();

    let entries = read_dir(target_folder)?;
    // ? provides an unwrapped ReadDir or returns an error

    for entry in entries.flatten() {
        // Flatten discards any failed files
        let path = entry.path();

        // Handles None case from extension() if the path is to a folder
        if let Some(ext) = path.extension() {
            // Windows allows capitals in extensions so ignore case
            if ext.eq_ignore_ascii_case("mp3") {
                songs.push(path);
            }
        }
    }

    Ok(songs)
}