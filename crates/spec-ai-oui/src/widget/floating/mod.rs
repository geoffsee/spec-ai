//! Floating widgets for information display
//!
//! Floating panels and cards that can appear in space or follow the user.

mod card;
mod menu;
mod tooltip;

pub use card::InfoCard;
pub use menu::{MenuItem, RadialMenu};
pub use tooltip::Tooltip;
