use crate::volume::Volume;
impl Volume {
    /// effectively performing the transformation, t is the transfer function
    pub fn map_point(&self, t: impl Fn(usize, f64) -> f64) -> Volume {
        let tables = [0, 1, 2].map(|c| build_table(self.maxval, |r| t(c, r)));
        let new_data: Vec<u16> = self.data.iter().enumerate().map(|(i, &v)| tables[i % 3][v as usize]).collect();

        Volume { h: self.h, w: self.w, maxval: self.maxval, data: new_data }
    }

    pub fn apply(&self, op: &PointOp) -> Result<Volume, String> {
        op.check()?;
        Ok(self.map_point(|c, r| op.eval(c, r)))
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

pub enum PointOp {
    Add([f64; 3]),
    Subtract([f64; 3]),
    Multiply([f64; 3]),
    Divide([f64; 3]),
}
impl PointOp {
    fn check(&self) -> Result<(), String> {
        // multiplication can only be a non-zero positive number,
        // same with division
        match *self {
            PointOp::Multiply(k) | PointOp::Divide(k) => {
                if !k.iter().all(|&c| c > 0.0) {
                    return Err("values for multiplication or division must be greater than 0".into());
                }
            }
            PointOp::Add(_) | PointOp::Subtract(_) => {}
        };
        Ok(())
    }

    fn eval(&self, c: usize, r: f64) -> f64 {
        match *self {
            PointOp::Add(k) => r + k[c],
            PointOp::Subtract(k) => r - k[c],
            PointOp::Multiply(k) => r * k[c],
            PointOp::Divide(k) => r / k[c],
        }
    }
}
