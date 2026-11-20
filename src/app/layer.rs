use eframe::egui;

use crate::volume::Volume;

/// a volume together with the GPU texture rendered from it.
pub(super) struct Layer {
    volume: Volume,
    texture: Option<egui::TextureHandle>,
}

impl Layer {
    pub(super) fn new(volume: Volume) -> Self {
        Self { volume, texture: None }
    }

    pub(super) fn volume(&self) -> &Volume {
        &self.volume
    }

    pub(super) fn texture(&self) -> Option<&egui::TextureHandle> {
        self.texture.as_ref()
    }

    /// the only way to change the image - drops the stale texture so it gets rebuilt
    pub(super) fn replace_volume(&mut self, volume: Volume) {
        self.volume = volume;
        self.texture = None;
    }

    /// renders and uploads the texture if there isn't one yet
    pub(super) fn ensure_texture(&mut self, ctx: &egui::Context, name: &str) {
        if self.texture.is_none() {
            let pixels = self.volume.render();
            let size = [self.volume.width(), self.volume.height()];
            let image = egui::ColorImage::from_rgb(size, &pixels);
            self.texture = Some(ctx.load_texture(name, image, egui::TextureOptions::NEAREST));
        }
    }
}
