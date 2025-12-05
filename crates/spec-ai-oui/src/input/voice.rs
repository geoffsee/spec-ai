//! Voice command recognition types

/// A recognized voice command
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    /// The recognized command text
    pub text: String,
    /// Recognition confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Whether the command is final or interim
    pub is_final: bool,
    /// Detected intent/action (if any)
    pub intent: Option<VoiceIntent>,
}

/// Common voice command intents
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceIntent {
    /// Navigate/select
    Select,
    /// Go back
    Back,
    /// Cancel current action
    Cancel,
    /// Confirm/accept
    Confirm,
    /// Open menu
    Menu,
    /// Scroll up/down
    Scroll { direction: ScrollDirection },
    /// Custom command
    Custom(String),
}

/// Scroll direction for voice commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl VoiceCommand {
    /// Create a new voice command
    pub fn new(text: impl Into<String>, confidence: f32) -> Self {
        Self {
            text: text.into(),
            confidence,
            is_final: true,
            intent: None,
        }
    }

    /// Set as interim (not final) result
    pub fn interim(mut self) -> Self {
        self.is_final = false;
        self
    }

    /// Parse intent from command text
    pub fn with_parsed_intent(mut self) -> Self {
        self.intent = Self::parse_intent(&self.text);
        self
    }

    /// Parse command text to determine intent
    fn parse_intent(text: &str) -> Option<VoiceIntent> {
        let lower = text.to_lowercase();

        if lower.contains("select") || lower.contains("choose") || lower.contains("pick") {
            return Some(VoiceIntent::Select);
        }
        if lower.contains("back") || lower.contains("previous") {
            return Some(VoiceIntent::Back);
        }
        if lower.contains("cancel") || lower.contains("stop") {
            return Some(VoiceIntent::Cancel);
        }
        if lower.contains("confirm")
            || lower.contains("yes")
            || lower.contains("okay")
            || lower.contains("ok")
        {
            return Some(VoiceIntent::Confirm);
        }
        if lower.contains("menu") || lower.contains("options") {
            return Some(VoiceIntent::Menu);
        }
        if lower.contains("scroll up") || lower.contains("up") {
            return Some(VoiceIntent::Scroll {
                direction: ScrollDirection::Up,
            });
        }
        if lower.contains("scroll down") || lower.contains("down") {
            return Some(VoiceIntent::Scroll {
                direction: ScrollDirection::Down,
            });
        }

        None
    }

    /// Check if command matches a keyword
    pub fn matches(&self, keyword: &str) -> bool {
        self.text.to_lowercase().contains(&keyword.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_command() {
        let cmd = VoiceCommand::new("select target", 0.95).with_parsed_intent();
        assert_eq!(cmd.intent, Some(VoiceIntent::Select));
    }

    #[test]
    fn test_voice_matches() {
        let cmd = VoiceCommand::new("go back", 0.9);
        assert!(cmd.matches("back"));
        assert!(!cmd.matches("forward"));
    }
}
