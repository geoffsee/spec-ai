//! Reticle/crosshair widget

use std::time::Duration;

use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::renderer::{RenderBackend, Color};
use crate::input::OpticalEvent;
use crate::context::{DisplayContext, Priority};
use crate::widget::OpticalWidget;

/// Reticle visual styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReticleStyle {
    /// Simple crosshair (+)
    Simple,
    /// Circle with center dot
    Circle,
    /// Military-style brackets
    Tactical,
    /// Animated scanning reticle
    Scanner,
    /// Dot only
    Dot,
}

/// Target lock information
#[derive(Debug, Clone)]
pub struct TargetLock {
    /// Target identifier
    pub target_id: String,
    /// Target name/label
    pub name: String,
    /// Lock progress (0.0 - 1.0)
    pub lock_progress: f32,
    /// Whether lock is complete
    pub locked: bool,
}

/// Center reticle/crosshair widget
pub struct Reticle {
    id: String,
    anchor: SpatialAnchor,
    style: ReticleStyle,
    color: Color,
    locked_color: Color,
    target: Option<TargetLock>,
    visibility: f32,
    animation_tick: u64,
}

impl Reticle {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::screen_space(&id_str, 0.5, 0.5),
            id: id_str,
            style: ReticleStyle::Simple,
            color: Color::HUD_CYAN,
            locked_color: Color::ALERT_RED,
            target: None,
            visibility: 1.0,
            animation_tick: 0,
        }
    }

    /// Set the reticle style
    pub fn style(mut self, style: ReticleStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the normal color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the locked color
    pub fn locked_color(mut self, color: Color) -> Self {
        self.locked_color = color;
        self
    }

    /// Set a target lock
    pub fn set_target(&mut self, target: Option<TargetLock>) {
        self.target = target;
    }

    /// Update lock progress
    pub fn update_lock_progress(&mut self, progress: f32) {
        if let Some(ref mut target) = self.target {
            target.lock_progress = progress.clamp(0.0, 1.0);
            target.locked = target.lock_progress >= 1.0;
        }
    }
}

impl OpticalWidget for Reticle {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::point(Point3D::ORIGIN)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {
        self.animation_tick = self.animation_tick.wrapping_add(1);
    }

    fn handle_event(&mut self, _event: &OpticalEvent) -> bool {
        false
    }

    fn render(&self, backend: &mut dyn RenderBackend, _camera: &Transform) {
        if self.visibility < 0.1 {
            return;
        }

        let Some((x, y)) = self.anchor.screen_coords() else {
            return;
        };

        let color = if self.target.as_ref().map(|t| t.locked).unwrap_or(false) {
            self.locked_color
        } else {
            self.color
        };

        match self.style {
            ReticleStyle::Simple => {
                backend.draw_hud_text(x, y, "+", color);
            }
            ReticleStyle::Circle => {
                backend.draw_hud_text(x, y, "◎", color);
            }
            ReticleStyle::Tactical => {
                // Draw bracket-style reticle
                backend.draw_hud_text(x - 0.02, y - 0.02, "┌", color);
                backend.draw_hud_text(x + 0.02, y - 0.02, "┐", color);
                backend.draw_hud_text(x - 0.02, y + 0.02, "└", color);
                backend.draw_hud_text(x + 0.02, y + 0.02, "┘", color);
                backend.draw_hud_text(x, y, "·", color);
            }
            ReticleStyle::Scanner => {
                // Animated scanning effect
                let frame = (self.animation_tick / 10) % 4;
                let symbols = ["◴", "◷", "◶", "◵"];
                backend.draw_hud_text(x, y, symbols[frame as usize], color);
            }
            ReticleStyle::Dot => {
                backend.draw_hud_text(x, y, "●", color);
            }
        }

        // Draw target lock info
        if let Some(ref target) = self.target {
            // Target name
            backend.draw_hud_text(x + 0.03, y - 0.01, &target.name, Color::White);

            // Lock progress bar
            if !target.locked {
                let progress_width = 10;
                let filled = (target.lock_progress * progress_width as f32) as usize;
                let bar = "▓".repeat(filled) + &"░".repeat(progress_width - filled);
                backend.draw_hud_text(x + 0.03, y + 0.01, &bar, Color::Yellow);
            } else {
                backend.draw_hud_text(x + 0.03, y + 0.01, "LOCKED", Color::ALERT_RED);
            }
        }
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
