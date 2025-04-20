use crate::entities::entity::Hittable;
use crate::math::rand::rand_f32;
use crate::math::{
    mat3::{dot_v3, Mat3},
    vec3::{dot, Vec3},
};
use crate::ray::Ray;

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f32;
    fn generate(&self) -> Vec3;
}

pub struct EmptyPDF;

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

pub struct HittablePDF<'a> {
    pub origin: Vec3,
    pub hittable: &'a dyn Hittable,
}

impl<'a> HittablePDF<'a> {
    pub fn new(origin: Vec3, hittable: &'a dyn Hittable) -> Self {
        Self { origin, hittable }
    }
}

pub struct MixedPDF<'a> {
    pub mix: [&'a dyn PDF; 2],
}

impl<'a> MixedPDF<'a> {
    pub fn new(a: &'a dyn PDF, b: &'a dyn PDF) -> Self {
        Self { mix: [a, b] }
    }
}

impl PDF for EmptyPDF {
    fn value(&self, _direction: &Vec3) -> f32 {
        0.0
    }

    fn generate(&self) -> Vec3 {
        Vec3::zero()
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

impl<'a> PDF for HittablePDF<'a> {
    fn value(&self, direction: &Vec3) -> f32 {
        self.hittable.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.hittable.random(&self.origin)
    }
}

impl<'a> PDF for MixedPDF<'a> {
    fn value(&self, direction: &Vec3) -> f32 {
        0.5 * self.mix[0].value(direction) + 0.5 * self.mix[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        if rand_f32() > 0.5 {
            self.mix[0].generate()
        } else {
            self.mix[1].generate()
        }
    }
}
