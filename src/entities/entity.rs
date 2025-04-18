use crate::aabb::{HasAABB, AABB};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::{vec2::*, vec3::*};
use crate::ray::Ray;
use std::sync::Arc;

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
        self.normal = if self.front_face {
            *out_normal
        } else {
            -*out_normal
        };
    }
}

pub trait Transformable {
    fn translate(&mut self, translation: Vec3);
    fn rotate(&mut self, axis: Vec3, angle: f32);
    //fn scale(&mut self, scale: Vec3);
}

pub trait Hittable: Transformable + HasAABB + Send + Sync {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool;

    fn pdf_value(&self, _origin: &Vec3, _direction: &Vec3) -> f32 {
        0.0
    }

    fn random(&self, _origin: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

pub struct EntityList {
    pub list: Vec<Box<dyn Hittable>>,
    pub bbox: AABB,
}

impl EntityList {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            bbox: AABB::default(),
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
            if entity.hit(
                ray,
                &Interval::new(t_interval.min, closest_so_far),
                &mut tmp_record,
            ) {
                is_hit = true;
                closest_so_far = tmp_record.t;
                *record = tmp_record.clone();
            }
        }
        is_hit
    }
}

impl HasAABB for EntityList {
    fn get_aabb(&self) -> AABB {
        self.bbox
    }

    fn compute_aabb(&self) -> AABB {
        let mut aabb = AABB::default();
        for entity in &self.list {
            aabb = AABB::combine(&aabb, &entity.get_aabb());
        }
        aabb
    }
}

impl Hittable for EntityList {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        self.hit(ray, t_interval, record)
    }
}

impl Transformable for EntityList {
    fn translate(&mut self, translation: Vec3) {
        for entity in &mut self.list {
            entity.translate(translation);
        }
        self.bbox = self.compute_aabb();
    }

    fn rotate(&mut self, axis: Vec3, angle: f32) {
        for entity in &mut self.list {
            entity.rotate(axis, angle);
        }
        self.bbox = self.compute_aabb();
    }
}
