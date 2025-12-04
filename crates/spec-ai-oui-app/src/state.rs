//! Application state for the OUI demo - practical AI assistant

use spec_ai_oui::{
    DisplayContext, DisplayMode, InformationDensity,
    spatial::Point3D,
    renderer::Color,
};

/// Calendar event
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub title: String,
    pub time: String,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub is_current: bool,
    pub minutes_until: i32,
}

/// A notification or message
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub source: String,
    pub title: String,
    pub preview: String,
    pub priority: NotificationPriority,
    pub timestamp: String,
    pub read: bool,
}

/// Notification priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn color(&self) -> Color {
        match self {
            NotificationPriority::Low => Color::Grey,
            NotificationPriority::Normal => Color::White,
            NotificationPriority::High => Color::Yellow,
            NotificationPriority::Urgent => Color::ALERT_RED,
        }
    }

    pub fn icon(&self) -> char {
        match self {
            NotificationPriority::Low => '○',
            NotificationPriority::Normal => '●',
            NotificationPriority::High => '◆',
            NotificationPriority::Urgent => '⚠',
        }
    }
}

/// Point of interest (person, place, or thing)
#[derive(Debug, Clone)]
pub struct PointOfInterest {
    pub id: String,
    pub name: String,
    pub category: PoiCategory,
    pub position: Point3D,
    pub distance_meters: f32,
    pub selected: bool,
    pub details: Vec<String>,
}

/// POI category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoiCategory {
    Person,
    Place,
    Navigation,
    Reminder,
    Search,
}

impl PoiCategory {
    pub fn color(&self) -> Color {
        match self {
            PoiCategory::Person => Color::HUD_CYAN,
            PoiCategory::Place => Color::STATUS_GREEN,
            PoiCategory::Navigation => Color::Yellow,
            PoiCategory::Reminder => Color::Rgb(255, 165, 0),
            PoiCategory::Search => Color::White,
        }
    }

    pub fn icon(&self) -> char {
        match self {
            PoiCategory::Person => '👤',
            PoiCategory::Place => '📍',
            PoiCategory::Navigation => '→',
            PoiCategory::Reminder => '⏰',
            PoiCategory::Search => '🔍',
        }
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub battery_percent: u8,
    pub is_charging: bool,
    pub wifi_connected: bool,
    pub wifi_strength: u8,
    pub bluetooth_connected: bool,
    pub location_enabled: bool,
}

/// Quick action button
#[derive(Debug, Clone)]
pub struct QuickAction {
    pub name: String,
    pub icon: char,
    pub available: bool,
    pub shortcut: String,
}

/// Current context/agenda
#[derive(Debug, Clone)]
pub struct DayContext {
    pub date: String,
    pub weather: String,
    pub temperature: String,
    pub location: String,
    pub upcoming_events: Vec<CalendarEvent>,
    pub tasks_remaining: u8,
}

/// UI panel focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    None,
    Calendar,
    Notifications,
    Navigation,
    Menu,
}

/// Demo application state
#[derive(Debug, Clone)]
pub struct DemoState {
    /// Current tick
    pub tick: u64,
    /// Current display mode
    pub mode: DisplayMode,
    /// Current information density
    pub density: InformationDensity,
    /// Current day context
    pub context: DayContext,
    /// System status
    pub system: SystemStatus,
    /// Notifications
    pub notifications: Vec<Notification>,
    /// Points of interest
    pub points_of_interest: Vec<PointOfInterest>,
    /// Quick actions
    pub quick_actions: Vec<QuickAction>,
    /// Focused panel
    pub focus: PanelFocus,
    /// Show calendar overlay
    pub show_calendar: bool,
    /// Show radial menu
    pub show_menu: bool,
    /// Selected menu item
    pub menu_selection: Option<usize>,
    /// Status message (toast)
    pub status_message: Option<String>,
    /// Compass heading
    pub heading: f32,
    /// Current gaze position (normalized)
    pub gaze_pos: (f32, f32),
    /// Current time display
    pub current_time: String,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            mode: DisplayMode::Navigate,
            density: InformationDensity::Normal,
            context: DayContext {
                date: "Wednesday, Dec 4".to_string(),
                weather: "Partly Cloudy".to_string(),
                temperature: "18°C".to_string(),
                location: "San Francisco".to_string(),
                upcoming_events: vec![
                    CalendarEvent {
                        title: "Team Standup".to_string(),
                        time: "10:00 AM".to_string(),
                        location: Some("Conf Room B".to_string()),
                        attendees: vec!["Alex".to_string(), "Jordan".to_string(), "Sam".to_string()],
                        is_current: false,
                        minutes_until: 45,
                    },
                    CalendarEvent {
                        title: "Product Review".to_string(),
                        time: "2:00 PM".to_string(),
                        location: Some("Main Hall".to_string()),
                        attendees: vec!["Product Team".to_string()],
                        is_current: false,
                        minutes_until: 285,
                    },
                ],
                tasks_remaining: 4,
            },
            system: SystemStatus {
                battery_percent: 78,
                is_charging: false,
                wifi_connected: true,
                wifi_strength: 3,
                bluetooth_connected: true,
                location_enabled: true,
            },
            notifications: vec![
                Notification {
                    id: "msg-1".to_string(),
                    source: "Messages".to_string(),
                    title: "Alex Chen".to_string(),
                    preview: "Can you review the PR when you get a chance?".to_string(),
                    priority: NotificationPriority::Normal,
                    timestamp: "9:12 AM".to_string(),
                    read: false,
                },
                Notification {
                    id: "cal-1".to_string(),
                    source: "Calendar".to_string(),
                    title: "Team Standup".to_string(),
                    preview: "Starting in 45 minutes".to_string(),
                    priority: NotificationPriority::Low,
                    timestamp: "9:15 AM".to_string(),
                    read: false,
                },
            ],
            points_of_interest: vec![
                PointOfInterest {
                    id: "nav-1".to_string(),
                    name: "Conf Room B".to_string(),
                    category: PoiCategory::Navigation,
                    position: Point3D::new(50.0, 0.0, 100.0),
                    distance_meters: 45.0,
                    selected: false,
                    details: vec!["Next meeting location".to_string()],
                },
                PointOfInterest {
                    id: "person-1".to_string(),
                    name: "Alex Chen".to_string(),
                    category: PoiCategory::Person,
                    position: Point3D::new(30.0, 0.0, 25.0),
                    distance_meters: 12.0,
                    selected: false,
                    details: vec!["Software Engineer".to_string(), "Messaged you".to_string()],
                },
                PointOfInterest {
                    id: "place-1".to_string(),
                    name: "Coffee Bar".to_string(),
                    category: PoiCategory::Place,
                    position: Point3D::new(-20.0, 0.0, 40.0),
                    distance_meters: 28.0,
                    selected: false,
                    details: vec!["Open now".to_string()],
                },
            ],
            quick_actions: vec![
                QuickAction {
                    name: "Call".to_string(),
                    icon: '📞',
                    available: true,
                    shortcut: "[1]".to_string(),
                },
                QuickAction {
                    name: "Message".to_string(),
                    icon: '💬',
                    available: true,
                    shortcut: "[2]".to_string(),
                },
                QuickAction {
                    name: "Navigate".to_string(),
                    icon: '🧭',
                    available: true,
                    shortcut: "[3]".to_string(),
                },
                QuickAction {
                    name: "Note".to_string(),
                    icon: '📝',
                    available: true,
                    shortcut: "[4]".to_string(),
                },
            ],
            focus: PanelFocus::None,
            show_calendar: false,
            show_menu: false,
            menu_selection: None,
            status_message: None,
            heading: 45.0,
            gaze_pos: (0.5, 0.5),
            current_time: "9:15 AM".to_string(),
        }
    }

    /// Update state based on context
    pub fn update(&mut self, ctx: &DisplayContext) {
        self.mode = ctx.mode;
        self.density = ctx.density;

        // Simulate time passing for demo
        if self.tick % 60 == 0 {
            // Decrease time until meeting
            for event in &mut self.context.upcoming_events {
                if event.minutes_until > 0 {
                    event.minutes_until -= 1;
                }
            }
        }

        // Clear status message after a while
        if self.tick % 100 == 0 {
            self.status_message = None;
        }
    }

    /// Select a POI
    pub fn select_poi(&mut self, poi_id: &str) {
        // Deselect all first
        for poi in &mut self.points_of_interest {
            poi.selected = poi.id == poi_id;
        }
    }

    /// Mark notification as read
    pub fn mark_read(&mut self, notif_id: &str) {
        if let Some(notif) = self.notifications.iter_mut().find(|n| n.id == notif_id) {
            notif.read = true;
        }
    }

    /// Trigger a quick action
    pub fn trigger_action(&mut self, index: usize) {
        if let Some(action) = self.quick_actions.get(index) {
            if action.available {
                self.status_message = Some(format!("{} activated", action.name));
            }
        }
    }

    /// Toggle calendar view
    pub fn toggle_calendar(&mut self) {
        self.show_calendar = !self.show_calendar;
        self.focus = if self.show_calendar { PanelFocus::Calendar } else { PanelFocus::None };
    }

    /// Get unread notification count
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read).count()
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}
