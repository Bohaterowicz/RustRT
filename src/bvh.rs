use std::cmp::Ordering;

use crate::{
    aabb::{Axis, HasAABB, AABB},
    entities::entity::{EntityList, HitRecord, Hittable},
    interval::Interval,
    math::rand::rand_i32_range,
    ray::Ray,
};

enum BVHNode {
    Branch { left: Box<BVH>, right: Box<BVH> },
    Leaf(Box<dyn Hittable>),
}

pub struct BVH {
    tree: BVHNode,
    pub bbox: AABB,
}

impl HasAABB for BVH {
    fn get_aabb(&self) -> AABB {
        self.bbox
    }

    fn compute_aabb(&self) -> AABB {
        match &self.tree {
            BVHNode::Leaf(leaf) => leaf.get_aabb(),
            BVHNode::Branch { left, right } => AABB::combine(&right.get_aabb(), &left.get_aabb()),
        }
    }
}

impl Hittable for BVH {
    fn hit<'a>(&'a self, ray: &Ray, t_interval: &Interval, record: &mut HitRecord<'a>) -> bool {
        if !self.bbox.hit(ray, *t_interval) {
            false
        } else {
            match &self.tree {
                BVHNode::Leaf(leaf) => leaf.hit(ray, t_interval, record),
                BVHNode::Branch { left, right } => {
                    let hit_left = left.hit(ray, t_interval, record);
                    let hit_right = right.hit(
                        ray,
                        &Interval::new(
                            t_interval.min,
                            if hit_left { record.t } else { t_interval.max },
                        ),
                        record,
                    );
                    hit_left || hit_right
                }
            }
        }
    }
}

impl BVH {
    pub fn new(entities: EntityList) -> Self {
        Self::compute_bvh(entities.list)
    }

    fn compute_bvh(mut entities: Vec<Box<dyn Hittable>>) -> Self {
        fn compare(axis: Axis) -> impl FnMut(&Box<dyn Hittable>, &Box<dyn Hittable>) -> Ordering {
            move |a, b| {
                let a_bbox = &a.get_aabb();
                let b_bbox = &b.get_aabb();

                let a_int = a_bbox.get_axis(axis).min;
                let b_int = b_bbox.get_axis(axis).min;
                a_int.partial_cmp(&b_int).unwrap()
            }
        }

        let span = entities.len();
        match span {
            0 => panic!("No elements..."),
            1 => {
                let leaf = entities.pop().unwrap();
                let bbox = leaf.get_aabb();
                BVH {
                    tree: BVHNode::Leaf(leaf),
                    bbox,
                }
            }
            _ => {
                let mut bbox = AABB::default();
                for entity in &entities {
                    bbox = AABB::combine(&bbox, &entity.get_aabb());
                }
                let axis = bbox.get_longest_axis();
                entities.sort_unstable_by(compare(axis));
                let right = Self::compute_bvh(entities.drain(span / 2..).collect());
                let left = Self::compute_bvh(entities);
                let bbox = AABB::combine(&left.bbox, &right.bbox);
                BVH {
                    tree: BVHNode::Branch {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    bbox,
                }
            }
        }
    }
}
