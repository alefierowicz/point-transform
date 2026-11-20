mod canvas;
mod controls;
mod sidebar;

use eframe::egui;

use super::ViewerApp;

pub(super) struct View {
    zoom: f32,
    offset: egui::Vec2,
}

impl ViewerApp {
    pub(super) fn show_main(&mut self, ui: &mut egui::Ui) {
        self.show_sidebar(ui);
        self.show_controls(ui);
        self.update_texture(ui);
        self.show_canvas(ui);
    }

    /// uploads textures for any layer whose volume changed since the last frame
    fn update_texture(&mut self, ui: &mut egui::Ui) {
        let Some(file) = &mut self.file else { return };
        file.original.ensure_texture(ui.ctx(), "original");
        file.result.ensure_texture(ui.ctx(), "result");
    }
}
