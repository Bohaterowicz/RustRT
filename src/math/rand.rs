
use rand::Rng;

pub fn rand_f32() -> f32 {
    rand::thread_rng().gen()
}

pub fn rand_f32_range(min: f32, max: f32) -> f32 {
    min + (max-min)*rand_f32()
}

pub fn rand_i32_range(min: i32, max: i32) -> i32 {
    rand_f32_range(min as f32, (max+1) as f32) as i32
}
