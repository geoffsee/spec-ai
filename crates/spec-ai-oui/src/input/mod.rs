//! Input abstraction for optical UI
//!
//! Provides unified input events for:
//! - Gaze tracking
//! - Hand gesture recognition
//! - Head pose tracking
//! - Voice commands
//! - Fallback keyboard input (for terminal simulation)

mod event;
mod gaze;
mod gesture;
mod head;
mod voice;
mod simulator;

pub use event::OpticalEvent;
pub use gaze::{GazeState, GazeTarget};
pub use gesture::{GestureType, GestureEvent, Hand, SwipeDirection};
pub use head::{HeadGestureType, HeadPose};
pub use voice::VoiceCommand;
pub use simulator::InputSimulator;
