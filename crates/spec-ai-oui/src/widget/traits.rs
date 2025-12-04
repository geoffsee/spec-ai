//! Widget trait definitions for optical UI

use std::time::Duration;

use crate::spatial::{Bounds, SpatialAnchor, Transform};
use crate::renderer::RenderBackend;
use crate::input::OpticalEvent;
use crate::context::DisplayContext;

/// Core trait for optical widgets
pub trait OpticalWidget: Send + Sync {
    /// Get the widget's unique identifier (for gaze/gesture targeting)
    fn id(&self) -> &str;

    /// Get the widget's spatial bounds for hit testing
    fn bounds(&self) -> Bounds;

    /// Get the widget's spatial anchor
    fn anchor(&self) -> &SpatialAnchor;

    /// Update the widget state
    fn update(&mut self, dt: Duration, ctx: &DisplayContext);

    /// Handle an input event, return true if consumed
    fn handle_event(&mut self, event: &OpticalEvent) -> bool;

    /// Render the widget to the backend
    fn render(&self, backend: &mut dyn RenderBackend, camera: &Transform);

    /// Get current visibility (0.0 - 1.0, affected by context/priority)
    fn visibility(&self) -> f32;

    /// Set visibility (for context-aware display)
    fn set_visibility(&mut self, visibility: f32);

    /// Get the widget's priority level
    fn priority(&self) -> crate::context::Priority {
        crate::context::Priority::Normal
    }

    /// Check if the widget is interactive
    fn is_interactive(&self) -> bool {
        true
    }

    /// Check if the widget is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Stateful optical widget with external state
pub trait StatefulOpticalWidget: Send + Sync {
    type State;

    /// Get the widget's unique identifier
    fn id(&self) -> &str;

    /// Get the widget's spatial bounds
    fn bounds(&self, state: &Self::State) -> Bounds;

    /// Get the widget's spatial anchor
    fn anchor(&self, state: &Self::State) -> &SpatialAnchor;

    /// Update the widget state
    fn update(&self, state: &mut Self::State, dt: Duration, ctx: &DisplayContext);

    /// Handle an input event
    fn handle_event(&self, state: &mut Self::State, event: &OpticalEvent) -> bool;

    /// Render the widget
    fn render(&self, state: &Self::State, backend: &mut dyn RenderBackend, camera: &Transform);

    /// Get visibility
    fn visibility(&self, state: &Self::State) -> f32;

    /// Set visibility
    fn set_visibility(&self, state: &mut Self::State, visibility: f32);
}
