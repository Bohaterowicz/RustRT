use std::sync::Arc;

use super::entity::{HitRecord, Hittable, Transformable};
use crate::aabb::{HasAABB, AABB};
use crate::interval::Interval;
use crate::material::Material;
use crate::math::vec3::{cross, dot, Vec3};
use crate::mesh::Mesh;
use crate::ray::Ray;

pub struct Object {
    mesh: Mesh,
    bbox: AABB,
    material: Arc<dyn Material>,
}

impl Object {
    pub fn new(mesh: Mesh, material: Arc<dyn Material>) -> Self {
        let mut obj = Object {
            mesh,
            bbox: AABB::default(),
            material,
        };
        obj.bbox = obj.compute_aabb();
        obj
    }
}

impl HasAABB for Object {
    fn compute_aabb(&self) -> AABB {
        assert!(self.mesh.vert_position.len() > 0);
        let mut min = self.mesh.vert_position[0];
        let mut max = self.mesh.vert_position[0];
        for vert in &self.mesh.vert_position {
            for i in 0..3 {
                if vert[i] > max[i] {
                    max[i] = vert[i];
                } else if vert[i] < min[i] {
                    min[i] = vert[i];
                }
            }
        }
        AABB::new(
            Interval {
                min: min.x,
                max: max.x,
            },
            Interval {
                min: min.y,
                max: max.y,
            },
            Interval {
                min: min.z,
                max: max.z,
            },
        )
    }

    fn get_aabb(&self) -> AABB {
        self.bbox
    }
}

impl Transformable for Object {
    fn translate(&mut self, translation: Vec3) {
        for pos in &mut self.mesh.vert_position {
            *pos = *pos + translation;
        }
        self.bbox = self.compute_aabb();
    }

    fn rotate(&mut self, axis: crate::math::vec3::Vec3, angle: f32) {}
}

impl Hittable for Object {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        if !self.bbox.hit(ray, *t_interval) {
            return false;
        }

        let mut local_interval = *t_interval;
        let mut is_hit = false;

        for face in &self.mesh.faces {
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
                record.is_mesh = true;
                local_interval.max = t;
                is_hit = true;
            }
        }
        is_hit
    }
}
