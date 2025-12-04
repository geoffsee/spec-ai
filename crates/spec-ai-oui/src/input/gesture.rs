//! Hand gesture recognition types

use crate::spatial::{Point3D, Vector3D};

/// Which hand performed the gesture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
    Both,
}

/// Direction of a swipe gesture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Types of recognized gestures
#[derive(Debug, Clone)]
pub enum GestureType {
    /// Pinch thumb and index finger together
    Pinch {
        /// Pinch strength (0.0 = released, 1.0 = fully pinched)
        strength: f32,
    },

    /// Point with index finger
    Point {
        /// Direction the finger is pointing
        direction: Vector3D,
    },

    /// Open palm facing camera
    OpenPalm,

    /// Closed fist
    Fist,

    /// Swipe gesture
    Swipe {
        /// Direction of the swipe
        direction: SwipeDirection,
        /// Speed of the swipe
        velocity: f32,
    },

    /// Tap in the air (quick pinch and release)
    AirTap {
        /// Position of the tap
        position: Point3D,
    },

    /// Two-finger spread or pinch for zooming
    Zoom {
        /// Zoom factor (> 1.0 = zoom in, < 1.0 = zoom out)
        factor: f32,
    },

    /// Grab and hold gesture
    Grab {
        /// Whether the grab is held
        held: bool,
    },

    /// Thumbs up
    ThumbsUp,

    /// Thumbs down
    ThumbsDown,
}

/// A complete gesture event
#[derive(Debug, Clone)]
pub struct GestureEvent {
    /// Which hand
    pub hand: Hand,
    /// Type of gesture
    pub gesture: GestureType,
    /// Position in 3D space
    pub position: Point3D,
    /// Recognition confidence (0.0 - 1.0)
    pub confidence: f32,
}

impl GestureEvent {
    /// Create a new gesture event
    pub fn new(hand: Hand, gesture: GestureType, position: Point3D) -> Self {
        Self {
            hand,
            gesture,
            position,
            confidence: 1.0,
        }
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Check if this is a selection gesture (tap or pinch)
    pub fn is_select(&self) -> bool {
        match &self.gesture {
            GestureType::AirTap { .. } => true,
            GestureType::Pinch { strength } if *strength > 0.8 => true,
            _ => false,
        }
    }

    /// Check if this is a navigation gesture (swipe)
    pub fn is_navigation(&self) -> bool {
        matches!(self.gesture, GestureType::Swipe { .. })
    }

    /// Get swipe direction if this is a swipe gesture
    pub fn swipe_direction(&self) -> Option<SwipeDirection> {
        match &self.gesture {
            GestureType::Swipe { direction, .. } => Some(*direction),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_event() {
        let gesture = GestureEvent::new(
            Hand::Right,
            GestureType::AirTap { position: Point3D::ORIGIN },
            Point3D::ORIGIN,
        );
        assert!(gesture.is_select());
        assert!(!gesture.is_navigation());
    }

    #[test]
    fn test_swipe_gesture() {
        let gesture = GestureEvent::new(
            Hand::Right,
            GestureType::Swipe {
                direction: SwipeDirection::Left,
                velocity: 1.0,
            },
            Point3D::ORIGIN,
        );
        assert!(!gesture.is_select());
        assert!(gesture.is_navigation());
        assert_eq!(gesture.swipe_direction(), Some(SwipeDirection::Left));
    }
}
