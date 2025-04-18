use crate::aabb::{HasAABB, AABB};
use crate::entities::entity::{EntityList, HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::rand::rand_f32;
use crate::math::{
    mat3::{dot_v3, Mat3},
    vec2::Vec2,
    vec3::{cross, dot, Vec3},
};
use crate::ray::Ray;
use std::sync::Arc;

use super::entity::Transformable;

#[derive(Debug, Clone)]
pub struct Quad {
    pub q: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub normal: Vec3,
    pub d: f32,
    pub w: Vec3,
    pub material: Arc<dyn Material>,
    aabb: AABB,
    area: f32,
}

impl Quad {
    pub fn new(q: Vec3, u: Vec3, v: Vec3, material: Arc<dyn Material>) -> Self {
        let n = cross(&u, &v);
        let mut new = Self {
            q,
            u,
            v,
            normal: n.normalize(),
            d: 0.0,
            w: Vec3::zero(),
            material,
            aabb: AABB::default(),
            area: n.length(),
        };
        new.aabb = new.compute_aabb();
        new.d = dot(&new.normal, &new.q);
        new.w = n / dot(&n, &n);
        new
    }

    pub fn is_interior(alpha: f32, beta: f32, record: &mut HitRecord) -> bool {
        let interval = Interval::new(0.0, 1.0);
        if !interval.contains(alpha) || !interval.contains(beta) {
            return false;
        }
        record.uv = Vec2::new(alpha, beta);
        true
    }
}

impl HasAABB for Quad {
    fn get_aabb(&self) -> AABB {
        self.aabb
    }

    fn compute_aabb(&self) -> AABB {
        let p0 = self.q;
        let p1 = self.q + self.u;
        let p2 = self.q + self.v;
        let p3 = self.q + self.u + self.v;

        let (min, max) = [p0, p1, p2, p3].iter().fold((p0, p0), |(min, max), p| {
            (
                Vec3::new(
                    f32::min(min.x, p.x),
                    f32::min(min.y, p.y),
                    f32::min(min.z, p.z),
                ),
                Vec3::new(
                    f32::max(max.x, p.x),
                    f32::max(max.y, p.y),
                    f32::max(max.z, p.z),
                ),
            )
        });
        AABB::construct(min, max)
    }
}

impl Hittable for Quad {
    fn hit<'a>(
        &'a self,
        ray: &crate::ray::Ray,
        t_interval: &crate::interval::Interval,
        record: &mut HitRecord<'a>,
    ) -> bool {
        let denom = dot(&self.normal, &ray.direction);
        if denom.abs() < 1e-6 {
            return false;
        }

        let t = (self.d - dot(&self.normal, &ray.origin)) / denom;
        if !t_interval.contains(t) {
            false
        } else {
            let hit_p = ray.at(t);
            let planar_hit_vector = hit_p - self.q;
            let alpha = dot(&self.w, &cross(&planar_hit_vector, &self.v));
            let beta = dot(&self.w, &cross(&self.u, &planar_hit_vector));

            if !Quad::is_interior(alpha, beta, record) {
                return false;
            }

            record.t = t;
            record.position = hit_p;
            record.material = Some(&self.material);
            record.set_face_normal(ray, &self.normal);
            true
        }
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f32 {
        let mut hit_rec = HitRecord::new();
        if self.hit(
            &Ray::new(*origin, *direction),
            &Interval::new(0.001, f32::MAX),
            hit_rec,
        ) {
            let dist_sq = hit_rec.t * hit_rec.t * direction.length_squared();
            let cosine = f32::abs(dot(direction, &hit_rec.normal) / direction.length());
            dist_sq / (cosine * self.area)
        } else {
            0.0
        }
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let p = self.q + self.u * rand_f32() + self.v * rand_f32();
        p - *origin
    }
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (std::f32::consts::PI / 180.0)
}

impl Transformable for Quad {
    fn translate(&mut self, translation: Vec3) {
        self.q += translation;
        self.d = dot(&self.normal, &self.q);
        self.aabb = self.compute_aabb();
    }

    fn rotate(&mut self, axis: Vec3, angle: f32) {
        let rotation_matrix = Mat3::rotation(axis, degrees_to_radians(angle));
        self.q = dot_v3(&rotation_matrix, &self.q);
        self.u = dot_v3(&rotation_matrix, &self.u);
        self.v = dot_v3(&rotation_matrix, &self.v);

        let n = cross(&self.u, &self.v);
        self.normal = n.normalize();
        self.d = dot(&self.normal, &self.q);
        self.w = n / dot(&n, &n);
        self.aabb = self.compute_aabb();
    }
}

pub fn create_box(p1: Vec3, p2: Vec3, material: Arc<dyn Material>) -> EntityList {
    let mut list = EntityList::new();

    let min = Vec3::new(
        f32::min(p1.x, p2.x),
        f32::min(p1.y, p2.y),
        f32::min(p1.z, p2.z),
    );

    let max = Vec3::new(
        f32::max(p1.x, p2.x),
        f32::max(p1.y, p2.y),
        f32::max(p1.z, p2.z),
    );

    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    list.add(Box::new(Quad::new(
        Vec3::new(min.x, min.y, max.z),
        dx,
        dy,
        material.clone(),
    )));
    list.add(Box::new(Quad::new(
        Vec3::new(max.x, min.y, max.z),
        -dz,
        dy,
        material.clone(),
    )));
    list.add(Box::new(Quad::new(
        Vec3::new(max.x, min.y, min.z),
        -dx,
        dy,
        material.clone(),
    )));
    list.add(Box::new(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dz,
        dy,
        material.clone(),
    )));
    list.add(Box::new(Quad::new(
        Vec3::new(min.x, max.y, max.z),
        dx,
        -dz,
        material.clone(),
    )));
    list.add(Box::new(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dx,
        dz,
        material.clone(),
    )));

    list
}
