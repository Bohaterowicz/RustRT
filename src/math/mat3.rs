use std::ops::{Index, IndexMut};

use crate::math::vec3::{cross, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Mat3 {
    pub columns: [Vec3; 3],
}

impl Mat3 {
    pub fn identity() -> Self {
        Self {
            columns: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
        }
    }

    pub fn zero() -> Self {
        Self {
            columns: [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
            ],
        }
    }

    pub fn rotation(axis: Vec3, angle: f32) -> Self {
        let [x, y, z] = axis.into();
        let norm = (x * x + y * y + z * z).sqrt();
        let (x, y, z) = (x / norm, y / norm, z / norm);

        let cos = angle.cos();
        let sin = angle.sin();
        let one_minus_cos = 1.0 - cos;

        Self {
            columns: [
                Vec3::new(
                    cos + x * x * one_minus_cos,
                    x * y * one_minus_cos - z * sin,
                    x * z * one_minus_cos + y * sin,
                ),
                Vec3::new(
                    y * x * one_minus_cos + z * sin,
                    cos + y * y * one_minus_cos,
                    y * z * one_minus_cos - x * sin,
                ),
                Vec3::new(
                    z * x * one_minus_cos - y * sin,
                    z * y * one_minus_cos + x * sin,
                    cos + z * z * one_minus_cos,
                ),
            ],
        }
    }

    pub fn get_orthonormal_basis(vec: &Vec3) -> Self {
        let mut onb = Self::zero();
        onb[2] = vec.normalize();
        let a = if onb[2].x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        onb[1] = cross(&onb[2], &a).normalize();
        onb[0] = cross(&onb[2], &onb[1]).normalize();
        onb
    }

    pub fn transpose(&self) -> Self {
        Self {
            columns: [
                Vec3::new(self.columns[0].x, self.columns[1].x, self.columns[2].x),
                Vec3::new(self.columns[0].y, self.columns[1].y, self.columns[2].y),
                Vec3::new(self.columns[0].z, self.columns[1].z, self.columns[2].z),
            ],
        }
    }
}

impl Index<usize> for Mat3 {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.columns[index]
    }
}

impl IndexMut<usize> for Mat3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.columns[index]
    }
}

pub fn dot(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut result = Mat3::identity();
    for i in 0..3 {
        for j in 0..3 {
            result.columns[i][j] = a.columns[i][0] * b.columns[0][j]
                + a.columns[i][1] * b.columns[1][j]
                + a.columns[i][2] * b.columns[2][j];
        }
    }
    result
}

pub fn dot_v3(mat: &Mat3, vec: &Vec3) -> Vec3 {
    Vec3::new(
        mat.columns[0][0] * vec.x + mat.columns[0][1] * vec.y + mat.columns[0][2] * vec.z,
        mat.columns[1][0] * vec.x + mat.columns[1][1] * vec.y + mat.columns[1][2] * vec.z,
        mat.columns[2][0] * vec.x + mat.columns[2][1] * vec.y + mat.columns[2][2] * vec.z,
    )
}
