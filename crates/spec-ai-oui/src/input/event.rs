//! Unified optical input events

use std::time::Duration;
use crossterm::event::KeyEvent;

use crate::spatial::{Point3D, Transform};
use super::{GestureEvent, HeadGestureType};

/// Unified input event for optical UI
#[derive(Debug, Clone)]
pub enum OpticalEvent {
    /// Gaze moved to a new point in space
    GazeMove {
        /// Point in 3D space where the user is looking
        point: Point3D,
        /// Normalized screen position (0-1)
        screen_pos: (f32, f32),
    },

    /// User has been looking at the same target for the dwell threshold
    GazeDwell {
        /// ID of the UI element being dwelled on
        target_id: String,
        /// How long the user has been looking
        duration: Duration,
    },

    /// Gaze entered a UI element
    GazeEnter {
        /// ID of the element being entered
        target_id: String,
    },

    /// Gaze exited a UI element
    GazeExit {
        /// ID of the element being exited
        target_id: String,
    },

    /// Hand gesture recognized
    Gesture(GestureEvent),

    /// Head position/orientation changed
    HeadPose {
        /// New head transform
        transform: Transform,
    },

    /// Head gesture detected (nod, shake, etc.)
    HeadGesture(HeadGestureType),

    /// Voice command recognized
    Voice {
        /// Recognized command text
        command: String,
        /// Recognition confidence (0-1)
        confidence: f32,
    },

    /// Fallback keyboard input
    Key(KeyEvent),

    /// Regular tick for animations
    Tick,

    /// Terminal/window resized
    Resize {
        width: u32,
        height: u32,
    },
}

impl OpticalEvent {
    /// Check if this is a tick event
    pub fn is_tick(&self) -> bool {
        matches!(self, OpticalEvent::Tick)
    }

    /// Check if this is a keyboard event
    pub fn is_key(&self) -> bool {
        matches!(self, OpticalEvent::Key(_))
    }

    /// Check if this is a gaze event
    pub fn is_gaze(&self) -> bool {
        matches!(
            self,
            OpticalEvent::GazeMove { .. }
                | OpticalEvent::GazeDwell { .. }
                | OpticalEvent::GazeEnter { .. }
                | OpticalEvent::GazeExit { .. }
        )
    }

    /// Check if this is a gesture event
    pub fn is_gesture(&self) -> bool {
        matches!(self, OpticalEvent::Gesture(_))
    }
}
