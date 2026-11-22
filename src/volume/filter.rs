use crate::volume::Volume;
impl Volume {
    pub fn convolve(&self, kernel: &[f64], kw: usize, kh: usize) -> Volume {
        let raw = self.convolve_raw(kernel, kw, kh);
        let data: Vec<u16> = raw.iter().map(|v| v.round().clamp(0.0, self.maxval as f64) as u16).collect();

        Volume { h: self.h, w: self.w, maxval: self.maxval, data }
    }

    fn convolve_raw(&self, kernel: &[f64], kw: usize, kh: usize) -> Vec<f64> {
        debug_assert!(kernel.len() == kw * kh && kw % 2 == 1 && kh % 2 == 1);

        let (w, h) = (self.w as isize, self.h as isize);

        // distance from edge to centre of mask
        let (rx, ry) = ((kw / 2) as isize, (kh / 2) as isize);
        let mut out = vec![0.0; self.data.len()];

        self.par_rows(&mut out, |y, row| {
            for x in 0..w {
                let mut sum = [0.0; 3];
                for ky in 0..kh {
                    let sy = (y + ky as isize - ry).clamp(0, h - 1) as usize;
                    for kx in 0..kw {
                        let sx = (x + kx as isize - rx).clamp(0, w - 1) as usize;
                        let weight = kernel[ky * kw + kx];
                        let i = (sy * self.w + sx) * 3;
                        for c in 0..3 {
                            sum[c] += weight * self.data[i + c] as f64;
                        }
                    }
                }
                let at = x as usize * 3;
                row[at..at + 3].copy_from_slice(&sum);
            }
        });
        out
    }

    /// median filter over an n x n window: each sample becomes the middle value of its
    /// window in the same channel. removes outliers but keeps edges sharp
    pub fn median(&self, n: usize) -> Volume {
        debug_assert!(n % 2 == 1);
        let (w, h) = (self.w as isize, self.h as isize);
        let r = (n / 2) as isize;
        let mut out = vec![0; self.data.len()];

        self.par_rows(&mut out, |y, row| {
            // one window per channel, reused for every pixel in the row
            let mut windows: [Vec<u16>; 3] = std::array::from_fn(|_| Vec::with_capacity(n * n));
            for x in 0..w {
                windows.iter_mut().for_each(Vec::clear);
                for dy in -r..=r {
                    // positions outside the image use the nearest edge pixel, as in convolve_raw
                    let sy = (y + dy).clamp(0, h - 1) as usize;
                    for dx in -r..=r {
                        let sx = (x + dx).clamp(0, w - 1) as usize;
                        let i = (sy * self.w + sx) * 3;
                        for c in 0..3 {
                            windows[c].push(self.data[i + c]);
                        }
                    }
                }
                // the middle value: only places that one element correctly, without sorting the rest
                let mid = n * n / 2;
                let at = x as usize * 3;
                for c in 0..3 {
                    row[at + c] = *windows[c].select_nth_unstable(mid).1;
                }
            }
        });
        Volume { h: self.h, w: self.w, maxval: self.maxval, data: out }
    }

    /// splits `out` into blocks of whole rows and fills them on all CPU cores at once.
    /// `fill_row(y, row)` computes image row `y` into `row`, which holds w * 3 samples
    fn par_rows<T: Send>(&self, out: &mut [T], fill_row: impl Fn(isize, &mut [T]) + Sync) {
        let row_len = self.w * 3;
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        let rows_per_block = self.h.div_ceil(threads);

        std::thread::scope(|scope| {
            // non-overlapping blocks, so no two threads ever write the same sample
            for (b, block) in out.chunks_mut(rows_per_block * row_len).enumerate() {
                let fill_row = &fill_row;
                scope.spawn(move || {
                    for (i, row) in block.chunks_mut(row_len).enumerate() {
                        fill_row((b * rows_per_block + i) as isize, row);
                    }
                });
            }
        });
    }

    pub fn gaussian(&self, sigma: f64) -> Volume {
        let ker = gaussian_kernel(sigma);
        let n = ker.len();
        // horizontal pass
        let result = self.convolve(&ker, n, 1);
        // vertical pass
        result.convolve(&ker, 1, n)
    }

    /// Sobel edge detection: edge strength sqrt(gx² + gy²) per sample,
    /// min-max normalised to [0, maxval] so the strongest edge becomes maxval
    pub fn sobel(&self) -> Volume {
        // raw, not clamped: gx and gy are negative for edges from bright to dark
        let gx = self.convolve_raw(&SOBEL_X, 3, 3);
        let gy = self.convolve_raw(&SOBEL_Y, 3, 3);
        let magnitude: Vec<f64> = gx.iter().zip(&gy).map(|(x, y)| (x * x + y * y).sqrt()).collect();

        let min = magnitude.iter().copied().fold(f64::INFINITY, f64::min);
        let max = magnitude.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        // a completely flat image has no edges at all: everything becomes 0
        let scale = if max > min { self.maxval as f64 / (max - min) } else { 0.0 };

        let data = magnitude.iter().map(|m| ((m - min) * scale).round() as u16).collect();
        Volume { h: self.h, w: self.w, maxval: self.maxval, data }
    }
}

/// n x n averaging mask
pub fn box_kernel(n: usize) -> Vec<f64> {
    vec![1.0 / (n * n) as f64; n * n]
}

/// 3 x 3 sharpen mask
#[rustfmt::skip]
pub const SHARPEN: [f64; 9] = [
     0.0, -1.0,  0.0,
    -1.0,  5.0, -1.0,
     0.0, -1.0,  0.0,
];

/// Sobel masks: horizontal and vertical change in brightness, both sum to 0
#[rustfmt::skip]
const SOBEL_X: [f64; 9] = [
    -1.0, 0.0, 1.0,
    -2.0, 0.0, 2.0,
    -1.0, 0.0, 1.0,
];
#[rustfmt::skip]
const SOBEL_Y: [f64; 9] = [
    -1.0, -2.0, -1.0,
     0.0,  0.0,  0.0,
     1.0,  2.0,  1.0,
];

fn gaussian_kernel(sigma: f64) -> Vec<f64> {
    // g(x) = e^(-x^2 / (2*sigma^2))
    let r = (3.0 * sigma).ceil() as i32;
    let mut mask: Vec<f64> = (-r..=r).map(|x| (-(x * x) as f64 / (2.0 * sigma * sigma)).exp()).collect();
    let s: f64 = mask.iter().sum();

    mask.iter_mut().for_each(|v| *v /= s);
    debug_assert!(mask.len() % 2 == 1 && mask.iter().eq(mask.iter().rev()));
    mask
}
