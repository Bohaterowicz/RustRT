use std::sync::Arc;

use super::entity::{HitRecord, Hittable, Transformable};
use crate::aabb::{BoundingBox, AABB};
use crate::bvh::BVH;
use crate::interval::Interval;
use crate::material::Material;
use crate::math::mat3::{dot_v3, Mat3};
use crate::math::vec3::{cross, dot, Vec3};
use crate::mesh::Mesh;
use crate::ray::Ray;

#[derive(Debug, Clone)]
pub struct Object {
    pub mesh: Mesh,
    bbox: AABB,
    pub material: Arc<dyn Material>,
    pub bvh: Option<BVH>,
}

impl Object {
    pub fn new(mesh: Mesh, material: Arc<dyn Material>) -> Self {
        let mut obj = Object {
            mesh,
            bbox: AABB::default(),
            bvh: None,
            material,
        };
        obj.bbox = obj.construct_bounding_box();
        obj
    }
}

impl BoundingBox for Object {
    fn construct_bounding_box(&self) -> AABB {
        Mesh::construct_aabb(&self.mesh)
    }

    fn get_bounding_box(&self) -> AABB {
        self.bbox
    }
}

impl Transformable for Object {
    fn translate(&mut self, translation: Vec3) {
        for pos in &mut self.mesh.vert_position {
            *pos += translation;
        }
        self.bbox = self.construct_bounding_box();
    }

    fn rotate(&mut self, axis: Vec3, angle: f32) {
        let rot = Mat3::rotation(axis, angle);
        for pos in &mut self.mesh.vert_position {
            *pos = dot_v3(&rot, pos);
        }

        for normal in &mut self.mesh.vert_normal {
            *normal = dot_v3(&rot, normal);
        }

        self.bbox = self.construct_bounding_box();
    }

    fn scale(&mut self, scale: Vec3) {
        for pos in &mut self.mesh.vert_position {
            pos.x *= scale.x;
            pos.y *= scale.y;
            pos.z *= scale.z;
        }
        self.bbox = self.construct_bounding_box();
    }
}

impl Hittable for Object {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        /*
        if !self.bbox.hit(ray, *t_interval).0 {
            return false;
        }
        */
        let mut faces = Vec::new();
        let bvh = self.bvh.as_ref().unwrap();
        if !bvh.hit(ray, t_interval, &mut faces) {
            return false;
        }

        let mut local_interval = *t_interval;
        let mut is_hit = false;
        let faces = faces; //&self.mesh.faces;

        for face in faces {
            let positions = &self.mesh.vert_position;
            let edge1 = positions[face.vert_pos_idx[1]] - positions[face.vert_pos_idx[0]];
            let edge2 = positions[face.vert_pos_idx[2]] - positions[face.vert_pos_idx[0]];
            let ray_cross_e2 = cross(&ray.direction, &edge2);
            let det = dot(&edge1, &ray_cross_e2);

            if det > -f32::EPSILON && det < f32::EPSILON {
                continue;
            }

            let inv_det = 1.0 / det;
            let s = ray.origin - positions[face.vert_pos_idx[0]];
            let u = inv_det * dot(&s, &ray_cross_e2);
            if u < 0.0 || u > 1.0 {
                continue;
            }

            let s_cross_e1 = cross(&s, &edge1);
            let v = inv_det * dot(&ray.direction, &s_cross_e1);
            if v < 0.0 || (u + v) > 1.0 {
                continue;
            }

            let t = inv_det * dot(&edge2, &s_cross_e1);
            if t > f32::EPSILON && local_interval.contains(t) {
                record.t = t;
                record.position = ray.at(t);
                record.set_face_normal(ray, &self.mesh.vert_normal[face.vert_normal_idx[0]]);
                record.material = Some(&self.material);
                local_interval.max = t;
                is_hit = true;
            }
        }
        is_hit
    }
}
