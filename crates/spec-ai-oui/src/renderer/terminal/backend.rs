//! Terminal backend implementation

use std::io::{self, Write};
use crossterm::{
    cursor, execute, queue,
    style::{self, SetBackgroundColor, SetForegroundColor, Print},
    terminal::{self, ClearType},
};

use crate::spatial::{Point3D, Transform};
use crate::renderer::{
    RenderBackend, RenderError, RenderGlyph,
    SurfaceCapabilities, Color,
};
use super::Projection;

/// Cell in the terminal buffer
#[derive(Clone)]
struct Cell {
    symbol: String,
    fg: Color,
    bg: Color,
    depth: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: " ".to_string(),
            fg: Color::White,
            bg: Color::Black,
            depth: f32::MAX,
        }
    }
}

/// Terminal rendering backend with depth buffer
pub struct TerminalBackend {
    /// Terminal width
    width: u32,
    /// Terminal height
    height: u32,
    /// Render buffer
    buffer: Vec<Cell>,
    /// Previous frame buffer for diff rendering
    prev_buffer: Vec<Cell>,
    /// Projection settings
    projection: Projection,
    /// Camera transform
    camera: Transform,
    /// Clear color
    clear_color: Color,
}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new() -> Result<Self, RenderError> {
        let (width, height) = terminal::size()
            .map_err(|e| RenderError::InitError(e.to_string()))?;

        let size = (width as usize) * (height as usize);
        let aspect = width as f32 / height as f32;

        Ok(Self {
            width: width as u32,
            height: height as u32,
            buffer: vec![Cell::default(); size],
            prev_buffer: vec![Cell::default(); size],
            projection: Projection::perspective(70.0, aspect),
            camera: Transform::identity(),
            clear_color: Color::Rgb(5, 7, 12), // Dark blue-black for HUD feel
        })
    }

    /// Refresh terminal size
    pub fn refresh_size(&mut self) -> Result<(), RenderError> {
        let (width, height) = terminal::size()
            .map_err(|e| RenderError::TerminalError(e.to_string()))?;

        if width as u32 != self.width || height as u32 != self.height {
            self.width = width as u32;
            self.height = height as u32;
            let size = (width as usize) * (height as usize);
            self.buffer = vec![Cell::default(); size];
            self.prev_buffer = vec![Cell::default(); size];
            self.projection.aspect = width as f32 / height as f32;
        }

        Ok(())
    }

    /// Get buffer index for coordinates
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width as u16 && y < self.height as u16 {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    /// Set a cell in the buffer with depth test
    fn set_cell(&mut self, x: u16, y: u16, symbol: String, fg: Color, depth: f32) {
        if let Some(idx) = self.index(x, y) {
            let cell = &mut self.buffer[idx];
            if depth < cell.depth {
                cell.symbol = symbol;
                cell.fg = fg;
                cell.depth = depth;
            }
        }
    }

    /// Set a cell without depth test (for HUD elements)
    fn set_cell_hud(&mut self, x: u16, y: u16, symbol: String, fg: Color) {
        if let Some(idx) = self.index(x, y) {
            let cell = &mut self.buffer[idx];
            cell.symbol = symbol;
            cell.fg = fg;
            cell.depth = 0.0; // HUD is always on top
        }
    }
}

impl Default for TerminalBackend {
    fn default() -> Self {
        Self::new().expect("Failed to initialize terminal backend")
    }
}

impl RenderBackend for TerminalBackend {
    fn capabilities(&self) -> SurfaceCapabilities {
        SurfaceCapabilities {
            width: self.width,
            height: self.height,
            supports_depth: true,
            supports_alpha: false, // Terminal has limited alpha support
            fov_horizontal: Some(self.projection.fov.to_degrees()),
            fov_vertical: Some(self.projection.fov.to_degrees() / self.projection.aspect),
        }
    }

    fn begin_frame(&mut self) -> Result<(), RenderError> {
        self.refresh_size()?;

        // Swap buffers
        std::mem::swap(&mut self.buffer, &mut self.prev_buffer);

        // Clear buffer
        for cell in &mut self.buffer {
            *cell = Cell {
                symbol: " ".to_string(),
                fg: Color::White,
                bg: self.clear_color,
                depth: f32::MAX,
            };
        }

        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        let mut stdout = io::stdout();

        // Hide cursor during rendering
        queue!(stdout, cursor::Hide)
            .map_err(|e| RenderError::FrameError(e.to_string()))?;

        // Diff render - only update changed cells
        for y in 0..self.height as u16 {
            for x in 0..self.width as u16 {
                if let Some(idx) = self.index(x, y) {
                    let cell = &self.buffer[idx];
                    let prev = &self.prev_buffer[idx];

                    // Only update if changed
                    if cell.symbol != prev.symbol || cell.fg != prev.fg || cell.bg != prev.bg {
                        queue!(
                            stdout,
                            cursor::MoveTo(x, y),
                            SetForegroundColor(cell.fg.to_crossterm()),
                            SetBackgroundColor(cell.bg.to_crossterm()),
                            Print(&cell.symbol)
                        ).map_err(|e| RenderError::FrameError(e.to_string()))?;
                    }
                }
            }
        }

        // Show cursor and flush
        queue!(stdout, cursor::Show)
            .map_err(|e| RenderError::FrameError(e.to_string()))?;
        stdout.flush()
            .map_err(|e| RenderError::FrameError(e.to_string()))?;

        Ok(())
    }

    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        for cell in &mut self.buffer {
            cell.bg = color;
            cell.symbol = " ".to_string();
            cell.depth = f32::MAX;
        }
    }

    fn draw_glyph(&mut self, glyph: &RenderGlyph, camera: &Transform) {
        if let Some((x, y, depth)) = self.projection.project_to_screen(
            glyph.position,
            camera,
            self.width,
            self.height,
        ) {
            // Apply alpha to color (simple threshold for terminal)
            if glyph.alpha > 0.3 {
                let color = if glyph.alpha < 0.7 {
                    // Dim the color for semi-transparent glyphs
                    match glyph.color {
                        Color::Rgb(r, g, b) => Color::Rgb(
                            (r as f32 * glyph.alpha) as u8,
                            (g as f32 * glyph.alpha) as u8,
                            (b as f32 * glyph.alpha) as u8,
                        ),
                        c => c,
                    }
                } else {
                    glyph.color
                };

                self.set_cell(x, y, glyph.symbol.clone(), color, depth);
            }
        }
    }

    fn draw_line(&mut self, from: Point3D, to: Point3D, color: Color, alpha: f32, camera: &Transform) {
        // Project both endpoints
        let from_screen = self.projection.project_to_screen(from, camera, self.width, self.height);
        let to_screen = self.projection.project_to_screen(to, camera, self.width, self.height);

        if let (Some((x1, y1, d1)), Some((x2, y2, d2))) = (from_screen, to_screen) {
            // Bresenham's line algorithm
            let dx = (x2 as i32 - x1 as i32).abs();
            let dy = -(y2 as i32 - y1 as i32).abs();
            let sx = if x1 < x2 { 1 } else { -1 };
            let sy = if y1 < y2 { 1 } else { -1 };
            let mut err = dx + dy;

            let mut x = x1 as i32;
            let mut y = y1 as i32;
            let steps = dx.max(-dy) as f32;

            loop {
                // Interpolate depth
                let t = if steps > 0.0 {
                    ((x - x1 as i32).abs() + (y - y1 as i32).abs()) as f32 / steps
                } else {
                    0.0
                };
                let depth = d1 + (d2 - d1) * t;

                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                    let symbol = if dx > -dy { "─" } else if -dy > dx { "│" } else { "·" };
                    self.set_cell(x as u16, y as u16, symbol.to_string(), color, depth);
                }

                if x == x2 as i32 && y == y2 as i32 {
                    break;
                }

                let e2 = 2 * err;
                if e2 >= dy {
                    if x == x2 as i32 {
                        break;
                    }
                    err += dy;
                    x += sx;
                }
                if e2 <= dx {
                    if y == y2 as i32 {
                        break;
                    }
                    err += dx;
                    y += sy;
                }
            }
        }
    }

    fn draw_hud_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let sx = (x * self.width as f32) as u16;
        let sy = (y * self.height as f32) as u16;
        let sw = (width * self.width as f32) as u16;
        let sh = (height * self.height as f32) as u16;

        // Draw box with border characters
        for dy in 0..sh {
            for dx in 0..sw {
                let symbol = if dy == 0 || dy == sh - 1 {
                    if dx == 0 || dx == sw - 1 {
                        if dy == 0 { if dx == 0 { "┌" } else { "┐" } }
                        else { if dx == 0 { "└" } else { "┘" } }
                    } else {
                        "─"
                    }
                } else if dx == 0 || dx == sw - 1 {
                    "│"
                } else {
                    " "
                };

                self.set_cell_hud(sx + dx, sy + dy, symbol.to_string(), color);
            }
        }
    }

    fn draw_hud_text(&mut self, x: f32, y: f32, text: &str, color: Color) {
        let sx = (x * self.width as f32) as u16;
        let sy = (y * self.height as f32) as u16;

        for (i, ch) in text.chars().enumerate() {
            let char_x = sx + i as u16;
            if char_x < self.width as u16 {
                self.set_cell_hud(char_x, sy, ch.to_string(), color);
            }
        }
    }

    fn project(&self, point: Point3D, camera: &Transform) -> Option<(f32, f32)> {
        self.projection.project(point, camera).map(|(x, y, _)| (x, y))
    }

    fn is_visible(&self, point: Point3D, camera: &Transform) -> bool {
        self.projection.is_visible(point, camera)
    }

    fn camera(&self) -> &Transform {
        &self.camera
    }

    fn set_camera(&mut self, camera: Transform) {
        self.camera = camera;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_calculation() {
        // Note: Can't easily test terminal backend without a terminal
        // This is more of a compile-time check
        let _ = Cell::default();
    }
}
