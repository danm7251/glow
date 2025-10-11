use eframe::{App as eframeApp, egui::Context as eguiContext, Frame as eframeFrame};
use std::time::Duration;

pub struct GlowApp;

impl Default for GlowApp {
    fn default() -> Self {
        Self
    }
}

impl eframeApp for GlowApp {
    fn update(&mut self, ctx: &eguiContext, _frame: &mut eframeFrame) {
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}