use std::f32::consts::PI;
use std::sync::Arc;

use crate::aabb::{HasAABB, AABB};
use crate::entities::entity::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::{vec2::*, vec3::*};

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Arc<dyn Material>,
    aabb: AABB,
}

impl HasAABB for Sphere {
    fn get_aabb(&self) -> AABB {
        self.aabb
    }

    fn compute_aabb(&self) -> AABB {
        let rvec = vec3(self.radius, self.radius, self.radius);
        AABB::construct(self.center - rvec, self.center + rvec)
    }
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<dyn Material>) -> Self {
        let mut new = Self {
            center,
            radius,
            material,
            aabb: AABB::default(),
        };
        new.aabb = new.compute_aabb();
        new
    }

    pub fn get_uv(p: &Vec3) -> Vec2 {
        let theta = f32::acos(-p.y);
        let phi = f32::atan2(-p.z, p.x) + PI;
        let u = phi / (2.0 * PI);
        let v = theta / PI;
        Vec2::new(u, v)
    }
}

impl Hittable for Sphere {
    fn hit<'a>(
        &'a self,
        ray: &crate::ray::Ray,
        t_interval: &Interval,
        record: &mut HitRecord<'a>,
    ) -> bool {
        let ray_sphere_vec = self.center - ray.origin;
        let a = dot(&ray.direction, &ray.direction);
        //let b = -2.0 * dot(&ray.direction, &ray_sphere_vec);
        let h = dot(&ray.direction, &ray_sphere_vec);
        let c = dot(&ray_sphere_vec, &ray_sphere_vec) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            false
        } else {
            let d_sqrt = discriminant.sqrt();
            let mut root = (h - d_sqrt) / a;

            if !t_interval.surrounds(root) {
                root = (h + d_sqrt) / a;
                if !t_interval.surrounds(root) {
                    return false;
                }
            }

            record.t = root;
            record.position = ray.at(root);
            let outward_normal = (record.position - self.center).normalize();
            record.set_face_normal(ray, &outward_normal);
            record.material = Some(&self.material);
            record.uv = Sphere::get_uv(&outward_normal);
            true
        }
    }
}
