use std::ops;
use crate::math::rand;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
} 

impl Vec3 {
    pub fn new(x: f32, y: f32, z:f32) -> Self {
        Self {x, y, z}
    }

    pub fn normalize(&self) -> Self{
        let length_squared = self.x * self.x + self.y * self.y + self.z * self.z;
        if length_squared > 0.0 {
            let length = length_squared.sqrt();
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
            }
        } else {
            // Return a zero vector if the original vector has zero length
            Self { x: 0.0, y: 0.0, z: 0.0 }
        }
    } 

    pub fn length_squared(&self) -> f32 {
        dot(self, self)
    }

    pub fn length(&self) -> f32 {
        dot(self, self).sqrt()
    }

    pub fn near_zero(&self) -> bool {
        let eps:f32 = 1e-8;
        self.x.abs() < eps && self.y.abs() < eps && self.z.abs() < eps
    }

    pub fn origin() -> Vec3 {
        vec3(0.0, 0.0, 0.0)
    }

    pub fn random() -> Vec3 {
        vec3(rand::rand_f32(), rand::rand_f32(), rand::rand_f32())
    }
    
    pub fn random_range(min: f32, max: f32) -> Vec3 {
        vec3(rand::rand_f32_range(min, max), rand::rand_f32_range(min, max), rand::rand_f32_range(min, max))
    }

    pub fn random_unit() -> Vec3 {
        loop {
            let v = Vec3::random_range(-1.0, 1.0);
            let len_sq = v.length_squared();
            if len_sq > f32::EPSILON && len_sq <= 1.0 {
                return v;
            }
        }
    }
}

impl ops::Deref for Vec3 {
    type Target = [f32;3];

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Vec3 as *const [f32;3])}
    }
}

impl ops::DerefMut for Vec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Vec3 as *mut [f32;3])}
    }
}

impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Add for &Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub<&Vec3> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: &Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub for &Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub<Vec3> for &Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self {
            x: rhs.x * self.x,
            y: rhs.y * self.y,
            z: rhs.z * self.z,
        }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl ops::Mul<&Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Neg for &Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

pub fn dot(a: &Vec3, b: &Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    *v - 2.0 * dot(v,n)*n
}

// A helper function to create a vector
pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_addition() {
        let v1 = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        let v2 = Vec3 { x: 4.0, y: 5.0, z: 6.0 };
        let result = v1 + v2;
        assert_eq!(result, Vec3 { x: 5.0, y: 7.0, z: 9.0 });
    }

    #[test]
    fn test_vec3_addition_zero() {
        let v1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let v2 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        let result = v1 + v2;
        assert_eq!(result, v2);  // Adding zero vector should give back v2
    }

    #[test]
    fn test_vec3_addition_negative() {
        let v1 = Vec3 { x: -1.0, y: -2.0, z: -3.0 };
        let v2 = Vec3 { x: 4.0, y: 5.0, z: 6.0 };
        let result = v1 + v2;
        assert_eq!(result, Vec3 { x: 3.0, y: 3.0, z: 3.0 });
    }

    #[test]
    fn test_vec3_subtraction() {
        let v1 = Vec3 { x: 5.0, y: 7.0, z: 9.0 };
        let v2 = Vec3 { x: 2.0, y: 3.0, z: 4.0 };
        let result = v1 - v2;
        assert_eq!(result, Vec3 { x: 3.0, y: 4.0, z: 5.0 });
    }

    #[test]
    fn test_vec3_subtraction_zero() {
        let v1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let v2 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        let result = v2 - v1;
        assert_eq!(result, v2);
    }

    #[test]
    fn test_vec3_subtraction_negative() {
        let v1 = Vec3 { x: 7.0, y: 8.0, z: 9.0 };
        let v2 = Vec3 { x: 4.0, y: 5.0, z: 6.0 };
        let result = v2 - v1;
        assert_eq!(result, Vec3 { x: -3.0, y: -3.0, z: -3.0 });
    }

    #[test]
    fn test_vec3_mul_scalar_right() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        let result = v * scalar;
        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 4.0);
        assert_eq!(result.z, 6.0);
    }

    #[test]
    fn test_vec3_mul_scalar_left() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        let result = scalar * v;
        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 4.0);
        assert_eq!(result.z, 6.0);
    }

    #[test]
    fn test_vec3_division() {
        // Create a Vec3 with arbitrary values
        let vec = Vec3::new(6.0, 9.0, 12.0);
        
        // Divide by a scalar (f32)
        let result = vec / 3.0;
        
        // Check if the result is as expected (each component divided by 3)
        assert_eq!(result.x, 2.0);  // 6.0 / 3.0 = 2.0
        assert_eq!(result.y, 3.0);  // 9.0 / 3.0 = 3.0
        assert_eq!(result.z, 4.0);  // 12.0 / 3.0 = 4.0
    }

}