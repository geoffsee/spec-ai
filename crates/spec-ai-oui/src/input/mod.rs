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
mod simulator;
mod voice;

pub use event::OpticalEvent;
pub use gaze::{GazeState, GazeTarget};
pub use gesture::{GestureEvent, GestureType, Hand, SwipeDirection};
pub use head::{HeadGestureType, HeadPose};
pub use simulator::InputSimulator;
pub use voice::VoiceCommand;
