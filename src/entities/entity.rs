
use std::sync::Arc;
use crate::aabb::{HasAABB, AABB};
use crate::math::{vec3::*, vec2::*};
use crate::ray::Ray;
use crate::interval::Interval;
use crate::material::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct HitRecord<'a> {
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material: Option<&'a Arc<dyn Material>>,
    pub front_face: bool,
}

impl HitRecord<'_> {
    pub fn new() -> Self {
        Self {
            t: 0.0,
            position: Vec3::zero(),
            material: None,
            normal: Vec3::zero(),
            uv: Vec2::zero(),
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, out_normal: &Vec3) {
        self.front_face = dot(&ray.direction, out_normal) < 0.0;
        self.normal = if self.front_face {*out_normal} else {-*out_normal};
    }
}

pub trait Hittable: HasAABB + Send + Sync {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool;
}

pub struct EntityList {
    pub list: Vec<Box<dyn Hittable>>,
    pub bbox: AABB
}

impl EntityList {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            bbox: AABB::default()
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bbox = AABB::combine(&self.bbox, &object.get_aabb());
        self.list.push(object);
    }

    pub fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        let mut tmp_record = HitRecord::new();
        let mut is_hit = false;
        let mut closest_so_far = t_interval.max;
        for entity in &self.list {
            if entity.hit(ray, &Interval::new(t_interval.min, closest_so_far), &mut tmp_record) {
                is_hit = true;
                closest_so_far = tmp_record.t;
                *record = tmp_record.clone();
            }
        }
        is_hit
    }
}