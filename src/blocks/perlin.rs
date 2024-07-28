use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub struct PerlinNoiseGenerator {
    p: Vec<i32>,
}

impl PerlinNoiseGenerator {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let mut permutation_table = vec![rng.gen_range(0..256); 256];
        permutation_table.extend(permutation_table.clone());

        PerlinNoiseGenerator {
            p: permutation_table,
        }
    }

    pub fn get_height_for(&self, x: f64, z: f64) -> f64 {
        let xi = (x.floor() as i32) & 255;
        let xf = x - x.floor();
        let u = fade(xf);

        let zi = (z.floor() as i32) & 255;
        let zf = z - z.floor();
        let v = fade(zf);

        let pxi = self.p[xi as usize];
        let pxi1 = self.p[(xi + 1) as usize];

        let (aa, ab, ba, bb) = (
            self.p[(pxi + zi) as usize],
            self.p[(pxi + zi + 1) as usize],
            self.p[(pxi1 + zi) as usize],
            self.p[(pxi1 + zi + 1) as usize],
        );

        println!("xi={}, xf={}, u={}, a={}, b={}", xi, xf, u, aa, bb);

        lerp(
            lerp(grad(aa, xf, zf), grad(ba, xf - 1.0, zf), u),
            lerp(grad(ab, xf, zf - 1.0), grad(bb, xf - 1.0, zf - 1.0), u),
            v,
        )
    }
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn grad(hash: i32, x: f64, z: f64) -> f64 {
    match hash & 0b11 {
        0b00 => x + z,
        0b01 => x - z,
        0b10 => -x + z,
        0b11 => -x - z,
        _ => 0.0,
    }
}

fn lerp(a: f64, b: f64, x: f64) -> f64 {
    a + x * (b - a)
}
