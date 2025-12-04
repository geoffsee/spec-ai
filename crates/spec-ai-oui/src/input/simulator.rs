//! Input simulator for terminal development
//!
//! Maps keyboard inputs to simulated spatial inputs for development without AR hardware.

use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::spatial::{Point3D, Transform, Quaternion, Vector3D};
use super::{OpticalEvent, GestureEvent, GestureType, Hand, SwipeDirection, HeadGestureType};

/// Simulates spatial inputs from keyboard for development
pub struct InputSimulator {
    /// Current simulated gaze position (screen-space 0-1)
    gaze_x: f32,
    gaze_y: f32,
    /// Current head yaw (left-right rotation)
    head_yaw: f32,
    /// Current head pitch (up-down rotation)
    head_pitch: f32,
    /// Whether grab gesture is active
    grab_active: bool,
    /// Pending events to emit
    pending_events: VecDeque<OpticalEvent>,
    /// Gaze movement speed
    gaze_speed: f32,
    /// Head rotation speed
    head_speed: f32,
}

impl Default for InputSimulator {
    fn default() -> Self {
        Self {
            gaze_x: 0.5,
            gaze_y: 0.5,
            head_yaw: 0.0,
            head_pitch: 0.0,
            grab_active: false,
            pending_events: VecDeque::new(),
            gaze_speed: 0.05,
            head_speed: 0.1,
        }
    }
}

impl InputSimulator {
    /// Create a new input simulator
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a keyboard event and generate simulated optical events
    pub fn process_key(&mut self, key: KeyEvent) -> Vec<OpticalEvent> {
        let mut events = Vec::new();

        // Only process key press events
        if key.kind != crossterm::event::KeyEventKind::Press {
            return events;
        }

        match key.code {
            // Arrow keys: Move gaze
            KeyCode::Up => {
                self.gaze_y = (self.gaze_y - self.gaze_speed).max(0.0);
                events.push(self.gaze_event());
            }
            KeyCode::Down => {
                self.gaze_y = (self.gaze_y + self.gaze_speed).min(1.0);
                events.push(self.gaze_event());
            }
            KeyCode::Left => {
                self.gaze_x = (self.gaze_x - self.gaze_speed).max(0.0);
                events.push(self.gaze_event());
            }
            KeyCode::Right => {
                self.gaze_x = (self.gaze_x + self.gaze_speed).min(1.0);
                events.push(self.gaze_event());
            }

            // WASD: Head rotation
            KeyCode::Char('w') | KeyCode::Char('W') => {
                self.head_pitch -= self.head_speed;
                events.push(self.head_pose_event());
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.head_pitch += self.head_speed;
                events.push(self.head_pose_event());
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.head_yaw -= self.head_speed;
                events.push(self.head_pose_event());
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.head_yaw += self.head_speed;
                events.push(self.head_pose_event());
            }

            // Space: Air tap
            KeyCode::Char(' ') => {
                events.push(OpticalEvent::Gesture(GestureEvent::new(
                    Hand::Right,
                    GestureType::AirTap {
                        position: self.gaze_3d_point(),
                    },
                    self.gaze_3d_point(),
                )));
            }

            // G: Toggle grab
            KeyCode::Char('g') | KeyCode::Char('G') => {
                self.grab_active = !self.grab_active;
                events.push(OpticalEvent::Gesture(GestureEvent::new(
                    Hand::Right,
                    GestureType::Grab {
                        held: self.grab_active,
                    },
                    self.gaze_3d_point(),
                )));
            }

            // P: Pinch
            KeyCode::Char('p') | KeyCode::Char('P') => {
                events.push(OpticalEvent::Gesture(GestureEvent::new(
                    Hand::Right,
                    GestureType::Pinch { strength: 1.0 },
                    self.gaze_3d_point(),
                )));
            }

            // H/J/K/L: Swipe gestures (vim-style)
            KeyCode::Char('h') => {
                events.push(self.swipe_event(SwipeDirection::Left));
            }
            KeyCode::Char('l') => {
                events.push(self.swipe_event(SwipeDirection::Right));
            }
            KeyCode::Char('j') => {
                events.push(self.swipe_event(SwipeDirection::Down));
            }
            KeyCode::Char('k') => {
                events.push(self.swipe_event(SwipeDirection::Up));
            }

            // Number keys: Simulated voice commands
            KeyCode::Char('1') => events.push(self.voice_event("select")),
            KeyCode::Char('2') => events.push(self.voice_event("back")),
            KeyCode::Char('3') => events.push(self.voice_event("menu")),
            KeyCode::Char('4') => events.push(self.voice_event("confirm")),
            KeyCode::Char('5') => events.push(self.voice_event("cancel")),
            KeyCode::Char('6') => events.push(self.voice_event("scroll up")),
            KeyCode::Char('7') => events.push(self.voice_event("scroll down")),
            KeyCode::Char('8') => events.push(self.voice_event("help")),
            KeyCode::Char('9') => events.push(self.voice_event("status")),

            // Head gestures with modifiers
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                events.push(OpticalEvent::HeadGesture(HeadGestureType::Nod));
            }
            KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                events.push(OpticalEvent::HeadGesture(HeadGestureType::Shake));
            }

            // Also pass through the raw key event
            _ => {
                events.push(OpticalEvent::Key(key));
            }
        }

        events
    }

    /// Get current gaze as a 3D point (projected forward from camera)
    fn gaze_3d_point(&self) -> Point3D {
        // Convert screen position to a point 2 meters in front of camera
        let x = (self.gaze_x - 0.5) * 2.0; // -1 to 1
        let y = (0.5 - self.gaze_y) * 2.0; // -1 to 1 (inverted)
        Point3D::new(x, y, 2.0)
    }

    /// Create a gaze move event
    fn gaze_event(&self) -> OpticalEvent {
        OpticalEvent::GazeMove {
            point: self.gaze_3d_point(),
            screen_pos: (self.gaze_x, self.gaze_y),
        }
    }

    /// Create a head pose event
    fn head_pose_event(&self) -> OpticalEvent {
        let rotation = Quaternion::from_euler(self.head_yaw, self.head_pitch, 0.0);
        OpticalEvent::HeadPose {
            transform: Transform::from_position_rotation(Point3D::ORIGIN, rotation),
        }
    }

    /// Create a swipe gesture event
    fn swipe_event(&self, direction: SwipeDirection) -> OpticalEvent {
        OpticalEvent::Gesture(GestureEvent::new(
            Hand::Right,
            GestureType::Swipe {
                direction,
                velocity: 1.0,
            },
            self.gaze_3d_point(),
        ))
    }

    /// Create a voice command event
    fn voice_event(&self, command: &str) -> OpticalEvent {
        OpticalEvent::Voice {
            command: command.to_string(),
            confidence: 1.0,
        }
    }

    /// Get current simulated head transform
    pub fn head_transform(&self) -> Transform {
        let rotation = Quaternion::from_euler(self.head_yaw, self.head_pitch, 0.0);
        Transform::from_position_rotation(Point3D::ORIGIN, rotation)
    }

    /// Get current gaze screen position
    pub fn gaze_position(&self) -> (f32, f32) {
        (self.gaze_x, self.gaze_y)
    }

    /// Reset simulator to default state
    pub fn reset(&mut self) {
        self.gaze_x = 0.5;
        self.gaze_y = 0.5;
        self.head_yaw = 0.0;
        self.head_pitch = 0.0;
        self.grab_active = false;
        self.pending_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaze_movement() {
        let mut sim = InputSimulator::new();
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        let events = sim.process_key(key);

        assert!(!events.is_empty());
        if let OpticalEvent::GazeMove { screen_pos, .. } = &events[0] {
            assert!(screen_pos.1 < 0.5); // Moved up
        } else {
            panic!("Expected GazeMove event");
        }
    }

    #[test]
    fn test_air_tap() {
        let mut sim = InputSimulator::new();
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        let events = sim.process_key(key);

        assert!(!events.is_empty());
        assert!(matches!(
            &events[0],
            OpticalEvent::Gesture(GestureEvent { gesture: GestureType::AirTap { .. }, .. })
        ));
    }

    #[test]
    fn test_voice_command() {
        let mut sim = InputSimulator::new();
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        let events = sim.process_key(key);

        assert!(!events.is_empty());
        if let OpticalEvent::Voice { command, .. } = &events[0] {
            assert_eq!(command, "select");
        } else {
            panic!("Expected Voice event");
        }
    }
}
