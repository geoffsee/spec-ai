//! Application state for the OUI demo

use spec_ai_oui::{
    DisplayContext, DisplayMode, InformationDensity, Priority,
    spatial::Point3D,
    renderer::Color,
};

/// Mission information
#[derive(Debug, Clone)]
pub struct Mission {
    pub codename: String,
    pub objective: String,
    pub status: MissionStatus,
    pub targets: Vec<Target>,
    pub intel: Vec<String>,
}

/// Mission status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Briefing,
    Active,
    Complete,
    Failed,
}

/// A target in the mission
#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub position: Point3D,
    pub threat_level: ThreatLevel,
    pub locked: bool,
    pub lock_progress: f32,
}

/// Threat level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn color(&self) -> Color {
        match self {
            ThreatLevel::None => Color::Grey,
            ThreatLevel::Low => Color::STATUS_GREEN,
            ThreatLevel::Medium => Color::Yellow,
            ThreatLevel::High => Color::Rgb(255, 165, 0), // Orange
            ThreatLevel::Critical => Color::ALERT_RED,
        }
    }
}

/// Agent status
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub health: f32,
    pub shields: f32,
    pub ammo: u32,
    pub gadgets: Vec<Gadget>,
}

/// Agent gadget
#[derive(Debug, Clone)]
pub struct Gadget {
    pub name: String,
    pub icon: char,
    pub ready: bool,
    pub cooldown: f32,
}

/// UI panel focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    None,
    Mission,
    Status,
    Targets,
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
    /// Active mission
    pub mission: Mission,
    /// Agent status
    pub agent: AgentStatus,
    /// Focused panel
    pub focus: PanelFocus,
    /// Show mission briefing
    pub show_briefing: bool,
    /// Show radial menu
    pub show_menu: bool,
    /// Selected menu item
    pub menu_selection: Option<usize>,
    /// Status message
    pub status_message: Option<String>,
    /// Compass heading
    pub heading: f32,
    /// Current gaze position (normalized)
    pub gaze_pos: (f32, f32),
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            mode: DisplayMode::Exploration,
            density: InformationDensity::Normal,
            mission: Mission {
                codename: "GOLDEN EYE".to_string(),
                objective: "Infiltrate facility and retrieve intel".to_string(),
                status: MissionStatus::Briefing,
                targets: vec![
                    Target {
                        id: "target-1".to_string(),
                        name: "Primary Objective".to_string(),
                        position: Point3D::new(50.0, 0.0, 100.0),
                        threat_level: ThreatLevel::None,
                        locked: false,
                        lock_progress: 0.0,
                    },
                    Target {
                        id: "hostile-1".to_string(),
                        name: "Guard Alpha".to_string(),
                        position: Point3D::new(30.0, 0.0, 50.0),
                        threat_level: ThreatLevel::Medium,
                        locked: false,
                        lock_progress: 0.0,
                    },
                    Target {
                        id: "hostile-2".to_string(),
                        name: "Guard Beta".to_string(),
                        position: Point3D::new(-20.0, 0.0, 70.0),
                        threat_level: ThreatLevel::High,
                        locked: false,
                        lock_progress: 0.0,
                    },
                ],
                intel: vec![
                    "Facility has 3 levels".to_string(),
                    "Security rotation every 10 minutes".to_string(),
                    "Target package in server room B".to_string(),
                ],
            },
            agent: AgentStatus {
                health: 100.0,
                shields: 75.0,
                ammo: 42,
                gadgets: vec![
                    Gadget {
                        name: "EMP".to_string(),
                        icon: '⚡',
                        ready: true,
                        cooldown: 0.0,
                    },
                    Gadget {
                        name: "Grapple".to_string(),
                        icon: '🪝',
                        ready: true,
                        cooldown: 0.0,
                    },
                    Gadget {
                        name: "Cloak".to_string(),
                        icon: '👻',
                        ready: false,
                        cooldown: 30.0,
                    },
                ],
            },
            focus: PanelFocus::None,
            show_briefing: true,
            show_menu: false,
            menu_selection: None,
            status_message: Some("Mission briefing active".to_string()),
            heading: 45.0,
            gaze_pos: (0.5, 0.5),
        }
    }

    /// Update state based on context
    pub fn update(&mut self, ctx: &DisplayContext) {
        self.mode = ctx.mode;
        self.density = ctx.density;

        // Update target lock progress
        for target in &mut self.mission.targets {
            if target.locked && target.lock_progress < 1.0 {
                target.lock_progress = (target.lock_progress + 0.02).min(1.0);
            }
        }

        // Update gadget cooldowns
        for gadget in &mut self.agent.gadgets {
            if gadget.cooldown > 0.0 {
                gadget.cooldown = (gadget.cooldown - 0.1).max(0.0);
                if gadget.cooldown <= 0.0 {
                    gadget.ready = true;
                }
            }
        }

        // Clear status message after a while
        if self.tick % 100 == 0 {
            self.status_message = None;
        }
    }

    /// Start mission
    pub fn start_mission(&mut self) {
        self.mission.status = MissionStatus::Active;
        self.show_briefing = false;
        self.status_message = Some("Mission active - proceed to objective".to_string());
    }

    /// Lock onto a target
    pub fn lock_target(&mut self, target_id: &str) {
        // Unlock all targets first
        for target in &mut self.mission.targets {
            if target.id != target_id {
                target.locked = false;
                target.lock_progress = 0.0;
            }
        }

        // Lock the specified target
        if let Some(target) = self.mission.targets.iter_mut().find(|t| t.id == target_id) {
            target.locked = true;
        }
    }

    /// Use a gadget
    pub fn use_gadget(&mut self, index: usize) {
        if let Some(gadget) = self.agent.gadgets.get_mut(index) {
            if gadget.ready {
                gadget.ready = false;
                gadget.cooldown = 30.0;
                self.status_message = Some(format!("{} activated", gadget.name));
            }
        }
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}
