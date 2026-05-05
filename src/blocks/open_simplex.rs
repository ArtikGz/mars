// from https://github.com/MasonFeurer/OpenSimplex/blob/master/src/lib.rs
#[derive(Clone)]
pub struct Noise2d {
    pub noise: Box<Noise>,
    pub scale: f64,
    pub freq: f64,
}
impl Noise2d {
    pub fn new(seed: i64, freq: f64, scale: f64) -> Self {
        let noise = Noise::boxed(seed);
        Self { noise, freq, scale }
    }

    pub fn get(&self, x: f64, y: f64) -> f64 {
        let val = self.noise.eval2d(x * self.freq, y * self.freq);
        ((val + 1.0) * 0.5) * self.scale
    }
}

/// Averages multiple Noise2d's
pub struct MixedNoise2d {
    sources: Vec<Noise2d>,
}
impl MixedNoise2d {
    pub fn new(sources: &[Noise2d]) -> Self {
        Self {
            sources: sources.to_vec(),
        }
    }
    pub fn get(&self, x: f64, y: f64) -> f64 {
        let mut sum = 0.0;
        for source in &self.sources {
            sum += source.get(x, y);
        }
        sum / self.sources.len() as f64
    }
}

///
/// 2014 OpenSimplex Noise in Java.
/// by Kurt Spencer
///
/// Ported to Rust by Mason Feurer (Excluding 4D)
/// I have no idea how this thing works, just translated the code.
///

const STRETCH_CONSTANT_2D: f64 = -0.211324865405187; // (1/(2+1).sqrt()-1)/2;
const SQUISH_CONSTANT_2D: f64 = 0.366025403784439; // ((2+1).sqrt()-1)/2;

const PSIZE: usize = 2048;
const PMASK: usize = 2047;

type Grad2 = (f64, f64);
#[derive(Clone)]
pub struct Noise {
    perm: [usize; PSIZE],
    perm_grad2: [Grad2; PSIZE],
}
impl Noise {
    #[allow(invalid_value)]
    pub fn new(seed: i64) -> Self {
        let mut result: Self = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        result.load_seed(seed);
        result
    }

    pub fn boxed(seed: i64) -> Box<Self> {
        let mut result: Box<Self> = unsafe { Box::new_uninit().assume_init() };
        result.load_seed(seed);
        result
    }

    pub fn load_seed(&mut self, mut seed: i64) {
        let mut source = [0; PSIZE];
        for i in 0..PSIZE {
            source[i] = i;
        }
        for i in (0..PSIZE).rev() {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = ((seed + 31) % (i as i64 + 1)) as isize;
            let r = if r < 0 { r + i as isize + 1 } else { r } as usize;
            let s = source[r];
            self.perm[i] = s;
            self.perm_grad2[i] = GRADIENTS_2D[s];
            source[r] = source[i];
        }
    }

    /// 2D OpenSimplex Noise
    pub fn eval2d(&self, x: f64, y: f64) -> f64 {
        // Place input coordinates onto grid.
        let stretch_offset = (x + y) * STRETCH_CONSTANT_2D;
        let xs = x + stretch_offset;
        let ys = y + stretch_offset;

        // Floor to get grid coordinates of rhombus (stretched square) super-cell origin.
        let mut xsb: i32 = xs.floor() as i32;
        let mut ysb: i32 = ys.floor() as i32;

        // Compute grid coordinates relative to rhombus origin.
        let xins: f64 = xs - xsb as f64;
        let yins: f64 = ys - ysb as f64;

        // Sum those together to get a value that determines which region we're in.
        let in_sum: f64 = xins + yins;

        // Positions relative to origin point.
        let squish_offset_ins: f64 = in_sum * SQUISH_CONSTANT_2D;
        let mut dx0: f64 = xins + squish_offset_ins;
        let mut dy0: f64 = yins + squish_offset_ins;

        // We'll be defining these inside the next block and using them afterwards.
        let dx_ext: f64;
        let dy_ext: f64;
        let xsv_ext: i32;
        let ysv_ext: i32;

        let mut value: f64 = 0.0;

        // Contribution (1,0)
        let dx1: f64 = dx0 - 1.0 - SQUISH_CONSTANT_2D;
        let dy1: f64 = dy0 - 0.0 - SQUISH_CONSTANT_2D;
        let mut attn1: f64 = 2.0 - dx1 * dx1 - dy1 * dy1;
        if attn1 > 0.0 {
            attn1 *= attn1;
            value += attn1 * attn1 * self.extrapolate2d(xsb + 1, ysb + 0, dx1, dy1);
        }

        // Contribution (0,1)
        let dx2: f64 = dx0 - 0.0 - SQUISH_CONSTANT_2D;
        let dy2: f64 = dy0 - 1.0 - SQUISH_CONSTANT_2D;
        let mut attn2: f64 = 2.0 - dx2 * dx2 - dy2 * dy2;
        if attn2 > 0.0 {
            attn2 *= attn2;
            value += attn2 * attn2 * self.extrapolate2d(xsb + 0, ysb + 1, dx2, dy2);
        }

        if in_sum <= 1.0 {
            // We're inside the triangle (2-Simplex) at (0,0)
            let zins: f64 = 1.0 - in_sum;
            if zins > xins || zins > yins {
                // (0,0) is one of the closest two triangular vertices
                if xins > yins {
                    xsv_ext = xsb + 1;
                    ysv_ext = ysb - 1;
                    dx_ext = dx0 - 1.0;
                    dy_ext = dy0 + 1.0;
                } else {
                    xsv_ext = xsb - 1;
                    ysv_ext = ysb + 1;
                    dx_ext = dx0 + 1.0;
                    dy_ext = dy0 - 1.0;
                }
            } else {
                // (1,0) and (0,1) are the closest two vertices.
                xsv_ext = xsb + 1;
                ysv_ext = ysb + 1;
                dx_ext = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
                dy_ext = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
            }
        } else {
            // We're inside the triangle (2-Simplex) at (1,1)
            let zins: f64 = 2.0 - in_sum;
            if zins < xins || zins < yins {
                // (0,0) is one of the closest two triangular vertices
                if xins > yins {
                    xsv_ext = xsb + 2;
                    ysv_ext = ysb + 0;
                    dx_ext = dx0 - 2.0 - 2.0 * SQUISH_CONSTANT_2D;
                    dy_ext = dy0 + 0.0 - 2.0 * SQUISH_CONSTANT_2D;
                } else {
                    xsv_ext = xsb + 0;
                    ysv_ext = ysb + 2;
                    dx_ext = dx0 + 0.0 - 2.0 * SQUISH_CONSTANT_2D;
                    dy_ext = dy0 - 2.0 - 2.0 * SQUISH_CONSTANT_2D;
                }
            } else {
                // (1,0) and (0,1) are the closest two vertices.
                dx_ext = dx0;
                dy_ext = dy0;
                xsv_ext = xsb;
                ysv_ext = ysb;
            }
            xsb += 1;
            ysb += 1;
            dx0 = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
            dy0 = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
        }

        // Contribution (0,0) or (1,1)
        let mut attn0: f64 = 2.0 - dx0 * dx0 - dy0 * dy0;
        if attn0 > 0.0 {
            attn0 *= attn0;
            value += attn0 * attn0 * self.extrapolate2d(xsb, ysb, dx0, dy0);
        }

        // Extra Vertex
        let mut attn_ext: f64 = 2.0 - dx_ext * dx_ext - dy_ext * dy_ext;
        if attn_ext > 0.0 {
            attn_ext *= attn_ext;
            value += attn_ext * attn_ext * self.extrapolate2d(xsv_ext, ysv_ext, dx_ext, dy_ext);
        }
        value
    }


    fn extrapolate2d(&self, xsb: i32, ysb: i32, dx: f64, dy: f64) -> f64 {
        let grad = &self.perm_grad2[self.perm[xsb as usize & PMASK] ^ (ysb as usize & PMASK)];
        grad.0 * dx + grad.1 * dy
    }
}

pub static GRADIENTS_2D: [(f64, f64); 2048] = include!("gradients_2d.expr");