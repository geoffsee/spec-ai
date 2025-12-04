//! spec-ai-oui-app: Demo application for optical user interface
//!
//! Practical AI assistant for smart eyewear featuring:
//! - Calendar and meeting reminders
//! - Notification management
//! - Points of interest and navigation
//! - Context-aware information display
//! - Ergonomic design based on clinical optometry principles

mod state;
mod handlers;
pub mod ui;

use std::time::Duration;

use spec_ai_oui::{
    OpticalApp,
    OpticalEvent,
    DisplayContext,
    renderer::RenderBackend,
};

use state::DemoState;
use handlers::handle_event;
use ui::render_demo;

/// The demo application
pub struct OuiDemo;

impl OpticalApp for OuiDemo {
    type State = DemoState;

    fn init(&self) -> Self::State {
        DemoState::new()
    }

    fn handle_event(&mut self, event: OpticalEvent, state: &mut Self::State) -> bool {
        handle_event(event, state)
    }

    fn update(&mut self, state: &mut Self::State, ctx: &DisplayContext) {
        state.update(ctx);
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
    let mut runner = OpticalAppRunner::new(app)?
        .with_tick_rate(Duration::from_millis(100));

    runner.run()
}
