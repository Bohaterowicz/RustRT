use std::sync::Arc;

use super::entity::{HitRecord, Hittable, Transformable};
use crate::aabb::{BoundingBox, AABB};
use crate::bvh::BVH;
use crate::interval::Interval;
use crate::material::Material;
use crate::math::mat3::{dot_v3, Mat3};
use crate::math::vec3::{cross, dot, Vec3};
use crate::mesh::{Face, Mesh};
use crate::ray::Ray;
use std::cell::RefCell;

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

// create a thread local faces vector such that we do not allocate it everytime we hit a mesh. Allocate with a capacity...
thread_local! {
    pub static BVH_FACES_VEC: RefCell<Vec<Face>> = RefCell::new(Vec::with_capacity(64 * 10));
}

const EPS: f32 = 1e-4;

impl Hittable for Object {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        let bvh = self.bvh.as_ref().unwrap();
        let mut is_hit = false;

        BVH_FACES_VEC.with_borrow_mut(|faces| {
            let mut _dbug_v = Vec::new();
            faces.clear();
            if bvh.hit(ray, t_interval, faces) {
                let mut local_interval = *t_interval;
                for face in faces {
                    let positions = &self.mesh.vert_position;
                    let vert0 = positions[face.vert_pos_idx[0]];
                    let vert1 = positions[face.vert_pos_idx[1]];
                    let vert2 = positions[face.vert_pos_idx[2]];

                    let edge1 = vert1 - vert0;
                    let edge2 = vert2 - vert0;
                    let ray_cross_e2 = cross(&ray.direction, &edge2);
                    let det = dot(&edge1, &ray_cross_e2);

                    if det > -EPS && det < EPS {
                        continue;
                    }

                    let inv_det = 1.0 / det;
                    let s = ray.origin - vert0;
                    let u = inv_det * dot(&s, &ray_cross_e2);
                    if !(0.0 - EPS..1.0 + EPS).contains(&u) {
                        continue;
                    }

                    let s_cross_e1 = cross(&s, &edge1);
                    let v = inv_det * dot(&ray.direction, &s_cross_e1);
                    if v < -EPS || (u + v - EPS) > 1.0 {
                        continue;
                    }

                    let t = inv_det * dot(&edge2, &s_cross_e1);
                    _dbug_v.push(t);
                    if t > EPS && local_interval.contains(t) {
                        record.t = t;
                        record.position = ray.at(t);
                        let w = (1.0 - u - v).max(0.0);
                        let n0 = &self.mesh.vert_normal[face.vert_normal_idx[0]];
                        let n1 = &self.mesh.vert_normal[face.vert_normal_idx[1]];
                        let n2 = &self.mesh.vert_normal[face.vert_normal_idx[2]];
                        let interpolated_normal = ((*n0 * w) + (*n1 * u) + (*n2 * v)).normalize();
                        record.set_face_normal(ray, &interpolated_normal, true);
                        record.material = Some(&self.material);
                        local_interval.max = t;
                        is_hit = true;
                    }
                }
            }
        });
        is_hit
    }
}
