use std::cmp::Ordering;

use crate::{
    aabb::{Axis, BoundingBox, AABB},
    entities::entity::{EntityList, HitRecord, Hittable, Transformable},
    interval::Interval,
    math::vec3::Vec3,
    mesh::{Face, Mesh},
    ray::Ray,
};

#[derive(Debug, Clone)]
enum BVHNode {
    Branch { left: Box<BVH>, right: Box<BVH> },
    Leaf(Vec<Face>),
}

#[derive(Debug, Clone)]
pub struct BVH {
    tree: BVHNode,
    pub bbox: AABB,
}

impl BVH {
    pub fn new(mesh: &Mesh) -> Self {
        Self::construct_bvh(mesh)
    }

    fn construct_bvh(mesh: &Mesh) -> Self {
        Self::construct_bvh_nodes(mesh, mesh.faces.clone(), 0)
    }

    fn construct_aabb_from_indicies(mesh: &Mesh, tris: &Vec<Face>) -> AABB {
        assert!(!tris.is_empty());
        let mut aabb = AABB::default();
        let mut x = Interval::empty();
        let mut y = Interval::empty();
        let mut z = Interval::empty();

        let positions = &mesh.vert_position;
        for tri in tris {
            for i in 0..3 {
                let pos = &positions[tri.vert_pos_idx[i]];
                x.include(pos.x);
                y.include(pos.y);
                z.include(pos.z);
            }
        }

        aabb.x = x;
        aabb.y = y;
        aabb.z = z;
        aabb
    }

    fn construct_bvh_nodes(mesh: &Mesh, tris: Vec<Face>, depth: i32) -> Self {
        let bbox = Self::construct_aabb_from_indicies(mesh, &tris);
        let tri_count = tris.len();

        // Base case: Create a leaf node if the triangle count is below the threshold
        if tri_count <= 128 || depth == 10 {
            return Self {
                tree: BVHNode::Leaf(tris),
                bbox,
            };
        }

        // Determine the splitting axis (longest axis of the bounding box)
        let axis = bbox.get_longest_axis();
        let axis_idx = Axis::to_idx(axis);

        // Compute the midpoint of the triangles along the splitting axis
        let positions = &mesh.vert_position;
        let mut midpoints: Vec<f32> = tris
            .iter()
            .map(|tri| {
                let v1 = positions[tri.vert_pos_idx[0]][axis_idx];
                let v2 = positions[tri.vert_pos_idx[1]][axis_idx];
                let v3 = positions[tri.vert_pos_idx[2]][axis_idx];
                (v1 + v2 + v3) / 3.0 // Compute the average position (midpoint)
            })
            .collect();

        midpoints.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = midpoints[tri_count / 2];

        // Split triangles into left and right groups
        let (mut tris_left, mut tris_right): (Vec<Face>, Vec<Face>) =
            tris.clone().into_iter().partition(|tri| {
                let v1 = positions[tri.vert_pos_idx[0]][axis_idx];
                let v2 = positions[tri.vert_pos_idx[1]][axis_idx];
                let v3 = positions[tri.vert_pos_idx[2]][axis_idx];
                let max = v1.max(v2).max(v3);
                max < median // Assign to left if the triangle is entirely on the left
            });

        // Handle edge case: If one group is empty, move a triangle to balance
        if tris_left.is_empty() {
            tris_left.push(tris_right.pop().unwrap());
        } else if tris_right.is_empty() {
            tris_right.push(tris_left.pop().unwrap());
        }

        // Recursively construct BVH nodes for left and right groups
        let depth = depth + 1;
        Self {
            tree: BVHNode::Branch {
                left: Box::new(Self::construct_bvh_nodes(mesh, tris_left, depth)),
                right: Box::new(Self::construct_bvh_nodes(mesh, tris_right, depth)),
            },
            bbox,
        }
    }

    pub fn hit(&self, ray: &Ray, t_interval: &Interval, faces: &mut Vec<Face>) -> bool {
        // Check if the ray intersects the bounding box
        if !self.bbox.hit(ray, *t_interval) {
            return false;
        }

        match &self.tree {
            // If this is a leaf node, return the triangles
            BVHNode::Leaf(leaf) => {
                let mut leaf = leaf.clone();
                faces.append(&mut leaf);
                true
            }

            // If this is a branch node, check the left and right children
            BVHNode::Branch { left, right } => {
                // Check the left child
                let hit_left = left.hit(ray, t_interval, faces);
                // Check the right child
                let hit_right = right.hit(ray, t_interval, faces);
                hit_left || hit_right
            }
        }
    }
}
