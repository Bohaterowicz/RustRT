use crate::{interval::Interval, math::vec3::Vec3, ray::Ray};

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut bbox = AABB { x, y, z };
        bbox.pad_to_minimum_size();
        bbox
    }

    pub fn construct(a: Vec3, b: Vec3) -> AABB {
        let mut bbox = AABB {
            x: if a[0] <= b[0] {
                Interval::new(a[0], b[0])
            } else {
                Interval::new(b[0], a[0])
            },
            y: if a[1] <= b[1] {
                Interval::new(a[1], b[1])
            } else {
                Interval::new(b[1], a[1])
            },
            z: if a[2] <= b[2] {
                Interval::new(a[2], b[2])
            } else {
                Interval::new(b[2], a[2])
            },
        };
        bbox.pad_to_minimum_size();
        bbox
    }

    pub fn combine(a: &AABB, b: &AABB) -> AABB {
        AABB {
            x: Interval::combine(&a.x, &b.x),
            y: Interval::combine(&a.y, &b.y),
            z: Interval::combine(&a.z, &b.z),
        }
    }

    pub fn get_axis(&self, axis: Axis) -> Interval {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn hit(&self, ray: &Ray, ray_t: Interval) -> bool {
        let mut ray_t = ray_t;
        let ray_origin = ray.origin;
        let ray_direction = ray.direction;

        for axis in Axis::ALL {
            let idx = axis as usize;
            let ax = self.get_axis(axis);
            let dir_inv = 1.0 / ray_direction[idx];

            let t0 = (ax.min - ray_origin[idx]) * dir_inv;
            let t1 = (ax.max - ray_origin[idx]) * dir_inv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }

    pub fn get_longest_axis(&self) -> Axis {
        let x_size = self.x.size();
        let y_size = self.y.size();
        let z_size = self.z.size();

        if x_size > y_size && x_size > z_size {
            Axis::X
        } else if y_size > x_size && y_size > z_size {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    fn pad_to_minimum_size(&mut self) {
        let min_delta = 0.0001;
        if self.x.size() < min_delta {
            self.x = self.x.expand(min_delta);
        }
        if self.y.size() < min_delta {
            self.y = self.y.expand(min_delta);
        }
        if self.z.size() < min_delta {
            self.z = self.z.expand(min_delta);
        }
    }
}

pub trait HasAABB {
    fn get_aabb(&self) -> AABB;
    fn compute_aabb(&self) -> AABB;
}
