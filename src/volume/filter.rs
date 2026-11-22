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
        
        // threading !!!
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        let rows_per_block = self.h.div_ceil(threads);
        let block_len = rows_per_block * self.w * 3;

        std::thread::scope(|scope| {
            for (b, block) in out.chunks_mut(block_len).enumerate() {
                scope.spawn(move || {
                    let first_row = (b * rows_per_block) as isize;
                    let rows = (block.len() / (self.w * 3)) as isize;

                    for (o, y) in (first_row..first_row + rows).enumerate() {
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
                            // write into this thread's block, not push onto a shared Vec
                            let at = (o * self.w + x as usize) * 3;
                            block[at..at + 3].copy_from_slice(&sum);
                        }
                    }
                });
            }
        });
        out
    }

    pub fn gaussian(&self, sigma: f64) -> Volume {
        let ker = gaussian_kernel(sigma);
        let n = ker.len();
        // horizontal pass
        let result = self.convolve(&ker, n, 1);
        // vertical pass
        result.convolve(&ker, 1, n)
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

fn gaussian_kernel(sigma: f64) -> Vec<f64> {
    // g(x) = e^(-x^2 / (2*sigma^2))
    let r = (3.0 * sigma).ceil() as i32;
    let mut mask: Vec<f64> = (-r..=r).map(|x| (-(x * x) as f64 / (2.0 * sigma * sigma)).exp()).collect();
    let s: f64 = mask.iter().sum();

    mask.iter_mut().for_each(|v| *v /= s);
    debug_assert!(mask.len() % 2 == 1 && mask.iter().eq(mask.iter().rev()));
    mask
}
