use crate::math::vec3::Vec3;

pub struct Face {
    pub vert_pos_idx: [usize; 3],
    pub vert_normal_idx: [usize; 3],
}

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
}
