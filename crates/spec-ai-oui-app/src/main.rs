//! OpenTelemetry Visualization UI
//!
//! A minimal optical interface that displays OpenTelemetry data streams.
//! The UI state is derived from incoming telemetry (spans, logs, metrics).
//!
//! Controls (simulating wearable ring):
//! - Up/Down or j/k: Scroll within focused panel
//! - Tab or Left/Right: Switch focus between menu and content
//! - Enter or Space: Select current item
//! - Esc or Backspace: Back to feed view
//! - Q: Quit
//!
//! Usage:
//!   oui-demo              # Run with mock telemetry data
//!   oui-demo --otlp 4317  # Run with OTLP receiver on port 4317

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let result = if args.len() > 2 && args[1] == "--otlp" {
        let port: u16 = args[2].parse().unwrap_or(4317);
        eprintln!("Starting OTLP receiver on port {}...", port);
        spec_ai_oui_app::run_with_otlp(port)
    } else {
        spec_ai_oui_app::run_demo()
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
