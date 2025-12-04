//! OUI Demo Application - Agent 007 Style Optical Interface
//!
//! Controls:
//! - Arrow keys: Move gaze
//! - WASD: Head rotation
//! - Space: Air tap (select)
//! - G: Grab gesture
//! - H/J/K/L: Swipe gestures
//! - 1-9: Simulated voice commands
//! - Tab: Switch focus
//! - M: Toggle menu
//! - Ctrl+Q: Quit

fn main() {
    if let Err(e) = spec_ai_oui_app::run_demo() {
        eprintln!("Error running demo: {}", e);
        std::process::exit(1);
    }
}
