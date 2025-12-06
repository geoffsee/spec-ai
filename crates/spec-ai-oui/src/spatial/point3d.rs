//! 3D point representation

use std::ops::{Add, Mul, Sub};

/// A point in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    /// Origin point (0, 0, 0)
    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Create a new 3D point
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate the Euclidean distance to another point
    pub fn distance(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate the squared distance (faster, avoids sqrt)
    pub fn distance_squared(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Linear interpolation between two points
    pub fn lerp(&self, other: &Point3D, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    /// Convert to a Vector3D (from origin)
    pub fn to_vector(&self) -> super::Vector3D {
        super::Vector3D::new(self.x, self.y, self.z)
    }
}

impl Add<super::Vector3D> for Point3D {
    type Output = Point3D;

    fn add(self, rhs: super::Vector3D) -> Self::Output {
        Point3D::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point3D {
    type Output = super::Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        super::Vector3D::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Point3D {
    type Output = Point3D;

    fn mul(self, rhs: f32) -> Self::Output {
        Point3D::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin() {
        let p = Point3D::ORIGIN;
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
        assert_eq!(p.z, 0.0);
    }

    #[test]
    fn test_distance() {
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp() {
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(10.0, 10.0, 10.0);
        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 5.0);
        assert_eq!(mid.z, 5.0);
    }
}
