//! Fade transition effect

use std::time::Duration;

/// Fade transition states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeState {
    Hidden,
    FadingIn,
    Visible,
    FadingOut,
}

/// Fade transition effect
#[derive(Debug, Clone)]
pub struct FadeTransition {
    pub state: FadeState,
    pub progress: f32,
    pub duration: Duration,
}

impl Default for FadeTransition {
    fn default() -> Self {
        Self {
            state: FadeState::Hidden,
            progress: 0.0,
            duration: Duration::from_millis(300),
        }
    }
}

impl FadeTransition {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    pub fn fade_in(&mut self) {
        self.state = FadeState::FadingIn;
    }

    pub fn fade_out(&mut self) {
        self.state = FadeState::FadingOut;
    }

    pub fn update(&mut self, dt: Duration) {
        let delta = dt.as_secs_f32() / self.duration.as_secs_f32();

        match self.state {
            FadeState::FadingIn => {
                self.progress = (self.progress + delta).min(1.0);
                if self.progress >= 1.0 {
                    self.state = FadeState::Visible;
                }
            }
            FadeState::FadingOut => {
                self.progress = (self.progress - delta).max(0.0);
                if self.progress <= 0.0 {
                    self.state = FadeState::Hidden;
                }
            }
            _ => {}
        }
    }

    pub fn alpha(&self) -> f32 {
        self.progress
    }

    pub fn is_visible(&self) -> bool {
        self.progress > 0.0
    }
}
