//! Audio backend trait

use super::Notification;
use crate::spatial::Transform;

/// Audio backend trait for optical UI
pub trait AudioBackend: Send + Sync {
    /// Play a notification sound
    fn play_notification(&mut self, notification: Notification);

    /// Update listener position (for spatial audio)
    fn set_listener(&mut self, transform: Transform);

    /// Set master volume (0.0 - 1.0)
    fn set_volume(&mut self, volume: f32);

    /// Check if audio is available
    fn is_available(&self) -> bool;
}

/// Null audio backend (no-op)
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn play_notification(&mut self, _notification: Notification) {}
    fn set_listener(&mut self, _transform: Transform) {}
    fn set_volume(&mut self, _volume: f32) {}
    fn is_available(&self) -> bool {
        false
    }
}
