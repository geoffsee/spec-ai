//! 3D to 2D projection for terminal rendering

use crate::spatial::{Point3D, Transform};

/// Projection settings for 3D to 2D conversion
#[derive(Debug, Clone)]
pub struct Projection {
    /// Field of view in radians
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
    /// Aspect ratio (width / height)
    pub aspect: f32,
}

impl Projection {
    /// Create a default perspective projection
    pub fn perspective(fov_degrees: f32, aspect: f32) -> Self {
        Self {
            fov: fov_degrees.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect,
        }
    }

    /// Project a 3D point to normalized device coordinates (-1 to 1)
    pub fn project(&self, point: Point3D, camera: &Transform) -> Option<(f32, f32, f32)> {
        // Transform point to camera space
        let local = camera.inverse_transform_point(point);

        // Check if point is behind camera
        if local.z <= self.near {
            return None;
        }

        // Check if point is beyond far plane
        if local.z >= self.far {
            return None;
        }

        // Perspective projection
        let tan_half_fov = (self.fov / 2.0).tan();
        let x_ndc = local.x / (local.z * tan_half_fov * self.aspect);
        let y_ndc = local.y / (local.z * tan_half_fov);

        // Depth value for z-ordering (normalized 0-1)
        let depth = (local.z - self.near) / (self.far - self.near);

        // Check if point is within view frustum
        if x_ndc.abs() > 1.0 || y_ndc.abs() > 1.0 {
            return None;
        }

        Some((x_ndc, y_ndc, depth))
    }

    /// Convert normalized device coordinates to screen coordinates
    pub fn ndc_to_screen(&self, x_ndc: f32, y_ndc: f32, width: u32, height: u32) -> (u16, u16) {
        let x = ((x_ndc + 1.0) / 2.0 * width as f32).round() as u16;
        let y = ((1.0 - y_ndc) / 2.0 * height as f32).round() as u16;

        // Clamp to screen bounds
        let x = x.min(width as u16 - 1);
        let y = y.min(height as u16 - 1);

        (x, y)
    }

    /// Project a 3D point directly to screen coordinates
    pub fn project_to_screen(
        &self,
        point: Point3D,
        camera: &Transform,
        width: u32,
        height: u32,
    ) -> Option<(u16, u16, f32)> {
        let (x_ndc, y_ndc, depth) = self.project(point, camera)?;
        let (x, y) = self.ndc_to_screen(x_ndc, y_ndc, width, height);
        Some((x, y, depth))
    }

    /// Check if a point is visible (within frustum)
    pub fn is_visible(&self, point: Point3D, camera: &Transform) -> bool {
        self.project(point, camera).is_some()
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::perspective(60.0, 16.0 / 9.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perspective_projection() {
        let proj = Projection::perspective(90.0, 1.0);
        let camera = Transform::identity();

        // Point directly in front should project to center
        let point = Point3D::new(0.0, 0.0, 5.0);
        let result = proj.project(point, &camera);
        assert!(result.is_some());

        let (x, y, _) = result.unwrap();
        assert!(x.abs() < 0.001);
        assert!(y.abs() < 0.001);
    }

    #[test]
    fn test_behind_camera() {
        let proj = Projection::default();
        let camera = Transform::identity();

        // Point behind camera should not be visible
        let point = Point3D::new(0.0, 0.0, -5.0);
        assert!(proj.project(point, &camera).is_none());
    }

    #[test]
    fn test_screen_coordinates() {
        let proj = Projection::default();

        // NDC (0, 0) should be center of screen
        let (x, y) = proj.ndc_to_screen(0.0, 0.0, 100, 50);
        assert_eq!(x, 50);
        assert_eq!(y, 25);

        // NDC (-1, 1) should be top-left
        let (x, y) = proj.ndc_to_screen(-1.0, 1.0, 100, 50);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }
}
