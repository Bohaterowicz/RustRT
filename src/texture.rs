use crate::math::{vec2::Vec2, vec3::Vec3};
use crate::perlin_noise::PerlinNoise;
use std::fmt::Debug;

pub trait TextureSampler: Debug + Send + Sync {
    fn value(&self, uv: &Vec2, p: &Vec3) -> Vec3;
}

#[derive(Debug, Clone)]
pub struct Texture {
    color: Vec3,
}

impl Texture {
    pub fn new(color: Vec3) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone)]
pub struct CheckerTexture {
    inv_scale: f32,
    odd: Texture,
    even: Texture,
}

impl CheckerTexture {
    pub fn new(odd: Texture, even: Texture, scale: f32) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            odd,
            even,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageTexture {
    image: image::RgbaImage,
}

impl ImageTexture {
    pub fn new(path: &str) -> Self {
        let img = image::open(path)
            .expect("Failed to open image")
            .flipv()
            .into_rgba8();
        Self { image: img }
    }
}

#[derive(Debug, Clone)]
pub struct NoiseTexture {
    scale: f32,
    noise: PerlinNoise,
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            noise: PerlinNoise::new(),
        }
    }
}

impl TextureSampler for Texture {
    fn value(&self, _uv: &Vec2, _p: &Vec3) -> Vec3 {
        self.color
    }
}

impl TextureSampler for CheckerTexture {
    fn value(&self, uv: &Vec2, p: &Vec3) -> Vec3 {
        let is_even = ((self.inv_scale * p.x).floor()
            + (self.inv_scale * p.y).floor()
            + (self.inv_scale * p.z).floor())
            % 2.0
            == 0.0;
        if is_even {
            self.odd.value(uv, p)
        } else {
            self.even.value(uv, p)
        }
    }
}

impl TextureSampler for ImageTexture {
    fn value(&self, uv: &Vec2, _p: &Vec3) -> Vec3 {
        if self.image.is_empty() {
            return Vec3::new(0.0, 1.0, 1.0);
        }
        let u = uv.x.clamp(0.0, 1.0);
        let v = uv.y.clamp(0.0, 1.0);
        let x =
            ((u * self.image.width() as f32) as usize).clamp(0, self.image.width() as usize - 1);
        let y =
            ((v * self.image.height() as f32) as usize).clamp(0, self.image.height() as usize - 1);

        let pixel = &self.image.get_pixel(x as u32, y as u32).0;
        let color_scale = 1.0 / 255.0f32;
        Vec3::new(
            pixel[0] as f32 * color_scale,
            pixel[1] as f32 * color_scale,
            pixel[2] as f32 * color_scale,
        )
    }
}

impl TextureSampler for NoiseTexture {
    fn value(&self, _uv: &Vec2, p: &Vec3) -> Vec3 {
        //let noise_value = 0.5 * (self.noise.noise(*p * self.scale) + 1.0);
        let turb_value = self.noise.turbulence(*p, 7);
        Vec3::one() * turb_value
    }
}
