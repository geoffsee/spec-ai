//! Agent capability tracking and expertise management.
//!
//! This module provides infrastructure for tracking agent capabilities,
//! recording task outcomes, and building expertise profiles over time.

use crate::types::{Domain, InstanceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an agent's capability in a specific domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// The domain this capability applies to (e.g., "code_review", "data_analysis")
    pub domain: Domain,

    /// Proficiency level from 0.0 (novice) to 1.0 (expert)
    pub proficiency: f32,

    /// Number of tasks completed in this domain
    pub experience_count: u64,

    /// Historical success rate (0.0 to 1.0)
    pub success_rate: f32,

    /// Average task completion time in milliseconds
    pub avg_duration_ms: Option<u64>,

    /// When this capability was last updated
    pub last_updated: DateTime<Utc>,
}

impl Capability {
    /// Create a new capability with default values.
    pub fn new(domain: Domain) -> Self {
        Self {
            domain,
            proficiency: 0.0,
            experience_count: 0,
            success_rate: 0.0,
            avg_duration_ms: None,
            last_updated: Utc::now(),
        }
    }

    /// Update capability based on a task outcome.
    pub fn update(&mut self, outcome: &TaskOutcome) {
        self.experience_count += 1;

        // Update success rate with exponential moving average
        let success_value = match outcome {
            TaskOutcome::Success { .. } => 1.0,
            TaskOutcome::Partial { completion_ratio } => *completion_ratio,
            TaskOutcome::Failure { .. } => 0.0,
        };

        let alpha = 0.1; // Learning rate
        self.success_rate = (1.0 - alpha) * self.success_rate + alpha * success_value;

        // Update average duration if available
        if let TaskOutcome::Success { duration_ms, .. } = outcome {
            self.avg_duration_ms = Some(match self.avg_duration_ms {
                Some(avg) => ((avg as f64 * 0.9) + (*duration_ms as f64 * 0.1)) as u64,
                None => *duration_ms,
            });
        }

        // Update proficiency based on experience and success rate
        self.proficiency = self.calculate_proficiency();
        self.last_updated = Utc::now();
    }

    /// Calculate proficiency based on experience and success rate.
    fn calculate_proficiency(&self) -> f32 {
        // Proficiency grows with experience but is bounded by success rate
        let experience_factor = (1.0 - (-0.01 * self.experience_count as f32).exp()).min(1.0);
        (experience_factor * self.success_rate).min(1.0)
    }

    /// Check if this agent is a specialist in this domain (proficiency > 0.8).
    pub fn is_specialist(&self) -> bool {
        self.proficiency > 0.8
    }
}

/// Outcome of a task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    /// Task completed successfully
    Success {
        /// Confidence in the result (0.0 to 1.0)
        confidence: f32,
        /// Duration in milliseconds
        duration_ms: u64,
    },
    /// Task failed
    Failure {
        /// Category of the error
        error_category: String,
        /// Whether the error is recoverable
        recoverable: bool,
    },
    /// Task partially completed
    Partial {
        /// Ratio of completion (0.0 to 1.0)
        completion_ratio: f32,
    },
}

/// A single learning event from task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Type of task performed
    pub task_type: String,

    /// Outcome of the task
    pub outcome: TaskOutcome,

    /// Strategy or approach used
    pub strategy_used: String,

    /// Optional context embedding for semantic matching
    pub context_embedding: Option<Vec<f32>>,

    /// When this event occurred
    pub timestamp: DateTime<Utc>,
}

/// Agent expertise profile tracking all capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertiseProfile {
    /// The instance ID of the agent
    pub instance_id: InstanceId,

    /// Capabilities by domain
    pub capabilities: HashMap<Domain, Capability>,

    /// Domains where the agent is a specialist (proficiency > 0.8)
    pub specializations: Vec<Domain>,

    /// Recent learning events
    pub learning_history: Vec<LearningEvent>,

    /// Maximum learning history to keep
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

fn default_max_history() -> usize {
    100
}

impl ExpertiseProfile {
    /// Create a new expertise profile for an agent.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            capabilities: HashMap::new(),
            specializations: Vec::new(),
            learning_history: Vec::new(),
            max_history: default_max_history(),
        }
    }

    /// Get capability for a domain, creating if necessary.
    pub fn get_or_create_capability(&mut self, domain: &str) -> &mut Capability {
        self.capabilities
            .entry(domain.to_string())
            .or_insert_with(|| Capability::new(domain.to_string()))
    }

    /// Record a task outcome and update capabilities.
    pub fn record_outcome(&mut self, domain: &str, outcome: TaskOutcome, strategy: String) {
        // Update capability
        let capability = self.get_or_create_capability(domain);
        capability.update(&outcome);

        // Add learning event
        let event = LearningEvent {
            task_type: domain.to_string(),
            outcome,
            strategy_used: strategy,
            context_embedding: None,
            timestamp: Utc::now(),
        };
        self.learning_history.push(event);

        // Trim history if needed
        while self.learning_history.len() > self.max_history {
            self.learning_history.remove(0);
        }

        // Update specializations
        self.update_specializations();
    }

    /// Update the list of specializations based on current capabilities.
    fn update_specializations(&mut self) {
        self.specializations = self
            .capabilities
            .iter()
            .filter(|(_, cap)| cap.is_specialist())
            .map(|(domain, _)| domain.clone())
            .collect();
    }

    /// Get the best capability match for required capabilities.
    pub fn match_score(&self, required: &[String]) -> f32 {
        if required.is_empty() {
            return 1.0;
        }

        let total: f32 = required
            .iter()
            .map(|domain| {
                self.capabilities
                    .get(domain)
                    .map(|c| c.proficiency)
                    .unwrap_or(0.0)
            })
            .sum();

        total / required.len() as f32
    }
}

/// Tracks capabilities for agents in the mesh.
#[derive(Debug)]
pub struct CapabilityTracker {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// This agent's expertise profile
    profile: ExpertiseProfile,

    /// Known peer profiles (from capability updates)
    peers: HashMap<InstanceId, ExpertiseProfile>,
}

impl CapabilityTracker {
    /// Create a new capability tracker.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id: instance_id.clone(),
            profile: ExpertiseProfile::new(instance_id),
            peers: HashMap::new(),
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get this agent's expertise profile.
    pub fn profile(&self) -> &ExpertiseProfile {
        &self.profile
    }

    /// Get mutable reference to this agent's expertise profile.
    pub fn profile_mut(&mut self) -> &mut ExpertiseProfile {
        &mut self.profile
    }

    /// Record a task outcome for this agent.
    pub fn record_task_outcome(&mut self, domain: &str, outcome: TaskOutcome, strategy: String) {
        self.profile.record_outcome(domain, outcome, strategy);
    }

    /// Update a peer's profile from a capability update message.
    pub fn update_peer_profile(&mut self, profile: ExpertiseProfile) {
        self.peers.insert(profile.instance_id.clone(), profile);
    }

    /// Get the best agent for a task requiring specific capabilities.
    pub fn get_best_agent(
        &self,
        required_capabilities: &[String],
    ) -> Option<RoutingRecommendation> {
        let mut best: Option<(String, f32)> = None;

        // Check self
        let self_score = self.profile.match_score(required_capabilities);
        if self_score > 0.0 {
            best = Some((self.instance_id.clone(), self_score));
        }

        // Check peers
        for (instance_id, profile) in &self.peers {
            let score = profile.match_score(required_capabilities);
            if let Some((_, best_score)) = &best {
                if score > *best_score {
                    best = Some((instance_id.clone(), score));
                }
            } else if score > 0.0 {
                best = Some((instance_id.clone(), score));
            }
        }

        best.map(|(instance_id, score)| {
            let is_self = instance_id == self.instance_id;
            RoutingRecommendation {
                instance_id,
                score,
                is_self,
            }
        })
    }

    /// Get all known agents with a minimum capability score.
    pub fn get_capable_agents(
        &self,
        required_capabilities: &[String],
        min_score: f32,
    ) -> Vec<RoutingRecommendation> {
        let mut agents = Vec::new();

        // Check self
        let self_score = self.profile.match_score(required_capabilities);
        if self_score >= min_score {
            agents.push(RoutingRecommendation {
                instance_id: self.instance_id.clone(),
                score: self_score,
                is_self: true,
            });
        }

        // Check peers
        for (instance_id, profile) in &self.peers {
            let score = profile.match_score(required_capabilities);
            if score >= min_score {
                agents.push(RoutingRecommendation {
                    instance_id: instance_id.clone(),
                    score,
                    is_self: false,
                });
            }
        }

        // Sort by score descending
        agents.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        agents
    }

    /// Get all known peer profiles.
    pub fn peers(&self) -> &HashMap<InstanceId, ExpertiseProfile> {
        &self.peers
    }
}

/// Recommendation for routing a task to an agent.
#[derive(Debug, Clone)]
pub struct RoutingRecommendation {
    /// The recommended instance ID
    pub instance_id: InstanceId,

    /// Match score (0.0 to 1.0)
    pub score: f32,

    /// Whether this is the local agent
    pub is_self: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_update() {
        let mut cap = Capability::new("code_review".to_string());
        assert_eq!(cap.proficiency, 0.0);

        // Simulate successful tasks
        for _ in 0..10 {
            cap.update(&TaskOutcome::Success {
                confidence: 0.9,
                duration_ms: 1000,
            });
        }

        assert!(cap.proficiency > 0.0);
        // With EMA alpha=0.1, after 10 successes: 1 - (0.9)^10 ≈ 0.65
        assert!(cap.success_rate > 0.5);
        assert_eq!(cap.experience_count, 10);
    }

    #[test]
    fn test_expertise_profile_matching() {
        let mut profile = ExpertiseProfile::new("agent-1".to_string());

        // Add some capabilities
        for _ in 0..20 {
            profile.record_outcome(
                "code_review",
                TaskOutcome::Success {
                    confidence: 0.9,
                    duration_ms: 1000,
                },
                "standard_review".to_string(),
            );
        }

        let score = profile.match_score(&["code_review".to_string()]);
        assert!(score > 0.0);

        let score2 = profile.match_score(&["unknown_domain".to_string()]);
        assert_eq!(score2, 0.0);
    }

    #[test]
    fn test_capability_tracker_routing() {
        let mut tracker = CapabilityTracker::new("agent-1".to_string());

        // Record some outcomes
        for _ in 0..10 {
            tracker.record_task_outcome(
                "data_analysis",
                TaskOutcome::Success {
                    confidence: 0.9,
                    duration_ms: 500,
                },
                "standard".to_string(),
            );
        }

        let rec = tracker.get_best_agent(&["data_analysis".to_string()]);
        assert!(rec.is_some());
        assert!(rec.unwrap().is_self);
    }
}
