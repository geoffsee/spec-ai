# spec-ai Architecture Diagram

## Component Overview

```mermaid
graph TB
    subgraph UI["User Interface"]
        CLI["CLI/REPL<br/>(spec-ai-cli)"]
        TUI["Terminal UI<br/>(spec-ai-tui-app)"]
        Spec["Agent Spec<br/>(TOML)"]
    end

    subgraph Config["Configuration & Registry"]
        ConfigCore["App/Model/Agent Config<br/>(spec-ai-config/src/config)"]
        AgentReg["Agent Registry"]
        ToolReg["Tool Registry"]
        PluginReg["Plugin Registry"]
    end

    subgraph Core["Core Execution Engine"]
        AgentCore["AgentCore<br/>(spec-ai-core/src/agent/core)"]
        ModelFactory["Model Factory<br/>(spec-ai-core/src/agent/factory)"]
        Tools["Built-in Tools<br/>(file, bash, graph, audio, util)"]
    end

    subgraph Providers["Model Providers"]
        ProvidersHub["OpenAI | Anthropic | LM Studio | MLX | Ollama"]
    end

    subgraph Knowledge["Knowledge & Memory"]
        Embeddings["Embeddings Service<br/>(spec-ai-core/src/embeddings)"]
        GraphDB["Knowledge Graph<br/>Nodes/Edges"]
    end

    subgraph MeshSync["Service Mesh & Sync"]
        MeshAPI["Mesh API<br/>(registry/message bus)"]
        MeshRegistry["MeshRegistry<br/>Leader + Peers"]
        MessageBus["Message Bus<br/>Delegation/GraphSync"]
        SyncAPI["Sync API<br/>(graph sync)"]
        SyncEngine["SyncEngine<br/>(spec-ai-graph-sync)"]
    end

    subgraph Collective["Collective Intelligence"]
        CapTracker["Capability Tracker"]
        LearningFabric["Learning Fabric"]
        Consensus["Consensus Coordinator"]
        Workflows["Workflow Engine"]
        Specialization["Specialization Engine"]
    end

    subgraph Persistence["Persistence Layer (DuckDB)"]
        DB["DuckDB"]
        Messages["Messages"]
        ToolLogs["Tool Logs"]
        GraphTables["Graph Tables"]
        PolicyCache["Policy Cache"]
        CollectiveTables["Collective Tables<br/>(capabilities, strategies,<br/>proposals, workflows)"]
    end

    CLI --> AgentCore
    TUI --> AgentCore
    Spec --> ConfigCore
    ConfigCore --> AgentReg
    AgentReg --> AgentCore
    ToolReg --> AgentCore
    PluginReg --> AgentCore
    AgentCore --> ModelFactory --> ProvidersHub
    AgentCore --> Tools --> ToolLogs
    AgentCore --> Embeddings
    Embeddings --> GraphDB
    AgentCore --> MeshAPI
    MeshAPI --> MeshRegistry
    MeshRegistry --> MessageBus
    MessageBus --> AgentCore
    AgentCore --> SyncAPI --> SyncEngine
    SyncEngine --> GraphTables
    SyncEngine --> DB
    AgentCore --> DB
    DB --> Messages
    DB --> PolicyCache
    AgentCore --> CapTracker
    CapTracker --> LearningFabric
    CapTracker --> Specialization
    MessageBus --> Consensus
    MessageBus --> Workflows
    Collective --> CollectiveTables

    style AgentCore fill:#ff6b6b
    style Collective fill:#9b59b6
    style DB fill:#2a8b9d
    style Embeddings fill:#2a8b9d
    style MeshAPI fill:#2a8b9d
    style SyncAPI fill:#2a8b9d
```

## Key Components

### User Interface
- **CLI/REPL** (`spec-ai-cli`): Command-line interface for interactive agent control
- **Terminal UI** (`spec-ai-tui-app`): Full-featured interactive terminal application built on the `spec-ai-tui` framework
- **Agent Spec**: TOML-based declarative specifications for structured execution

### Configuration & Registry
- **AppConfig**: Global application settings (database, logging, UI, audio)
- **Agent Registry**: Named agent profiles with per-agent settings
- **Tool Registry**: Available tools with execution implementations
- **Plugin Registry**: Bootstrap plugins for codebase analysis

### Core Execution Engine
- **AgentCore**: Main execution loop orchestrating the entire agent workflow
- **Model Factory**: Creates appropriate model provider instances

### Model Providers
Multi-provider support:
- OpenAI (GPT-4, etc.)
- Anthropic (Claude)
- LM Studio (local models)
- MLX (Apple Silicon optimization)
- Ollama (open-source models)

### Tool System
**Tool Trait**: Extensible interface for tools

**Built-in Tools**:
- **File Operations**: read, write, extract
- **Bash/Shell**: Command execution
- **Web Tools**: Search, scraping
- **Graph Operations**: Knowledge graph queries
- **Audio**: Transcription
- **Utilities**: Calculator, echo, prompt

### Knowledge & Memory
- **Embeddings Service**: Vector generation for semantic search
- **Knowledge Graph** (`spec-ai-knowledge-graph`): Isolated crate for graph storage, vector clocks, and graph types (GraphNodes, GraphEdges) for relationship tracking

### Terminal UI Framework
- **spec-ai-tui**: Low-level TUI framework built from scratch on crossterm, providing geometry primitives, cell-based buffer rendering, constraint-based layout, widget system, and async event loop
- **spec-ai-tui-app**: Interactive terminal application using the TUI framework, with chat interface, backend integration, and Elm-inspired state management

### Distributed Coordination & Sync
- **Mesh Registry & Messaging**: Agents register, exchange heartbeats, and route inter-agent messages (task delegation, notifications, sync triggers) via the mesh API and tooling (`crates/spec-ai-api/src/api/mesh.rs`, `crates/spec-ai-core/src/tools/builtin/mesh_communication.rs`).
- **Graph Sync Pipeline** (`spec-ai-graph-sync`): Vector-clock negotiation chooses full vs incremental graph exchange; conflict resolution merges concurrent edits before persisting. Key modules: `engine.rs`, `protocol.rs`, `resolver.rs`.
- **State Persistence**: Sync state, changelog, tombstones, and vector clocks are stored alongside graph data in DuckDB (`crates/spec-ai-config/src/persistence`).

### Collective Intelligence (`spec-ai-collective`)
Emergent multi-agent coordination enabling agents to learn from each other and develop specializations.

- **Capability Tracking**: Agents track proficiency per domain using EMA-based updates. The `CapabilityTracker` routes tasks to the most capable agent.
- **Task Delegation**: `DelegationManager` handles task routing, delegation chains, and result aggregation across the mesh.
- **Learning Fabric**: Agents share successful strategies. Strategies include approach steps and semantic embeddings for similarity-based discovery.
- **Consensus Coordinator**: Expertise-weighted voting on proposals (strategy adoption, policy changes, resource allocation, conflict resolution).
- **Workflow Engine**: Multi-agent workflow orchestration with Sequential, Parallel, MapReduce, Consensus, and ConditionalBranch stage types.
- **Specialization Engine**: Detects emergent specialists, identifies capability gaps, and routes queries to domain experts.

See [`docs/COLLECTIVE_INTELLIGENCE.md`](COLLECTIVE.md) for detailed documentation.

### Persistence Layer (DuckDB)
- **Messages**: Conversation history
- **Memory Vectors**: Embeddings for semantic search
- **Tool Logs**: Execution records
- **Graph Tables**: Knowledge graph entities and relationships
- **Policy Cache**: Authorization rules

### Access Control
- **Policy Engine**: Evaluates Allow/Deny rules for tool execution based on (agent, action, resource) tuples

### Analysis & Discovery
- **Bootstrap Self**: Codebase self-discovery system
- **Plugins**: Modular analysis for specific languages
  - Cargo Plugin (Rust projects)
  - TOAK Tokenizer (code tokenization)
  - Universal Code Plugin (generic code analysis)

## Data Flow

1. **Initialization**:
   - CLI or TUI loads configuration
   - Agent profile selected from registry
   - AgentCore initialized with tools and model provider

2. **Execution Loop**:
   - Retrieve semantic memory via embeddings
   - Query knowledge graph for context
   - Call model with context and available tools
   - Parse model response for tool calls
   - Check policy engine for permissions
   - Execute authorized tools
   - Log results to persistence
   - Add to conversation history
   - Repeat until goal satisfied

3. **Persistence**:
   - All state saved to DuckDB
   - Messages, vectors, logs, graph data, policies stored
   - Enables agent continuity across sessions

4. **Knowledge Building**:
   - Tool results and messages analyzed for entities and relationships
   - GraphNodes created for discovered concepts
   - GraphEdges created for relationships
   - Embeddings generated for semantic recall
