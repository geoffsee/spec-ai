# spec-ai-oui

Optical User Interface framework for AR/glasses displays.

## Overview

This crate provides a complete framework for building optical user interfaces designed for augmented reality and smart glasses displays. It includes 3D spatial primitives, multi-modal input handling, context-aware rendering, and AR-optimized widgets.

**Status:** Stable but not yet functional with real AR hardware. Currently provides a terminal backend for development and simulation.

## Features

- **3D Spatial System**: Right-handed coordinate system with Point3D, Vector3D, Quaternion, Transform, and SpatialAnchor types
- **Abstract Render Backend**: Pluggable rendering with terminal simulation for development
- **Multi-Modal Input**: Gaze tracking, gesture detection, head tracking, and voice input abstractions
- **Context Awareness**: Display modes (Ambient, Active, Focused), attention state, and information density
- **Optical Widgets**: AR-optimized widgets including anchored labels, floating cards, HUD panels, and visual effects
- **Animation System**: Tween-based animations with easing functions
- **Glass-Morphism Theming**: Visual themes designed for optical displays

## Module Structure

```
src/
├── spatial/      # 3D coordinate system and transforms
│   ├── point3d   # 3D point type
│   ├── vector3d  # 3D vector with operations
│   ├── quaternion # Rotation representation
│   ├── transform # Combined position/rotation/scale
│   ├── bounds    # 3D bounding boxes
│   └── anchor    # Spatial anchoring for AR content
├── renderer/     # Rendering abstraction
│   ├── backend   # RenderBackend trait
│   ├── surface   # Surface capabilities
│   └── terminal/ # Terminal backend for development
├── context/      # Display context and awareness
│   ├── mode      # DisplayMode (Ambient/Active/Focused)
│   ├── attention # User attention state
│   ├── density   # Information density levels
│   └── priority  # Content priority
├── input/        # Multi-modal input
│   ├── gaze      # Eye tracking events
│   ├── gesture   # Gesture detection
│   ├── head      # Head tracking
│   ├── voice     # Voice input
│   ├── event     # OpticalEvent union type
│   └── simulator # Input simulation for testing
├── widget/       # Optical widgets
│   ├── traits    # OpticalWidget trait
│   ├── anchored/ # World-space anchored widgets
│   │   ├── label, marker, waypoint
│   ├── floating/ # Screen-relative widgets
│   │   ├── card, menu, tooltip
│   ├── hud/      # HUD widgets
│   │   ├── panel, indicator, compass, reticle
│   └── effects/  # Visual effects
│       ├── fade, glow, scan_line
├── layout/       # Spatial layout engine
│   ├── zone      # AttentionZone management
│   ├── screen_space # Screen-space layout
│   └── spatial   # 3D spatial layout
├── animation/    # Animation system
│   ├── tween     # Value interpolation
│   └── easing    # Easing functions
├── theme/        # Visual theming
│   ├── glass     # Glass-morphism theme
│   └── palette   # Color palettes
├── audio/        # Audio feedback
│   ├── backend   # Audio backend trait
│   └── notification # Notification sounds
└── app/          # Application framework
    └── framework # OpticalApp trait and runner
```

## Coordinate System

OUI uses a right-handed coordinate system:
- **X-axis**: Positive = right, Negative = left
- **Y-axis**: Positive = up, Negative = down
- **Z-axis**: Positive = forward (into screen), Negative = backward

This matches common AR/VR conventions where the user faces the positive Z direction.

## Usage

```rust
use spec_ai_oui::{
    OpticalApp, DisplayContext, DisplayMode,
    renderer::terminal::TerminalBackend,
    Point3D, SpatialAnchor, AnchorType,
};

// Implement OpticalApp trait for your application
struct MyApp { /* state */ }

impl OpticalApp for MyApp {
    fn update(&mut self, context: &DisplayContext) {
        // Update app state based on context
    }

    fn handle_event(&mut self, event: OpticalEvent) -> bool {
        // Handle input events, return false to quit
        true
    }

    fn render(&self, backend: &mut dyn RenderBackend) {
        // Render your UI
    }
}
```

## Widget Types

### Anchored Widgets
Attached to real-world positions, remain fixed in 3D space:
- `AnchoredLabel` - Text labels at world positions
- `Marker` - Visual markers for points of interest
- `Waypoint` - Navigation waypoints with distance indicators

### Floating Widgets
Screen-relative, move with the user's view:
- `Card` - Information cards with titles and content
- `Menu` - Selection menus
- `Tooltip` - Contextual tooltips

### HUD Widgets
Fixed to the display, always visible:
- `Panel` - Information panels
- `Indicator` - Status indicators
- `Compass` - Directional compass
- `Reticle` - Center-screen reticle

### Effects
Visual enhancements:
- `Fade` - Opacity transitions
- `Glow` - Glowing effects
- `ScanLine` - Scan line animations

## License

MIT OR Apache-2.0
