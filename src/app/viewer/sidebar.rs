use eframe::egui;

use crate::app::ViewerApp;

const SIDEBAR_WIDTH: f32 = 280.0;

impl ViewerApp {
    /// left panel: file name and parameters on top, file controls and errors below
    pub(super) fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("sidebar").resizable(false).exact_size(SIDEBAR_WIDTH).show(ui, |ui| {
            if let Some(file) = &self.file {
                ui.heading(&file.name);
                ui.label(format!(
                    "{} x {}, max: {}",
                    file.volume.width(),
                    file.volume.height(),
                    file.volume.maxval()
                ));
                ui.separator();
            }

            let full_width = egui::vec2(ui.available_width(), 0.0);
            if ui.add(egui::Button::new("open file").min_size(full_width)).clicked() {
                self.open_dialog();
            }
            if ui.add(egui::Button::new("export to JPEG").min_size(full_width)).clicked() {
                self.save_dialog();
            }
            ui.add(egui::Slider::new(&mut self.jpeg_quality, 1..=100).text("JPEG quality"));

            if let Some(msg) = &self.error {
                ui.separator();
                ui.colored_label(ui.visuals().error_fg_color, msg);
            }
        });
    }
}
