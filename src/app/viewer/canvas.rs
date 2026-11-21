use eframe::egui;

use super::View;
use crate::app::ViewerApp;
use crate::app::layer::Layer;
use crate::volume::Volume;

const MIN_LABEL_ZOOM: f32 = 30.0;
const PANE_GAP: f32 = 8.0;

impl View {
    /// zoom and offset that fit the whole image into a pane, centered
    fn fit(pane: egui::Vec2, image: egui::Vec2) -> Self {
        let zoom = (pane.x / image.x).min(pane.y / image.y);
        let offset = (pane - image * zoom) / 2.0;
        // ^ measured from left to right, divided by two so the image is centered
        Self { zoom, offset }
    }

    /// dragging and scroll-zooming on one pane. the offset is measured from the pane's
    /// top-left corner, so one View works for every pane of the same size
    fn handle_input(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response) {
        self.offset += response.drag_delta();

        if let Some(cursor) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);

            if scroll != 0.0 {
                let factor = (scroll * 0.002).exp();
                let cursor_in_pane = cursor - rect.min;
                self.offset = cursor_in_pane - (cursor_in_pane - self.offset) * factor;
                self.zoom *= factor;
            }
        }
    }
}

/// splits the canvas into `count` equally wide panes with a gap between them
fn split(area: egui::Rect, count: usize) -> Vec<egui::Rect> {
    let width = (area.width() - PANE_GAP * (count - 1) as f32) / count as f32;
    (0..count)
        .map(|i| {
            let left = area.min.x + i as f32 * (width + PANE_GAP);
            egui::Rect::from_min_size(egui::pos2(left, area.min.y), egui::vec2(width, area.height()))
        })
        .collect()
}

impl ViewerApp {
    /// central panel: the result, or original and result side by side, sharing one View
    pub(super) fn show_canvas(&mut self, ui: &mut egui::Ui) {
        let Some(file) = &self.file else { return };

        egui::CentralPanel::default().show(ui, |ui| {
            let panes = if self.side_by_side {
                vec![("original", &file.original), ("result", &file.result)]
            } else {
                vec![("result", &file.result)]
            };

            // lay out the panes and reserve their space
            let rects = split(ui.available_rect_before_wrap(), panes.len());
            let responses: Vec<egui::Response> =
                rects.iter().map(|&rect| ui.allocate_rect(rect, egui::Sense::click_and_drag())).collect();

            // update the shared view from whichever pane got the input,
            // before painting, so both panes show the same frame
            if self.view.is_none() || responses.iter().any(|r| r.double_clicked()) {
                let volume = file.result.volume();
                let image_size = egui::vec2(volume.width() as f32, volume.height() as f32);
                self.view = Some(View::fit(rects[0].size(), image_size));
            }
            let Some(view) = &mut self.view else { return };
            for (rect, response) in rects.iter().zip(&responses) {
                view.handle_input(ui, *rect, response);
            }

            // paint every pane with the same view
            for ((title, layer), rect) in panes.iter().zip(&rects) {
                Self::paint_pane(ui, *rect, title, layer, view);
            }
        });
    }

    fn paint_pane(ui: &egui::Ui, rect: egui::Rect, title: &str, layer: &Layer, view: &View) {
        let painter = ui.painter_at(rect);

        let Some(texture) = layer.texture() else { return };
        let image_size = texture.size_vec2();
        let image_rect = egui::Rect::from_min_size(rect.min + view.offset, image_size * view.zoom);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(texture.id(), image_rect, uv, egui::Color32::WHITE);

        if view.zoom >= MIN_LABEL_ZOOM {
            Self::draw_pixel_values(&painter, rect, image_rect, view.zoom, layer.volume());
        }

        // title in the top-left corner, with a shadow so it reads on any image
        let pos = rect.left_top() + egui::vec2(8.0, 8.0);
        let font = egui::FontId::proportional(14.0);
        painter.text(pos + egui::vec2(1.0, 1.0), egui::Align2::LEFT_TOP, title, font.clone(), egui::Color32::BLACK);
        painter.text(pos, egui::Align2::LEFT_TOP, title, font, egui::Color32::WHITE);
    }

    fn draw_pixel_values(
        painter: &egui::Painter,
        rect: egui::Rect,
        image_rect: egui::Rect,
        zoom: f32,
        volume: &Volume,
    ) {
        let image_size = egui::vec2(volume.width() as f32, volume.height() as f32);
        let first = ((rect.min - image_rect.min) / zoom).floor().max(egui::Vec2::ZERO);

        let last = ((rect.max - image_rect.min) / zoom).ceil().min(image_size);
        let font = egui::FontId::monospace(10.0);

        for y in first.y as usize..last.y as usize {
            for x in first.x as usize..last.x as usize {
                let Some(val) = volume.pixel_value(x, y) else {
                    continue;
                };

                let text = format!("{}\n{}\n{}", val[0], val[1], val[2]);
                let center = image_rect.min + egui::vec2(x as f32 + 0.5, y as f32 + 0.5) * zoom;
                // cool shadow
                painter.text(
                    center + egui::vec2(1.0, 1.0),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font.clone(),
                    egui::Color32::BLACK,
                );
                painter.text(center, egui::Align2::CENTER_CENTER, &text, font.clone(), egui::Color32::WHITE);
            }
        }
    }
}
