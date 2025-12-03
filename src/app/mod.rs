use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{
        CentralPanel, Context as eguiContext, Frame, Label, Margin, ProgressBar, ScrollArea, Sense,
        SidePanel, Slider, TopBottomPanel,
    },
};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};

pub mod edit_window;

use crate::{app::edit_window::EditWindow, audio::AudioEngine, library::Library};

// Temporary hardcoded filepath, will be upgraded once permanent config is implemented
const SONG_FOLDER: &str = "test";
// Visual settings
const STD_MARGIN: i8 = 10;
const CONTROL_MARGIN: i8 = 5;
// Test flags
const ALLOW_NON_MP3: bool = false;

pub struct GlowApp {
    library: Library,
    audio_engine: AudioEngine,
    current_id: Option<usize>,
    error_queue: VecDeque<anyhow::Error>, // A VecDeque is used for FIFO action
    edit_window: Option<EditWindow>,
}

impl Default for GlowApp {
    fn default() -> Self {
        let mut error_queue = VecDeque::<anyhow::Error>::new();
        let library = match Library::new(SONG_FOLDER, ALLOW_NON_MP3) {
            Ok(lib) => lib,
            Err(e) => {
                error_queue.push_back(e);
                Library::empty(SONG_FOLDER, ALLOW_NON_MP3)
            }
        };

        Self {
            library,
            audio_engine: AudioEngine::new(),
            current_id: None,
            error_queue,
            edit_window: None,
        }
    }
}

impl eframeApp for GlowApp {
    fn update(&mut self, ctx: &eguiContext, _frame: &mut eframeFrame) {
        // TODO: [SOON] Consider how the main loop can facilitate clean communication between modules
        self.audio_engine.update();
        self.render_ui(ctx);
    }
}

impl GlowApp {
    /// Main GUI loop
    fn render_ui(&mut self, ctx: &eguiContext) {
        self.process_dropped_files(ctx);
        self.ui_playback_bar(ctx);
        self.ui_playlist_panel(ctx);
        self.ui_tracklist(ctx);
        self.ui_edit_window(ctx);
        self.process_errors();
        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn process_dropped_files(&mut self, ctx: &eguiContext) {
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in dropped_files {
                #[allow(clippy::collapsible_if, reason = "Readability")]
                if let Some(path) = file.path {
                    if let Err(e) = self.library.import_from_path(path.as_path()) {
                        self.error_queue.push_back(e);
                    }
                }
            }
        }
    }

    fn process_errors(&mut self) {
        if let Some(error) = self.error_queue.pop_front() {
            std::thread::spawn(move || {
                let _ = DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Error!")
                    .set_text(error.to_string())
                    .alert()
                    .show();
            });
        }
    }

    fn ui_tracklist(&mut self, ctx: &eguiContext) {
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(Margin::same(STD_MARGIN)))
            .show(ctx, |ui| {
                ui.heading("Songs");
                ui.separator();
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    // Reserves space for the scroll bar
                    ui.set_max_width(ui.available_width() - ui.spacing().scroll.bar_width);

                    if self.library.songs().is_empty() {
                        ui.label("No songs found...");
                    } else {
                        // TODO: [LATER] Need some kind of intermediate Vector to represent 'views' into the Library. Consider that the Library showcases data and the new layer will encapsulate actions
                        for song in self.library.songs() {
                            ui.horizontal_wrapped(|ui| {
                                // --- Labels ---
                                let title_label =
                                    ui.add(Label::new(song.title()).sense(Sense::click()));
                                ui.label("by");
                                let artist_label =
                                    ui.add(Label::new(song.artist()).sense(Sense::click()));
                                ui.label(song.formatted_duration());

                                // --- Actions ---
                                if title_label.clicked() {
                                    self.current_id = Some(song.id());

                                    if let Err(e) =
                                        self.audio_engine.play_song(song.path(), song.id())
                                    {
                                        self.error_queue.push_back(e.context("Playback Failed"));
                                    }
                                }
                                if artist_label.clicked() {
                                    // TODO: [LATER] Filter tracklist by artist
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
            });
    }

    // TODO: [LATER] Finalize the playback bar design
    fn ui_playback_bar(&mut self, ctx: &eguiContext) {
        TopBottomPanel::bottom("Playback Bar")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(CONTROL_MARGIN)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.add_enabled_ui(self.current_id.is_some(), |ui| {
                                // FIXME: [LATER] Consider this side of the playbacks role in the AudioEngine::play_song bug
                                #[allow(clippy::collapsible_else_if, reason = "Readability")]
                                if self.audio_engine.is_playing() {
                                    if ui.button("Pause").clicked() {
                                        self.audio_engine.pause();
                                    }
                                } else {
                                    if ui.button("Play").clicked() {
                                        self.audio_engine.resume();
                                    }
                                }

                                if ui.button("Stop").clicked() {
                                    self.current_id = None;
                                    self.audio_engine.stop();
                                }
                            });
                        });
                        // TODO: [SOON] Consider another layer of abstraction
                        ui.horizontal_centered(|ui| {
                            let volume =
                                ui.add(Slider::new(self.audio_engine.mut_volume(), 0..=100));
                            if volume.changed() {
                                self.audio_engine.set_volume();
                            }
                        });
                    });

                    // TODO: [LATER] Current song should be dropped when song ends
                    // TODO: [LATER] Style the seek bar
                    // TODO: [SOON] Review and revise seek logic according to pending arch. review
                    ui.vertical(|ui| {
                        if let Some(id) = self.current_id
                            && let Some(current_song) = self.library.get_song(id)
                        {
                            ui.horizontal(|ui| {
                                ui.label(current_song.title());
                                ui.label("by");
                                ui.label(current_song.artist());
                            });

                            match current_song.duration() {
                                Some(duration) => {
                                    let mut current_time =
                                        self.audio_engine.time_elapsed().as_secs_f64();

                                    let total_time = duration.as_secs_f64();

                                    println!(
                                        "Time elapsed: {current_time}\nTotal time: {total_time}"
                                    );

                                    let seek_bar =
                                        ui.add(Slider::new(&mut current_time, 0.0..=total_time));
                                    if seek_bar.changed()
                                        && let Err(e) = self
                                            .audio_engine
                                            .seek(Duration::from_secs_f64(current_time))
                                    {
                                        self.error_queue.push_back(e);
                                    }
                                }
                                None => {
                                    ui.add_enabled(false, ProgressBar::new(1.0));
                                }
                            }
                        }
                    });
                });
            });
    }

    #[allow(clippy::unused_self)] // This will require &mut later
    fn ui_playlist_panel(&mut self, ctx: &eguiContext) {
        SidePanel::left("Playlists")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(STD_MARGIN)))
            .show(ctx, |ui| {
                ui.heading("Playlists");
                ui.separator();
            });
    }

    fn ui_edit_window(&mut self, ctx: &eguiContext) {
        if let Some(edit_window) = &mut self.edit_window {
            match edit_window.show(&mut self.library, ctx) {
                Ok(()) => (),
                Err(e) => self.error_queue.push_back(e),
            }

            if !edit_window.open() {
                self.edit_window = None;
            }
        }
    }
}
