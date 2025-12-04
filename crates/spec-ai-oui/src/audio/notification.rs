//! Notification sound types

/// Notification sound types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notification {
    /// UI click/tap
    Click,
    /// Item selected
    Select,
    /// Action confirmed
    Confirm,
    /// Action cancelled
    Cancel,
    /// Error occurred
    Error,

    /// General alert
    Alert,
    /// Warning
    Warning,
    /// Critical alert
    Critical,

    /// Objective update
    ObjectiveUpdate,
    /// Message received
    MessageReceived,
    /// Target acquired
    TargetAcquired,
    /// Target lost
    TargetLost,
}
