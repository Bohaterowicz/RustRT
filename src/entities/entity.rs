use crate::aabb::{BoundingBox, AABB};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::rand::rand_i32_range;
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

    pub fn set_face_normal(&mut self, ray: &Ray, out_normal: &Vec3, adjust_back_normal: bool) {
        self.front_face = dot(&ray.direction, out_normal) + 1e-3 < 0.0;
        self.normal = if self.front_face || !adjust_back_normal {
            *out_normal
        } else {
            -*out_normal
        };
    }
}

pub trait Transformable {
    fn translate(&mut self, _translation: Vec3) {}
    fn rotate(&mut self, _axis: Vec3, _angle: f32) {}
    fn scale(&mut self, _scale: Vec3) {}
}

pub trait Hittable: Transformable + BoundingBox + Send + Sync {
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
        self.bbox = AABB::combine(&self.bbox, &object.get_bounding_box());
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

impl BoundingBox for EntityList {
    fn get_bounding_box(&self) -> AABB {
        self.bbox
    }

    fn construct_bounding_box(&self) -> AABB {
        let mut aabb = AABB::default();
        for entity in &self.list {
            aabb = AABB::combine(&aabb, &entity.get_bounding_box());
        }
        aabb
    }
}

impl Hittable for EntityList {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        self.hit(ray, t_interval, record)
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f32 {
        let weight = 1.0 / self.list.len() as f32;
        let mut sum = 0.0;
        for entity in self.list.as_slice() {
            sum = sum + (weight * entity.pdf_value(origin, direction))
        }
        sum
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let int_size = self.list.len() as i32;
        self.list[rand_i32_range(0, int_size - 1) as usize].random(origin)
    }
}

impl Transformable for EntityList {
    fn translate(&mut self, translation: Vec3) {
        for entity in &mut self.list {
            entity.translate(translation);
        }
        self.bbox = self.construct_bounding_box();
    }

    fn rotate(&mut self, axis: Vec3, angle: f32) {
        for entity in &mut self.list {
            entity.rotate(axis, angle);
        }
        self.bbox = self.construct_bounding_box();
    }
}
