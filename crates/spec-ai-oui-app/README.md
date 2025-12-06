# spec-ai-oui-app

Demo application for the spec-ai-oui optical interface framework, featuring OpenTelemetry data visualization.

## Overview

This application demonstrates the OUI framework by providing a minimal optical interface for visualizing OpenTelemetry data streams. The UI displays traces, spans, and services from either mock data or a live OTLP receiver.

**Status:** Stable but not functional with real AR hardware. Uses the terminal backend for development and demonstration.

## Features

- Real-time OpenTelemetry span and trace visualization
- Two-panel interface (menu + content feed)
- Mock telemetry data for offline demos
- OTLP gRPC receiver for live telemetry
- Ring-style control scheme (designed for wearable input)

## Installation

```bash
# From workspace root
cargo build -p spec-ai-oui-app --release

# Binary is available at target/release/oui-demo
```

## Usage

```bash
# Run with mock telemetry data (default)
oui-demo

# Run with OTLP receiver on specified port
oui-demo --otlp 4317
```

## Controls

Designed to simulate a wearable ring controller:

| Key | Action |
|-----|--------|
| `Up` / `k` | Navigate up / scroll up |
| `Down` / `j` | Navigate down / scroll down |
| `Tab` / `Left` / `Right` | Switch focus between panels |
| `Enter` / `Space` | Select current item |
| `Esc` / `Backspace` | Back to feed view |
| `q` | Quit |
| `Ctrl+Q` | Force quit |

## Interface Layout

```
┌─────────┬───────────────────────────┐
│  Menu   │      Content Feed         │
├─────────┤                           │
│ Traces  │  [Span] user-service      │
│ Spans   │    GET /api/users         │
│ Services│    duration: 45ms         │
│         │                           │
│         │  [Span] db-service        │
│         │    SELECT * FROM users    │
│         │    duration: 12ms         │
│         │                           │
└─────────┴───────────────────────────┘
```

## Configuration

The app uses `AppConfig` for customization:

```rust
use spec_ai_oui_app::{run_app, AppConfig};

let config = AppConfig {
    tick_rate: Duration::from_millis(100),  // UI refresh rate
    otlp_port: 4317,                         // OTLP receiver port
    use_mock_data: false,                    // Use real telemetry
};

run_app(config).await?;
```

## Sending Telemetry

When running with `--otlp`, the app starts a gRPC server that accepts OpenTelemetry traces. Configure your application to send traces to `localhost:4317`:

```bash
# Example with OpenTelemetry collector
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export OTEL_TRACES_EXPORTER="otlp"
```

## Architecture

```
main.rs          # Entry point, CLI argument parsing
lib.rs           # App runner, main event loop
├── state.rs     # Application state management
├── ui.rs        # UI rendering logic
├── handlers.rs  # Event handling
├── telemetry.rs # Telemetry data types
└── receiver/    # OTLP receiver
    ├── mod.rs   # Receiver config and startup
    └── mock.rs  # Mock telemetry generator
```

## Dependencies

- `spec-ai-oui` - OUI framework
- `opentelemetry` / `opentelemetry_sdk` - OpenTelemetry integration
- `tonic` - gRPC server for OTLP receiver
- `crossterm` - Terminal handling

## License

MIT OR Apache-2.0
