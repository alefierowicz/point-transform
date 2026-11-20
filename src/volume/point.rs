use crate::volume::Volume;
impl Volume {
    #[allow(dead_code)]

    /// effectively performing the transformation, t is the transfer function
    pub fn map_point(&self, t: impl Fn(usize, f64) -> f64) -> Volume {
        let tables = [0, 1, 2].map(|c| build_table(self.maxval, |r| t(c, r)));
        let new_data: Vec<u16> = self.data.iter().enumerate().map(|(i, &v)| tables[i % 3][v as usize]).collect();

        Volume { h: self.h, w: self.w, maxval: self.maxval, data: new_data }
    }
}
/// building lookup tables for map_point to use and cut down on calculations
fn build_table(maxval: u16, t: impl Fn(f64) -> f64) -> Vec<u16> {
    let mut table = Vec::with_capacity(maxval as usize + 1);
    for r in 0..=maxval {
        let s = t(r as f64);
        let s = s.clamp(0.0, maxval as f64);
        table.push(s.round() as u16);
    }
    table
}
