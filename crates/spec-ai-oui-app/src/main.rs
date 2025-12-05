//! Minimal OUI demo - ring-style navigation
//!
//! Controls (simulating wearable ring):
//! - Up/Down or j/k: Scroll within focused panel
//! - Tab or Left/Right: Switch focus between menu and events
//! - Enter or Space: Select current item
//! - Esc or Backspace: Back to events view
//! - Q: Quit

fn main() {
    if let Err(e) = spec_ai_oui_app::run_demo() {
        eprintln!("Error running demo: {}", e);
        std::process::exit(1);
    }
}
