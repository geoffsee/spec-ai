//! 3D bounding volumes for hit testing and visibility

use super::{Point3D, Vector3D};

/// 3D bounding volume types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bounds {
    /// Axis-aligned bounding box
    AABB { min: Point3D, max: Point3D },
    /// Bounding sphere
    Sphere { center: Point3D, radius: f32 },
    /// A single point (for very small elements)
    Point(Point3D),
}

impl Bounds {
    /// Create an AABB from center and half-extents
    pub fn aabb_centered(center: Point3D, half_extents: Vector3D) -> Self {
        Self::AABB {
            min: Point3D::new(
                center.x - half_extents.x,
                center.y - half_extents.y,
                center.z - half_extents.z,
            ),
            max: Point3D::new(
                center.x + half_extents.x,
                center.y + half_extents.y,
                center.z + half_extents.z,
            ),
        }
    }

    /// Create a bounding sphere
    pub fn sphere(center: Point3D, radius: f32) -> Self {
        Self::Sphere { center, radius }
    }

    /// Create a point bounds
    pub fn point(point: Point3D) -> Self {
        Self::Point(point)
    }

    /// Get the center of the bounds
    pub fn center(&self) -> Point3D {
        match self {
            Bounds::AABB { min, max } => Point3D::new(
                (min.x + max.x) / 2.0,
                (min.y + max.y) / 2.0,
                (min.z + max.z) / 2.0,
            ),
            Bounds::Sphere { center, .. } => *center,
            Bounds::Point(p) => *p,
        }
    }

    /// Check if a point is inside the bounds
    pub fn contains(&self, point: Point3D) -> bool {
        match self {
            Bounds::AABB { min, max } => {
                point.x >= min.x
                    && point.x <= max.x
                    && point.y >= min.y
                    && point.y <= max.y
                    && point.z >= min.z
                    && point.z <= max.z
            }
            Bounds::Sphere { center, radius } => center.distance_squared(&point) <= radius * radius,
            Bounds::Point(p) => point == *p,
        }
    }

    /// Check if this bounds intersects with another
    pub fn intersects(&self, other: &Bounds) -> bool {
        match (self, other) {
            (
                Bounds::AABB {
                    min: min1,
                    max: max1,
                },
                Bounds::AABB {
                    min: min2,
                    max: max2,
                },
            ) => {
                min1.x <= max2.x
                    && max1.x >= min2.x
                    && min1.y <= max2.y
                    && max1.y >= min2.y
                    && min1.z <= max2.z
                    && max1.z >= min2.z
            }
            (
                Bounds::Sphere {
                    center: c1,
                    radius: r1,
                },
                Bounds::Sphere {
                    center: c2,
                    radius: r2,
                },
            ) => c1.distance_squared(c2) <= (r1 + r2) * (r1 + r2),
            (Bounds::AABB { min, max }, Bounds::Sphere { center, radius })
            | (Bounds::Sphere { center, radius }, Bounds::AABB { min, max }) => {
                // Find closest point on AABB to sphere center
                let closest = Point3D::new(
                    center.x.clamp(min.x, max.x),
                    center.y.clamp(min.y, max.y),
                    center.z.clamp(min.z, max.z),
                );
                closest.distance_squared(center) <= radius * radius
            }
            (Bounds::Point(p), other) | (other, Bounds::Point(p)) => other.contains(*p),
        }
    }

    /// Get the distance from a point to the closest edge of the bounds
    pub fn distance_to(&self, point: Point3D) -> f32 {
        match self {
            Bounds::AABB { min, max } => {
                let closest = Point3D::new(
                    point.x.clamp(min.x, max.x),
                    point.y.clamp(min.y, max.y),
                    point.z.clamp(min.z, max.z),
                );
                closest.distance(&point)
            }
            Bounds::Sphere { center, radius } => (center.distance(&point) - radius).max(0.0),
            Bounds::Point(p) => p.distance(&point),
        }
    }

    /// Expand the bounds by a margin
    pub fn expand(&self, margin: f32) -> Self {
        match self {
            Bounds::AABB { min, max } => Bounds::AABB {
                min: Point3D::new(min.x - margin, min.y - margin, min.z - margin),
                max: Point3D::new(max.x + margin, max.y + margin, max.z + margin),
            },
            Bounds::Sphere { center, radius } => Bounds::Sphere {
                center: *center,
                radius: radius + margin,
            },
            Bounds::Point(p) => Bounds::Sphere {
                center: *p,
                radius: margin,
            },
        }
    }

    /// Create a bounds that encompasses both this and another bounds
    pub fn union(&self, other: &Bounds) -> Self {
        let (self_min, self_max) = self.aabb_extents();
        let (other_min, other_max) = other.aabb_extents();

        Bounds::AABB {
            min: Point3D::new(
                self_min.x.min(other_min.x),
                self_min.y.min(other_min.y),
                self_min.z.min(other_min.z),
            ),
            max: Point3D::new(
                self_max.x.max(other_max.x),
                self_max.y.max(other_max.y),
                self_max.z.max(other_max.z),
            ),
        }
    }

    /// Get AABB extents for any bounds type
    fn aabb_extents(&self) -> (Point3D, Point3D) {
        match self {
            Bounds::AABB { min, max } => (*min, *max),
            Bounds::Sphere { center, radius } => (
                Point3D::new(center.x - radius, center.y - radius, center.z - radius),
                Point3D::new(center.x + radius, center.y + radius, center.z + radius),
            ),
            Bounds::Point(p) => (*p, *p),
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Bounds::Point(Point3D::ORIGIN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_contains() {
        let bounds = Bounds::AABB {
            min: Point3D::new(0.0, 0.0, 0.0),
            max: Point3D::new(10.0, 10.0, 10.0),
        };

        assert!(bounds.contains(Point3D::new(5.0, 5.0, 5.0)));
        assert!(bounds.contains(Point3D::new(0.0, 0.0, 0.0)));
        assert!(!bounds.contains(Point3D::new(-1.0, 5.0, 5.0)));
    }

    #[test]
    fn test_sphere_contains() {
        let bounds = Bounds::sphere(Point3D::ORIGIN, 5.0);

        assert!(bounds.contains(Point3D::new(0.0, 0.0, 0.0)));
        assert!(bounds.contains(Point3D::new(3.0, 0.0, 0.0)));
        assert!(!bounds.contains(Point3D::new(6.0, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_intersection() {
        let a = Bounds::AABB {
            min: Point3D::new(0.0, 0.0, 0.0),
            max: Point3D::new(10.0, 10.0, 10.0),
        };
        let b = Bounds::AABB {
            min: Point3D::new(5.0, 5.0, 5.0),
            max: Point3D::new(15.0, 15.0, 15.0),
        };
        let c = Bounds::AABB {
            min: Point3D::new(20.0, 20.0, 20.0),
            max: Point3D::new(30.0, 30.0, 30.0),
        };

        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_sphere_intersection() {
        let a = Bounds::sphere(Point3D::ORIGIN, 5.0);
        let b = Bounds::sphere(Point3D::new(8.0, 0.0, 0.0), 5.0);
        let c = Bounds::sphere(Point3D::new(20.0, 0.0, 0.0), 5.0);

        assert!(a.intersects(&b)); // Overlapping
        assert!(!a.intersects(&c)); // Too far
    }

    #[test]
    fn test_distance_to() {
        let bounds = Bounds::sphere(Point3D::ORIGIN, 5.0);
        assert_eq!(bounds.distance_to(Point3D::new(10.0, 0.0, 0.0)), 5.0);
        assert_eq!(bounds.distance_to(Point3D::new(3.0, 0.0, 0.0)), 0.0); // Inside
    }

    #[test]
    fn test_expand() {
        let bounds = Bounds::sphere(Point3D::ORIGIN, 5.0);
        let expanded = bounds.expand(2.0);

        if let Bounds::Sphere { radius, .. } = expanded {
            assert_eq!(radius, 7.0);
        } else {
            panic!("Expected sphere");
        }
    }
}
