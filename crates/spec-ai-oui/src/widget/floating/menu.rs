//! Radial menu widget

use crate::context::{DisplayContext, Priority};
use crate::input::{GestureType, OpticalEvent, SwipeDirection};
use crate::renderer::{Color, RenderBackend};
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform, Vector3D};
use crate::widget::OpticalWidget;
use std::time::Duration;

/// A menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub icon: char,
    pub label: String,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<String>, icon: char, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Radial menu for gesture-based selection
pub struct RadialMenu {
    id: String,
    anchor: SpatialAnchor,
    items: Vec<MenuItem>,
    selected: Option<usize>,
    open: bool,
    visibility: f32,
    animation_progress: f32,
}

impl RadialMenu {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::screen_space(&id_str, 0.5, 0.5),
            id: id_str,
            items: Vec::new(),
            selected: None,
            open: false,
            visibility: 1.0,
            animation_progress: 0.0,
        }
    }

    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn open(&mut self) {
        self.open = true;
        self.animation_progress = 0.0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.selected = None;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.selected.and_then(|i| self.items.get(i))
    }

    /// Select item by direction (for swipe gestures)
    pub fn select_by_direction(&mut self, direction: SwipeDirection) {
        if self.items.is_empty() {
            return;
        }

        // Map directions to item indices based on position
        let count = self.items.len();
        let index = match direction {
            SwipeDirection::Up => 0,
            SwipeDirection::Right => count / 4,
            SwipeDirection::Down => count / 2,
            SwipeDirection::Left => 3 * count / 4,
        } % count;

        if self.items[index].enabled {
            self.selected = Some(index);
        }
    }
}

impl OpticalWidget for RadialMenu {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::sphere(Point3D::ORIGIN, 1.0)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, dt: Duration, _ctx: &DisplayContext) {
        if self.open && self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + dt.as_secs_f32() * 4.0).min(1.0);
        } else if !self.open && self.animation_progress > 0.0 {
            self.animation_progress = (self.animation_progress - dt.as_secs_f32() * 4.0).max(0.0);
        }
    }

    fn handle_event(&mut self, event: &OpticalEvent) -> bool {
        if !self.open {
            return false;
        }

        match event {
            OpticalEvent::Gesture(gesture) => {
                match &gesture.gesture {
                    GestureType::Swipe { direction, .. } => {
                        self.select_by_direction(*direction);
                        true
                    }
                    GestureType::AirTap { .. } => {
                        // Confirm selection
                        if self.selected.is_some() {
                            self.close();
                            true
                        } else {
                            false
                        }
                    }
                    GestureType::Pinch { strength } if *strength > 0.8 => {
                        // Confirm selection
                        if self.selected.is_some() {
                            self.close();
                            true
                        } else {
                            false
                        }
                    }
                    GestureType::Fist | GestureType::OpenPalm => {
                        self.close();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn render(&self, backend: &mut dyn RenderBackend, _camera: &Transform) {
        if self.animation_progress < 0.01 || self.visibility < 0.1 {
            return;
        }

        let Some((cx, cy)) = self.anchor.screen_coords() else {
            return;
        };

        let count = self.items.len();
        if count == 0 {
            return;
        }

        let radius = 0.1 * self.animation_progress;
        let angle_step = std::f32::consts::TAU / count as f32;

        for (i, item) in self.items.iter().enumerate() {
            let angle = angle_step * i as f32 - std::f32::consts::FRAC_PI_2;
            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius * 0.5; // Squash for aspect ratio

            let is_selected = self.selected == Some(i);
            let color = if !item.enabled {
                Color::DarkGrey
            } else if is_selected {
                Color::GOLD
            } else {
                Color::White
            };

            // Draw item
            let icon_str = item.icon.to_string();
            backend.draw_hud_text(x, y, &icon_str, color);

            // Draw label for selected item
            if is_selected {
                backend.draw_hud_text(cx - 0.05, cy, &item.label, Color::White);
            }
        }

        // Draw center indicator
        backend.draw_hud_text(cx, cy, "◯", Color::HUD_CYAN);
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        Priority::High // Menu should be visible
    }
}
