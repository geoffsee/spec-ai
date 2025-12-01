# spec-ai-tui-app

Interactive terminal UI application for spec-ai built on spec-ai-tui.

## Overview

This crate provides a full-featured terminal application for interacting with spec-ai agents. It uses the `spec-ai-tui` framework for rendering and the `spec-ai-core` runtime for agent execution.

- **Chat Interface**: Interactive conversation with AI agents
- **Backend Integration**: Async communication with spec-ai-core
- **State Management**: Elm-inspired application state handling
- **Event Handling**: Keyboard and terminal event processing

## Architecture

```
spec-ai-tui-app
├── backend.rs    # Async backend for agent communication
├── handlers.rs   # Event handlers for user input
├── models.rs     # Data models for UI state
├── state.rs      # Application state management
└── ui.rs         # UI rendering logic
```

## Running

The TUI can be launched via the `run_tui` function:

```rust
use spec_ai_tui_app::run_tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_tui(None).await
}
```

Or with an explicit config path:

```rust
use spec_ai_tui_app::run_tui;
use std::path::PathBuf;

run_tui(Some(PathBuf::from("~/.config/spec-ai/config.toml"))).await?;
```

## Dependencies

- `spec-ai-core` - Agent runtime and tool execution
- `spec-ai-tui` - TUI framework for rendering
- `tokio` - Async runtime
- `chrono` - Timestamp handling

## Usage

This crate is typically invoked through `spec-ai-cli` rather than used directly.

For end-user documentation, see the main [spec-ai README](../../README.md).
