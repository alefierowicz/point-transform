use crate::{jpeg, ppm, volume::Volume};
use std::path::Path;

pub fn load_file(path: &Path) -> Result<Volume, String> {
    let data = std::fs::read(path).map_err(|e| format!("could not read {}: {}", path.display(), e))?;
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        jpeg::parse(&data)
    } else if data.starts_with(b"P") {
        ppm::parse(&data)
    } else {
        Err("unsupported file format".into())
    }
}
