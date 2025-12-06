//! Emergent agent specialization tracking.
//!
//! This module provides infrastructure for detecting and leveraging
//! emergent agent specializations based on performance history.

use crate::capability::{CapabilityTracker, ExpertiseProfile};
use crate::types::{Domain, InstanceId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of an agent's specialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpecializationStatus {
    /// Agent is learning this domain
    Learning,
    /// Agent is proficient in this domain
    Proficient,
    /// Agent is a recognized specialist
    Specialist,
    /// Agent is an expert (top performer)
    Expert,
}

impl SpecializationStatus {
    /// Get the minimum proficiency for this status.
    pub fn min_proficiency(&self) -> f32 {
        match self {
            SpecializationStatus::Learning => 0.0,
            SpecializationStatus::Proficient => 0.5,
            SpecializationStatus::Specialist => 0.8,
            SpecializationStatus::Expert => 0.95,
        }
    }

    /// Get the routing priority for this status.
    pub fn routing_priority(&self) -> u32 {
        match self {
            SpecializationStatus::Learning => 1,
            SpecializationStatus::Proficient => 2,
            SpecializationStatus::Specialist => 3,
            SpecializationStatus::Expert => 4,
        }
    }
}

/// A specialist in a particular domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specialist {
    /// The agent's instance ID
    pub instance_id: InstanceId,

    /// The domain of specialization
    pub domain: Domain,

    /// Current status
    pub status: SpecializationStatus,

    /// Current proficiency
    pub proficiency: f32,

    /// Number of tasks completed in this domain
    pub task_count: u64,

    /// Success rate in this domain
    pub success_rate: f32,

    /// When this specialization was first detected
    pub detected_at: DateTime<Utc>,

    /// When the status was last updated
    pub updated_at: DateTime<Utc>,

    /// Whether this agent is currently available
    pub available: bool,

    /// Last time the agent was active
    pub last_active: DateTime<Utc>,
}

impl Specialist {
    /// Create a new specialist entry.
    pub fn new(instance_id: InstanceId, domain: Domain, proficiency: f32) -> Self {
        let status = Self::status_for_proficiency(proficiency);
        let now = Utc::now();

        Self {
            instance_id,
            domain,
            status,
            proficiency,
            task_count: 0,
            success_rate: 0.0,
            detected_at: now,
            updated_at: now,
            available: true,
            last_active: now,
        }
    }

    /// Determine status based on proficiency.
    fn status_for_proficiency(proficiency: f32) -> SpecializationStatus {
        if proficiency >= 0.95 {
            SpecializationStatus::Expert
        } else if proficiency >= 0.8 {
            SpecializationStatus::Specialist
        } else if proficiency >= 0.5 {
            SpecializationStatus::Proficient
        } else {
            SpecializationStatus::Learning
        }
    }

    /// Update the specialist with new data.
    pub fn update(&mut self, proficiency: f32, task_count: u64, success_rate: f32) {
        self.proficiency = proficiency;
        self.task_count = task_count;
        self.success_rate = success_rate;
        self.status = Self::status_for_proficiency(proficiency);
        self.updated_at = Utc::now();
    }

    /// Mark as active.
    pub fn mark_active(&mut self) {
        self.last_active = Utc::now();
        self.available = true;
    }

    /// Mark as unavailable.
    pub fn mark_unavailable(&mut self) {
        self.available = false;
    }

    /// Check if the specialist is stale (not seen recently).
    pub fn is_stale(&self, timeout: Duration) -> bool {
        Utc::now() - self.last_active > timeout
    }
}

/// Manages specialization tracking and routing.
#[derive(Debug)]
pub struct SpecializationEngine {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// Known specialists by domain
    specialists: HashMap<Domain, Vec<Specialist>>,

    /// This agent's specializations
    my_specializations: HashMap<Domain, Specialist>,

    /// Minimum proficiency to be considered a specialist
    specialist_threshold: f32,

    /// Timeout for considering an agent stale
    stale_timeout: Duration,

    /// Maximum specialists to track per domain
    max_per_domain: usize,
}

impl SpecializationEngine {
    /// Create a new specialization engine.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            specialists: HashMap::new(),
            my_specializations: HashMap::new(),
            specialist_threshold: 0.8,
            stale_timeout: Duration::minutes(30),
            max_per_domain: 10,
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Set the specialist threshold.
    pub fn set_specialist_threshold(&mut self, threshold: f32) {
        self.specialist_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Update specializations from capability tracker.
    pub fn update_from_tracker(&mut self, tracker: &CapabilityTracker) {
        let profile = tracker.profile();
        self.update_from_profile(profile);
    }

    /// Update specializations from an expertise profile.
    pub fn update_from_profile(&mut self, profile: &ExpertiseProfile) {
        for (domain, capability) in &profile.capabilities {
            if capability.proficiency >= self.specialist_threshold {
                let entry = self
                    .my_specializations
                    .entry(domain.clone())
                    .or_insert_with(|| {
                        Specialist::new(
                            self.instance_id.clone(),
                            domain.clone(),
                            capability.proficiency,
                        )
                    });

                entry.update(
                    capability.proficiency,
                    capability.experience_count,
                    capability.success_rate,
                );
            }
        }
    }

    /// Register a peer specialist.
    pub fn register_specialist(&mut self, specialist: Specialist) {
        // Don't register ourselves
        if specialist.instance_id == self.instance_id {
            return;
        }

        let domain_specialists = self
            .specialists
            .entry(specialist.domain.clone())
            .or_default();

        // Update existing or add new
        if let Some(existing) = domain_specialists
            .iter_mut()
            .find(|s| s.instance_id == specialist.instance_id)
        {
            existing.update(
                specialist.proficiency,
                specialist.task_count,
                specialist.success_rate,
            );
            existing.mark_active();
        } else {
            domain_specialists.push(specialist);

            // Limit per domain
            if domain_specialists.len() > self.max_per_domain {
                // Remove lowest proficiency
                domain_specialists.sort_by(|a, b| {
                    b.proficiency
                        .partial_cmp(&a.proficiency)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                domain_specialists.truncate(self.max_per_domain);
            }
        }
    }

    /// Mark a specialist as unavailable.
    pub fn mark_unavailable(&mut self, instance_id: &str, domain: &str) {
        if let Some(specialists) = self.specialists.get_mut(domain) {
            if let Some(specialist) = specialists
                .iter_mut()
                .find(|s| s.instance_id == instance_id)
            {
                specialist.mark_unavailable();
            }
        }
    }

    /// Get specialists for a domain, sorted by proficiency.
    pub fn get_specialists(&self, domain: &str) -> Vec<&Specialist> {
        let mut specialists: Vec<_> = self
            .specialists
            .get(domain)
            .map(|s| {
                s.iter()
                    .filter(|s| s.available && !s.is_stale(self.stale_timeout))
                    .collect()
            })
            .unwrap_or_default();

        // Include self if applicable
        if let Some(my_spec) = self.my_specializations.get(domain) {
            specialists.push(my_spec);
        }

        specialists.sort_by(|a, b| {
            b.proficiency
                .partial_cmp(&a.proficiency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        specialists
    }

    /// Get the best specialist for a domain.
    pub fn get_best_specialist(&self, domain: &str) -> Option<&Specialist> {
        self.get_specialists(domain).first().copied()
    }

    /// Get all domains with available specialists.
    pub fn covered_domains(&self) -> Vec<&str> {
        let mut domains: Vec<_> = self
            .specialists
            .keys()
            .filter(|d| !self.get_specialists(d).is_empty())
            .map(|s| s.as_str())
            .collect();

        // Add our own specializations
        for domain in self.my_specializations.keys() {
            if !domains.contains(&domain.as_str()) {
                domains.push(domain.as_str());
            }
        }

        domains
    }

    /// Check domain coverage and identify gaps.
    pub fn identify_gaps(&self, required_domains: &[String]) -> Vec<String> {
        let covered: std::collections::HashSet<_> = self.covered_domains().into_iter().collect();
        required_domains
            .iter()
            .filter(|d| !covered.contains(d.as_str()))
            .cloned()
            .collect()
    }

    /// Get this agent's specializations.
    pub fn my_specializations(&self) -> Vec<&Specialist> {
        self.my_specializations.values().collect()
    }

    /// Clean up stale specialists.
    pub fn cleanup_stale(&mut self) -> usize {
        let mut removed = 0;

        for specialists in self.specialists.values_mut() {
            let before = specialists.len();
            specialists.retain(|s| !s.is_stale(self.stale_timeout));
            removed += before - specialists.len();
        }

        removed
    }

    /// Get domain statistics.
    pub fn domain_stats(&self, domain: &str) -> Option<DomainStats> {
        let specialists = self.get_specialists(domain);
        if specialists.is_empty() {
            return None;
        }

        let avg_proficiency =
            specialists.iter().map(|s| s.proficiency).sum::<f32>() / specialists.len() as f32;
        let total_tasks: u64 = specialists.iter().map(|s| s.task_count).sum();
        let expert_count = specialists
            .iter()
            .filter(|s| s.status == SpecializationStatus::Expert)
            .count();

        Some(DomainStats {
            domain: domain.to_string(),
            specialist_count: specialists.len(),
            expert_count,
            avg_proficiency,
            total_tasks,
        })
    }
}

/// Statistics for a domain.
#[derive(Debug, Clone)]
pub struct DomainStats {
    /// The domain
    pub domain: String,

    /// Number of specialists
    pub specialist_count: usize,

    /// Number of experts
    pub expert_count: usize,

    /// Average proficiency across specialists
    pub avg_proficiency: f32,

    /// Total tasks completed across specialists
    pub total_tasks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_creation() {
        let specialist = Specialist::new("agent-1".to_string(), "code_review".to_string(), 0.85);

        assert_eq!(specialist.status, SpecializationStatus::Specialist);
        assert!(specialist.available);
    }

    #[test]
    fn test_specialization_engine() {
        let mut engine = SpecializationEngine::new("agent-1".to_string());

        // Register some specialists
        engine.register_specialist(Specialist::new(
            "agent-2".to_string(),
            "code_review".to_string(),
            0.9,
        ));
        engine.register_specialist(Specialist::new(
            "agent-3".to_string(),
            "code_review".to_string(),
            0.85,
        ));

        let specialists = engine.get_specialists("code_review");
        assert_eq!(specialists.len(), 2);
        assert_eq!(specialists[0].instance_id, "agent-2"); // Higher proficiency first
    }

    #[test]
    fn test_domain_gaps() {
        let mut engine = SpecializationEngine::new("agent-1".to_string());

        engine.register_specialist(Specialist::new(
            "agent-2".to_string(),
            "code_review".to_string(),
            0.9,
        ));

        let gaps = engine.identify_gaps(&[
            "code_review".to_string(),
            "data_analysis".to_string(),
            "testing".to_string(),
        ]);

        assert_eq!(gaps.len(), 2);
        assert!(gaps.contains(&"data_analysis".to_string()));
        assert!(gaps.contains(&"testing".to_string()));
    }
}
