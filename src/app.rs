use eframe::{App as eframeApp, egui::{Context as eguiContext, Sense, Label}, Frame as eframeFrame, egui::CentralPanel};
use std::time::Duration;
use std::path::PathBuf;
use std::fs::read_dir;

use crate::audio::AudioEngine;

/*
Todo list:
    Functionality:
        - Implement scrolling for large song lists
        - Make the song labels prettier, includes investigating title and artist fields
        - Allow metadata editing or at least file renaming
    Quality:
        - Check for any variables that should be borrowed
*/

pub struct GlowApp {
    songs: Vec<PathBuf>,
    audio_engine: AudioEngine,
}

impl Default for GlowApp {
    fn default() -> Self {
        Self {
            songs: load_songs("songs").unwrap_or_default(),
            audio_engine: AudioEngine::new(),
            // Creates an empty vector if load_songs returns an error
            // Consider storing the result later to display the error in app
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
                    let song_title = song.file_name().expect("Failed to get file name of song"); 
                    // Using the add method allows the use of sense to make the label interactive
                    // Some depth to display() to explore
                    let label = ui.add(Label::new(song_title.display().to_string()).sense(Sense::click()));

                    if label.clicked() {
                        self.audio_engine.play_song(song);
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