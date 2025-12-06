//! Gaze tracking abstraction

use crate::spatial::Point3D;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Current gaze state
#[derive(Debug, Clone)]
pub struct GazeState {
    /// Current gaze point in 3D space
    pub point: Point3D,
    /// Screen-space position (normalized 0-1)
    pub screen_pos: (f32, f32),
    /// Current target element ID (if any)
    pub target: Option<GazeTarget>,
    /// Gaze velocity (for smooth movement)
    pub velocity: Point3D,
    /// Recent gaze history for gesture detection
    pub history: VecDeque<(Point3D, Instant)>,
}

/// A gaze target with dwell tracking
#[derive(Debug, Clone)]
pub struct GazeTarget {
    /// Target element ID
    pub id: String,
    /// When the gaze entered this target
    pub entered_at: Instant,
    /// Time spent looking at target
    pub dwell_time: Duration,
}

impl GazeTarget {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entered_at: Instant::now(),
            dwell_time: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.dwell_time = self.entered_at.elapsed();
    }
}

impl Default for GazeState {
    fn default() -> Self {
        Self {
            point: Point3D::new(0.0, 0.0, 1.0), // Looking forward
            screen_pos: (0.5, 0.5),             // Center of screen
            target: None,
            velocity: Point3D::ORIGIN,
            history: VecDeque::with_capacity(60), // ~1 second at 60fps
        }
    }
}

impl GazeState {
    /// Update gaze with new position
    pub fn update(&mut self, point: Point3D, screen_pos: (f32, f32)) {
        // Calculate velocity
        let dt = 1.0 / 60.0; // Assume 60fps
        self.velocity = Point3D::new(
            (point.x - self.point.x) / dt,
            (point.y - self.point.y) / dt,
            (point.z - self.point.z) / dt,
        );

        // Update position
        self.point = point;
        self.screen_pos = screen_pos;

        // Add to history
        self.history.push_back((point, Instant::now()));
        while self.history.len() > 60 {
            self.history.pop_front();
        }

        // Update target dwell time
        if let Some(ref mut target) = self.target {
            target.update();
        }
    }

    /// Set the current gaze target
    pub fn set_target(&mut self, id: Option<String>) {
        match (id, &self.target) {
            (Some(new_id), Some(current)) if new_id == current.id => {
                // Same target, just update dwell time
                if let Some(ref mut target) = self.target {
                    target.update();
                }
            }
            (Some(new_id), _) => {
                // New target
                self.target = Some(GazeTarget::new(new_id));
            }
            (None, _) => {
                self.target = None;
            }
        }
    }

    /// Check if currently dwelling on a target
    pub fn is_dwelling(&self, threshold: Duration) -> bool {
        self.target
            .as_ref()
            .map(|t| t.dwell_time >= threshold)
            .unwrap_or(false)
    }

    /// Get the current target ID if dwelling
    pub fn dwelling_target(&self, threshold: Duration) -> Option<&str> {
        self.target.as_ref().and_then(|t| {
            if t.dwell_time >= threshold {
                Some(t.id.as_str())
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaze_target() {
        let mut target = GazeTarget::new("button1");
        std::thread::sleep(Duration::from_millis(10));
        target.update();
        assert!(target.dwell_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_gaze_state_update() {
        let mut state = GazeState::default();
        state.update(Point3D::new(1.0, 0.0, 1.0), (0.6, 0.5));
        assert_eq!(state.point.x, 1.0);
        assert_eq!(state.history.len(), 1);
    }
}
