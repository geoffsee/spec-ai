//! Optical application framework

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::context::DisplayContext;
use crate::input::{InputSimulator, OpticalEvent};
use crate::renderer::{terminal::TerminalBackend, RenderBackend};

/// Optical application trait
pub trait OpticalApp {
    /// Application state type
    type State;

    /// Initialize application state
    fn init(&self) -> Self::State;

    /// Handle an optical event, return true to continue, false to quit
    fn handle_event(&mut self, event: OpticalEvent, state: &mut Self::State) -> bool;

    /// Update application state
    fn update(&mut self, state: &mut Self::State, ctx: &DisplayContext);

    /// Render the application
    fn render(&self, state: &Self::State, backend: &mut dyn RenderBackend);

    /// Called each tick (for animations)
    fn on_tick(&mut self, _state: &mut Self::State) {}
}

/// Application runner for optical apps
pub struct OpticalAppRunner<A: OpticalApp> {
    app: A,
    backend: TerminalBackend,
    input_simulator: InputSimulator,
    context: DisplayContext,
    tick_rate: Duration,
    running: bool,
}

impl<A: OpticalApp> OpticalAppRunner<A> {
    /// Create a new app runner
    pub fn new(app: A) -> io::Result<Self> {
        let backend = TerminalBackend::new().map_err(|e| io::Error::other(e.to_string()))?;

        Ok(Self {
            app,
            backend,
            input_simulator: InputSimulator::new(),
            context: DisplayContext::default(),
            tick_rate: Duration::from_millis(100),
            running: true,
        })
    }

    /// Set tick rate
    pub fn with_tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Run the application
    pub fn run(&mut self) -> io::Result<()> {
        // Enter alternate screen and raw mode
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        // Initialize state
        let mut state = self.app.init();
        let mut last_tick = Instant::now();

        // Main loop
        while self.running {
            // Poll for events
            let timeout = self
                .tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_default();

            if event::poll(timeout)? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    // Check for quit
                    if key.code == KeyCode::Char('q')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        self.running = false;
                        continue;
                    }

                    // Convert to optical events
                    let events = self.input_simulator.process_key(key);
                    for event in events {
                        if !self.app.handle_event(event, &mut state) {
                            self.running = false;
                            break;
                        }
                    }
                }
            }

            // Check for tick
            if last_tick.elapsed() >= self.tick_rate {
                // Update context
                self.context.update(last_tick.elapsed());

                // Update app
                self.app.update(&mut state, &self.context);
                self.app.on_tick(&mut state);

                // Update camera from simulator
                self.backend
                    .set_camera(self.input_simulator.head_transform());

                // Render
                self.backend
                    .begin_frame()
                    .map_err(|e| io::Error::other(e.to_string()))?;

                self.app.render(&state, &mut self.backend);

                self.backend
                    .end_frame()
                    .map_err(|e| io::Error::other(e.to_string()))?;

                // Send tick event
                self.app.handle_event(OpticalEvent::Tick, &mut state);

                last_tick = Instant::now();
            }
        }

        // Cleanup
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }
}
