use core::f32;
use std::f32::consts::PI;
use std::sync::Arc;

use crate::aabb::{BoundingBox, AABB};
use crate::entities::entity::{HitRecord, Hittable, Transformable};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::mat3::{dot_v3, Mat3};
use crate::math::rand::rand_f32;
use crate::math::{vec2::*, vec3::*};
use crate::ray::Ray;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Arc<dyn Material>,
    aabb: AABB,
}

impl BoundingBox for Sphere {
    fn get_bounding_box(&self) -> AABB {
        self.aabb
    }

    fn construct_bounding_box(&self) -> AABB {
        let rvec = Vec3::new(self.radius, self.radius, self.radius);
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
        new.aabb = new.construct_bounding_box();
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

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f32 {
        let mut hit_rec = HitRecord::new();
        if !self.hit(
            &Ray::new(*origin, *direction),
            &Interval::new(0.001, f32::MAX),
            &mut hit_rec,
        ) {
            return 0.0;
        }

        let dis_sq = (self.center - origin).length_squared();
        let cos_theta_max = f32::sqrt(1.0 - ((self.radius * self.radius) / dis_sq));
        let solid_angle = 2.0 * f32::consts::PI * (1.0 - cos_theta_max);
        1.0 / solid_angle
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let direction = self.center - origin;
        let dis_sq = direction.length_squared();
        let onb = Mat3::get_orthonormal_basis(&direction);
        dot_v3(&onb.transpose(), &random_in_cone(self.radius, dis_sq))
    }
}

impl Transformable for Sphere {
    fn translate(&mut self, translation: Vec3) {
        self.center += translation;
        self.aabb = self.construct_bounding_box();
    }

    fn rotate(&mut self, _axis: Vec3, _angle: f32) {
        // No rotation for sphere
    }
}

fn random_in_cone(radius: f32, distance_sq: f32) -> Vec3 {
    let r1 = rand_f32();
    let r2 = rand_f32();
    let phi = 2.0 * f32::consts::PI * r1;

    let z = 1.0 + r2 * (f32::sqrt(1.0 - ((radius * radius) / distance_sq)) - 1.0);
    let x = f32::cos(phi) * f32::sqrt(1.0 - (z * z));
    let y = f32::sin(phi) * f32::sqrt(1.0 - (z * z));
    Vec3::new(x, y, z)
}
