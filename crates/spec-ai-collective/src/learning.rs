//! Inter-agent learning and strategy sharing.
//!
//! This module provides infrastructure for agents to share successful
//! strategies with each other and learn from peer experiences.

use crate::types::{InstanceId, StrategyId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned strategy that can be shared between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Unique strategy identifier
    pub strategy_id: StrategyId,

    /// Type of task this strategy applies to
    pub task_type: String,

    /// Human-readable description of the strategy
    pub description: String,

    /// Detailed steps or approach
    pub approach: Vec<String>,

    /// Optional context embedding for semantic matching
    pub context_embedding: Option<Vec<f32>>,

    /// Number of times this strategy succeeded
    pub success_count: u64,

    /// Total number of times this strategy was used
    pub total_uses: u64,

    /// The agent that created this strategy
    pub created_by: InstanceId,

    /// When the strategy was created
    pub created_at: DateTime<Utc>,

    /// When the strategy was last used
    pub last_used: Option<DateTime<Utc>>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Strategy {
    /// Create a new strategy.
    pub fn new(
        task_type: impl Into<String>,
        description: impl Into<String>,
        approach: Vec<String>,
        created_by: InstanceId,
    ) -> Self {
        Self {
            strategy_id: uuid::Uuid::new_v4().to_string(),
            task_type: task_type.into(),
            description: description.into(),
            approach,
            context_embedding: None,
            success_count: 0,
            total_uses: 0,
            created_by,
            created_at: Utc::now(),
            last_used: None,
            tags: Vec::new(),
        }
    }

    /// Record a usage of this strategy.
    pub fn record_usage(&mut self, success: bool) {
        self.total_uses += 1;
        if success {
            self.success_count += 1;
        }
        self.last_used = Some(Utc::now());
    }

    /// Calculate success rate.
    pub fn success_rate(&self) -> f32 {
        if self.total_uses == 0 {
            0.0
        } else {
            self.success_count as f32 / self.total_uses as f32
        }
    }

    /// Set context embedding for semantic matching.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.context_embedding = Some(embedding);
        self
    }

    /// Add tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// A strategy match result from a query.
#[derive(Debug, Clone)]
pub struct StrategyMatch {
    /// The matched strategy
    pub strategy: Strategy,

    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,

    /// Match type
    pub match_type: MatchType,
}

/// How the strategy was matched.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Exact task type match
    ExactType,
    /// Semantic similarity match via embeddings
    Semantic,
    /// Tag-based match
    Tagged,
}

/// The learning fabric for sharing strategies between agents.
#[derive(Debug)]
pub struct LearningFabric {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// Local strategies (created by this agent)
    local_strategies: HashMap<StrategyId, Strategy>,

    /// Strategies received from peers
    peer_strategies: HashMap<StrategyId, Strategy>,

    /// Index by task type for fast lookup
    type_index: HashMap<String, Vec<StrategyId>>,

    /// Minimum success rate to consider a strategy
    min_success_rate: f32,

    /// Maximum strategies to keep per task type
    max_per_type: usize,
}

impl LearningFabric {
    /// Create a new learning fabric.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            local_strategies: HashMap::new(),
            peer_strategies: HashMap::new(),
            type_index: HashMap::new(),
            min_success_rate: 0.5,
            max_per_type: 10,
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Set minimum success rate for strategies.
    pub fn set_min_success_rate(&mut self, rate: f32) {
        self.min_success_rate = rate;
    }

    /// Add a new local strategy.
    pub fn add_strategy(&mut self, strategy: Strategy) {
        let strategy_id = strategy.strategy_id.clone();
        let task_type = strategy.task_type.clone();

        self.local_strategies.insert(strategy_id.clone(), strategy);

        // Update index
        self.type_index
            .entry(task_type)
            .or_default()
            .push(strategy_id);
    }

    /// Record that a strategy was used.
    pub fn record_usage(&mut self, strategy_id: &str, success: bool) {
        if let Some(strategy) = self.local_strategies.get_mut(strategy_id) {
            strategy.record_usage(success);
        } else if let Some(strategy) = self.peer_strategies.get_mut(strategy_id) {
            strategy.record_usage(success);
        }
    }

    /// Import a strategy from a peer.
    pub fn import_strategy(&mut self, strategy: Strategy) {
        let strategy_id = strategy.strategy_id.clone();
        let task_type = strategy.task_type.clone();

        // Don't import our own strategies
        if strategy.created_by == self.instance_id {
            return;
        }

        self.peer_strategies.insert(strategy_id.clone(), strategy);

        // Update index
        self.type_index
            .entry(task_type)
            .or_default()
            .push(strategy_id);

        // Cleanup if too many strategies
        self.cleanup_strategies();
    }

    /// Query strategies for a task type.
    pub fn query_by_type(&self, task_type: &str) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();

        if let Some(strategy_ids) = self.type_index.get(task_type) {
            for id in strategy_ids {
                if let Some(strategy) = self.get_strategy(id) {
                    if strategy.success_rate() >= self.min_success_rate {
                        matches.push(StrategyMatch {
                            strategy: strategy.clone(),
                            relevance: strategy.success_rate(),
                            match_type: MatchType::ExactType,
                        });
                    }
                }
            }
        }

        // Sort by relevance (success rate)
        matches.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Query strategies by semantic similarity (requires embeddings).
    pub fn query_by_embedding(&self, query_embedding: &[f32], threshold: f32) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();

        for strategy in self.all_strategies() {
            if let Some(embedding) = &strategy.context_embedding {
                let similarity = cosine_similarity(query_embedding, embedding);
                if similarity >= threshold && strategy.success_rate() >= self.min_success_rate {
                    matches.push(StrategyMatch {
                        strategy: strategy.clone(),
                        relevance: similarity * strategy.success_rate(),
                        match_type: MatchType::Semantic,
                    });
                }
            }
        }

        matches.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Query strategies by tags.
    pub fn query_by_tags(&self, tags: &[String]) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();

        for strategy in self.all_strategies() {
            let tag_match_count = tags
                .iter()
                .filter(|t| strategy.tags.contains(t))
                .count();

            if tag_match_count > 0 && strategy.success_rate() >= self.min_success_rate {
                let relevance = (tag_match_count as f32 / tags.len() as f32) * strategy.success_rate();
                matches.push(StrategyMatch {
                    strategy: strategy.clone(),
                    relevance,
                    match_type: MatchType::Tagged,
                });
            }
        }

        matches.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Get a strategy by ID.
    pub fn get_strategy(&self, strategy_id: &str) -> Option<&Strategy> {
        self.local_strategies
            .get(strategy_id)
            .or_else(|| self.peer_strategies.get(strategy_id))
    }

    /// Get all strategies (local and peer).
    pub fn all_strategies(&self) -> impl Iterator<Item = &Strategy> {
        self.local_strategies
            .values()
            .chain(self.peer_strategies.values())
    }

    /// Get local strategies for sharing.
    pub fn local_strategies(&self) -> impl Iterator<Item = &Strategy> {
        self.local_strategies.values()
    }

    /// Get strategies worth sharing (high success rate).
    pub fn shareable_strategies(&self, min_success_rate: f32) -> Vec<&Strategy> {
        self.local_strategies
            .values()
            .filter(|s| s.success_rate() >= min_success_rate && s.total_uses >= 3)
            .collect()
    }

    /// Clean up low-performing or old strategies.
    fn cleanup_strategies(&mut self) {
        // Collect strategy rates first to avoid borrow issues
        let mut rates: HashMap<StrategyId, f32> = HashMap::new();
        for (id, strategy) in &self.local_strategies {
            rates.insert(id.clone(), strategy.success_rate());
        }
        for (id, strategy) in &self.peer_strategies {
            rates.insert(id.clone(), strategy.success_rate());
        }

        // Group by task type and keep only top performers
        let mut to_remove = Vec::new();
        for strategy_ids in self.type_index.values_mut() {
            if strategy_ids.len() > self.max_per_type {
                // Sort by success rate
                strategy_ids.sort_by(|a, b| {
                    let rate_a = rates.get(a).copied().unwrap_or(0.0);
                    let rate_b = rates.get(b).copied().unwrap_or(0.0);
                    rate_b
                        .partial_cmp(&rate_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Mark excess for removal
                for id in strategy_ids.drain(self.max_per_type..) {
                    to_remove.push(id);
                }
            }
        }

        // Remove excess strategies
        for id in to_remove {
            self.peer_strategies.remove(&id);
        }
    }
}

/// Calculate cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = Strategy::new(
            "code_review",
            "Standard code review process",
            vec![
                "Check for style issues".to_string(),
                "Verify logic correctness".to_string(),
                "Look for security vulnerabilities".to_string(),
            ],
            "agent-1".to_string(),
        );

        assert_eq!(strategy.task_type, "code_review");
        assert_eq!(strategy.approach.len(), 3);
        assert_eq!(strategy.success_rate(), 0.0);
    }

    #[test]
    fn test_strategy_usage() {
        let mut strategy = Strategy::new(
            "testing",
            "Test strategy",
            vec!["Step 1".to_string()],
            "agent-1".to_string(),
        );

        strategy.record_usage(true);
        strategy.record_usage(true);
        strategy.record_usage(false);

        assert_eq!(strategy.total_uses, 3);
        assert_eq!(strategy.success_count, 2);
        assert!((strategy.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_learning_fabric_query() {
        let mut fabric = LearningFabric::new("agent-1".to_string());
        fabric.set_min_success_rate(0.0);

        let mut strategy = Strategy::new(
            "data_analysis",
            "Standard analysis approach",
            vec!["Load data".to_string(), "Analyze".to_string()],
            "agent-1".to_string(),
        );

        strategy.record_usage(true);
        fabric.add_strategy(strategy);

        let matches = fabric.query_by_type("data_analysis");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].relevance > 0.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.0001);
    }
}
