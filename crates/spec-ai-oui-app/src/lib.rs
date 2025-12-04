//! spec-ai-oui-app: Super OUI - Combined Intelligence Assistant
//!
//! Features combined from three domains:
//!
//! Social Intelligence:
//! - Person recognition with relationship context
//! - Emotional state detection and engagement tracking
//! - Rapport indicators (eye contact, mirroring, turn-taking)
//! - Conversation cues and topic suggestions
//!
//! Journalist Superpowers:
//! - Real-time fact-checking with verdict display
//! - Source reliability tracking
//! - Recording and capture modes
//! - Research documents with relevance scoring
//!
//! Practical Assistant:
//! - Calendar integration with countdown
//! - Smart notification triage
//! - Context-aware alerts
//! - Private mode for sensitive situations
//!
//! 8 Display Modes: Ambient, Social, Meeting, Research, Recording, Navigation, Private, Focus

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

/// The social intelligence demo application
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
