//! Transform representing position, rotation, and scale in 3D space

use super::{Point3D, Quaternion, Vector3D};

/// A complete 3D transform (position + rotation + scale)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Point3D,
    pub rotation: Quaternion,
    pub scale: Vector3D,
}

impl Transform {
    /// Identity transform (origin, no rotation, unit scale)
    pub fn identity() -> Self {
        Self {
            position: Point3D::ORIGIN,
            rotation: Quaternion::IDENTITY,
            scale: Vector3D::new(1.0, 1.0, 1.0),
        }
    }

    /// Create a transform with just position
    pub fn from_position(position: Point3D) -> Self {
        Self {
            position,
            rotation: Quaternion::IDENTITY,
            scale: Vector3D::new(1.0, 1.0, 1.0),
        }
    }

    /// Create a transform with position and rotation
    pub fn from_position_rotation(position: Point3D, rotation: Quaternion) -> Self {
        Self {
            position,
            rotation,
            scale: Vector3D::new(1.0, 1.0, 1.0),
        }
    }

    /// Get the forward direction of this transform
    pub fn forward(&self) -> Vector3D {
        self.rotation.forward()
    }

    /// Get the right direction of this transform
    pub fn right(&self) -> Vector3D {
        self.rotation.right()
    }

    /// Get the up direction of this transform
    pub fn up(&self) -> Vector3D {
        self.rotation.up()
    }

    /// Rotate the transform to look at a target point
    pub fn look_at(&mut self, target: Point3D) {
        let direction = (target - self.position).normalize();
        if direction.magnitude_squared() < 0.0001 {
            return;
        }

        // Calculate yaw and pitch from direction
        let yaw = direction.x.atan2(direction.z);
        let pitch = (-direction.y).asin();

        self.rotation = Quaternion::from_euler(yaw, pitch, 0.0);
    }

    /// Transform a point from local space to world space
    pub fn transform_point(&self, local: Point3D) -> Point3D {
        let scaled = Vector3D::new(
            local.x * self.scale.x,
            local.y * self.scale.y,
            local.z * self.scale.z,
        );
        let rotated = self.rotation.rotate_vector(scaled);
        self.position + rotated
    }

    /// Transform a direction vector (ignores position and scale)
    pub fn transform_direction(&self, direction: Vector3D) -> Vector3D {
        self.rotation.rotate_vector(direction)
    }

    /// Inverse transform a point from world space to local space
    pub fn inverse_transform_point(&self, world: Point3D) -> Point3D {
        let relative = world - self.position;
        let inv_rotation = self.rotation.inverse();
        let unrotated = inv_rotation.rotate_vector(relative);
        Point3D::new(
            unrotated.x / self.scale.x,
            unrotated.y / self.scale.y,
            unrotated.z / self.scale.z,
        )
    }

    /// Linearly interpolate between two transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Self {
        Self {
            position: self.position.lerp(&other.position, t),
            rotation: self.rotation.slerp(&other.rotation, t),
            scale: self.scale.lerp(&other.scale, t),
        }
    }

    /// Translate the transform by a vector
    pub fn translate(&mut self, offset: Vector3D) {
        self.position = self.position + offset;
    }

    /// Rotate the transform by a quaternion
    pub fn rotate(&mut self, rotation: Quaternion) {
        self.rotation = rotation * self.rotation;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        assert_eq!(t.position, Point3D::ORIGIN);
        assert_eq!(t.rotation, Quaternion::IDENTITY);
    }

    #[test]
    fn test_transform_point() {
        let mut t = Transform::identity();
        t.position = Point3D::new(10.0, 0.0, 0.0);

        let local = Point3D::new(1.0, 0.0, 0.0);
        let world = t.transform_point(local);

        assert_eq!(world.x, 11.0);
        assert_eq!(world.y, 0.0);
        assert_eq!(world.z, 0.0);
    }

    #[test]
    fn test_look_at() {
        let mut t = Transform::identity();
        t.look_at(Point3D::new(0.0, 0.0, 10.0));

        let forward = t.forward();
        assert!((forward.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_inverse_transform() {
        let mut t = Transform::identity();
        t.position = Point3D::new(5.0, 5.0, 5.0);
        t.scale = Vector3D::new(2.0, 2.0, 2.0);

        let world = Point3D::new(7.0, 7.0, 7.0);
        let local = t.inverse_transform_point(world);

        assert!((local.x - 1.0).abs() < 0.0001);
        assert!((local.y - 1.0).abs() < 0.0001);
        assert!((local.z - 1.0).abs() < 0.0001);
    }
}
