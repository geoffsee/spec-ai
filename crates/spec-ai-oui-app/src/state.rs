//! Super OUI - Combined Intelligence Assistant
//!
//! All-in-one features:
//! - Person recognition with emotional/engagement tracking
//! - Rapport indicators and conversation cues
//! - Real-time fact-checking and verification
//! - Smart notifications with context awareness
//! - Calendar/meeting integration
//! - Recording and capture
//! - Research document access

use spec_ai_oui::{
    DisplayContext, DisplayMode, InformationDensity,
    spatial::Point3D,
    renderer::Color,
};

// ============================================================================
// PERSON INTELLIGENCE
// ============================================================================

#[derive(Debug, Clone)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub organization: Option<String>,
    pub relationship: Relationship,
    pub reliability: SourceReliability,
    pub position: Point3D,
    pub distance_meters: f32,
    pub selected: bool,
    pub emotional_state: EmotionalState,
    pub comm_style: CommStyle,
    pub engagement: f32,
    pub interests: Vec<String>,
    pub last_interaction: Option<String>,
    pub shared_context: Vec<String>,
    pub hooks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship { Unknown, Acquaintance, Colleague, Friend, VIP, Source }

impl Relationship {
    pub fn color(&self) -> Color {
        match self {
            Self::Unknown => Color::Grey, Self::Acquaintance => Color::White,
            Self::Colleague => Color::HUD_CYAN, Self::Friend => Color::STATUS_GREEN,
            Self::VIP => Color::GOLD, Self::Source => Color::Yellow,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unknown => "NEW", Self::Acquaintance => "MET",
            Self::Colleague => "WORK", Self::Friend => "FRIEND",
            Self::VIP => "VIP", Self::Source => "SOURCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceReliability { Verified, Established, Unknown, Caution }

impl SourceReliability {
    pub fn color(&self) -> Color {
        match self {
            Self::Verified => Color::STATUS_GREEN, Self::Established => Color::HUD_CYAN,
            Self::Unknown => Color::Grey, Self::Caution => Color::Yellow,
        }
    }
    pub fn icon(&self) -> char {
        match self { Self::Verified => '✓', Self::Established => '●', Self::Unknown => '?', Self::Caution => '!' }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionalState { Neutral, Positive, Engaged, Guarded, Stressed, Distracted }

impl EmotionalState {
    pub fn color(&self) -> Color {
        match self {
            Self::Neutral => Color::Grey, Self::Positive => Color::STATUS_GREEN,
            Self::Engaged => Color::HUD_CYAN, Self::Guarded => Color::Yellow,
            Self::Stressed => Color::Rgb(255, 165, 0), Self::Distracted => Color::Rgb(150, 150, 150),
        }
    }
    pub fn icon(&self) -> char {
        match self { Self::Neutral => '─', Self::Positive => '↑', Self::Engaged => '●', Self::Guarded => '◐', Self::Stressed => '!', Self::Distracted => '~' }
    }
    pub fn label(&self) -> &'static str {
        match self { Self::Neutral => "Neutral", Self::Positive => "Positive", Self::Engaged => "Engaged", Self::Guarded => "Guarded", Self::Stressed => "Stressed", Self::Distracted => "Distracted" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommStyle { Direct, Analytical, Relational, Expressive }

impl CommStyle {
    pub fn tip(&self) -> &'static str {
        match self { Self::Direct => "Be concise, outcomes", Self::Analytical => "Data and logic", Self::Relational => "Personal connection", Self::Expressive => "Enthusiasm, big picture" }
    }
}

// ============================================================================
// RAPPORT & CONVERSATION
// ============================================================================

#[derive(Debug, Clone)]
pub struct RapportIndicator { pub metric: String, pub level: f32, pub trend: Trend }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend { Rising, Stable, Falling }

impl Trend {
    pub fn icon(&self) -> char { match self { Self::Rising => '↑', Self::Stable => '→', Self::Falling => '↓' } }
    pub fn color(&self) -> Color { match self { Self::Rising => Color::STATUS_GREEN, Self::Stable => Color::Grey, Self::Falling => Color::Yellow } }
}

#[derive(Debug, Clone)]
pub struct ConversationCue { pub cue_type: CueType, pub content: String, pub relevance: f32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueType { Topic, Question, Callback, Shared, Avoid, FactCheck }

impl CueType {
    pub fn color(&self) -> Color {
        match self { Self::Topic => Color::HUD_CYAN, Self::Question => Color::White, Self::Callback => Color::STATUS_GREEN, Self::Shared => Color::GOLD, Self::Avoid => Color::ALERT_RED, Self::FactCheck => Color::Yellow }
    }
    pub fn icon(&self) -> char {
        match self { Self::Topic => '◇', Self::Question => '?', Self::Callback => '↩', Self::Shared => '∩', Self::Avoid => '✗', Self::FactCheck => '✓' }
    }
}

// ============================================================================
// FACT-CHECKING
// ============================================================================

#[derive(Debug, Clone)]
pub struct FactCheck { pub claim: String, pub verdict: FactVerdict, pub source: String, pub timestamp: String }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactVerdict { Verified, Misleading, Unverified, False }

impl FactVerdict {
    pub fn color(&self) -> Color { match self { Self::Verified => Color::STATUS_GREEN, Self::Misleading => Color::Yellow, Self::Unverified => Color::Grey, Self::False => Color::ALERT_RED } }
    pub fn icon(&self) -> char { match self { Self::Verified => '✓', Self::Misleading => '~', Self::Unverified => '?', Self::False => '✗' } }
    pub fn label(&self) -> &'static str { match self { Self::Verified => "VERIFIED", Self::Misleading => "MISLEADING", Self::Unverified => "UNVERIFIED", Self::False => "FALSE" } }
}

// ============================================================================
// CALENDAR & NOTIFICATIONS
// ============================================================================

#[derive(Debug, Clone)]
pub struct CalendarEvent { pub title: String, pub time: String, pub location: Option<String>, pub attendees: Vec<String>, pub minutes_until: i32 }

#[derive(Debug, Clone)]
pub struct Notification { pub id: String, pub source: String, pub title: String, pub preview: String, pub priority: NotificationPriority, pub timestamp: String, pub dismissed: bool, pub quick_response: Option<String>, pub context_relevant: bool }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority { Low, Normal, High, Urgent }

impl NotificationPriority {
    pub fn color(&self) -> Color { match self { Self::Low => Color::DarkGrey, Self::Normal => Color::Grey, Self::High => Color::Yellow, Self::Urgent => Color::ALERT_RED } }
}

// ============================================================================
// RECORDING & RESEARCH
// ============================================================================

#[derive(Debug, Clone)]
pub struct RecordingSession { pub active: bool, pub duration_secs: u32, pub audio_level: f32 }

#[derive(Debug, Clone)]
pub struct ResearchDoc { pub title: String, pub doc_type: String, pub snippet: String, pub relevance: f32 }

// ============================================================================
// SYSTEM
// ============================================================================

#[derive(Debug, Clone)]
pub struct SystemStatus { pub battery_percent: u8, pub secure_connection: bool, pub recording: bool, pub private_mode: bool }

#[derive(Debug, Clone)]
pub struct ContextInfo { pub current_time: String, pub date: String, pub location: String, pub weather: String }

#[derive(Debug, Clone)]
pub struct QuickAction { pub name: String, pub icon: char, pub shortcut: String }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus { None, Person, Cues, Facts, Calendar, Notifications, Research, Menu }

// ============================================================================
// MAIN STATE
// ============================================================================

#[derive(Debug, Clone)]
pub struct DemoState {
    pub tick: u64, pub mode: DisplayMode, pub density: InformationDensity,
    pub system: SystemStatus, pub context: ContextInfo,
    pub people: Vec<Person>, pub rapport: Vec<RapportIndicator>, pub cues: Vec<ConversationCue>,
    pub fact_checks: Vec<FactCheck>, pub events: Vec<CalendarEvent>, pub notifications: Vec<Notification>,
    pub recording: RecordingSession, pub research_docs: Vec<ResearchDoc>, pub quick_actions: Vec<QuickAction>,
    pub focus: PanelFocus, pub show_menu: bool, pub show_research: bool, pub show_calendar: bool,
    pub menu_selection: Option<usize>, pub status_message: Option<String>, pub gaze_pos: (f32, f32), pub heading: f32,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            tick: 0, mode: DisplayMode::Social, density: InformationDensity::Normal,
            system: SystemStatus { battery_percent: 82, secure_connection: true, recording: false, private_mode: false },
            context: ContextInfo { current_time: "2:47 PM".to_string(), date: "Wed, Dec 4".to_string(), location: "Conference Center".to_string(), weather: "18°C Cloudy".to_string() },
            people: vec![
                Person {
                    id: "person-1".to_string(), name: "Sarah Chen".to_string(), title: Some("VP Engineering".to_string()), organization: Some("TechCorp".to_string()),
                    relationship: Relationship::Colleague, reliability: SourceReliability::Verified, position: Point3D::new(2.0, 0.0, 3.0), distance_meters: 1.8, selected: true,
                    emotional_state: EmotionalState::Engaged, comm_style: CommStyle::Analytical, engagement: 0.85,
                    interests: vec!["AI/ML".to_string(), "photography".to_string()], last_interaction: Some("Q3 roadmap (2 days ago)".to_string()),
                    shared_context: vec!["Same team".to_string(), "AI conf attendee".to_string()], hooks: vec!["Iceland photo trip".to_string()],
                },
                Person {
                    id: "person-2".to_string(), name: "Marcus Webb".to_string(), title: Some("Managing Partner".to_string()), organization: Some("Webb Ventures".to_string()),
                    relationship: Relationship::VIP, reliability: SourceReliability::Established, position: Point3D::new(-1.5, 0.0, 4.0), distance_meters: 2.5, selected: false,
                    emotional_state: EmotionalState::Neutral, comm_style: CommStyle::Direct, engagement: 0.6,
                    interests: vec!["startups".to_string(), "investing".to_string()], last_interaction: Some("Launch event (1 month ago)".to_string()),
                    shared_context: vec!["Series B investor".to_string()], hooks: vec!["Growth metrics update".to_string()],
                },
            ],
            rapport: vec![
                RapportIndicator { metric: "Eye contact".to_string(), level: 0.75, trend: Trend::Stable },
                RapportIndicator { metric: "Mirroring".to_string(), level: 0.6, trend: Trend::Rising },
                RapportIndicator { metric: "Turn-taking".to_string(), level: 0.85, trend: Trend::Stable },
            ],
            cues: vec![
                ConversationCue { cue_type: CueType::Callback, content: "Her Iceland photography trip".to_string(), relevance: 0.95 },
                ConversationCue { cue_type: CueType::Shared, content: "Both use Fujifilm cameras".to_string(), relevance: 0.85 },
                ConversationCue { cue_type: CueType::Question, content: "How's the ML pipeline?".to_string(), relevance: 0.8 },
                ConversationCue { cue_type: CueType::FactCheck, content: "40% improvement claim".to_string(), relevance: 0.75 },
                ConversationCue { cue_type: CueType::Avoid, content: "Project delay (sensitive)".to_string(), relevance: 0.9 },
            ],
            fact_checks: vec![
                FactCheck { claim: "40% latency improvement".to_string(), verdict: FactVerdict::Verified, source: "Metrics dashboard".to_string(), timestamp: "just now".to_string() },
            ],
            events: vec![
                CalendarEvent { title: "Product Review".to_string(), time: "3:00 PM".to_string(), location: Some("Room 204".to_string()), attendees: vec!["Sarah Chen".to_string()], minutes_until: 13 },
                CalendarEvent { title: "Investor Call".to_string(), time: "4:30 PM".to_string(), location: None, attendees: vec!["Marcus Webb".to_string()], minutes_until: 103 },
            ],
            notifications: vec![
                Notification { id: "n1".to_string(), source: "Calendar".to_string(), title: "Product Review in 13min".to_string(), preview: "Room 204 - Sarah attending".to_string(), priority: NotificationPriority::High, timestamp: "now".to_string(), dismissed: false, quick_response: None, context_relevant: true },
                Notification { id: "n2".to_string(), source: "Slack".to_string(), title: "Alex: API question".to_string(), preview: "Review endpoint changes?".to_string(), priority: NotificationPriority::Normal, timestamp: "2:42 PM".to_string(), dismissed: false, quick_response: Some("After meeting".to_string()), context_relevant: false },
            ],
            recording: RecordingSession { active: false, duration_secs: 0, audio_level: 0.0 },
            research_docs: vec![
                ResearchDoc { title: "Q3 Performance Report".to_string(), doc_type: "PDF".to_string(), snippet: "Key metrics...".to_string(), relevance: 0.95 },
                ResearchDoc { title: "Sarah Chen - Profile".to_string(), doc_type: "LinkedIn".to_string(), snippet: "15 years engineering...".to_string(), relevance: 0.85 },
            ],
            quick_actions: vec![
                QuickAction { name: "Record".to_string(), icon: '●', shortcut: "[R]".to_string() },
                QuickAction { name: "Photo".to_string(), icon: '◎', shortcut: "[F]".to_string() },
                QuickAction { name: "Note".to_string(), icon: '✎', shortcut: "[N]".to_string() },
                QuickAction { name: "Verify".to_string(), icon: '✓', shortcut: "[V]".to_string() },
            ],
            focus: PanelFocus::None, show_menu: false, show_research: false, show_calendar: false,
            menu_selection: None, status_message: None, gaze_pos: (0.5, 0.5), heading: 0.0,
        }
    }

    pub fn update(&mut self, ctx: &DisplayContext) {
        self.mode = ctx.mode; self.density = ctx.density;
        if self.tick % 30 == 0 { for p in &mut self.people { if p.selected { p.engagement = (p.engagement + (self.tick as f32 * 0.1).sin() * 0.03).clamp(0.0, 1.0); } } }
        if self.recording.active && self.tick % 10 == 0 { self.recording.duration_secs += 1; self.recording.audio_level = 0.3 + (self.tick as f32 * 0.1).sin().abs() * 0.5; }
        if self.tick % 100 == 0 { self.status_message = None; }
    }

    pub fn select_person(&mut self, id: &str) { for p in &mut self.people { p.selected = p.id == id; } }
    pub fn selected_person(&self) -> Option<&Person> { self.people.iter().find(|p| p.selected) }

    pub fn toggle_recording(&mut self) {
        self.recording.active = !self.recording.active;
        if self.recording.active { self.mode = DisplayMode::Recording; self.system.recording = true; self.status_message = Some("Recording".to_string()); }
        else { self.mode = DisplayMode::Social; self.system.recording = false; self.status_message = Some(format!("Saved {}:{:02}", self.recording.duration_secs / 60, self.recording.duration_secs % 60)); }
    }

    pub fn toggle_private(&mut self) {
        self.system.private_mode = !self.system.private_mode;
        self.mode = if self.system.private_mode { DisplayMode::Private } else { DisplayMode::Social };
        self.status_message = Some(if self.system.private_mode { "Private ON".to_string() } else { "Private OFF".to_string() });
    }

    pub fn dismiss_notification(&mut self, id: &str) { if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) { n.dismissed = true; } }
    pub fn first_active_notification_id(&self) -> Option<String> { self.notifications.iter().find(|n| !n.dismissed).map(|n| n.id.clone()) }
    pub fn active_notification_count(&self) -> usize { self.notifications.iter().filter(|n| !n.dismissed).count() }
    pub fn context_alert(&self) -> Option<&Notification> { self.notifications.iter().find(|n| !n.dismissed && n.context_relevant) }
    pub fn next_event(&self) -> Option<&CalendarEvent> { self.events.iter().filter(|e| e.minutes_until > 0).min_by_key(|e| e.minutes_until) }

    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            DisplayMode::Ambient => DisplayMode::Social, DisplayMode::Social => DisplayMode::Meeting,
            DisplayMode::Meeting => DisplayMode::Research, DisplayMode::Research => DisplayMode::Navigation,
            DisplayMode::Navigation => DisplayMode::Ambient, DisplayMode::Recording | DisplayMode::Private | DisplayMode::Focus => self.mode,
        };
    }
}

impl Default for DemoState { fn default() -> Self { Self::new() } }
