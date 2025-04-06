
#[derive(Debug, Default, Clone, Copy)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn combine(a: &Interval, b: &Interval) -> Interval {
        Interval { 
            min: f32::min(a.min, b.min),
            max: f32::max(a.max, b.max)
        }
    }

    pub fn empty() -> Self {
        Self {
            min: f32::MAX,
            max: f32::MIN,
        }
    }

    pub fn universe() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn contains(&self, value: f32) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn expand(&self, delta: f32) -> Interval {
        let padding = delta / 2.0;
        Interval { min: self.min-padding, max: self.max+padding }
    }

    pub fn surrounds(&self, value: f32) -> bool {
        self.min < value && value < self.max
    }

    pub fn clamp(&self, value: f32) -> f32 {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
}
