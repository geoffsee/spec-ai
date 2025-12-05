//! spec-ai-oui-app: Minimal OUI demo
//!
//! Simple two-panel interface:
//! - Left: Menu (Mode, Alerts, Settings)
//! - Right: Event feed (default) or alternate views
//!
//! Ring-style controls:
//! - Up/Down or j/k: Navigate
//! - Tab or Left/Right: Switch panel focus
//! - Enter or Space: Select
//! - Esc or Backspace: Back
//! - Q: Quit

mod handlers;
mod state;
pub mod ui;

use std::time::Duration;

use spec_ai_oui::{renderer::RenderBackend, DisplayContext, OpticalApp, OpticalEvent};

use handlers::handle_event;
use state::DemoState;
use ui::render_demo;

/// Minimal OUI demo application
pub struct OuiDemo;

impl OpticalApp for OuiDemo {
    type State = DemoState;

    fn init(&self) -> Self::State {
        DemoState::new()
    }

    fn handle_event(&mut self, event: OpticalEvent, state: &mut Self::State) -> bool {
        handle_event(event, state)
    }

    fn update(&mut self, _state: &mut Self::State, _ctx: &DisplayContext) {
        // No context-dependent updates needed
    }

    fn render(&self, state: &Self::State, backend: &mut dyn RenderBackend) {
        render_demo(state, backend);
    }

    fn on_tick(&mut self, state: &mut Self::State) {
        state.tick = state.tick.wrapping_add(1);
    }
}

/// Run the demo application
pub fn run_demo() -> std::io::Result<()> {
    use spec_ai_oui::app::OpticalAppRunner;

    let app = OuiDemo;
    let mut runner = OpticalAppRunner::new(app)?.with_tick_rate(Duration::from_millis(100));

    runner.run()
}
