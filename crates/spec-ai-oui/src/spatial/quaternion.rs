//! Quaternion representation for 3D rotations

use super::Vector3D;
use std::ops::Mul;

/// A quaternion for representing 3D rotations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    /// Identity quaternion (no rotation)
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    /// Create a new quaternion
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Create a quaternion from axis-angle representation
    pub fn from_axis_angle(axis: Vector3D, angle_radians: f32) -> Self {
        let half_angle = angle_radians / 2.0;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();
        let normalized = axis.normalize();

        Self {
            x: normalized.x * sin_half,
            y: normalized.y * sin_half,
            z: normalized.z * sin_half,
            w: cos_half,
        }
    }

    /// Create a quaternion from Euler angles (in radians)
    /// Order: yaw (Y), pitch (X), roll (Z)
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();

        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    /// Get the magnitude of the quaternion
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Normalize the quaternion
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
                w: self.w / mag,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// Get the conjugate of the quaternion
    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// Get the inverse of the quaternion
    pub fn inverse(&self) -> Self {
        let mag_sq = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;
        if mag_sq > 0.0 {
            let inv_mag_sq = 1.0 / mag_sq;
            Self {
                x: -self.x * inv_mag_sq,
                y: -self.y * inv_mag_sq,
                z: -self.z * inv_mag_sq,
                w: self.w * inv_mag_sq,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// Rotate a vector by this quaternion
    pub fn rotate_vector(&self, v: Vector3D) -> Vector3D {
        let q_vec = Vector3D::new(self.x, self.y, self.z);
        let uv = q_vec.cross(&v);
        let uuv = q_vec.cross(&uv);
        v + (uv * self.w + uuv) * 2.0
    }

    /// Spherical linear interpolation between two quaternions
    pub fn slerp(&self, other: &Quaternion, t: f32) -> Self {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        // If the dot product is negative, negate one quaternion to take the shorter path
        let (other, dot) = if dot < 0.0 {
            (Quaternion::new(-other.x, -other.y, -other.z, -other.w), -dot)
        } else {
            (*other, dot)
        };

        // If quaternions are very close, use linear interpolation
        if dot > 0.9995 {
            return Quaternion::new(
                self.x + t * (other.x - self.x),
                self.y + t * (other.y - self.y),
                self.z + t * (other.z - self.z),
                self.w + t * (other.w - self.w),
            ).normalize();
        }

        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Quaternion::new(
            s0 * self.x + s1 * other.x,
            s0 * self.y + s1 * other.y,
            s0 * self.z + s1 * other.z,
            s0 * self.w + s1 * other.w,
        )
    }

    /// Get the forward vector (where this rotation points)
    pub fn forward(&self) -> Vector3D {
        self.rotate_vector(Vector3D::FORWARD)
    }

    /// Get the right vector
    pub fn right(&self) -> Vector3D {
        self.rotate_vector(Vector3D::RIGHT)
    }

    /// Get the up vector
    pub fn up(&self) -> Vector3D {
        self.rotate_vector(Vector3D::UP)
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        Quaternion::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity() {
        let q = Quaternion::IDENTITY;
        let v = Vector3D::FORWARD;
        let rotated = q.rotate_vector(v);
        assert!((rotated.x - v.x).abs() < 0.0001);
        assert!((rotated.y - v.y).abs() < 0.0001);
        assert!((rotated.z - v.z).abs() < 0.0001);
    }

    #[test]
    fn test_90_degree_rotation() {
        // Rotate around Y axis by 90 degrees
        let q = Quaternion::from_axis_angle(Vector3D::UP, PI / 2.0);
        let v = Vector3D::FORWARD;
        let rotated = q.rotate_vector(v);
        // FORWARD should become RIGHT (approximately)
        assert!((rotated.x - 1.0).abs() < 0.0001);
        assert!(rotated.y.abs() < 0.0001);
        assert!(rotated.z.abs() < 0.0001);
    }

    #[test]
    fn test_normalize() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let n = q.normalize();
        assert!((n.magnitude() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_inverse() {
        let q = Quaternion::from_axis_angle(Vector3D::UP, PI / 4.0);
        let inv = q.inverse();
        let result = q * inv;
        // Should be approximately identity
        assert!((result.w - 1.0).abs() < 0.0001);
        assert!(result.x.abs() < 0.0001);
        assert!(result.y.abs() < 0.0001);
        assert!(result.z.abs() < 0.0001);
    }
}
