

use std::sync::Arc;

use crate::entities::entity::{Hittable, HitRecord};
use crate::material::Material;
use crate::math::vec3::*;
use crate::interval::Interval;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<dyn Material>) -> Self {
        Self {center, radius, material}
    }
}

impl Hittable for Sphere {
    fn hit<'a>(&'a self, ray: &crate::ray::Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        let ray_sphere_vec = self.center - ray.origin;
        let a = dot(&ray.direction, &ray.direction);
        //let b = -2.0 * dot(&ray.direction, &ray_sphere_vec);
        let h = dot(&ray.direction, &ray_sphere_vec);
        let c = dot(&ray_sphere_vec, &ray_sphere_vec) - self.radius*self.radius;
        let discriminant = h*h - a*c;
        if discriminant < 0.0 {
            false
        } else {
            let d_sqrt = discriminant.sqrt();
            let mut root = (h - d_sqrt) / a;

            if !t_interval.surrounds(root) {
                root = (h + d_sqrt) / a;
                if !t_interval.surrounds(root) {
                    return false
                }
            }

            record.t = root;
            record.position = ray.at(root);
            record.set_face_normal(ray,  &((record.position - self.center) / self.radius));
            record.material = Some(&self.material);
            true
        }
    }
}