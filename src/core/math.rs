use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn rotate(&self, angle: f32) -> Self {
        let rad = angle.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let x = self.x * cos - self.y * sin;
        let y = self.x * sin + self.y * cos;
        Self { x, y }
    }

    pub fn angle(&self) -> f32 {
        let angle = self.y.atan2(self.x).to_degrees();
        if angle < 0.0 { angle + 360.0 } else { angle }
    }

    pub fn dist(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Self) -> Self::Output {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Self) -> Self::Output {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Vector2,
    pub end: Vector2,
}

impl Line {
    pub fn new(start: Vector2, end: Vector2) -> Self {
        Self { start, end }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::Vector2;

    fn assert_eq_f32(actual: f32, expected: f32) {
        let epsilon = 0.000001;
        assert!((actual - expected).abs() < epsilon);
    }

    fn assert_eq_vec2(actual: Vector2, expected: Vector2) {
        assert_eq_f32(actual.x, expected.x);
        assert_eq_f32(actual.y, expected.y);
    }

    #[test]
    fn test_vec2_rotate() {
        for (angle, expected) in [
            (0.0, Vector2::new(1.0, 0.0)),
            (90.0, Vector2::new(0.0, 1.0)),
            (180.0, Vector2::new(-1.0, 0.0)),
            (270.0, Vector2::new(0.0, -1.0)),
        ] {
            let actual = Vector2::new(1.0, 0.0).rotate(angle);
            assert_eq_vec2(actual, expected);
        }
    }

    #[test]
    fn test_vec2_angle() {
        for (vec, expected) in [
            (Vector2::new(1.0, 0.0), 0.0),
            (Vector2::new(0.0, 1.0), 90.0),
            (Vector2::new(-1.0, 0.0), 180.0),
            (Vector2::new(0.0, -1.0), 270.0),
        ] {
            let actual = vec.angle();
            assert_eq_f32(actual, expected);
        }
    }

    #[test]
    fn test_vec2_dist() {
        let v1 = Vector2::new(0.0, 0.0);
        let v2 = Vector2::new(3.0, 4.0);
        assert_eq_f32(v1.dist(&v2), 5.0);
    }
}
