# spec-ai-tui

Terminal User Interface framework for spec-ai built from scratch on crossterm.

## Overview

This crate provides a complete TUI framework with:

- **Geometry Primitives**: `Rect`, `Point`, `Size` for layout calculations
- **Cell-Based Buffer**: Efficient diff rendering system
- **Terminal Abstraction**: Backend over crossterm for cross-platform support
- **Layout Engine**: Constraint-based layout with flex support
- **Widget System**: Stateful and interactive widget traits
- **Event Loop**: Async event handling integrated with tokio
- **Application Framework**: Elm-inspired architecture for building apps

## Architecture

```
spec-ai-tui
├── app         # Application framework and runner
├── buffer      # Cell-based screen buffer with diff rendering
├── event       # Input events and async event loop
├── geometry    # Point, Rect, Size primitives
├── layout      # Constraint-based layout engine
├── style       # Colors, modifiers, and text styling
├── terminal    # Terminal backend abstraction
└── widget      # Widget traits and built-in widgets
```

## Built-in Widgets

- **Block**: Container with borders and titles
- **Paragraph**: Text display with wrapping
- **Input**: Single-line text input
- **Editor**: Multi-line text editor
- **StatusBar**: Status line display
- **SlashMenu**: Command menu overlay
- **Overlay**: Modal overlay container

## Usage

```rust
use spec_ai_tui::{
    App, AppRunner, Buffer, Event, Rect,
    Constraint, Direction, Layout,
    Color, Style, Widget,
};

struct MyApp;

impl App for MyApp {
    type State = MyState;

    fn init(&self) -> Self::State {
        MyState::default()
    }

    fn handle_event(&mut self, event: Event, state: &mut Self::State) -> bool {
        // Return true to quit
        false
    }

    fn render(&self, state: &Self::State, area: Rect, buf: &mut Buffer) {
        // Render your UI
    }
}
```

## Dependencies

- `crossterm` - Cross-platform terminal manipulation
- `tokio` - Async runtime for event loop
- `unicode-width` - Proper Unicode character width handling

## Usage

This is an internal crate primarily used by:
- `spec-ai-tui-app` - The interactive terminal application

For end-user documentation, see the main [spec-ai README](../../README.md).
