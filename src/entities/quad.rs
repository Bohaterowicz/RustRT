use crate::aabb::{HasAABB, AABB};
use crate::entities::entity::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::{
    vec2::Vec2,
    vec3::{cross, dot, Vec3},
};
use std::sync::Arc;

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
}
