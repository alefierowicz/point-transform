use eframe::egui;

use crate::app::{Channel, ViewerApp};
use crate::theme;
use crate::volume::filter;
use crate::volume::point::{self, PointOp};

const SIDEBAR_WIDTH: f32 = 280.0;

impl ViewerApp {
    /// left panel: file name and parameters on top, file controls and errors below
    pub(super) fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("sidebar").resizable(false).exact_size(SIDEBAR_WIDTH).show(ui, |ui| {
            if let Some(file) = &self.file {
                ui.heading(&file.name);
                let volume = file.original.volume();
                ui.label(format!("{} x {}, max: {}", volume.width(), volume.height(), volume.maxval()));
                // the pane width changes, so the old fit no longer fits
                if ui.checkbox(&mut self.side_by_side, "side by side").changed() {
                    self.view = None;
                }
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

            if self.file.is_some() {
                ui.separator();
                section_header(ui, "point transformations");

                labelled_row(ui, "channel", |ui| {
                    egui::ComboBox::from_id_salt("point_channel")
                        .width(ui.available_width())
                        .selected_text(self.point_channel.label())
                        .show_ui(ui, |ui| {
                            for channel in Channel::ALL {
                                ui.selectable_value(&mut self.point_channel, channel, channel.label());
                            }
                        });
                });
                labelled_row(ui, "value", |ui| {
                    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                    ui.add_sized(size, egui::DragValue::new(&mut self.point_value).speed(0.1).max_decimals(2));
                });
                ui.add_space(4.0);

                // two rows of two, so the buttons fit the sidebar width
                let half_width = egui::vec2((ui.available_width() - ui.spacing().item_spacing.x) / 2.0, 0.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("add").min_size(half_width)).clicked() {
                        self.apply_point_op(PointOp::Add(self.channel_values(0.0)));
                    }
                    if ui.add(egui::Button::new("subtract").min_size(half_width)).clicked() {
                        self.apply_point_op(PointOp::Subtract(self.channel_values(0.0)));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("multiply").min_size(half_width)).clicked() {
                        self.apply_point_op(PointOp::Multiply(self.channel_values(1.0)));
                    }
                    if ui.add(egui::Button::new("divide").min_size(half_width)).clicked() {
                        self.apply_point_op(PointOp::Divide(self.channel_values(1.0)));
                    }
                });

                // brightness and greyscale always work on all channels
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // the slider's track plus its value box take up the left half
                    let spacing = ui.spacing();
                    let track_width = half_width.x - spacing.interact_size.x - spacing.item_spacing.x;
                    ui.spacing_mut().slider_width = track_width;
                    ui.add(egui::Slider::new(&mut self.point_brightness, -100.0..=100.0));

                    if ui.add(egui::Button::new("brightness").min_size(half_width)).clicked() {
                        self.apply_point_op(PointOp::Brightness(self.point_brightness));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("grey avg").min_size(half_width)).clicked() {
                        self.apply_greyscale(point::GREY_AVERAGE);
                    }
                    if ui.add(egui::Button::new("grey luma").min_size(half_width)).clicked() {
                        self.apply_greyscale(point::GREY_LUMINANCE);
                    }
                });

                // filters: they look at each pixel's neighbourhood, not just its own value
                ui.separator();
                section_header(ui, "filters");
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("average").min_size(half_width)).clicked() {
                        self.apply_filter(&filter::box_kernel(3), 3, 3);
                    }
                    if ui.add(egui::Button::new("sharpen").min_size(half_width)).clicked() {
                        self.apply_filter(&filter::SHARPEN, 3, 3);
                    }
                });
                ui.horizontal(|ui| {
                    // the slider's track plus its value box take up the left half
                    let spacing = ui.spacing();
                    let track_width = half_width.x - spacing.interact_size.x - spacing.item_spacing.x;
                    ui.spacing_mut().slider_width = track_width;
                    ui.add(egui::Slider::new(&mut self.filter_sigma, 0.1..=10.0));

                    if ui.add(egui::Button::new("gaussian").min_size(half_width)).clicked() {
                        self.apply_gaussian();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("sobel").min_size(half_width)).clicked() {
                        self.apply_sobel();
                    }
                });
                ui.separator();
                if ui.add(egui::Button::new("reset").min_size(full_width)).clicked() {
                    self.reset_result();
                }
            }

            if let Some(msg) = &self.error {
                ui.separator();
                ui.colored_label(ui.visuals().error_fg_color, msg);
            }
        });
    }
}

const LABEL_WIDTH: f32 = 64.0;

/// bold heading at the start of a sidebar section
fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(text).size(16.0).family(egui::FontFamily::Name(theme::BOLD.into())));
    ui.add_space(4.0);
}

/// a label with a fixed width on the left, so the widgets after it line up from row to row
fn labelled_row(ui: &mut egui::Ui, label: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        let size = egui::vec2(LABEL_WIDTH, ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(LABEL_WIDTH);
            ui.label(label);
        });
        add_contents(ui);
    });
}
