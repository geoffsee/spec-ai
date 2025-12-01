mod backend;
mod handlers;
mod models;
mod state;
mod ui;

use anyhow::Result;
use backend::{spawn_backend, BackendEvent, BackendHandle, BackendRequest};
use handlers::{handle_event, on_tick};
use spec_ai_tui::{
    app::{App, AppRunner},
    buffer::Buffer,
    event::Event,
    geometry::Rect,
};
use state::AppState;
use std::path::PathBuf;
use std::sync::Mutex;

struct SpecAiTuiApp {
    backend_tx: tokio::sync::mpsc::UnboundedSender<BackendRequest>,
    backend_rx: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<BackendEvent>>>,
}

impl SpecAiTuiApp {
    fn new(handle: BackendHandle) -> Self {
        Self {
            backend_tx: handle.request_tx,
            backend_rx: Mutex::new(Some(handle.event_rx)),
        }
    }
}

impl App for SpecAiTuiApp {
    type State = AppState;

    fn init(&self) -> Self::State {
        let rx = self
            .backend_rx
            .lock()
            .expect("backend receiver poisoned")
            .take()
            .expect("backend receiver already taken");
        AppState::new(rx)
    }

    fn handle_event(&mut self, event: Event, state: &mut Self::State) -> bool {
        handle_event(event, state, &self.backend_tx)
    }

    fn on_tick(&mut self, state: &mut Self::State) {
        on_tick(state);
    }

    fn render(&self, state: &Self::State, area: Rect, buf: &mut Buffer) {
        ui::render(state, area, buf);
    }
}

/// Run the spec-ai TUI app, optionally providing an explicit config path.
pub async fn run_tui(config_path: Option<PathBuf>) -> Result<()> {
    let backend = spawn_backend(config_path)?;
    let app = SpecAiTuiApp::new(backend);
    let mut runner = AppRunner::new(app)?;
    runner.run().await?;
    Ok(())
}
