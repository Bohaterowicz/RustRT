use crate::aabb::AABB;
use crate::interval::Interval;
use crate::math::vec3::Vec3;
#[derive(Debug, Clone)]
pub struct Face {
    pub vert_pos_idx: [usize; 3],
    pub vert_normal_idx: [usize; 3],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vert_position: Vec<Vec3>,
    pub vert_normal: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vert_position: Vec::new(),
            vert_normal: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn construct_aabb(mesh: &Mesh) -> AABB {
        assert!(!mesh.vert_position.is_empty());
        let mut min = mesh.vert_position[0];
        let mut max = mesh.vert_position[0];
        for vert in &mesh.vert_position {
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
}
