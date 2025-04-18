use crate::entities::entity::Hittable;
use crate::math::{
    mat3::{dot_v3, Mat3},
    vec3::{dot, Vec3},
};

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f32;
    fn generate(&self) -> Vec3;
}

pub struct SpherePDF;

pub struct CosinePDF {
    uvw: Mat3,
}

impl CosinePDF {
    pub fn new(normal: &Vec3) -> Self {
        Self {
            uvw: Mat3::get_orthonormal_basis(normal),
        }
    }
}

pub struct HittablePDF {
    pub origin: Vec3,
    pub hittable: Box<dyn Hittable>,
}

impl HittablePDF {
    pub fn new(origin: Vec3, hittable: Box<dyn Hittable>) -> Self {
        Self { origin, hittable }
    }
}

impl PDF for SpherePDF {
    fn value(&self, _direction: &Vec3) -> f32 {
        1.0 / (4.0 * std::f32::consts::PI)
    }

    fn generate(&self) -> Vec3 {
        Vec3::random_unit()
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> f32 {
        let cosine = dot(&direction.normalize(), &self.uvw[2]);
        f32::max(0.0, cosine / std::f32::consts::PI)
    }
    fn generate(&self) -> Vec3 {
        dot_v3(
            &self.uvw.transpose(),
            &Vec3::random_cosine_hemisphere_direction(),
        )
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: &Vec3) -> f32 {
        self.hittable.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.hittable.random(&self.origin)
    }
}
