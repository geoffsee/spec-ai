# spec-ai-collective

Emergent collective intelligence for multi-agent coordination in spec-ai.

## Overview

This crate provides the building blocks for agents to coordinate, learn from each other, and develop specialized expertise over time. Unlike knowledge graph sync (which shares *data*), collective intelligence enables agents to share *experience* and coordinate *behavior*.

## Modules

### `capability`
Track agent proficiency across domains using EMA-based updates.

```rust
use spec_ai_collective::capability::{Capability, CapabilityTracker, TaskOutcome};

let mut capability = Capability::new("code_review");
capability.update(TaskOutcome::Success, Duration::from_secs(30));
// Proficiency updates via exponential moving average
```

### `delegation`
Route tasks to capable peers in the mesh.

```rust
use spec_ai_collective::delegation::{DelegatedTask, DelegationManager, TaskPriority};

let task = DelegatedTask::new(
    "refactor_module",
    TaskPriority::Normal,
    vec!["rust".into(), "refactoring".into()],
);
```

### `learning`
Share and discover successful strategies.

```rust
use spec_ai_collective::learning::{Strategy, LearningFabric};

let strategy = Strategy::new(
    "api_design",
    vec!["Define endpoints", "Add validation", "Document"],
    None, // optional embedding
);
```

### `consensus`
Expertise-weighted collective decision-making.

```rust
use spec_ai_collective::consensus::{Proposal, ProposalType, ConsensusCoordinator};

let proposal = Proposal::new(
    proposer_id,
    ProposalType::StrategyAdoption,
    "Adopt caching strategy",
    "Use LRU for API responses",
    0.6, // quorum
    deadline,
);
```

### `orchestration`
Multi-agent workflow coordination.

```rust
use spec_ai_collective::orchestration::{Workflow, WorkflowStage, StageType};

let workflow = Workflow::new("pipeline", vec![
    WorkflowStage::new("fetch", StageType::Parallel, vec![]),
    WorkflowStage::new("process", StageType::MapReduce {
        map_capability: "parsing".into(),
        reduce_capability: "aggregation".into(),
    }, vec!["fetch"]),
]);
```

### `specialization`
Detect emergent expertise and capability gaps.

```rust
use spec_ai_collective::specialization::{SpecializationEngine, SpecializationStatus};

let engine = SpecializationEngine::new();
// Returns specialists sorted by proficiency
let experts = engine.get_specialists("security_review");
```

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Capability** | Proficiency + experience in a domain |
| **Strategy** | Proven approach steps for a task type |
| **Proposal** | Request for collective decision |
| **Workflow** | Multi-stage, multi-agent task plan |
| **Specialist** | Agent with high proficiency in a domain |

## Integration

This crate is used by:
- `spec-ai-core` for collective intelligence tools
- `spec-ai-config` for persistence of capabilities, strategies, etc.
- `spec-ai-api` for mesh message handling

## License

MIT License - see repository root for details.
