mod landing;
mod layer;
mod viewer;

use crate::{jpeg, loader::load_file, volume::Volume};
use eframe::egui;
use layer::Layer;
use std::path::Path;
pub struct ViewerApp {
    file: Option<LoadedFile>,
    error: Option<String>,
    view: Option<viewer::View>,
    jpeg_quality: u8,
}

/// an open image: the untouched original and the result that operations are applied to.
struct LoadedFile {
    name: String,
    original: Layer,
    result: Layer,
}

impl LoadedFile {
    /// the only place the second copy of the image is made
    fn new(name: String, volume: Volume) -> Self {
        Self { name, original: Layer::new(volume.clone()), result: Layer::new(volume) }
    }
}

impl Default for ViewerApp {
    fn default() -> Self {
        Self { file: None, error: None, view: None, jpeg_quality: 90 }
    }
}

impl ViewerApp {
    fn open_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["ppm", "jpg", "jpeg"])
            .add_filter("All files", &["*"])
            .pick_file()
        else {
            return;
        };

        let Some(name) = path.file_name() else {
            self.error = Some("could not read file name.".into());
            return;
        };
        let name = name.to_string_lossy().into_owned();

        match load_file(&path) {
            Ok(volume) => self.set_file(LoadedFile::new(name, volume)),
            Err(msg) => self.error = Some(msg),
        }
    }

    fn save_dialog(&mut self) {
        let Some(file) = &self.file else { return };
        let name_as_jpeg = Path::new(&file.name).with_extension("jpg");

        match rfd::FileDialog::new()
            .add_filter("JPEG", &["jpg"])
            .set_file_name(name_as_jpeg.to_string_lossy())
            .save_file()
        {
            None => (),
            Some(path) => match jpeg::save(file.result.volume(), &path, self.jpeg_quality) {
                Ok(_) => println!("file saved successfully: {}", &path.display()),
                Err(msg) => self.error = Some(msg),
            },
        };
    }

    fn set_file(&mut self, file: LoadedFile) {
        self.file = Some(file);
        self.error = None;
        self.view = None;
        self.jpeg_quality = 90;
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.file.is_none() {
            self.show_landing(ui);
        } else {
            self.show_main(ui);
        }
    }
}
