mod landing;
mod layer;
mod viewer;

use crate::{
    jpeg,
    loader::load_file,
    volume::{Volume, point::PointOp},
};
use eframe::egui;
use layer::Layer;
use std::path::Path;
pub struct ViewerApp {
    file: Option<LoadedFile>,
    error: Option<String>,
    view: Option<viewer::View>,
    jpeg_quality: u8,
    /// original and result next to each other, or only the result
    side_by_side: bool,
    /// value for the point operations, in the image's own units
    point_value: f64,
    /// which channel(s) the point operations change
    point_channel: Channel,
    point_brightness: f64,
}

/// channel choice for the point operations
#[derive(Clone, Copy, PartialEq)]
enum Channel {
    All,
    R,
    G,
    B,
}

impl Channel {
    const ALL: [Channel; 4] = [Channel::All, Channel::R, Channel::G, Channel::B];

    fn label(self) -> &'static str {
        match self {
            Channel::All => "all",
            Channel::R => "R",
            Channel::G => "G",
            Channel::B => "B",
        }
    }
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
        Self {
            file: None,
            error: None,
            view: None,
            jpeg_quality: 90,
            side_by_side: true,
            point_value: 0.0,
            point_channel: Channel::All,
            point_brightness: 0.0,
        }
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

    fn set_result(&mut self, vol: Volume) {
        let Some(file) = &mut self.file else {
            return;
        };

        self.error = None;
        file.result.replace_volume(vol);
    }

    fn reset_result(&mut self) {
        let Some(file) = &self.file else { return };
        let original = file.original.volume().clone();

        self.set_result(original);
    }

    /// R, G, B values for a point operation: the value on the selected channel(s),
    /// `neutral` (the value that leaves a sample unchanged) on the others
    fn channel_values(&self, neutral: f64) -> [f64; 3] {
        let v = self.point_value;
        match self.point_channel {
            Channel::All => [v, v, v],
            Channel::R => [v, neutral, neutral],
            Channel::G => [neutral, v, neutral],
            Channel::B => [neutral, neutral, v],
        }
    }

    /// applies a point operation to the result, so operations stack
    fn apply_point_op(&mut self, op: PointOp) {
        let Some(file) = &self.file else { return };

        match file.result.volume().apply(&op) {
            Ok(volume) => self.set_result(volume),
            Err(msg) => self.error = Some(msg),
        }
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
