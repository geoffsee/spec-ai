//! Minimal OUI state - menu + event feed for ring-based navigation

use spec_ai_oui::renderer::Color;

/// Menu items on the left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Mode,
    Alerts,
    Settings,
}

impl MenuItem {
    pub fn all() -> &'static [MenuItem] {
        &[MenuItem::Mode, MenuItem::Alerts, MenuItem::Settings]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::Mode => "Mode",
            MenuItem::Alerts => "Alerts",
            MenuItem::Settings => "Settings",
        }
    }
}

/// The view currently displayed on the right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Events,
    Mode,
    Alerts,
    Settings,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Events => "Events",
            View::Mode => "Mode",
            View::Alerts => "Alerts",
            View::Settings => "Settings",
        }
    }
}

/// An event in the rolling feed
#[derive(Debug, Clone)]
pub struct Event {
    pub id: usize,
    pub title: String,
    pub detail: String,
    pub timestamp: String,
    pub priority: EventPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPriority {
    Low,
    Normal,
    High,
}

impl EventPriority {
    pub fn color(&self) -> Color {
        match self {
            EventPriority::Low => Color::DarkGrey,
            EventPriority::Normal => Color::Grey,
            EventPriority::High => Color::Yellow,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            EventPriority::Low => "○",
            EventPriority::Normal => "●",
            EventPriority::High => "◆",
        }
    }
}

/// Which panel has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Menu,
    Events,
}

/// Main application state
#[derive(Debug, Clone)]
pub struct DemoState {
    pub tick: u64,
    pub focus: Focus,
    pub view: View,
    pub menu_index: usize,
    pub event_index: usize,
    pub events: Vec<Event>,
    pub scroll_offset: usize,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            focus: Focus::Menu,
            view: View::Events,
            menu_index: 0,
            event_index: 0,
            scroll_offset: 0,
            events: vec![
                Event {
                    id: 1,
                    title: "System initialized".into(),
                    detail: "All subsystems nominal".into(),
                    timestamp: "09:00".into(),
                    priority: EventPriority::Low,
                },
                Event {
                    id: 2,
                    title: "Connection established".into(),
                    detail: "Ring input active".into(),
                    timestamp: "09:01".into(),
                    priority: EventPriority::Normal,
                },
                Event {
                    id: 3,
                    title: "Calibration complete".into(),
                    detail: "Focus tracking ready".into(),
                    timestamp: "09:02".into(),
                    priority: EventPriority::Normal,
                },
                Event {
                    id: 4,
                    title: "Alert pending".into(),
                    detail: "Review recommended".into(),
                    timestamp: "09:05".into(),
                    priority: EventPriority::High,
                },
                Event {
                    id: 5,
                    title: "Status update".into(),
                    detail: "Environment scan done".into(),
                    timestamp: "09:08".into(),
                    priority: EventPriority::Normal,
                },
                Event {
                    id: 6,
                    title: "Sync complete".into(),
                    detail: "Data refreshed".into(),
                    timestamp: "09:10".into(),
                    priority: EventPriority::Low,
                },
            ],
        }
    }

    /// Move selection up within current focus
    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::Menu => {
                let len = MenuItem::all().len();
                self.menu_index = if self.menu_index == 0 {
                    len - 1
                } else {
                    self.menu_index - 1
                };
            }
            Focus::Events => {
                if self.event_index > 0 {
                    self.event_index -= 1;
                    if self.event_index < self.scroll_offset {
                        self.scroll_offset = self.event_index;
                    }
                }
            }
        }
    }

    /// Move selection down within current focus
    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::Menu => {
                let len = MenuItem::all().len();
                self.menu_index = (self.menu_index + 1) % len;
            }
            Focus::Events => {
                if self.event_index < self.events.len().saturating_sub(1) {
                    self.event_index += 1;
                    // Keep selected item visible (assume 5 visible)
                    if self.event_index >= self.scroll_offset + 5 {
                        self.scroll_offset = self.event_index.saturating_sub(4);
                    }
                }
            }
        }
    }

    /// Toggle focus between menu and events
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Menu => Focus::Events,
            Focus::Events => Focus::Menu,
        };
    }

    /// Select current item (ring tap)
    pub fn select(&mut self) {
        match self.focus {
            Focus::Menu => {
                let item = MenuItem::all()[self.menu_index];
                self.view = match item {
                    MenuItem::Mode => View::Mode,
                    MenuItem::Alerts => View::Alerts,
                    MenuItem::Settings => View::Settings,
                };
                self.focus = Focus::Events;
            }
            Focus::Events => {
                // Event selected - could trigger action
                // For now just toggles back to default view
                self.view = View::Events;
            }
        }
    }

    /// Back to default events view
    pub fn back(&mut self) {
        self.view = View::Events;
        self.focus = Focus::Menu;
    }

    pub fn selected_event(&self) -> Option<&Event> {
        self.events.get(self.event_index)
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}
