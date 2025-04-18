use crate::math::vec3::*;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }

    pub fn default() -> Self {
        Self {
            origin: Vec3::origin(),
            direction: Vec3::origin(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_creation_and_at() {
        // Create a ray with an origin and direction
        let origin = Vec3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(0.0, 1.0, 0.0); // direction is along the y-axis
        let ray = Ray::new(origin, direction);

        // Check that the ray's origin is correctly set
        assert_eq!(ray.origin, origin);
        // Check that the direction is normalized
        assert_eq!(ray.direction, Vec3::new(0.0, 1.0, 0.0)); // already normalized

        // Test the `at` function at t = 0, which should return the origin
        let point_at_t_0 = ray.at(0.0);
        assert_eq!(point_at_t_0, origin);

        // Test the `at` function at t = 5, which should move the ray 5 units along the y-axis
        let point_at_t_5 = ray.at(5.0);
        assert_eq!(point_at_t_5, Vec3::new(1.0, 7.0, 3.0)); // origin + 5 * direction
    }

    #[test]
    fn test_zero_direction_normalization() {
        // Handle edge case where direction is a zero vector
        let origin = Vec3::zero();
        let direction = Vec3::zero(); // zero direction vector
        let ray = Ray::new(origin, direction);

        assert_eq!(ray.direction, Vec3::zero());
    }

    #[test]
    fn test_ray_at_different_times() {
        // Test with a ray along a non-orthogonal direction
        let origin = Vec3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(1.0, 0.0, 0.0); // arbitrary direction
        let ray = Ray::new(origin, direction);

        // Test `at` at different `t` values
        let point_at_t_1 = ray.at(1.0);
        assert_eq!(point_at_t_1, Vec3::new(2.0, 2.0, 3.0)); // origin + direction

        let point_at_t_2 = ray.at(2.0);
        assert_eq!(point_at_t_2, Vec3::new(3.0, 2.0, 3.0)); // origin + 2 * direction
    }
}
