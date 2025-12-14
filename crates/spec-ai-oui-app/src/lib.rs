//! spec-ai-oui-app: OpenTelemetry Visualization UI
//!
//! A minimal optical interface that displays OpenTelemetry data streams.
//! The UI state is derived from incoming telemetry (spans, logs, metrics).
//!
//! Two-panel interface:
//! - Left: Menu (Traces, Spans, Services)
//! - Right: Event feed (default) or filtered views
//!
//! Ring-style controls:
//! - Up/Down or j/k: Navigate
//! - Tab or Left/Right: Switch panel focus
//! - Enter or Space: Select
//! - Esc or Backspace: Back
//! - Q: Quit

mod handlers;
pub mod receiver;
pub mod state;
pub mod telemetry;
pub mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::receiver::mock_telemetry_stream;
use crate::state::AppState;
use handlers::handle_event;
use spec_ai_oui::{
    context::DisplayContext,
    input::InputSimulator,
    renderer::{terminal::TerminalBackend, RenderBackend},
    OpticalEvent,
};
use ui::render_app;

/// Configuration for the OUI app
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Tick rate for UI updates
    pub tick_rate: Duration,
    /// OTLP receiver port (0 to disable, use mock data instead)
    pub otlp_port: u16,
    /// Use mock telemetry data for demo
    pub use_mock_data: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(100),
            otlp_port: 4317,
            use_mock_data: true, // Default to mock data for demo
        }
    }
}

/// Run the OpenTelemetry visualization app
pub async fn run_app(config: AppConfig) -> io::Result<()> {
    // Set up telemetry stream
    let mut telemetry_rx = if config.use_mock_data {
        mock_telemetry_stream()
    } else {
        let receiver_config = receiver::ReceiverConfig {
            grpc_addr: format!("127.0.0.1:{}", config.otlp_port).parse().unwrap(),
        };
        let handle = receiver::start_receiver(receiver_config)
            .await
            .map_err(|e| io::Error::other(e.to_string()))?;
        handle.events_rx
    };

    // Initialize terminal
    let mut backend = TerminalBackend::new().map_err(|e| io::Error::other(e.to_string()))?;
    let mut input_simulator = InputSimulator::new();
    let mut context = DisplayContext::default();

    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Initialize state
    let mut state = AppState::new();
    let mut last_tick = Instant::now();
    let mut running = true;

    // Main loop
    while running {
        // Poll for telemetry events (non-blocking)
        while let Ok(event) = telemetry_rx.try_recv() {
            state.process_telemetry(event);
        }

        // Poll for input events
        let timeout = config
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Check for quit
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    running = false;
                    continue;
                }

                // Convert to optical events
                let events = input_simulator.process_key(key);
                for event in events {
                    if !handle_event(event, &mut state) {
                        running = false;
                        break;
                    }
                }
            }
        }

        // Check for tick
        if last_tick.elapsed() >= config.tick_rate {
            // Update context
            context.update(last_tick.elapsed());

            // Update tick counter
            state.tick = state.tick.wrapping_add(1);

            // Update camera from simulator
            backend.set_camera(input_simulator.head_transform());

            // Render
            backend
                .begin_frame()
                .map_err(|e| io::Error::other(e.to_string()))?;

            render_app(&state, &mut backend);

            backend
                .end_frame()
                .map_err(|e| io::Error::other(e.to_string()))?;

            // Send tick event
            handle_event(OpticalEvent::Tick, &mut state);

            last_tick = Instant::now();
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

/// Run the demo application with default config
pub fn run_demo() -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_app(AppConfig::default()))
}

/// Run with OTLP receiver enabled (no mock data)
pub fn run_with_otlp(port: u16) -> io::Result<()> {
    let config = AppConfig {
        use_mock_data: false,
        otlp_port: port,
        ..Default::default()
    };
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_app(config))
}
