
use rand::Rng;

pub fn rand_f32() -> f32 {
    rand::thread_rng().gen()
}

pub fn rand_f32_range(min: f32, max: f32) -> f32 {
    min + (max-min)*rand_f32()
}