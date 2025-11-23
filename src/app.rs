use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{
        Button, CentralPanel, Context as eguiContext, DroppedFile, Label, Sense, TextEdit,
        TopBottomPanel, Window,
    },
};
use id3::{Tag, TagLike};
use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::{copy, read_dir};
use std::path::PathBuf;
use std::{collections::VecDeque, time::Duration};

use crate::{audio::AudioEngine, library::Library, song::Song};

// Temporary hardcoded filepath, will be upgraded once permanent config is implemented
const SONG_FOLDER: &str = "test";
// Test flags
const ALLOW_NON_MP3: bool = false;

// Temporary state for song metadata editing
pub struct EditWindowBuffer {
    song_id: usize,
    title: String,
    artist: String,
}

impl EditWindowBuffer {
    pub fn new(song: &Song) -> Self {
        Self {
            song_id: song.id,
            title: song.display_title.clone(),
            artist: song.display_artist.clone(),
        }
    }
}

pub struct GlowApp {
    library: Library, // To replace songs when complete
    songs: Vec<Song>,
    audio_engine: AudioEngine,
    // A VecDeque is used for FIFO action
    error_queue: VecDeque<String>,
    edit_window: Option<EditWindowBuffer>,
}

impl Default for GlowApp {
    fn default() -> Self {
        let mut error_queue = VecDeque::new();
        let songs = match load_songs(SONG_FOLDER) {
            Ok(list) => list,
            Err(error) => {
                error_queue.push_back(format!("Failed to load songs: {error}"));
                Vec::new()
            }
        };

        Self {
            // Logic to fallback to empty Library to replace this
            library: Library::empty(SONG_FOLDER, ALLOW_NON_MP3),
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
    /// Returns a mutable reference to a Song struct from it's ID
    fn get_song_mut(&mut self, song_id: usize) -> Option<&mut Song> {
        self.songs.iter_mut().find(|s| s.id == song_id)
    }

    /// Returns a result after attempting to copy a ``DroppedFile``
    /// into a desired folder, construct a song for each one and push it
    /// to self.songs
    fn copy_and_add_file(&mut self, file: &DroppedFile, target_folder: &str) -> Result<(), String> {
        // Path uses as_ref() rather than & to obtain Option<&PathBuf> over &Option<PathBuf>
        // Ok_or() transforms an Option to a Result to simplify error propogation
        let path = file.path.as_ref().ok_or("Failed to get path from file")?;

        if !path
            .extension()
            .is_some_and(|p| p.eq_ignore_ascii_case("mp3") || ALLOW_NON_MP3)
        {
            return Err(format!("Skipped non-mp3 file: {}", path.to_string_lossy()));
        }

        let filename = path
            .file_name()
            .ok_or("Failed to get filename from path")?
            .to_string_lossy();

        let target: PathBuf = [target_folder, &filename].iter().collect();

        // Map_err() is used to return the desired error type simply
        copy(path, &target).map_err(|e| format!("Failed to copy: {e:?}"))?;

        let song = Song::new(self.songs.len(), &target).ok_or("Failed to create song")?;
        self.songs.push(song);

        Ok(())
    }

    /// Main GUI loop
    fn render_ui(&mut self, ctx: &eguiContext) {
        // --- WIP --- (Ready for testing)
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in dropped_files {
                self.copy_and_add_file(&file, SONG_FOLDER)
                    .unwrap_or_else(|e| self.error_queue.push_back(e));
            }
        }
        // --- WIP ---

        TopBottomPanel::bottom("playback_control").show(ctx, |ui| {
            ui.horizontal(|ui| {
                #[allow(clippy::collapsible_else_if)] // Readability
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
                            if let Err(e) = self.audio_engine.play_song(&song.path) {
                                self.error_queue.push_back(format!("Playback failed: {e}"));
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
                let mut enable_save = true;

                if buffer.title.is_empty() || buffer.artist.is_empty() {
                    enable_save = false;
                }

                if ui.add_enabled(enable_save, Button::new("Save")).clicked() {
                    if let Some(song) = self.get_song_mut(buffer.song_id) {
                        song.display_title.clone_from(&buffer.title);
                        song.display_artist.clone_from(&buffer.artist);

                        if let Err(e) = save_metadata(song) {
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

// Generates a Vector of Song structs from a target folder containing .mp3 files
fn load_songs(target_folder: &str) -> std::io::Result<Vec<Song>> {
    let mut songs = Vec::new();
    let entries = read_dir(target_folder)?;

    // Uses flatten to discard any failed entries
    for entry in entries.flatten() {
        let path = entry.path();

        // Uses conditional pattern matching to discard folder paths with no extension
        // Case is ignored as file extensions are not case sensitive in windows
        if path
            .extension()
            .is_some_and(|p| p.eq_ignore_ascii_case("mp3") || ALLOW_NON_MP3)
        {
            // Conditional pattern matching in case any malformed Song structs return as None
            if let Some(song) = Song::new(songs.len(), &path) {
                songs.push(song);
            }
        }
    }

    Ok(songs)
}
