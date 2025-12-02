//! Collective decision-making through voting and consensus.
//!
//! This module provides infrastructure for agents to make collective
//! decisions through expertise-weighted voting.

use crate::capability::CapabilityTracker;
use crate::types::{CollectiveError, Domain, InstanceId, ProposalId, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of proposal for collective decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalType {
    /// Adopt a new strategy for a task type
    StrategyAdoption,
    /// Change collective policy
    PolicyChange,
    /// Allocate resources to a task
    ResourceAllocation,
    /// Resolve a conflict between agents
    ConflictResolution,
    /// Custom proposal type
    Custom(String),
}

/// Status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    /// Proposal is open for voting
    Open,
    /// Proposal was approved
    Approved,
    /// Proposal was rejected
    Rejected,
    /// Proposal expired without reaching quorum
    Expired,
    /// Proposal was cancelled by proposer
    Cancelled,
}

/// A proposal for collective decision-making.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal identifier
    pub proposal_id: ProposalId,

    /// The agent that created this proposal
    pub proposer_id: InstanceId,

    /// Type of proposal
    pub proposal_type: ProposalType,

    /// Title of the proposal
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Content/payload of the proposal
    pub content: serde_json::Value,

    /// Deadline for voting
    pub deadline: DateTime<Utc>,

    /// Required quorum (0.0 to 1.0, proportion of agents that must vote)
    pub required_quorum: f32,

    /// Required approval ratio (0.0 to 1.0)
    pub required_approval: f32,

    /// Domains relevant to this proposal (for vote weighting)
    pub relevant_domains: Vec<Domain>,

    /// Current status
    pub status: ProposalStatus,

    /// When the proposal was created
    pub created_at: DateTime<Utc>,

    /// When the proposal was resolved (if applicable)
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Proposal {
    /// Create a new proposal.
    pub fn new(
        proposer_id: InstanceId,
        proposal_type: ProposalType,
        title: impl Into<String>,
        description: impl Into<String>,
        content: serde_json::Value,
        duration: Duration,
    ) -> Self {
        Self {
            proposal_id: uuid::Uuid::new_v4().to_string(),
            proposer_id,
            proposal_type,
            title: title.into(),
            description: description.into(),
            content,
            deadline: Utc::now() + duration,
            required_quorum: 0.5,
            required_approval: 0.5,
            relevant_domains: Vec::new(),
            status: ProposalStatus::Open,
            created_at: Utc::now(),
            resolved_at: None,
        }
    }

    /// Set required quorum.
    pub fn with_quorum(mut self, quorum: f32) -> Self {
        self.required_quorum = quorum.clamp(0.0, 1.0);
        self
    }

    /// Set required approval ratio.
    pub fn with_approval(mut self, approval: f32) -> Self {
        self.required_approval = approval.clamp(0.0, 1.0);
        self
    }

    /// Set relevant domains for vote weighting.
    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.relevant_domains = domains;
        self
    }

    /// Check if the proposal is still open for voting.
    pub fn is_open(&self) -> bool {
        self.status == ProposalStatus::Open && Utc::now() < self.deadline
    }

    /// Check if the proposal has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.deadline && self.status == ProposalStatus::Open
    }
}

/// Decision on a vote.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
}

/// A vote on a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// The agent casting the vote
    pub voter_id: InstanceId,

    /// The proposal being voted on
    pub proposal_id: ProposalId,

    /// The vote decision
    pub decision: VoteDecision,

    /// Weight of this vote (based on expertise)
    pub weight: f32,

    /// Optional rationale for the vote
    pub rationale: Option<String>,

    /// When the vote was cast
    pub voted_at: DateTime<Utc>,
}

impl Vote {
    /// Create a new vote.
    pub fn new(
        voter_id: InstanceId,
        proposal_id: ProposalId,
        decision: VoteDecision,
        weight: f32,
    ) -> Self {
        Self {
            voter_id,
            proposal_id,
            decision,
            weight: weight.clamp(0.0, 1.0),
            rationale: None,
            voted_at: Utc::now(),
        }
    }

    /// Add a rationale.
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}

/// Result of tallying votes on a proposal.
#[derive(Debug, Clone)]
pub struct TallyResult {
    /// Total weighted approval votes
    pub weighted_approval: f32,

    /// Total weighted rejection votes
    pub weighted_rejection: f32,

    /// Total weighted abstention votes
    pub weighted_abstention: f32,

    /// Number of voters
    pub voter_count: usize,

    /// Total eligible voters (known agents)
    pub eligible_voters: usize,

    /// Whether quorum was reached
    pub quorum_reached: bool,

    /// Whether the proposal is approved
    pub approved: bool,

    /// Final status
    pub status: ProposalStatus,
}

/// Coordinates collective decision-making.
#[derive(Debug)]
pub struct ConsensusCoordinator {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// Active proposals
    proposals: HashMap<ProposalId, Proposal>,

    /// Votes by proposal
    votes: HashMap<ProposalId, Vec<Vote>>,

    /// Known eligible voters
    eligible_voters: Vec<InstanceId>,

    /// Default voting duration
    default_duration: Duration,

    /// Minimum weight for a vote to count
    min_vote_weight: f32,
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            eligible_voters: Vec::new(),
            default_duration: Duration::hours(24),
            min_vote_weight: 0.1,
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Set the list of eligible voters.
    pub fn set_eligible_voters(&mut self, voters: Vec<InstanceId>) {
        self.eligible_voters = voters;
    }

    /// Add an eligible voter.
    pub fn add_eligible_voter(&mut self, voter: InstanceId) {
        if !self.eligible_voters.contains(&voter) {
            self.eligible_voters.push(voter);
        }
    }

    /// Create a new proposal.
    pub fn create_proposal(&mut self, proposal: Proposal) -> ProposalId {
        let proposal_id = proposal.proposal_id.clone();
        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), Vec::new());
        proposal_id
    }

    /// Get a proposal by ID.
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    /// Calculate vote weight based on expertise in relevant domains.
    pub fn calculate_vote_weight(
        &self,
        voter_id: &str,
        proposal: &Proposal,
        tracker: &CapabilityTracker,
    ) -> f32 {
        if proposal.relevant_domains.is_empty() {
            // No specific domains, equal weight
            return 1.0;
        }

        // Get voter's profile
        let profile = if voter_id == tracker.instance_id() {
            Some(tracker.profile())
        } else {
            tracker.peers().get(voter_id)
        };

        match profile {
            Some(profile) => {
                let score = profile.match_score(&proposal.relevant_domains);
                // Base weight of 0.5 + up to 0.5 based on expertise
                (0.5 + 0.5 * score).max(self.min_vote_weight)
            }
            None => self.min_vote_weight,
        }
    }

    /// Cast a vote on a proposal.
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        decision: VoteDecision,
        tracker: &CapabilityTracker,
        rationale: Option<String>,
    ) -> Result<Vote> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| CollectiveError::ProposalNotFound(proposal_id.to_string()))?;

        if !proposal.is_open() {
            return Err(CollectiveError::ProposalExpired(proposal_id.to_string()));
        }

        // Calculate weight
        let weight = self.calculate_vote_weight(&self.instance_id, proposal, tracker);

        let mut vote = Vote::new(
            self.instance_id.clone(),
            proposal_id.to_string(),
            decision,
            weight,
        );

        if let Some(r) = rationale {
            vote = vote.with_rationale(r);
        }

        // Remove any existing vote from this voter
        if let Some(votes) = self.votes.get_mut(proposal_id) {
            votes.retain(|v| v.voter_id != self.instance_id);
            votes.push(vote.clone());
        }

        Ok(vote)
    }

    /// Record a vote from another agent.
    pub fn record_vote(&mut self, vote: Vote) -> Result<()> {
        let proposal = self
            .proposals
            .get(&vote.proposal_id)
            .ok_or_else(|| CollectiveError::ProposalNotFound(vote.proposal_id.clone()))?;

        if !proposal.is_open() {
            return Err(CollectiveError::ProposalExpired(vote.proposal_id.clone()));
        }

        if let Some(votes) = self.votes.get_mut(&vote.proposal_id) {
            // Remove any existing vote from this voter
            votes.retain(|v| v.voter_id != vote.voter_id);
            votes.push(vote);
        }

        Ok(())
    }

    /// Tally votes for a proposal.
    pub fn tally_votes(&self, proposal_id: &str) -> Result<TallyResult> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| CollectiveError::ProposalNotFound(proposal_id.to_string()))?;

        let votes = self.votes.get(proposal_id).map(|v| v.as_slice()).unwrap_or(&[]);

        let mut weighted_approval = 0.0;
        let mut weighted_rejection = 0.0;
        let mut weighted_abstention = 0.0;

        for vote in votes {
            match vote.decision {
                VoteDecision::Approve => weighted_approval += vote.weight,
                VoteDecision::Reject => weighted_rejection += vote.weight,
                VoteDecision::Abstain => weighted_abstention += vote.weight,
            }
        }

        let voter_count = votes.len();
        let eligible_voters = self.eligible_voters.len().max(1);

        // Quorum is based on number of voters, not weighted votes
        let quorum_ratio = voter_count as f32 / eligible_voters as f32;
        let quorum_reached = quorum_ratio >= proposal.required_quorum;

        // Approval is based on weighted votes (excluding abstentions)
        let total_decisive = weighted_approval + weighted_rejection;
        let approval_ratio = if total_decisive > 0.0 {
            weighted_approval / total_decisive
        } else {
            0.0
        };

        let approved = quorum_reached && approval_ratio >= proposal.required_approval;

        let status = if proposal.is_expired() {
            if quorum_reached {
                if approved {
                    ProposalStatus::Approved
                } else {
                    ProposalStatus::Rejected
                }
            } else {
                ProposalStatus::Expired
            }
        } else if quorum_reached {
            // Early resolution if quorum reached and clear majority
            if approval_ratio >= 0.9 {
                ProposalStatus::Approved
            } else if approval_ratio <= 0.1 {
                ProposalStatus::Rejected
            } else {
                ProposalStatus::Open
            }
        } else {
            ProposalStatus::Open
        };

        Ok(TallyResult {
            weighted_approval,
            weighted_rejection,
            weighted_abstention,
            voter_count,
            eligible_voters,
            quorum_reached,
            approved,
            status,
        })
    }

    /// Resolve a proposal (finalize its status).
    pub fn resolve_proposal(&mut self, proposal_id: &str) -> Result<TallyResult> {
        let tally = self.tally_votes(proposal_id)?;

        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
            proposal.status = tally.status.clone();
            proposal.resolved_at = Some(Utc::now());
        }

        Ok(tally)
    }

    /// Cancel a proposal (only by proposer).
    pub fn cancel_proposal(&mut self, proposal_id: &str) -> Result<()> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| CollectiveError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.proposer_id != self.instance_id {
            return Err(CollectiveError::Internal(anyhow::anyhow!(
                "Only the proposer can cancel a proposal"
            )));
        }

        proposal.status = ProposalStatus::Cancelled;
        proposal.resolved_at = Some(Utc::now());

        Ok(())
    }

    /// Get all open proposals.
    pub fn open_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().filter(|p| p.is_open()).collect()
    }

    /// Check and resolve expired proposals.
    pub fn check_expired(&mut self) -> Vec<(ProposalId, TallyResult)> {
        let expired: Vec<_> = self
            .proposals
            .values()
            .filter(|p| p.is_expired())
            .map(|p| p.proposal_id.clone())
            .collect();

        let mut results = Vec::new();
        for proposal_id in expired {
            if let Ok(tally) = self.resolve_proposal(&proposal_id) {
                results.push((proposal_id, tally));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            "agent-1".to_string(),
            ProposalType::StrategyAdoption,
            "Adopt new code review strategy",
            "Proposal to adopt the new code review strategy",
            serde_json::json!({"strategy_id": "strat-123"}),
            Duration::hours(24),
        )
        .with_quorum(0.5)
        .with_approval(0.6);

        assert!(proposal.is_open());
        assert!(!proposal.is_expired());
        assert_eq!(proposal.required_quorum, 0.5);
        assert_eq!(proposal.required_approval, 0.6);
    }

    #[test]
    fn test_voting() {
        let mut coordinator = ConsensusCoordinator::new("agent-1".to_string());
        coordinator.set_eligible_voters(vec![
            "agent-1".to_string(),
            "agent-2".to_string(),
            "agent-3".to_string(),
        ]);

        let proposal = Proposal::new(
            "agent-1".to_string(),
            ProposalType::PolicyChange,
            "Test proposal",
            "Description",
            serde_json::json!({}),
            Duration::hours(24),
        )
        .with_quorum(0.5);

        let proposal_id = coordinator.create_proposal(proposal);

        // Record votes
        coordinator
            .record_vote(Vote::new(
                "agent-1".to_string(),
                proposal_id.clone(),
                VoteDecision::Approve,
                1.0,
            ))
            .unwrap();

        coordinator
            .record_vote(Vote::new(
                "agent-2".to_string(),
                proposal_id.clone(),
                VoteDecision::Approve,
                0.8,
            ))
            .unwrap();

        let tally = coordinator.tally_votes(&proposal_id).unwrap();

        assert_eq!(tally.voter_count, 2);
        assert!(tally.quorum_reached);
        assert!(tally.weighted_approval > 0.0);
    }
}
