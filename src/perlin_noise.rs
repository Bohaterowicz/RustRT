use rand::seq::SliceRandom;

use crate::math::{
    rand::rand_f32,
    vec3::{dot, Vec3},
};

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    rand_vec: [Vec3; Self::POINT_COUNT],
    perm_x: [u8; Self::POINT_COUNT],
    perm_y: [u8; Self::POINT_COUNT],
    perm_z: [u8; Self::POINT_COUNT],
}

impl PerlinNoise {
    const POINT_COUNT: usize = 256;
    pub fn new() -> Self {
        let rand_vec = std::array::from_fn(|_| Vec3::random_unit());
        Self {
            rand_vec,
            perm_x: PerlinNoise::generate_permutation(),
            perm_y: PerlinNoise::generate_permutation(),
            perm_z: PerlinNoise::generate_permutation(),
        }
    }

    pub fn noise(&self, p: Vec3) -> f32 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let interpolation_points = std::array::from_fn(|x| {
            std::array::from_fn(|y| {
                std::array::from_fn(|z| {
                    self.rand_vec[(self.perm_x[((x as i32 + i) & 255) as usize]
                        ^ self.perm_y[((y as i32 + j) & 255) as usize]
                        ^ self.perm_z[((z as i32 + k) & 255) as usize])
                        as usize]
                })
            })
        });

        perlin_interpolate(interpolation_points, u, v, w)
    }

    pub fn turbulence(&self, p: Vec3, depth: i32) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }
        accum.abs()
    }

    fn generate_permutation() -> [u8; Self::POINT_COUNT] {
        let mut perm = [0u8; Self::POINT_COUNT];
        perm.iter_mut().enumerate().for_each(|(i, p)| *p = i as u8);
        perm.shuffle(&mut rand::thread_rng());
        perm
    }
}

fn perlin_interpolate(points: [[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
    let uu = hermite_fade(u);
    let vv = hermite_fade(v);
    let ww = hermite_fade(w);
    let mut accum = 0.0;
    points.iter().enumerate().for_each(|(i, x)| {
        x.iter().enumerate().for_each(|(j, y)| {
            y.iter().enumerate().for_each(|(k, z)| {
                let weight = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                accum += (i as f32 * uu + (1 - i) as f32 * (1.0 - uu))
                    * (j as f32 * vv + (1 - j) as f32 * (1.0 - vv))
                    * (k as f32 * ww + (1 - k) as f32 * (1.0 - ww))
                    * dot(z, &weight);
            });
        });
    });
    accum
}

fn trilinear_interpolate(points: [[[f32; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
    let mut accum = 0.0;
    points.iter().enumerate().for_each(|(i, x)| {
        x.iter().enumerate().for_each(|(j, y)| {
            y.iter().enumerate().for_each(|(k, z)| {
                accum += (i as f32 * u + (1 - i) as f32 * (1.0 - u))
                    * (j as f32 * v + (1 - j) as f32 * (1.0 - v))
                    * (k as f32 * w + (1 - k) as f32 * (1.0 - w))
                    * z;
            });
        });
    });
    accum
}

fn hermite_fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
