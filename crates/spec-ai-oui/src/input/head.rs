//! Head tracking and head gesture types

use crate::spatial::{Point3D, Quaternion, Transform};

/// Types of head gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadGestureType {
    /// Nodding (yes)
    Nod,
    /// Shaking (no)
    Shake,
    /// Tilting head to the side
    Tilt,
    /// Looking up
    LookUp,
    /// Looking down
    LookDown,
}

/// Head pose tracking state
#[derive(Debug, Clone)]
pub struct HeadPose {
    /// Current head transform
    pub transform: Transform,
    /// Previous frame transform (for velocity)
    prev_transform: Transform,
    /// Detected gesture (if any)
    pub gesture: Option<HeadGestureType>,
    /// Tracking quality (0.0 - 1.0)
    pub tracking_quality: f32,
}

impl Default for HeadPose {
    fn default() -> Self {
        Self {
            transform: Transform::identity(),
            prev_transform: Transform::identity(),
            gesture: None,
            tracking_quality: 1.0,
        }
    }
}

impl HeadPose {
    /// Update head pose with new transform
    pub fn update(&mut self, transform: Transform) {
        self.prev_transform = self.transform;
        self.transform = transform;
        self.gesture = self.detect_gesture();
    }

    /// Get angular velocity (rotation change per frame)
    pub fn angular_velocity(&self) -> Quaternion {
        self.transform.rotation * self.prev_transform.rotation.conjugate()
    }

    /// Get position velocity
    pub fn velocity(&self) -> Point3D {
        Point3D::new(
            self.transform.position.x - self.prev_transform.position.x,
            self.transform.position.y - self.prev_transform.position.y,
            self.transform.position.z - self.prev_transform.position.z,
        )
    }

    /// Detect head gesture from recent motion
    fn detect_gesture(&self) -> Option<HeadGestureType> {
        let forward = self.transform.forward();
        let prev_forward = self.prev_transform.forward();

        // Calculate rotation change
        let y_delta = forward.y - prev_forward.y;
        let x_delta = forward.x - prev_forward.x;

        const THRESHOLD: f32 = 0.05;

        if y_delta.abs() > THRESHOLD {
            if y_delta > 0.0 {
                return Some(HeadGestureType::LookUp);
            } else {
                return Some(HeadGestureType::LookDown);
            }
        }

        if x_delta.abs() > THRESHOLD {
            // Rapid left-right motion could be a shake
            // This would need more sophisticated detection in practice
            return Some(HeadGestureType::Shake);
        }

        None
    }

    /// Get the forward gaze direction
    pub fn gaze_direction(&self) -> Point3D {
        let forward = self.transform.forward();
        Point3D::new(forward.x, forward.y, forward.z)
    }

    /// Get a point in space where the user is looking at a given distance
    pub fn gaze_point(&self, distance: f32) -> Point3D {
        let forward = self.transform.forward();
        self.transform.position + forward * distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_pose_default() {
        let pose = HeadPose::default();
        assert_eq!(pose.tracking_quality, 1.0);
        assert!(pose.gesture.is_none());
    }

    #[test]
    fn test_gaze_point() {
        let pose = HeadPose::default();
        let point = pose.gaze_point(5.0);
        // Default pose looks forward (+Z)
        assert!(point.z > 0.0);
    }
}
