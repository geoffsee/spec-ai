# Collective Intelligence

Emergent multi-agent coordination for spec-ai.

## Overview

The Collective Intelligence system enables agents to coordinate, learn from each other, and develop specialized expertise over time. Unlike Knowledge Graph Sync (which shares *data*), Collective Intelligence enables agents to share *experience* and coordinate *behavior*.

| System | What it shares | Purpose |
|--------|---------------|---------|
| Knowledge Graph Sync | Facts, entities, relationships | Shared world state |
| Collective Intelligence | Strategies, capabilities, decisions | Shared experience |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Collective Intelligence                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Capability  │  │   Learning   │  │  Consensus   │          │
│  │   Tracking   │  │    Fabric    │  │ Coordinator  │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐          │
│  │  Delegation  │  │  Workflow    │  │Specialization│          │
│  │   Manager    │  │   Engine     │  │   Engine     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                      Mesh Communication                          │
│  (CapabilityUpdate, LearningShare, ProposalVote, Workflow...)   │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Capability Tracking

Agents track their proficiency in different domains using an Exponential Moving Average (EMA) that adapts based on task outcomes.

```rust
// Capability updates based on task results
capability.update(TaskOutcome::Success, duration);
// Proficiency adjusts: new = α * outcome + (1-α) * old
```

**Key Types:**
- `Capability` - Proficiency, experience count, success rate for a domain
- `ExpertiseProfile` - All capabilities for an agent instance
- `CapabilityTracker` - Routes tasks to the most capable agent

### 2. Task Delegation

Agents can delegate tasks to peers with higher capability scores.

```toml
# Agent configuration for delegation
[agents.worker]
enable_collective = true
accept_delegations = true
min_delegation_score = 0.3
max_concurrent_tasks = 3
```

**Delegation Flow:**
1. Agent receives a task with required capabilities
2. `CapabilityTracker` evaluates local vs peer proficiency
3. If a peer scores higher, task is delegated via mesh message
4. Results flow back through the delegation chain

### 3. Inter-Agent Learning

Agents share successful strategies that others can discover and apply.

```rust
// Share a strategy after successful task completion
let strategy = Strategy::new(
    "code_refactoring",
    vec!["Analyze dependencies", "Extract interface", "Update callers"],
    Some(embedding),  // For semantic search
);
learning_fabric.add_strategy(strategy);
```

**Strategy Discovery:**
- Query by task type: exact match lookup
- Query by embedding: semantic similarity search
- Strategies track success/failure counts for ranking

### 4. Collective Decision-Making

Agents vote on proposals with expertise-weighted ballots.

```rust
// Submit a proposal
let proposal = Proposal::new(
    proposer_id,
    ProposalType::StrategyAdoption,
    "Adopt new caching strategy",
    "Use LRU cache for API responses",
    0.6,  // 60% quorum required
    deadline,
);

// Cast an expertise-weighted vote
coordinator.cast_vote(proposal_id, voter_id, Vote::Approve, weight, rationale);
```

**Proposal Types:**
- `StrategyAdoption` - Adopt a shared strategy as standard practice
- `PolicyChange` - Modify collective policies
- `ResourceAllocation` - Allocate shared resources
- `ConflictResolution` - Resolve disagreements

### 5. Workflow Orchestration

Complex tasks can be broken into multi-agent workflows.

```rust
let workflow = Workflow::new("data_pipeline", vec![
    WorkflowStage::new("fetch", StageType::Parallel, vec!["http"]),
    WorkflowStage::new("transform", StageType::MapReduce {
        map_capability: "parsing".into(),
        reduce_capability: "aggregation".into(),
    }, vec!["fetch"]),
    WorkflowStage::new("validate", StageType::Consensus {
        min_agreement: 0.8
    }, vec!["transform"]),
]);
```

**Stage Types:**
- `Sequential` - One agent, one task
- `Parallel` - Multiple agents work concurrently
- `MapReduce` - Distribute then aggregate
- `Consensus` - Require agreement threshold
- `ConditionalBranch` - Dynamic routing based on results

### 6. Emergent Specialization

Over time, agents develop specializations based on their task history.

```rust
// Specialization levels (determined by proficiency + experience)
enum SpecializationStatus {
    Learning,     // < 0.3 proficiency
    Proficient,   // 0.3 - 0.6
    Specialist,   // 0.6 - 0.85
    Expert,       // > 0.85 + significant experience
}
```

The `SpecializationEngine`:
- Detects emerging specialists in the mesh
- Identifies capability gaps (domains with no specialists)
- Routes queries to the best-matched expert

## Configuration

### Agent Profile Options

```toml
[agents.collaborative]
# Enable collective intelligence
enable_collective = true

# Accept delegated tasks from peers
accept_delegations = true

# Domains this agent prefers
preferred_domains = ["code_review", "testing"]

# Maximum concurrent delegated tasks
max_concurrent_tasks = 3

# Minimum capability score to accept delegation
min_delegation_score = 0.3

# Share successful strategies with the mesh
share_learnings = true

# Participate in collective voting
participate_in_voting = true
```

## Tools

The following tools are available when `enable_collective = true`:

| Tool | Purpose |
|------|---------|
| `delegate_task` | Route a task to a capable peer |
| `query_capabilities` | Discover peer capabilities |
| `share_capabilities` | Broadcast capability updates |
| `share_strategy` | Share a learned strategy |
| `submit_proposal` | Submit a proposal for voting |
| `cast_vote` | Vote on a proposal |
| `create_workflow` | Create a multi-agent workflow |
| `report_stage_result` | Report workflow stage completion |

## Message Types

Collective Intelligence uses these mesh message types:

- `CapabilityUpdate` - Broadcast capability changes
- `CapabilityQuery` - Request peer capabilities
- `LearningShare` - Share a strategy
- `ProposalSubmit` - Submit a proposal
- `ProposalVote` - Cast a vote
- `WorkflowAssignment` - Assign a workflow stage
- `WorkflowStageResult` - Report stage completion

## Database Schema

Migration v9 adds these tables:

```sql
-- Agent capabilities and expertise
CREATE TABLE agent_capabilities (
    instance_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    proficiency REAL,
    experience_count INTEGER,
    success_rate REAL,
    ...
);

-- Shared strategies
CREATE TABLE strategies (
    strategy_id TEXT UNIQUE,
    origin_instance TEXT,
    task_type TEXT,
    approach_steps TEXT,  -- JSON
    embedding TEXT,
    success_count INTEGER,
    ...
);

-- Collective proposals
CREATE TABLE proposals (
    proposal_id TEXT UNIQUE,
    proposer_instance TEXT,
    proposal_type TEXT,
    status TEXT,  -- open, passed, rejected, expired
    quorum_required REAL,
    ...
);

-- Votes with expertise weighting
CREATE TABLE votes (
    proposal_id TEXT,
    voter_instance TEXT,
    vote_value TEXT,
    weight REAL,
    ...
);

-- Multi-agent workflows
CREATE TABLE workflows (...);
CREATE TABLE workflow_executions (...);
```

## Example: Emergent Code Review Specialization

```
1. Mesh starts with 3 generalist agents (A, B, C)

2. Agent A completes several code review tasks successfully
   → A's "code_review" proficiency increases via EMA
   → A shares successful review strategies

3. Agent B attempts code review, struggles
   → B's proficiency stays low
   → CapabilityTracker routes future reviews to A

4. After many iterations:
   → A becomes "Specialist" in code_review
   → B and C learn A's strategies
   → New review tasks auto-route to A unless overloaded

5. Agent A proposes "mandatory linting before review"
   → B and C vote (weighted by their review experience)
   → If passed, becomes collective policy
```

## Comparison: Graph Sync vs Collective Intelligence

| Aspect | Graph Sync | Collective |
|--------|-----------|------------|
| Shares | Entities, relationships | Strategies, capabilities |
| Conflict resolution | Vector clocks, merge | Voting, consensus |
| Purpose | "What is true" | "What works" |
| Granularity | Per-node/edge | Per-task/agent |
| Use case | Shared knowledge base | Coordinated behavior |

Both systems complement each other:
- Graph Sync ensures all agents know the same facts
- Collective Intelligence ensures agents act on those facts effectively

## Crate Structure

```
crates/spec-ai-collective/
├── src/
│   ├── lib.rs              # Module exports
│   ├── types.rs            # Common types (IDs, errors)
│   ├── capability.rs       # Capability tracking, ExpertiseProfile
│   ├── delegation.rs       # Task delegation, DelegationManager
│   ├── learning.rs         # Strategy sharing, LearningFabric
│   ├── consensus.rs        # Proposals, voting, ConsensusCoordinator
│   ├── orchestration.rs    # Workflows, WorkflowEngine
│   └── specialization.rs   # Emergent specialization detection
└── Cargo.toml
```
