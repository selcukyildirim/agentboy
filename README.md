# AgentBoy

AI-powered autonomous business agent framework. Free Desktop version with 60 department-specific agents.

## Overview

AgentBoy is an open-source AI agent platform that automates business workflows across Finance, Accounting, Procurement, Sales, HR, Operations, Logistics, Management, PMO, and Legal departments.

### Features

- **60 Business Agents** — Department-specific AI agents with LLM integration
- **CSV/Data Analysis** — Parse, analyze, and generate insights from business data
- **LLM-Powered** — Integrates with OpenAI, Anthropic, Gemini, and Ollama
- **Desktop App** — Tauri-based cross-platform desktop application
- **Extensible** — Build custom agents with the Agent trait and SDK

### Departments

| Department | Agents | Capabilities |
|---|---|---|
| Finance | 6 | Bank reconciliation, budget variance, expense analysis, financial risk, revenue recognition, cash flow forecast |
| Accounting | 6 | Account reconciliation, invoice control, invoice reader, tax compliance, journal entries, reconcile reports |
| Procurement | 6 | Price history, procurement decisions, supplier comparison, PO validation, spend analytics, vendor risk |
| Sales | 6 | Sales forecast, lead scoring, pipeline health, win/loss analysis, customer segmentation, pricing optimization |
| HR | 6 | Attrition risk, headcount planning, compensation analysis, training ROI, employee engagement, hiring pipeline |
| Operations | 6 | Inventory optimization, production scheduling, quality assurance, capacity planning, maintenance, supply chain risk |
| Logistics | 6 | Route optimization, fleet management, warehouse optimization, delivery tracking, freight analysis, last mile |
| Management | 6 | KPI reporting, strategic initiatives, budget tracking, board reports, OKR tracking, decision matrix |
| PMO | 6 | Project tracking, risk register, resource allocation, stakeholder reports, change requests, lessons learned |
| Legal | 6 | Contract analysis, compliance checks, risk assessment, document review, deadline tracking, regulatory monitoring |

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for desktop app)

### Build

```bash
# Clone the repository
git clone https://github.com/agentboy/agentboy.git
cd agentboy

# Build the workspace
cargo build

# Run tests
cargo test --workspace
```

### Desktop App

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Architecture

```
agentboy/
├── crates/
│   ├── common/              # Core types and error handling
│   ├── agent-runtime/       # Agent trait, context, execution
│   ├── agents/              # 60 free business agents
│   ├── orchestrator/        # Agent orchestration
│   ├── llm-gateway/         # LLM provider abstraction
│   ├── provider-*/          # OpenAI, Anthropic, Gemini, Ollama
│   ├── rag-core/            # Retrieval-augmented generation
│   ├── spreadsheet-engine/  # CSV/Excel parsing
│   ├── document-parser/     # Multi-format document parsing
│   └── ...                  # Infrastructure crates
└── apps/
    └── desktop/             # Tauri desktop application
```

## Agent Development

Implement the `Agent` trait to create custom agents:

```rust
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

pub struct MyAgent;

#[async_trait::async_trait]
impl Agent for MyAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "custom.my-agent".to_string(),
            version: "1.0.0".to_string(),
            name: "My Agent".to_string(),
            department: "Custom".to_string(),
            description: "A custom business agent".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string()],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 30,
                timeout_seconds: 120,
            },
        }
    }

    fn supports_context(&self) -> bool { true }

    async fn execute_with_context(
        &self,
        input: serde_json::Value,
        ctx: &dyn AgentContext,
    ) -> AppResult<serde_json::Value> {
        // Your agent logic here
        let llm_analysis = ctx.call_llm("System prompt", "User prompt").await?;
        Ok(serde_json::json!({ "result": llm_analysis }))
    }
}
```

## Configuration

Agents accept CSV data as input and return structured JSON:

```json
{
  "input_csv": "name,amount\nAcme,1000\nBeta,2000",
  "options": { "threshold": 500 }
}
```

Output format:

```json
{
  "summary": { "total": 2, "flagged": 1 },
  "details": [...],
  "llm_analysis": "Analysis text from LLM"
}
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific department tests
cargo test -p agents -- finance
cargo test -p agents -- hr
cargo test -p agents -- sales

# Run with output
cargo test -- --nocapture
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting a PR.
