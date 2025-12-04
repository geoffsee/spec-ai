//! Attention tracking for context-aware display

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use crate::spatial::Point3D;

/// Current attention state based on gaze tracking
#[derive(Debug, Clone)]
pub struct AttentionState {
    /// Current gaze target widget ID
    pub gaze_target: Option<String>,
    /// Time spent on current target
    pub dwell_time: Duration,
    /// Recent gaze history
    gaze_history: VecDeque<(Point3D, Instant)>,
    /// Detected focus level (0.0 = distracted, 1.0 = focused)
    pub focus_level: f32,
    /// Whether user appears to be actively engaged
    pub is_engaged: bool,
}

impl Default for AttentionState {
    fn default() -> Self {
        Self {
            gaze_target: None,
            dwell_time: Duration::ZERO,
            gaze_history: VecDeque::with_capacity(60),
            focus_level: 1.0,
            is_engaged: true,
        }
    }
}

impl AttentionState {
    /// Update attention state with new gaze data
    pub fn update(&mut self, gaze_point: Point3D, target: Option<String>) {
        let now = Instant::now();

        // Update gaze history
        self.gaze_history.push_back((gaze_point, now));
        while self.gaze_history.len() > 60 {
            self.gaze_history.pop_front();
        }

        // Update target and dwell time
        match (&self.gaze_target, &target) {
            (Some(old), Some(new)) if old == new => {
                // Same target, increase dwell time
                self.dwell_time += Duration::from_millis(16); // Assume ~60fps
            }
            _ => {
                // New target or no target
                self.gaze_target = target;
                self.dwell_time = Duration::ZERO;
            }
        }

        // Calculate focus level based on gaze stability
        self.focus_level = self.calculate_focus_level();

        // Determine engagement based on recent movement
        self.is_engaged = self.gaze_history.len() > 5;
    }

    /// Calculate focus level from gaze stability
    fn calculate_focus_level(&self) -> f32 {
        if self.gaze_history.len() < 2 {
            return 1.0;
        }

        // Calculate average movement in recent gaze history
        let mut total_movement = 0.0;
        let mut prev_point = self.gaze_history.front().map(|(p, _)| *p);

        for (point, _) in self.gaze_history.iter().skip(1) {
            if let Some(prev) = prev_point {
                total_movement += prev.distance(point);
            }
            prev_point = Some(*point);
        }

        let avg_movement = total_movement / (self.gaze_history.len() - 1) as f32;

        // Lower movement = higher focus (inverse relationship)
        // Movement of 0.1 or more = low focus, 0.01 or less = high focus
        (1.0 - (avg_movement * 10.0).min(1.0)).max(0.0)
    }

    /// Check if user is focused on a specific element
    pub fn is_focused_on(&self, id: &str) -> bool {
        self.gaze_target.as_ref().map(|t| t == id).unwrap_or(false)
    }

    /// Check if user has dwelled on current target long enough
    pub fn has_dwelled(&self, threshold: Duration) -> bool {
        self.dwell_time >= threshold
    }

    /// Get attention score for an element (higher = more attention)
    pub fn attention_score(&self, id: &str) -> f32 {
        if self.is_focused_on(id) {
            // Currently focused - score based on dwell time
            let dwell_secs = self.dwell_time.as_secs_f32();
            (dwell_secs / 2.0).min(1.0) * self.focus_level
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_state_default() {
        let state = AttentionState::default();
        assert!(state.gaze_target.is_none());
        assert_eq!(state.focus_level, 1.0);
    }

    #[test]
    fn test_dwell_time_accumulation() {
        let mut state = AttentionState::default();
        state.update(Point3D::ORIGIN, Some("button1".to_string()));
        state.update(Point3D::ORIGIN, Some("button1".to_string()));
        state.update(Point3D::ORIGIN, Some("button1".to_string()));

        assert!(state.dwell_time > Duration::ZERO);
        assert!(state.is_focused_on("button1"));
    }

    #[test]
    fn test_target_change_resets_dwell() {
        let mut state = AttentionState::default();
        state.update(Point3D::ORIGIN, Some("button1".to_string()));
        state.update(Point3D::ORIGIN, Some("button1".to_string()));
        let dwell1 = state.dwell_time;

        state.update(Point3D::ORIGIN, Some("button2".to_string()));
        assert!(state.dwell_time < dwell1);
    }
}
