use crate::volume::Volume;

pub const GREY_AVERAGE: [f64; 3] = [1.0 / 3.0; 3];
/// recommendation BT.601
pub const GREY_LUMINANCE: [f64; 3] = [0.299, 0.587, 0.114];

impl Volume {
    /// effectively performing the transformation, t is the transfer function
    fn map_point(&self, t: impl Fn(usize, f64) -> f64) -> Volume {
        let tables = [0, 1, 2].map(|c| build_table(self.maxval, |r| t(c, r)));
        let new_data: Vec<u16> = self.data.iter().enumerate().map(|(i, &v)| tables[i % 3][v as usize]).collect();

        Volume { h: self.h, w: self.w, maxval: self.maxval, data: new_data }
    }

    pub fn apply(&self, op: &PointOp) -> Result<Volume, String> {
        op.check()?;
        Ok(self.map_point(|c, r| op.eval(c, r)))
    }

    pub fn greyscale(&self, weights: [f64; 3]) -> Volume {
        let mut out = Vec::with_capacity(self.data.len());
        for chunk in self.data.chunks_exact(3) {
            let grey = chunk.iter().zip(weights).map(|(&v, w)| v as f64 * w).sum::<f64>();
            let g = grey.round().clamp(0.0, self.maxval as f64) as u16;
            out.extend_from_slice(&[g, g, g]);
        }

        Volume { h: self.h, w: self.w, maxval: self.maxval, data: out }
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
    Brightness(f64),
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
            // below -100 % the factor would turn negative
            PointOp::Brightness(v) => {
                if v < -100.0 {
                    return Err("brightness must be at least -100 %".into());
                }
            }
            PointOp::Add(_) | PointOp::Subtract(_) => {}
        };
        Ok(())
    }
    /// what we evaluate the pointOp to
    fn eval(&self, c: usize, r: f64) -> f64 {
        match *self {
            PointOp::Add(k) => r + k[c],
            PointOp::Subtract(k) => r - k[c],
            PointOp::Multiply(k) => r * k[c],
            PointOp::Divide(k) => r / k[c],
            PointOp::Brightness(v) => r * (1.0 + v / 100.0),
        }
    }
}
