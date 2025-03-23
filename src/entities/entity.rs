
use std::sync::Arc;
use crate::math::vec3::*;
use crate::ray::Ray;
use crate::interval::Interval;
use crate::material::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct HitRecord<'a> {
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub material: Option<&'a Arc<dyn Material>>,
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new() -> Self {
        Self {
            t: 0.0,
            position: vec3(0.0, 0.0, 0.0),
            material: None,
            normal: vec3(0.0, 0.0, 0.0),
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, out_normal: &Vec3) {
        self.front_face = dot(&ray.direction, out_normal) < 0.0;
        self.normal = if self.front_face {*out_normal} else {-*out_normal};
    }
}

pub trait Hittable: Send + Sync {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool;
}

pub struct EntityList {
    pub list: Vec<Box<dyn Hittable>>,
}

impl EntityList {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
        }
    }
}

impl Hittable for EntityList {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
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