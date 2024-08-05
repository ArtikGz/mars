use rand::{distributions::uniform::SampleRange, prelude::*};
use rand_chacha::ChaCha8Rng;

pub struct PerlinNoiseGenerator {}

impl PerlinNoiseGenerator {
    pub fn new() -> Self {
        PerlinNoiseGenerator {}
    }
    pub fn get_height_for(&self, x: f64, z: f64) -> f64 {
        self.perlin(x, z)
    }

    fn perlin(&self, x: f64, z: f64) -> f64 {
        let x0 = x.floor();
        let z0 = z.floor();
        let x1 = x0 + 1.0;
        let z1 = z0 + 1.0;

        let diff_x = x.fract();
        let diff_z = z.fract();

        let sx = fade(diff_x);
        let sz = fade(diff_z);

        let top0 = self.perlin_dot_product(x0, z0, diff_x, diff_z);
        let top1 = self.perlin_dot_product(x1, z0, diff_x - 1.0, diff_z);
        let top_i = lerp(top0, top1, sx);

        let bot0 = self.perlin_dot_product(x0, z1, diff_x, diff_z - 1.0);
        let bot1 = self.perlin_dot_product(x1, z1, diff_x - 1.0, diff_z - 1.0);
        let bot_i = lerp(bot0, bot1, sx);

        lerp(top_i, bot_i, sz)
    }

    fn perlin_dot_product(&self, ix: f64, iz: f64, dx: f64, dz: f64) -> f64 {
        let (gx, gz) = rgrad(ix, iz);

        gx * dx + gz * dz
    }
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, x: f64) -> f64 {
    a + x * (b - a)
}

fn mod289(x: f64) -> f64 {
    x - ((x / 289.0).floor() * 289.0)
}

fn permute(x: f64) -> f64 {
    mod289(((x * 34.0) + 10.0) * x)
}

fn rgrad(x: f64, z: f64) -> (f64, f64) {
    let mut u = permute(permute(x) + z) * 0.0243902439;
    u = u.fract() * 6.28318530718; // 2*pi

    (u.cos(), u.sin())
}
