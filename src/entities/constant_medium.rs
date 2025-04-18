use crate::{
    aabb::{Axis, HasAABB, AABB},
    entities::entity::{HitRecord, Hittable},
    interval::Interval,
    material::Material,
    math::{rand, vec3::Vec3},
    ray::Ray,
};
use std::sync::Arc;

use super::entity::Transformable;

pub struct ConstantMedium {
    pub boundary: Box<dyn Hittable>,
    pub phase_function: Arc<dyn Material>,
    pub neg_inv_density: f32,
}

impl ConstantMedium {
    pub fn new(
        boundary: Box<dyn Hittable>,
        density: f32,
        phase_function: Arc<dyn Material>,
    ) -> Self {
        Self {
            boundary,
            phase_function,
            neg_inv_density: -1.0 / density,
        }
    }
}

impl HasAABB for ConstantMedium {
    fn get_aabb(&self) -> AABB {
        self.boundary.get_aabb()
    }

    fn compute_aabb(&self) -> AABB {
        self.boundary.compute_aabb()
    }
}

impl Transformable for ConstantMedium {
    fn translate(&mut self, translation: Vec3) {
        self.boundary.translate(translation);
    }

    fn rotate(&mut self, axis: Vec3, angle: f32) {
        self.boundary.rotate(axis, angle);
    }
}

impl Hittable for ConstantMedium {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        let mut rec1 = HitRecord::new();
        if !self.boundary.hit(ray, &Interval::universe(), &mut rec1) {
            return false;
        }

        let mut rec2 = HitRecord::new();
        if !self
            .boundary
            .hit(ray, &Interval::new(rec1.t + 0.0001, f32::MAX), &mut rec2)
        {
            return false;
        }

        if rec1.t < t_interval.min {
            rec1.t = t_interval.min;
        }
        if rec2.t > t_interval.max {
            rec2.t = t_interval.max;
        }

        if rec1.t >= rec2.t {
            return false;
        }
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let distance_inside_boundary = (rec2.t - rec1.t) * ray.direction.length();
        let hit_distance = self.neg_inv_density * f32::ln(rand::rand_f32());

        if hit_distance > distance_inside_boundary {
            return false;
        }

        record.t = rec1.t + hit_distance / ray.direction.length();
        record.position = ray.at(record.t);
        record.set_face_normal(ray, &Vec3::new(1.0, 0.0, 0.0));
        record.front_face = true;
        record.material = Some(&self.phase_function);
        true
    }
}
