use eframe::{
    App as eframeApp, Frame as eframeFrame,
    egui::{
        CentralPanel, Context as eguiContext, Frame, Label, Margin, ProgressBar, ScrollArea, Sense,
        SidePanel, Slider, TopBottomPanel,
    },
};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{collections::VecDeque, time::Duration};
use tracing::error;
pub mod edit_window;

use crate::{app::edit_window::EditWindow, audio::AudioEngine, library::Library, player::Player};

// Temporary hardcoded filepath, will be upgraded once permanent config is implemented
const SONG_FOLDER: &str = "test";
// Visual settings
const STD_MARGIN: i8 = 10;
const CONTROL_MARGIN: i8 = 5;
// Test flags
const ALLOW_NON_MP3: bool = false;

pub struct GlowApp {
    library: Library,
    player: Player<AudioEngine>,
    error_queue: VecDeque<anyhow::Error>, // A VecDeque is used for FIFO action
    edit_window: Option<EditWindow>,
}

impl Default for GlowApp {
    fn default() -> Self {
        let mut error_queue = VecDeque::<anyhow::Error>::new();
        let player = match Player::with_audio_engine() {
            Ok(p) => p,
            Err(e) => {
                panic!("Failed to construct audio engine!\n{e}"); // TODO: [SOON] Replace panic
            }
        };
        let library = match Library::new(SONG_FOLDER, ALLOW_NON_MP3) {
            Ok(lib) => lib,
            Err(e) => {
                error_queue.push_back(e);
                Library::empty(SONG_FOLDER, ALLOW_NON_MP3)
            }
        };

        Self {
            library,
            player,
            error_queue,
            edit_window: None,
        }
    }
}

impl eframeApp for GlowApp {
    fn update(&mut self, ctx: &eguiContext, _frame: &mut eframeFrame) {
        #[cfg(debug_assertions)]
        ctx.set_debug_on_hover(true);

        // TODO: [SOON] Consider how the main loop can facilitate clean communication between modules
        self.player.update();
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
            error!("{error}");

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
                                if title_label.clicked()
                                    && let Err(e) = self.player.play(&self.library, song.id())
                                {
                                    self.error_queue.push_back(e.context("Playback Failed"));
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

    #[allow(clippy::unused_self)] // This will require &mut later
    fn ui_playlist_panel(&mut self, ctx: &eguiContext) {
        SidePanel::left("Playlists")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(STD_MARGIN)))
            .show(ctx, |ui| {
                ui.heading("Playlists");
                ui.separator();
                ui.label("All Songs");
            });
    }

    fn ui_playback_bar(&mut self, ctx: &eguiContext) {
        TopBottomPanel::bottom("Playback Bar")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(CONTROL_MARGIN)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 1st Section: Playback Buttons
                    // TODO: [SOON] Force all buttons to be equal size
                    ui.vertical(|ui| {
                        ui.add_enabled_ui(self.player.active_id().is_some(), |ui| {
                            let playing = self.player.is_playing();

                            ui.horizontal(|ui| {
                                let label = if playing { "Pause" } else { "Play" };

                                if ui.button(label).clicked() {
                                    if playing {
                                        self.player.pause();
                                    } else {
                                        self.player.resume();
                                    }
                                }
                                if ui.button("Stop").clicked() {
                                    self.player.stop();
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button("Back").clicked() {
                                    // TODO: [LATER] Add functionality
                                }
                                if ui.button("Skip").clicked() {
                                    // TODO: [LATER] Add functionality
                                }
                            });
                        });
                    });

                    // 2nd Section: Playback info and seek bar
                    // TODO: Consider cases: No audio loaded, audio has no duration
                    ui.vertical(|ui| {
                        if let Some(id) = self.player.active_id()
                            && let Some(song) = self.library.get_song(id)
                        {
                            ui.horizontal(|ui| {
                                ui.label(song.title());
                                ui.label("by");
                                ui.label(song.artist());
                                ui.label(song.formatted_duration());
                            });

                            match (song.duration(), self.player.position()) {
                                (Some(duration), Some(mut position)) => {
                                    let total_duration = duration.as_secs_f64();

                                    let seek_bar = ui.add(
                                        Slider::new(&mut position, 0.0..=total_duration)
                                            .show_value(false),
                                    );

                                    if seek_bar.drag_stopped()
                                        && let Err(e) = self.player.set_position(position)
                                    {
                                        self.error_queue.push_back(e);
                                    }
                                }
                                _ => {
                                    ui.add_enabled(false, ProgressBar::new(1.0));
                                }
                            }
                        }
                    });

                    // 3rd Section: Volume
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Log Active ID").clicked() {
                                tracing::info!("Active ID = {:?}", self.player.active_id());
                            }
                            if ui.button("Log State").clicked() {
                                tracing::info!("Player state = {:?}", self.player.state());
                            }
                        });

                        ui.horizontal(|ui| {
                            let mut volume = self.player.volume();

                            let volume_slider =
                                ui.add(Slider::new(&mut volume, 0..=100).show_value(false));
                            if volume_slider.changed() {
                                self.player.set_volume(volume);
                            }

                            ui.label(format!("{volume}%"));
                        });
                    });
                });
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
