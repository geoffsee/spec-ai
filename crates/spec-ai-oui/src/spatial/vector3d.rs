//! 3D vector representation for directions and velocities

use std::ops::{Add, Mul, Neg, Sub};

/// A vector in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3D {
    /// Zero vector
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    /// Unit vector pointing right (+X)
    pub const RIGHT: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    /// Unit vector pointing left (-X)
    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    /// Unit vector pointing up (+Y)
    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    /// Unit vector pointing down (-Y)
    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    /// Unit vector pointing forward (+Z)
    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    /// Unit vector pointing backward (-Z)
    pub const BACK: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    /// Create a new 3D vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate the magnitude (length) of the vector
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Calculate the squared magnitude (faster, avoids sqrt)
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Normalize the vector (make it unit length)
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            Self::ZERO
        }
    }

    /// Calculate the dot product with another vector
    pub fn dot(&self, other: &Vector3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculate the cross product with another vector
    pub fn cross(&self, other: &Vector3D) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Linear interpolation between two vectors
    pub fn lerp(&self, other: &Vector3D, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    /// Calculate the angle between two vectors in radians
    pub fn angle(&self, other: &Vector3D) -> f32 {
        let dot = self.dot(other);
        let mags = self.magnitude() * other.magnitude();
        if mags > 0.0 {
            (dot / mags).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        }
    }

    /// Convert to a Point3D
    pub fn to_point(&self) -> super::Point3D {
        super::Point3D::new(self.x, self.y, self.z)
    }
}

impl Add for Vector3D {
    type Output = Vector3D;

    fn add(self, rhs: Self) -> Self::Output {
        Vector3D::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3D::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vector3D {
    type Output = Vector3D;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector3D::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Neg for Vector3D {
    type Output = Vector3D;

    fn neg(self) -> Self::Output {
        Vector3D::new(-self.x, -self.y, -self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_magnitude() {
        let v = Vector3D::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize() {
        let v = Vector3D::new(3.0, 0.0, 0.0);
        let n = v.normalize();
        assert!((n.magnitude() - 1.0).abs() < 0.0001);
        assert_eq!(n.x, 1.0);
    }

    #[test]
    fn test_dot_product() {
        let a = Vector3D::RIGHT;
        let b = Vector3D::UP;
        assert_eq!(a.dot(&b), 0.0); // Perpendicular vectors

        let c = Vector3D::new(1.0, 0.0, 0.0);
        assert_eq!(a.dot(&c), 1.0); // Parallel vectors
    }

    #[test]
    fn test_cross_product() {
        let right = Vector3D::RIGHT;
        let up = Vector3D::UP;
        let result = right.cross(&up);
        // In right-handed system, RIGHT x UP = -BACK = (0, 0, -1)
        // (1,0,0) x (0,1,0) = (0*0-0*1, 0*0-1*0, 1*1-0*0) = (0, 0, 1)
        assert!((result.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_angle() {
        let a = Vector3D::RIGHT;
        let b = Vector3D::UP;
        let angle = a.angle(&b);
        assert!((angle - PI / 2.0).abs() < 0.0001); // 90 degrees
    }
}
