//! Styling system for terminal text

mod color;
pub mod markdown;
mod modifier;
mod style;
mod styled;
pub mod text_utils;

pub use color::Color;
pub use markdown::{parse_markdown, MarkdownConfig};
pub use modifier::Modifier;
pub use style::Style;
pub use styled::{Line, Span, Text};
pub use text_utils::{truncate, wrap_text};
