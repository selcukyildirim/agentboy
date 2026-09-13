# AI Workforce Platform --- Coding Agent Technical Specification v4

**Status:** Implementation Ready\
**Purpose:** This file is designed to be read directly by a coding agent
and executed phase-by-phase.\
**Product:** Free Desktop Client + Paid Company Server\
**Core principle:** Local-first processing, minimum necessary data
egress, provider-agnostic LLM layer, reusable skills, advanced RAG,
strict Free/Paid capability boundaries, Clean Architecture, SOLID, DRY
and YAGNI.

------------------------------------------------------------------------

# 0. CODING AGENT EXECUTION PROTOCOL

The coding agent MUST follow this section before writing code.

## 0.1 Execution rules

1.  Work **phase by phase**.
2.  Do not implement future phases early unless a dependency explicitly
    requires it.
3.  Before each phase:
    -   inspect the current repository,
    -   compare current state with the tasks in that phase,
    -   create/update a local implementation checklist,
    -   identify missing dependencies.
4.  For every completed task:
    -   write production code,
    -   add tests,
    -   run formatting/linting,
    -   run relevant test suites,
    -   update implementation status in `docs/IMPLEMENTATION_STATUS.md`.
5.  Do not mark a task complete unless its acceptance criteria pass.
6.  If an architectural decision is ambiguous:
    -   create an ADR under `docs/adr/`,
    -   choose the simplest option compatible with this specification,
    -   continue implementation.
7.  Never bypass security, permission, egress, entitlement, or
    structured-output validation for convenience.
8.  Never hard-code provider model names into agent business logic.
9.  Free/Paid restrictions MUST be enforced in runtime/backend logic,
    not only hidden in UI.
10. All external side effects must be explicit, auditable and
    permission-controlled.

## 0.2 Required implementation status file

Create:

`docs/IMPLEMENTATION_STATUS.md`

Format:

``` md
# Implementation Status

## Current Phase
PHASE-01

## Completed
- [x] TASK-001 ...

## In Progress
- [ ] TASK-004 ...

## Blocked
- None

## Decisions
- ADR-001 ...

## Test Status
- cargo test: PASS
- frontend tests: PASS
```

## 0.3 Required task completion behavior

For each task:

``` text
READ SPEC
→ IMPLEMENT
→ TEST
→ VERIFY ACCEPTANCE CRITERIA
→ UPDATE STATUS
→ COMMIT-READY STATE
→ NEXT TASK
```

Do not jump from planning directly to marking a phase complete.

------------------------------------------------------------------------

# 1. PRODUCT DEFINITION

The platform consists of two product tiers.

## 1.1 Free Desktop Client

A Rust-based local AI worker for individual employees.

Primary jobs:

-   spreadsheet work,
-   document analysis,
-   local file operations,
-   department-specific analysis,
-   personal knowledge/RAG,
-   decision support based on user-provided documents,
-   manually triggered local workflows.

## 1.2 Paid Company Server

A centrally managed AI workforce layer for companies.

Primary jobs:

-   ERP/database/API connectivity,
-   shared company knowledge,
-   department business agents,
-   scheduled/event-driven work,
-   approvals,
-   RBAC,
-   audit,
-   autonomous or semi-autonomous business execution.

------------------------------------------------------------------------

# 2. FREE / PAID BOUNDARY

This boundary is mandatory.

## 2.1 Free includes

-   Rust local agent runtime
-   Local file access with explicit user-selected scopes
-   XLSX/CSV processing
-   PDF/DOCX/TXT processing
-   JSON/XML transformations
-   Department-specific agents
-   Local workflows
-   Manual workflow execution
-   Personal/local RAG
-   Local vector index
-   Local metadata store
-   Local cache
-   BYOK provider configuration
-   OpenAI provider
-   Anthropic Claude provider
-   Google Gemini provider
-   OpenAI-compatible custom endpoint
-   Ollama/local endpoint
-   Model selection
-   Provider capability discovery
-   Controlled fallback
-   Data Egress Guard
-   Secret/PII redaction
-   Local audit/history
-   Token usage and estimated cost visibility
-   Evidence-first answers
-   Local decision memory

## 2.2 Free excludes

-   ERP connectors
-   Central/shared DB connectors
-   Company Workspace
-   Shared knowledge base
-   Shared department workflows
-   Shared company agents
-   Central RBAC
-   Company policy management
-   Central approvals
-   Central secrets vault
-   Scheduler
-   Cron execution
-   Event-driven execution
-   Webhook-triggered execution
-   24/7 server execution
-   Autonomous ERP writes
-   Company Knowledge Graph
-   Company-wide Decision Memory
-   SSO
-   Multi-tenancy
-   Central audit
-   Server observability
-   HA/SLA deployment
-   On-prem server package

## 2.3 Paid includes

Everything in Free plus:

-   Company Workspace
-   Tenant isolation
-   Users / groups / departments
-   RBAC
-   Company Policy Engine
-   Approval Engine
-   Secrets Vault abstraction
-   ERP/API/DB connectors
-   Shared Knowledge Fabric
-   Company Knowledge Graph
-   Company Decision Memory
-   Shared workflows
-   Scheduled workflows
-   Event-driven workflows
-   Long-running workflows
-   Human-in-the-loop execution
-   Autonomous agent execution within policy
-   Central audit
-   Central metrics/tracing
-   Usage and budget controls
-   Model/provider policy
-   Data residency controls
-   On-prem/private cloud deployment
-   Distributed workers
-   Queue/retry/DLQ
-   Idempotent business actions

## 2.4 Commercial principle

**Free is not a deliberately crippled demo.**

The commercial boundary is:

> Free = work on my device.\
> Paid = work for my company.

------------------------------------------------------------------------

# 3. CAPABILITY / ENTITLEMENT MODEL

Every skill/tool capability MUST be identified and tiered.

Example:

``` yaml
id: erp.sales_order.write
tier: paid
scope: company
risk: financial
requires:
  - connector.erp
  - permission.sales_order.write
```

Runtime order:

``` text
Agent
  ↓
Skill
  ↓
Entitlement Check
  ↓
Permission Check
  ↓
Policy Check
  ↓
Tool Invocation
```

Required outcomes:

-   `ALLOW`
-   `DENY`
-   `REQUIRE_APPROVAL`

UI visibility is not security.

------------------------------------------------------------------------

# 4. TARGET REPOSITORY STRUCTURE

Create a monorepo.

``` text
/
├── apps/
│   ├── desktop/
│   └── server/
├── crates/
│   ├── agent-runtime/
│   ├── orchestrator/
│   ├── skill-sdk/
│   ├── tool-runtime/
│   ├── workflow-engine/
│   ├── llm-gateway/
│   ├── provider-openai/
│   ├── provider-anthropic/
│   ├── provider-gemini/
│   ├── provider-openai-compatible/
│   ├── provider-ollama/
│   ├── egress-guard/
│   ├── policy-core/
│   ├── entitlement-core/
│   ├── rag-core/
│   ├── document-parser/
│   ├── spreadsheet-engine/
│   ├── cache-core/
│   ├── secure-store/
│   ├── audit-core/
│   ├── usage-metering/
│   └── common/
├── packages/
│   ├── agent-packs/
│   ├── skill-packs/
│   └── schemas/
├── docs/
│   ├── adr/
│   ├── architecture/
│   └── IMPLEMENTATION_STATUS.md
└── tests/
```

If the actual repository already has a reasonable structure, preserve it
and map the logical modules to the existing layout rather than rewriting
everything.

------------------------------------------------------------------------

# 5. CORE DOMAIN CONTRACTS

The following abstractions MUST exist before department agents are
implemented.

## 5.1 Agent

``` yaml
id: finance.bank_reconciliation
version: 1.0.0
department: finance
tier: free
skills:
  - spreadsheet.read
  - records.normalize
  - records.reconcile
  - report.generate
permissions:
  filesystem:
    read: user_selected
    write: user_selected
  network:
    llm_gateway_only: true
rag:
  enabled: true
  scopes:
    - user_workspace
execution:
  max_steps: 20
  timeout_seconds: 300
```

## 5.2 Skill

A Skill is a reusable business capability.

Examples:

-   `spreadsheet.merge`
-   `records.reconcile`
-   `document.extract`
-   `document.compare`
-   `analysis.detect_anomaly`
-   `rag.search`
-   `decision.build_context`
-   `report.generate`

## 5.3 Tool

A Tool is a deterministic executor.

Examples:

-   filesystem read/write
-   XLSX parser/writer
-   CSV parser
-   PDF parser
-   DOCX parser
-   local DB
-   HTTP connector
-   ERP connector

## 5.4 Execution

Every execution needs:

``` text
execution_id
agent_id
agent_version
workflow_id?
status
started_at
finished_at
input_manifest
egress_manifest
tool_calls
model_calls
usage
result
error
```

------------------------------------------------------------------------

# 6. LLM PROVIDER SYSTEM

## 6.1 MVP providers

Must support:

1.  OpenAI
2.  Anthropic Claude
3.  Google Gemini
4.  OpenAI-compatible endpoint
5.  Ollama/local endpoint

Do not hard-code a single model as the application's architecture.

## 6.2 API key settings

UI section:

`Settings → AI Providers`

For each provider:

-   API key
-   endpoint if applicable
-   connection status
-   available models
-   default model
-   test connection
-   remove credential

## 6.3 Secure credential storage

Desktop:

-   macOS: Keychain
-   Windows: secure OS credential storage
-   Linux: Secret Service/libsecret abstraction

Server:

-   Vault/KMS-compatible secrets abstraction

The application DB stores only a reference:

``` text
provider_credentials
- id
- provider
- secret_ref
- created_at
- last_validated_at
- status
```

Never log secrets.

## 6.4 Provider contract

Implement a provider-neutral interface conceptually equivalent to:

``` rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn validate_credentials(&self) -> Result<ProviderStatus>;
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, req: CompletionRequest) -> Result<ResponseStream>;
    async fn embed(&self, req: EmbeddingRequest) -> Result<EmbeddingResponse>;
    fn capabilities(&self) -> ProviderCapabilities;
}
```

Not every provider must implement every capability. Unsupported
capabilities must be explicit.

## 6.5 Model capability registry

Normalize capabilities:

``` json
{
  "provider": "provider-id",
  "model": "provider-model-id",
  "capabilities": {
    "text": true,
    "vision": true,
    "tools": true,
    "structured_output": true,
    "reasoning": true,
    "prompt_cache": true,
    "embeddings": false
  }
}
```

Agent business logic requests capabilities, not specific branded model
names.

------------------------------------------------------------------------

# 7. MODEL ROUTER

Support three modes.

## 7.1 Manual

User explicitly selects provider/model.

## 7.2 Smart

Runtime selects based on:

-   task type
-   required capabilities
-   latency preference
-   privacy classification
-   user budget
-   availability

Example routing request:

``` json
{
  "task_type": "document_analysis",
  "required_capabilities": ["text", "structured_output"],
  "privacy_level": "L2",
  "max_cost_usd": 0.10,
  "latency_profile": "interactive"
}
```

## 7.3 Privacy First

Preference order:

``` text
local deterministic tool
→ local model
→ sanitized cloud request
→ minimum-required cloud request
```

If policy does not allow external data transfer, fail closed.

------------------------------------------------------------------------

# 8. PROVIDER FALLBACK

Fallback is allowed only for infrastructure-type failures.

Allowed triggers:

-   timeout
-   rate limit
-   temporary provider outage
-   network error
-   5xx

Do not silently switch providers because of:

-   safety refusal
-   policy refusal
-   semantic disagreement
-   invalid user request

Every fallback must be audited.

------------------------------------------------------------------------

# 9. CACHE ARCHITECTURE

Caching is a first-class subsystem.

## 9.1 L1 in-memory cache

Use for:

-   provider model catalog
-   capability registry
-   policy result
-   skill manifests
-   short-lived retrieval results
-   parser metadata

Requirements:

-   bounded
-   TTL
-   LRU or equivalent

## 9.2 L2 persistent local cache

Desktop default:

-   SQLite metadata
-   encrypted local file/blob cache where required

Cache:

-   document hashes
-   parsed text
-   extracted structure
-   chunk data
-   embeddings
-   deterministic tool outputs
-   selected model metadata
-   provider capability metadata

## 9.3 Content-addressable cache

Key concept:

``` text
SHA256(
  operation_version +
  input_hash +
  normalized_parameters +
  parser_or_skill_version
)
```

The same unchanged document must not be reparsed unnecessarily.

## 9.4 LLM response cache

Do NOT indiscriminately cache chat responses.

Cache only when safe and semantically stable, e.g.:

-   classification
-   extraction
-   deterministic tagging
-   schema mapping

Cache key MUST include:

``` text
provider
model
system_prompt_version
prompt_hash
structured_schema_version
temperature
toolset_version
policy_version
```

## 9.5 Native provider prompt cache

Create a provider-neutral abstraction:

``` rust
pub enum PromptCacheMode {
    Disabled,
    Auto,
    ProviderNative,
    Explicit { ttl_seconds: u64 }
}
```

Normalize metrics:

``` text
cache_hit
cache_read_tokens
cache_write_tokens
cache_provider
cache_ttl
estimated_saving
```

## 9.6 Paid server cache

Logical layers:

``` text
Worker Memory
→ Distributed Cache
→ Document/Object Cache
→ Vector Index
→ Provider Native Cache
```

All keys must be tenant-scoped.

Cross-tenant cache reuse is forbidden.

## 9.7 Cache invalidation

Invalidate when any relevant version changes:

-   input/document hash
-   parser version
-   chunking version
-   embedding model
-   prompt version
-   agent version
-   skill version
-   tool schema version
-   policy version
-   provider/model
-   RAG pipeline version

------------------------------------------------------------------------

# 10. DATA EGRESS GUARD

Every outbound LLM request must pass:

``` text
Context Builder
→ Data Classifier
→ PII/Secret Detector
→ Redactor
→ Policy
→ Optional User Approval
→ LLM Gateway
```

## 10.1 Data levels

  Level   Meaning
  ------- -----------------------------------
  L0      Local only
  L1      Metadata only
  L2      Sanitized/redacted content
  L3      Minimum required business content
  L4      Full content/document

Rules:

-   L4 is never the default.
-   Free client defaults to denying full-document cloud upload.
-   The user can explicitly allow a workflow/provider policy where
    appropriate.
-   Secrets are never intentionally sent to the model.

## 10.2 Egress manifest

Every outbound request produces:

``` json
{
  "provider": "provider-id",
  "classification": "L2",
  "files_uploaded": 0,
  "fields_included": ["description", "amount_bucket"],
  "pii_removed": true,
  "secrets_removed": true,
  "reason": "expense classification"
}
```

------------------------------------------------------------------------

# 11. TOOL SECURITY

Risk classes:

``` text
READ
WRITE
DESTRUCTIVE
EXTERNAL
FINANCIAL
ADMIN
```

Free defaults:

-   READ → allow within explicit local scope
-   WRITE → preview + policy
-   DESTRUCTIVE → explicit approval
-   EXTERNAL → Egress/Network policy
-   FINANCIAL → analysis/recommendation only
-   ADMIN → unavailable

Paid may elevate actions via company policies and approvals.

------------------------------------------------------------------------

# 12. STRUCTURED OUTPUT

Business side effects must never be derived from free-form assistant
text.

Example:

``` json
{
  "action": "FLAG_ORDER",
  "risk_score": 87,
  "reason_codes": ["LOW_MARGIN", "UNUSUAL_DISCOUNT"],
  "evidence_ids": ["ev-123", "ev-456"]
}
```

Flow:

``` text
LLM output
→ schema validation
→ policy validation
→ capability validation
→ optional approval
→ tool execution
```

If structured validation fails:

``` text
retry structured generation
→ fail closed
```

------------------------------------------------------------------------

# 13. WORKFLOW ENGINE

## 13.1 Free

Supports:

-   manual trigger
-   single-user local workflow
-   parameterized workflow
-   save/replay
-   execution history

No server cron/event execution.

## 13.2 Paid

Adds:

-   cron/schedule
-   event triggers
-   webhook
-   ERP event
-   queue
-   approval pause/resume
-   long-running execution
-   shared workflow
-   retry
-   compensation hooks

Example:

``` yaml
id: weekly-sales-report
version: 1

trigger:
  type: manual

steps:
  - skill: spreadsheet.merge
  - skill: sales.analyze
  - skill: report.generate
```

------------------------------------------------------------------------

# 14. ADVANCED RAG

The RAG system is not a simple "upload PDF and chat".

Pipeline:

``` text
Document
→ Parser
→ Structure Extraction
→ Document Classification
→ Metadata Extraction
→ Structure-aware Chunking
→ Embedding
→ Indexing
→ Query Understanding
→ Hybrid Retrieval
→ Reranking
→ Decision Context Builder
→ LLM
→ Evidence Validation
→ Answer
```

## 14.1 Supported MVP formats

-   PDF
-   DOCX
-   XLSX
-   CSV
-   TXT
-   Markdown
-   JSON
-   XML

## 14.2 Normalized document model

``` json
{
  "document_id": "uuid",
  "type": "contract",
  "sections": [],
  "tables": [],
  "entities": [],
  "metadata": {}
}
```

## 14.3 Chunking strategies

Implement abstractions for:

-   heading-aware
-   paragraph-aware
-   clause-aware
-   table-aware
-   semantic
-   sliding overlap

Do not treat spreadsheets like normal prose.

For XLSX/CSV prefer:

-   schema extraction
-   column semantics
-   statistics
-   key/entity detection
-   structured filtering/querying
-   grouped row summaries

------------------------------------------------------------------------

# 15. HYBRID RETRIEVAL

Pipeline:

``` text
Query Understanding
  ↓
Metadata Filters
  ├── Vector Search
  ├── Keyword/BM25
  ├── Structured Query
  └── Entity Lookup
          ↓
        Merge
          ↓
       Reranker
          ↓
      Context Pack
```

The architecture must not couple RAG to a single vector database.

------------------------------------------------------------------------

# 16. LOCAL KNOWLEDGE WORKSPACE

Free local storage should logically contain:

``` text
documents
document_sections
document_chunks
document_metadata
document_entities
embeddings
conversations
decisions
citations
rag_indexes
```

Default privacy model:

-   index locally,
-   retrieve locally,
-   send only selected evidence through Egress Guard if a cloud LLM is
    used.

------------------------------------------------------------------------

# 17. EVIDENCE-FIRST ANSWERS

Every decision-oriented RAG answer should support:

``` text
Recommendation
Confidence
Evidence
Missing Information
Assumptions
Required Approval
```

Each evidence item MUST reference source metadata:

``` json
{
  "document_id": "contract-123",
  "document_name": "SupplierContract.pdf",
  "page": 7,
  "section": "4.2 Payment",
  "chunk_id": "chunk-456",
  "retrieval_score": 0.91
}
```

If evidence is insufficient, the agent must say so rather than fabricate
certainty.

------------------------------------------------------------------------

# 18. DECISION CONTEXT BUILDER

Do not dump raw chunks directly into the decision prompt.

Required flow:

``` text
Question
→ Identify decision type
→ Determine required facts
→ Retrieve evidence
→ Query structured data
→ Retrieve applicable rules
→ Retrieve historical cases
→ Identify missing facts
→ Build DecisionContext
→ Ask model for structured recommendation
```

Example:

``` json
{
  "decision_type": "supplier_selection",
  "facts": {},
  "rules": [],
  "historical_cases": [],
  "missing_information": [],
  "evidence": []
}
```

------------------------------------------------------------------------

# 19. DECISION MEMORY

Free:

-   local user decision memory

Paid:

-   permission-aware company decision memory

Schema:

``` text
decision_id
decision_type
subject
context_snapshot
recommendation
human_decision
human_reason
evidence
approved_by
agent_version
policy_version
created_at
```

Past decisions are evidence, not immutable rules.

------------------------------------------------------------------------

# 20. DEPARTMENT AGENT PACKS

The long-term catalog includes 60 Free agent experiences.

## Finance

-   Cash Flow Analyst
-   Collection Analyst
-   Bank Reconciliation Agent
-   Budget Variance Agent
-   Financial Report Agent
-   Expense Analyst

## Accounting

-   Invoice Reader
-   Invoice Control Agent
-   Account Reconciliation Agent
-   Closing Assistant
-   Ledger Analysis Agent
-   Accounting Document Organizer

## Sales

-   Sales Performance Analyst
-   Quotation Analyst
-   Customer Analysis Agent
-   Lost Customer Finder
-   Margin Analysis Agent
-   Sales Report Agent

## Procurement

-   Supplier Comparison Agent
-   Price History Agent
-   Purchase Order Analyst
-   Supplier Performance Agent
-   RFQ Assistant
-   Procurement Decision Agent

## HR

-   CV Analysis Agent
-   Employee Document Agent
-   Leave Analysis Agent
-   Attendance Analyst
-   HR Report Agent
-   HR Policy Assistant

## Operations

-   Operations Analyst
-   Exception Finder
-   SLA Monitor Agent
-   Process Bottleneck Agent
-   Quality Analysis Agent
-   Daily Operations Brief Agent

## Logistics / Warehouse

-   Inventory Analysis Agent
-   Slow Moving Agent
-   Stockout Risk Agent
-   Shipment Analysis Agent
-   Inventory Reconciliation Agent
-   Warehouse Report Agent

## Management

-   Executive Brief Agent
-   KPI Analyst
-   Anomaly Agent
-   Decision Assistant
-   Meeting Prep Agent
-   Management Report Agent

## PMO

-   Project Status Agent
-   Risk Agent
-   Action Tracker
-   Project Cost Analyst
-   Timeline Analyzer
-   Project Brief Agent

## Legal / Contracts

-   Contract Reader
-   Contract Comparison Agent
-   Obligation Extractor
-   Renewal Tracker
-   Clause Finder
-   Contract Policy Checker

**Important:** These are not 60 independent codebases.

They must be compositions of reusable skills.

------------------------------------------------------------------------

# 21. FIRST RELEASE AGENT SCOPE

Do not implement all 60 agents first.

Production Release 1:

## Finance

-   Bank Reconciliation Agent
-   Budget Variance Agent
-   Expense Analyst

## Accounting

-   Invoice Reader
-   Invoice Control Agent
-   Account Reconciliation Agent

## Procurement

-   Supplier Comparison Agent
-   Price History Agent
-   Procurement Decision Agent

Total: **9 production-quality agents**

This is the first vertical validation set.

------------------------------------------------------------------------

# 22. PAID BUSINESS AGENTS --- INITIAL

Paid agent difference:

``` text
Free:
Analyze → Recommend → User Executes

Paid:
Observe → Analyze → Recommend → Approve → Execute → Verify
```

Initial Paid agents:

## Sales/Revenue

-   Margin Guardian
-   Order Exception Agent
-   Discount Control Agent
-   Pricing Anomaly Agent

## Finance

-   Collection Agent
-   CashFlow Agent
-   Invoice Control Agent
-   Payment Risk Agent

## Procurement

-   Procurement Agent
-   Supplier Risk Agent
-   Purchase Price Guardian
-   RFQ Agent

## Inventory

-   Stockout Agent
-   Replenishment Agent
-   Slow Moving Agent
-   Inventory Anomaly Agent

------------------------------------------------------------------------

# 23. ERP ABSTRACTION

Business agents must not be written directly against
SAP/Logo/Netsis/Dynamics-specific objects.

Canonical model:

``` text
Customer
Supplier
Product
Inventory
PriceList
SalesOrder
SalesOrderLine
PurchaseOrder
PurchaseOrderLine
Invoice
Payment
Shipment
Account
Employee
Department
```

Flow:

``` text
ERP
→ Connector Adapter
→ Canonical Model
→ Agent Skill
```

------------------------------------------------------------------------

# 24. PAID PERMISSION LEVELS

``` text
L0 READ
L1 RECOMMEND
L2 DRAFT
L3 APPROVED_EXECUTE
L4 AUTONOMOUS
```

Example:

``` yaml
agent: margin_guardian

permissions:
  sales_order:
    read: true
    block:
      mode: approved_execute
      max_amount: 100000
  price:
    update: false
```

------------------------------------------------------------------------

# 25. AUDIT REQUIREMENTS

Every execution must be traceable.

Store:

``` text
execution_id
tenant_id?
user_id
agent_id
agent_version
workflow_id?
skill_ids
tool_calls
provider
model
prompt_version
input_classification
egress_manifest
retrieved_evidence
decision
approval
action
result
token_usage
cache_usage
estimated_cost
duration
timestamps
```

Never store raw API keys or secrets.

------------------------------------------------------------------------

# 26. OBSERVABILITY

Core metrics:

``` text
agent_execution_total
agent_execution_failed
agent_step_duration
tool_call_duration
llm_request_duration
llm_input_tokens
llm_output_tokens
llm_cache_read_tokens
llm_estimated_cost
rag_retrieval_duration
rag_result_count
workflow_retry
policy_denied
egress_blocked
```

Trace structure:

``` text
Execution
→ Agent
→ Skill
→ Retrieval
→ LLM
→ Tool
→ Connector
→ External System
```

------------------------------------------------------------------------

# 27. COST GUARD

Free:

-   show usage
-   show provider
-   show model
-   show tokens
-   show estimated cost
-   show cache saving

Paid adds:

-   tenant budget
-   department budget
-   agent budget
-   per-run limit
-   monthly warning
-   monthly hard stop

------------------------------------------------------------------------

# 28. PROMPT MANAGEMENT

Prompts must be versioned assets.

Schema:

``` text
prompt_id
version
agent_id
template
variables
provider_overrides
status
created_at
```

Execution audit stores prompt version.

------------------------------------------------------------------------

# 29. SECURITY REQUIREMENTS

Mandatory:

-   least privilege
-   secure secret storage
-   TLS
-   encrypted sensitive local cache
-   signed desktop updates
-   dependency scanning
-   parser sandboxing
-   path traversal prevention
-   archive bomb protection
-   max document limits
-   prompt injection defense
-   SSRF protection
-   outbound domain controls
-   tenant isolation
-   secret redaction
-   audit integrity

Retrieved documents are untrusted input.

A document must never be able to:

-   elevate permissions,
-   enable a tool,
-   override system policies,
-   disable egress rules,
-   reveal secrets.

------------------------------------------------------------------------

# 30. LOCAL DATABASE

Use SQLite for Free MVP unless an ADR finds a blocker.

Minimum schema:

``` text
settings
providers
provider_credentials_ref
agents
skills
workflows
workflow_runs
executions
documents
document_sections
document_chunks
document_metadata
knowledge_entities
rag_indexes
conversations
messages
decisions
audit_events
cache_entries
usage_events
```

Database migrations are mandatory from Phase 1.

------------------------------------------------------------------------

# 31. SERVER STORAGE --- LOGICAL REQUIREMENTS

Paid logical layers:

``` text
Relational DB
→ tenant/config/workflow/audit metadata

Distributed Cache
→ shared cache/rate limit/locks

Queue
→ events/execution

Object Storage
→ documents/blobs

Vector Store
→ embedding retrieval

Secrets Vault
→ connector/provider credentials
```

Exact technologies must be captured in ADRs.

------------------------------------------------------------------------

# 32. IDEMPOTENCY / RETRY

Paid ERP writes require idempotency.

Suggested key:

``` text
tenant_id
+ workflow_id
+ business_entity_id
+ action_type
+ business_version
```

Retry only transient failures:

-   timeout
-   429
-   temporary network
-   502/503
-   temporary connector failure

Do not retry:

-   validation errors
-   permission denied
-   policy denied
-   business rule rejection

Use exponential backoff + jitter.

Paid server must support DLQ.

------------------------------------------------------------------------

# 33. IMPLEMENTATION PHASES

The coding agent MUST execute these phases in order.

------------------------------------------------------------------------

## PHASE-00 --- Repository & Engineering Foundation

### Goal

Create a buildable, testable, documented project foundation.

### Tasks

#### TASK-0001 --- Workspace bootstrap

-   [ ] Create/validate Rust workspace
-   [ ] Create desktop app shell
-   [ ] Create core crates/modules
-   [ ] Add workspace-level lint/format config

**Acceptance** - `cargo check` passes - `cargo fmt --check` passes -
empty app launches

#### TASK-0002 --- Error model

-   [ ] Define typed application errors
-   [ ] Separate user-facing vs internal errors
-   [ ] Add stable error codes

**Acceptance** - no core library uses ad-hoc string-only errors

#### TASK-0003 --- Configuration

-   [ ] layered config
-   [ ] environment overrides
-   [ ] local user settings
-   [ ] safe defaults

#### TASK-0004 --- Local DB

-   [ ] SQLite integration
-   [ ] migration framework
-   [ ] base tables

**Acceptance** - fresh DB creation passes - migration repeat is safe

#### TASK-0005 --- Logging

-   [ ] structured logging
-   [ ] correlation/execution IDs
-   [ ] secret filtering

### Exit criteria

All tasks complete and all foundation tests pass.

------------------------------------------------------------------------

## PHASE-01 --- Secure Provider & LLM Gateway

### Goal

User can enter API keys and use multiple LLM providers through one
internal contract.

### Tasks

#### TASK-0101 --- SecureStore

-   [ ] OS-backed credential abstraction
-   [ ] save/read/delete credential
-   [ ] no plaintext DB storage

#### TASK-0102 --- Provider contract

-   [ ] request/response normalization
-   [ ] streaming abstraction
-   [ ] usage normalization
-   [ ] capability interface

#### TASK-0103 --- OpenAI adapter

-   [ ] credential validation
-   [ ] model discovery/config
-   [ ] completion
-   [ ] streaming
-   [ ] structured output where supported

#### TASK-0104 --- Anthropic adapter

Same acceptance contract as provider abstraction.

#### TASK-0105 --- Gemini adapter

Same acceptance contract as provider abstraction.

#### TASK-0106 --- OpenAI-compatible adapter

-   [ ] configurable base URL
-   [ ] configurable auth
-   [ ] manual/discovered model list

#### TASK-0107 --- Ollama/local adapter

-   [ ] local endpoint
-   [ ] no cloud egress classification
-   [ ] model listing

#### TASK-0108 --- Provider settings UI

-   [ ] add provider
-   [ ] test connection
-   [ ] choose default
-   [ ] remove credentials

#### TASK-0109 --- Router & fallback

-   [ ] manual mode
-   [ ] smart mode skeleton
-   [ ] privacy-first mode skeleton
-   [ ] infrastructure-only fallback

### Tests

-   mock provider contract tests
-   invalid key
-   timeout
-   rate limit
-   streaming cancellation
-   unsupported capability

### Exit criteria

At least OpenAI, Anthropic, Gemini, custom compatible endpoint and
Ollama function behind the same gateway.

------------------------------------------------------------------------

## PHASE-02 --- Cache Subsystem

### Goal

Avoid repeated parsing, embeddings and eligible LLM work.

### Tasks

#### TASK-0201 --- In-memory cache

-   [ ] TTL
-   [ ] bounded size
-   [ ] LRU/equivalent eviction

#### TASK-0202 --- Persistent cache

-   [ ] SQLite cache metadata
-   [ ] blob/file cache
-   [ ] encryption option

#### TASK-0203 --- Content hash

-   [ ] SHA-256 content identity
-   [ ] operation-version aware keys

#### TASK-0204 --- Parse cache

-   [ ] hit/miss
-   [ ] invalidation by parser version

#### TASK-0205 --- Embedding cache

-   [ ] key by content + model + chunk version

#### TASK-0206 --- Eligible LLM result cache

-   [ ] structured extraction/classification only
-   [ ] policy-controlled

#### TASK-0207 --- Provider-native prompt cache abstraction

-   [ ] capability detection
-   [ ] normalized cache metrics

#### TASK-0208 --- Usage metrics

-   [ ] hit ratio
-   [ ] token savings
-   [ ] estimated monetary savings

### Exit criteria

Repeated identical supported operations demonstrably use cache without
changing results.

------------------------------------------------------------------------

## PHASE-03 --- Local Tool Runtime

### Goal

Deterministic local work happens without asking an LLM to do what code
can do.

### Tasks

#### TASK-0301 --- Tool manifest/schema

#### TASK-0302 --- Filesystem scoped read

#### TASK-0303 --- Filesystem write + preview

#### TASK-0304 --- CSV engine

#### TASK-0305 --- XLSX read/write

#### TASK-0306 --- JSON/XML transformation

#### TASK-0307 --- PDF text/table parser interface

#### TASK-0308 --- DOCX parser interface

#### TASK-0309 --- Tool risk classification

#### TASK-0310 --- Tool execution audit

### Security acceptance

-   path traversal tests
-   denied path access
-   destructive action requires approval
-   tool cannot bypass runtime permission checks

------------------------------------------------------------------------

## PHASE-04 --- Data Egress Guard

### Goal

No cloud LLM call bypasses privacy controls.

### Tasks

#### TASK-0401 --- Data classification L0-L4

#### TASK-0402 --- Secret detection

#### TASK-0403 --- PII detection abstraction

#### TASK-0404 --- Redaction/tokenization

#### TASK-0405 --- Egress manifest

#### TASK-0406 --- User approval flow

#### TASK-0407 --- Provider-specific policy

#### TASK-0408 --- Egress audit

### Acceptance

A test agent attempting to directly send raw file content to a provider
is blocked unless the policy permits it.

------------------------------------------------------------------------

## PHASE-05 --- Agent Runtime & Skill SDK

### Goal

Agents are declarative compositions of reusable skills and tools.

### Tasks

#### TASK-0501 --- Agent manifest schema

#### TASK-0502 --- Skill manifest schema

#### TASK-0503 --- Skill registry

#### TASK-0504 --- Agent registry

#### TASK-0505 --- Execution state machine

#### TASK-0506 --- Cancellation/timeout

#### TASK-0507 --- Max-step guard

#### TASK-0508 --- Structured output validator

#### TASK-0509 --- Orchestrator

#### TASK-0510 --- Entitlement check

### Acceptance

A sample agent can: - receive task, - plan, - call deterministic
tools, - call LLM through gateway, - validate output, - return result, -
persist audit.

------------------------------------------------------------------------

## PHASE-06 --- Advanced Local RAG

### Goal

Users can upload documents and obtain source-grounded analysis/decision
support.

### Tasks

#### TASK-0601 --- Ingestion pipeline

#### TASK-0602 --- Normalized document model

#### TASK-0603 --- Structure-aware chunker

#### TASK-0604 --- Spreadsheet structural indexer

#### TASK-0605 --- Embedding abstraction

#### TASK-0606 --- Local vector index adapter

#### TASK-0607 --- Keyword/BM25 retrieval adapter

#### TASK-0608 --- Metadata filtering

#### TASK-0609 --- Retrieval merger

#### TASK-0610 --- Reranker interface

#### TASK-0611 --- Citation/evidence model

#### TASK-0612 --- Context budget manager

#### TASK-0613 --- Prompt-injection defense boundary

### Acceptance

For a provided document, answer includes correct evidence source pointer
and does not invent unsupported facts.

------------------------------------------------------------------------

## PHASE-07 --- Decision Intelligence

### Goal

Move from document Q&A to evidence-backed business analysis.

### Tasks

#### TASK-0701 --- Decision type registry

#### TASK-0702 --- Required-facts schema

#### TASK-0703 --- Decision Context Builder

#### TASK-0704 --- Missing-information detector

#### TASK-0705 --- Evidence validator

#### TASK-0706 --- Recommendation structured schema

#### TASK-0707 --- Confidence representation

#### TASK-0708 --- Local Decision Memory

#### TASK-0709 --- Past-decision retrieval

### Acceptance

Procurement example can combine: - vendor quotes, - historical prices, -
user policy document, - missing data, and produce evidence-backed
recommendation.

------------------------------------------------------------------------

## PHASE-08 --- First 9 Department Agents

### Goal

Ship high-quality use cases instead of 60 shallow agents.

### Finance

#### TASK-0801 --- Bank Reconciliation Agent

Inputs: - bank statement XLSX/CSV - ledger/current account XLSX/CSV

Output: - matched records - unmatched records - likely matches -
reconciliation report

#### TASK-0802 --- Budget Variance Agent

Output: - budget/actual variance - material deviations - explanations
where evidence exists

#### TASK-0803 --- Expense Analyst

Output: - category analysis - unusual increases - duplicate/possible
anomalies - report

### Accounting

#### TASK-0810 --- Invoice Reader

#### TASK-0811 --- Invoice Control Agent

#### TASK-0812 --- Account Reconciliation Agent

### Procurement

#### TASK-0820 --- Supplier Comparison Agent

#### TASK-0821 --- Price History Agent

#### TASK-0822 --- Procurement Decision Agent

### Acceptance for every agent

-   manifest exists
-   uses reusable skills
-   does not duplicate tool code
-   sample fixtures
-   unit/integration tests
-   failure behavior documented
-   egress classification defined
-   expected outputs schema-defined

------------------------------------------------------------------------

## PHASE-09 --- Local Workflow Engine

### Goal

"Do this again" becomes a reusable workflow.

### Tasks

#### TASK-0901 --- Workflow schema

#### TASK-0902 --- Manual workflow runner

#### TASK-0903 --- Parameter binding

#### TASK-0904 --- Workflow versioning

#### TASK-0905 --- Save from completed execution

#### TASK-0906 --- Replay

#### TASK-0907 --- Execution history

#### TASK-0908 --- Rollback metadata for supported tools

### Free boundary

No cron, webhook or server-triggered automation.

------------------------------------------------------------------------

## PHASE-10 --- Free MVP Hardening

### Goal

Production-ready Free Desktop release.

### Tasks

#### TASK-1001 --- Crash recovery

#### TASK-1002 --- execution resume policy

#### TASK-1003 --- provider failure UX

#### TASK-1004 --- cache management UX

#### TASK-1005 --- knowledge workspace UX

#### TASK-1006 --- usage/cost dashboard

#### TASK-1007 --- local audit UX

#### TASK-1008 --- signed update design

#### TASK-1009 --- security tests

#### TASK-1010 --- packaging

### Free MVP Definition of Done

-   [ ] 5 provider types behind one gateway
-   [ ] secure BYOK
-   [ ] local deterministic tool runtime
-   [ ] local cache
-   [ ] provider-native cache abstraction
-   [ ] Egress Guard
-   [ ] Advanced local RAG
-   [ ] evidence/citations
-   [ ] Decision Context Builder
-   [ ] 9 production-quality agents
-   [ ] workflow save/replay
-   [ ] usage/cost visibility
-   [ ] structured output
-   [ ] local audit
-   [ ] security tests
-   [ ] no secret business payload in telemetry

------------------------------------------------------------------------

## PHASE-11 --- Paid Server Foundation

Do not start before Free runtime boundaries are stable.

### Tasks

#### TASK-1101 --- Server app bootstrap

#### TASK-1102 --- tenant model

#### TASK-1103 --- identity/auth

#### TASK-1104 --- RBAC

#### TASK-1105 --- policy service

#### TASK-1106 --- entitlement service

#### TASK-1107 --- central secrets abstraction

#### TASK-1108 --- central audit

#### TASK-1109 --- distributed cache

#### TASK-1110 --- queue abstraction

#### TASK-1111 --- worker runtime

#### TASK-1112 --- health/readiness

------------------------------------------------------------------------

## PHASE-12 --- Paid Workflow & Approval

### Tasks

#### TASK-1201 --- scheduler

#### TASK-1202 --- event trigger

#### TASK-1203 --- webhook trigger

#### TASK-1204 --- retry policy

#### TASK-1205 --- DLQ

#### TASK-1206 --- long-running state

#### TASK-1207 --- Approval Engine

#### TASK-1208 --- pause/resume

#### TASK-1209 --- idempotency

#### TASK-1210 --- compensation hooks

------------------------------------------------------------------------

## PHASE-13 --- Connector Platform

### Tasks

#### TASK-1301 --- Connector SDK

#### TASK-1302 --- connection health

#### TASK-1303 --- capability manifest

#### TASK-1304 --- canonical business model

#### TASK-1305 --- read connector example

#### TASK-1306 --- write connector example

#### TASK-1307 --- ERP action audit

#### TASK-1308 --- connector secret isolation

------------------------------------------------------------------------

## PHASE-14 --- First Paid Business Agents

### TASK-1401 --- Margin Guardian

Must detect: - negative/low margin - unusual discount - stale pricing -
cost drift

### TASK-1402 --- Order Exception Agent

Must detect: - customer risk - limit issues - overdue balance - stock
exception - pricing exception - unusual quantity

Default execution:

``` text
Observe
→ Analyze
→ Recommend
→ Approval
→ Execute
→ Verify
```

No autonomous ERP write in initial production configuration.

------------------------------------------------------------------------

## PHASE-15 --- Company Knowledge Fabric

### Tasks

#### TASK-1501 --- shared knowledge scopes

#### TASK-1502 --- ACL-aware retrieval

#### TASK-1503 --- company document ingestion

#### TASK-1504 --- server hybrid retrieval

#### TASK-1505 --- company entity graph

#### TASK-1506 --- permission-aware Decision Memory

#### TASK-1507 --- tenant-isolated indexes

#### TASK-1508 --- retention policy

------------------------------------------------------------------------

# 34. REQUIRED ADRs

Create ADRs as implementation begins.

Minimum:

-   ADR-001 Desktop UI framework
-   ADR-002 SQLite/migration library
-   ADR-003 Async/runtime strategy
-   ADR-004 LLM provider contract
-   ADR-005 HTTP client/retry policy
-   ADR-006 Secure credential storage
-   ADR-007 Cache implementation
-   ADR-008 Local vector index
-   ADR-009 BM25/full-text implementation
-   ADR-010 Document parsing stack
-   ADR-011 Workflow state machine
-   ADR-012 Server relational DB
-   ADR-013 Distributed cache
-   ADR-014 Queue/event technology
-   ADR-015 Secrets backend
-   ADR-016 Connector protocol

ADR format:

``` md
# ADR-XXX Title

## Status
Accepted

## Context

## Decision

## Alternatives

## Consequences
```

------------------------------------------------------------------------

# 35. REQUIRED SCHEMAS

The coding agent must create machine-readable schemas for:

-   Agent Manifest
-   Skill Manifest
-   Tool Manifest
-   Workflow Definition
-   Capability Manifest
-   Egress Policy
-   Structured Agent Result
-   Evidence
-   Decision Context
-   Provider Configuration

Store in:

`packages/schemas/`

------------------------------------------------------------------------

# 36. TEST STRATEGY

Every phase needs automated tests.

Required layers:

## Unit

Pure logic.

## Contract

Provider/tool/skill contracts.

## Integration

SQLite, parser, cache, RAG pipeline.

## Security

-   path traversal
-   secret leakage
-   egress bypass
-   prompt injection
-   entitlement bypass
-   tenant isolation on Paid

## Golden datasets

Use fixed fixtures for: - reconciliation - invoice parsing - supplier
comparison - RAG citations - decision recommendation

LLM-dependent tests should separate: - deterministic contract tests with
mocks, - optional live provider smoke tests.

CI must not require personal provider API keys for normal test
execution.

------------------------------------------------------------------------

# 37. CODE QUALITY GATES

Before completing any phase:

``` text
format
lint
compile/check
unit tests
integration tests for changed modules
security tests where relevant
documentation/status update
```

No ignored failing tests.

No secrets in repo.

No TODO used as a substitute for acceptance criteria.

------------------------------------------------------------------------

# 38. FIRST CODING COMMAND FOR AN AGENT

When an implementation agent starts from this file, its first action
should be:

``` text
1. Inspect repository.
2. Create docs/IMPLEMENTATION_STATUS.md.
3. Map current repository to this specification.
4. Mark already-completed requirements only after verifying code/tests.
5. Start PHASE-00.
6. Complete tasks sequentially.
7. Do not begin PHASE-01 until PHASE-00 exit criteria pass.
```

------------------------------------------------------------------------

# 39. FINAL PRODUCT PRINCIPLES

1.  Free/Paid boundary is execution scope and governance, not model
    quality.
2.  User can bring OpenAI/Claude/Gemini-compatible API credentials in
    Free.
3.  Provider-specific SDK logic stays behind adapters.
4.  Deterministic work is done by tools, not wasted LLM calls.
5.  Business data is local-first.
6.  Every cloud payload goes through Data Egress Guard.
7.  Every business side effect is structured and validated.
8.  RAG is hybrid, source-aware and decision-oriented.
9.  Cache is layered, version-aware and privacy-aware.
10. Department agents are reusable skill compositions.
11. Start with 9 high-quality Free agents, not 60 shallow ones.
12. Paid server is where ERP connectivity, scheduling, shared knowledge,
    approvals and autonomy live.
13. ERP writes are auditable, idempotent and policy-controlled.
14. Security controls are part of runtime architecture, not optional UI
    behavior.
15. The coding agent must always prefer a working, tested vertical slice
    over premature breadth.

------------------------------------------------------------------------

# 40. ENGINEERING PRINCIPLES --- MANDATORY

These rules are architectural constraints, not style preferences.

## 40.1 Clean Architecture

The system MUST preserve dependency direction.

``` text
Frameworks / Drivers
        ↓
Adapters
        ↓
Application / Use Cases
        ↓
Domain
```

The Domain layer must not depend on:

-   HTTP frameworks,
-   database libraries,
-   desktop UI frameworks,
-   concrete LLM SDKs,
-   concrete cache implementations,
-   operating-system credential APIs,
-   PostgreSQL drivers,
-   external provider SDKs.

Infrastructure must implement ports/contracts defined by inner layers.

### Required dependency rule

``` text
Domain
  ↑
Application
  ↑
Ports
  ↑
Adapters / Infrastructure
```

Never import infrastructure-specific types into domain entities or use
cases.

### Examples

Correct:

``` text
RegisterUserUseCase
    ↓
UserRepository interface
    ↓
PostgresUserRepository
```

Incorrect:

``` text
RegisterUserUseCase
    ↓
sqlx::PgPool / gorm.DB / concrete HTTP client
```

## 40.2 SOLID

### Single Responsibility Principle

A module must have one primary reason to change.

Examples:

-   `llm-gateway` routes LLM calls.
-   `provider-openai` implements OpenAI-specific protocol.
-   `egress-guard` evaluates outbound data.
-   `cache-core` defines caching abstractions.
-   `spreadsheet-engine` performs deterministic spreadsheet operations.

Do not create "god services" such as `AgentService` containing provider
calls, DB access, parsing, cache, policy and workflow logic together.

### Open/Closed Principle

New providers, skills, tools and connectors should be added by
implementing contracts rather than modifying central switch statements.

Avoid:

``` text
if provider == openai ...
else if provider == anthropic ...
else if provider == gemini ...
```

Prefer registry/adapter dispatch.

### Liskov Substitution Principle

All implementations of common interfaces must preserve documented
behavior, error semantics and capability declarations.

### Interface Segregation Principle

Prefer focused ports:

``` text
CompletionProvider
EmbeddingProvider
ModelDiscoveryProvider
CredentialValidator
```

over one oversized interface if provider capabilities materially differ.

### Dependency Inversion Principle

Business use cases depend on abstractions.

External systems depend on those abstractions through adapters.

## 40.3 DRY

Do not duplicate:

-   provider error normalization,
-   HTTP retry logic,
-   cache key construction,
-   audit event structures,
-   entitlement checks,
-   permission checks,
-   egress checks,
-   document metadata models,
-   pagination primitives,
-   validation rules,
-   API response envelopes.

However, do not over-generalize unrelated business behavior just to
remove a few repeated lines.

DRY applies to **knowledge duplication**, not merely textual
duplication.

## 40.4 YAGNI

Do not implement capabilities that are not required by the current
phase.

Examples:

-   Do not build a plugin marketplace in MVP.
-   Do not build Kubernetes operators before server deployment requires
    them.
-   Do not support ten databases before the first PostgreSQL-backed API
    is stable.
-   Do not build generic BPMN before the workflow requirements need it.
-   Do not add complex CQRS/event sourcing unless a concrete requirement
    justifies it.

When uncertain, implement the smallest architecture that keeps a clean
extension point.

## 40.5 KISS

Prefer simple, explicit implementations.

Avoid hidden magic, reflection-heavy dependency discovery and overly
generic meta-frameworks.

## 40.6 Composition over inheritance

Agents are compositions of skills.

Skills are compositions of deterministic tools and controlled LLM calls.

Avoid deep inheritance trees.

## 40.7 Explicit boundaries

No layer may bypass:

-   Entitlement
-   Permissions
-   Policy
-   Egress Guard
-   Structured Output Validation
-   Audit

------------------------------------------------------------------------

# 41. APPLICATION TOPOLOGY --- FINAL MVP DECISION

The product has three distinct runtime surfaces.

``` text
┌──────────────────────────────────────────────┐
│              FREE DESKTOP APP                │
│                                              │
│ UI                                           │
│  ↓                                           │
│ Rust Agent Backend / Local Runtime           │
│  ├─ Agent Runtime                            │
│  ├─ Skills / Tools                           │
│  ├─ RAG                                      │
│  ├─ Cache                                    │
│  ├─ SQLite                                   │
│  ├─ Secure Store                             │
│  └─ LLM Gateway                              │
└──────────────────┬───────────────────────────┘
                   │ HTTPS
                   │ only central-account/product APIs
                   ▼
┌──────────────────────────────────────────────┐
│                 GO CONTROL API               │
│                                              │
│ Auth                                         │
│ Account                                      │
│ Device / Session                             │
│ Agent Suggestions                            │
│ Error / Feedback Intake                      │
│ Release / Remote Config metadata             │
│ Admin APIs                                   │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
              PostgreSQL
```

Paid Company Server is a separate runtime surface and may reuse Rust
business-agent libraries where practical.

## 41.1 Mandatory language split

### Rust owns

All local agent execution logic:

-   orchestrator,
-   agent runtime,
-   skill runtime,
-   tool runtime,
-   local workflow,
-   RAG,
-   document parsing,
-   spreadsheet operations,
-   cache,
-   Data Egress Guard,
-   local audit,
-   provider gateway,
-   local persistence.

### Go owns

The central control-plane API for the Free product:

-   login,
-   registration,
-   logout/session lifecycle,
-   refresh token handling,
-   account profile,
-   optional device registration,
-   agent suggestion intake,
-   error report intake,
-   feedback intake,
-   release metadata,
-   remote feature/config metadata,
-   admin APIs.

Go MUST NOT become a proxy for normal Free agent execution.

### PostgreSQL owns

Central account/product operational data only.

It must not silently become storage for the user's business documents or
local RAG corpus.

------------------------------------------------------------------------

# 42. FREE DESKTOP NETWORK BOUNDARY

The Free client may communicate externally with only explicit
categories.

## 42.1 Control API traffic

Allowed central API use cases:

-   Register
-   Login
-   Refresh session
-   Logout
-   Account/profile
-   Agent suggestion
-   Error report
-   General feedback
-   Check application release/update metadata
-   Remote configuration / feature metadata
-   Optional anonymous product telemetry only if explicitly enabled

## 42.2 LLM provider traffic

User-configured provider traffic goes directly through the Rust LLM
Gateway and Data Egress Guard.

It is not required to pass through the Go API.

``` text
Rust Desktop
   ↓
Data Egress Guard
   ↓
OpenAI / Anthropic / Gemini / Custom / Ollama
```

This keeps BYOK credentials out of the central platform.

## 42.3 Local execution availability

The Free desktop MUST remain useful when the Go API is unavailable.

After a valid local session/bootstrap state exists, these capabilities
should remain functional offline where technically possible:

-   local spreadsheet operations,
-   local file operations,
-   local RAG,
-   local workflows,
-   Ollama/local model usage,
-   local execution history.

Account-dependent functionality may degrade gracefully.

------------------------------------------------------------------------

# 43. GO CONTROL API --- CLEAN ARCHITECTURE

Recommended logical layout:

``` text
/apps/control-api
├── cmd/
│   └── api/
├── internal/
│   ├── domain/
│   │   ├── user/
│   │   ├── session/
│   │   ├── feedback/
│   │   ├── suggestion/
│   │   ├── errorreport/
│   │   ├── release/
│   │   └── admin/
│   ├── application/
│   │   ├── auth/
│   │   ├── account/
│   │   ├── feedback/
│   │   ├── suggestion/
│   │   ├── errorreport/
│   │   └── admin/
│   ├── ports/
│   ├── adapters/
│   │   ├── postgres/
│   │   ├── http/
│   │   ├── email/
│   │   └── objectstore/
│   └── platform/
│       ├── config/
│       ├── logging/
│       ├── auth/
│       └── observability/
├── migrations/
└── tests/
```

Framework choice is secondary to the dependency rule.

The coding agent must capture the selected HTTP router, DB access
library and migration library in ADRs.

------------------------------------------------------------------------

# 44. GO API --- AUTHENTICATION FEATURES

## 44.1 Registration

Required fields:

-   email
-   password
-   display name or optional name
-   terms/privacy acceptance version
-   application version
-   locale

Optional later:

-   company name
-   department
-   referral source

Registration flow:

``` text
POST /v1/auth/register
→ validate
→ normalize email
→ verify uniqueness
→ hash password
→ create user
→ create verification state
→ issue verification email/token if enabled
→ return safe account/session response
```

Password storage:

-   Argon2id preferred, or equivalent modern password hashing algorithm.
-   Never encrypt passwords reversibly.
-   Never log passwords.

## 44.2 Login

``` text
POST /v1/auth/login
```

Requirements:

-   email/password
-   generic authentication error
-   brute-force/rate-limit protection
-   security audit event
-   access token
-   refresh token

## 44.3 Sessions

Use short-lived access tokens and rotatable refresh tokens.

Refresh tokens should be persisted as secure hashes, not raw values.

Required operations:

``` text
POST /v1/auth/refresh
POST /v1/auth/logout
POST /v1/auth/logout-all
GET  /v1/account/sessions
DELETE /v1/account/sessions/{sessionId}
```

## 44.4 Email verification

Recommended even for Free accounts.

Endpoints:

``` text
POST /v1/auth/verify-email
POST /v1/auth/resend-verification
```

## 44.5 Password reset

``` text
POST /v1/auth/forgot-password
POST /v1/auth/reset-password
```

Tokens:

-   one-time use,
-   short TTL,
-   hash at rest,
-   invalidate after use.

------------------------------------------------------------------------

# 45. FREE USER PRODUCT API

The central API intentionally remains small.

## 45.1 Account

``` text
GET   /v1/account
PATCH /v1/account
DELETE /v1/account
```

Profile may include:

-   display name
-   locale
-   selected department
-   product preferences
-   onboarding status

Do not sync local document paths or document content.

## 45.2 Agent Suggestion

Users can suggest a new agent/use case from the desktop.

Desktop UX:

``` text
Ajan Öner

Departman
[ Satın Alma ]

Ajan adı / fikir
[ Tedarikçi sözleşme risk ajanı ]

Ne çözmeli?
[ ... ]

Örnek dosyalar/veriler
[ Optional textual description only ]

[ Gönder ]
```

API:

``` text
POST /v1/suggestions/agents
GET  /v1/account/suggestions
```

Data:

``` text
id
user_id
department
title
problem_description
expected_output
status
admin_note?
created_at
updated_at
```

Default submission MUST NOT upload the user's business files.

## 45.3 Error Report

Desktop option:

``` text
Hata Bildir
```

The user should be able to review what will be sent.

Default payload:

-   app version
-   OS
-   architecture
-   anonymous/local execution error code
-   stack trace if safe
-   component
-   timestamp
-   provider name if relevant
-   model ID if relevant
-   correlation ID
-   user-written description

Must not automatically include:

-   API keys
-   document contents
-   spreadsheet rows
-   RAG evidence contents
-   full prompts containing business data
-   local filesystem paths unless sanitized

API:

``` text
POST /v1/error-reports
```

Optional attachments require explicit opt-in and redaction.

## 45.4 Feedback

``` text
POST /v1/feedback
```

Types:

-   bug
-   feature request
-   usability
-   agent quality
-   general

## 45.5 Release metadata

``` text
GET /v1/releases/latest?platform=macos&arch=arm64&channel=stable
```

Response should provide metadata and signed artifact information, not
arbitrary executable instructions.

------------------------------------------------------------------------

# 46. POSTGRESQL --- CONTROL PLANE SCHEMA

Minimum central tables:

``` text
users
user_profiles
email_verification_tokens
password_reset_tokens
sessions
devices
agent_suggestions
error_reports
feedback
release_channels
app_releases
remote_config
feature_flags
admin_users
admin_roles
admin_permissions
admin_role_permissions
admin_user_roles
audit_events
```

## 46.1 Important boundaries

DO NOT store in central PostgreSQL by default:

-   uploaded business documents,
-   local RAG chunks,
-   local embeddings,
-   spreadsheet content,
-   provider API keys,
-   full local chat history,
-   arbitrary file paths,
-   local workflow business payloads.

The local Free database remains SQLite.

------------------------------------------------------------------------

# 47. API RESPONSE AND ERROR CONTRACT

Use a stable API envelope.

Example success:

``` json
{
  "data": {
    "id": "..."
  },
  "meta": {
    "request_id": "..."
  }
}
```

Example error:

``` json
{
  "error": {
    "code": "AUTH_INVALID_CREDENTIALS",
    "message": "Invalid email or password.",
    "fields": null
  },
  "meta": {
    "request_id": "..."
  }
}
```

Requirements:

-   machine-readable stable error codes,
-   user-safe messages,
-   no internal stack trace in public response,
-   request ID on every request.

------------------------------------------------------------------------

# 48. API SECURITY BASELINE

Mandatory for Go API:

-   TLS-only production traffic
-   password hashing
-   access/refresh token separation
-   refresh token rotation
-   hashed refresh token storage
-   login rate limits
-   register rate limits
-   password reset rate limits
-   email enumeration resistance
-   input size limits
-   strict JSON decoding
-   SQL parameterization
-   CORS policy if browser surfaces are added
-   CSRF protection where cookie auth is used
-   security headers for Admin Web
-   IP/user rate limiting
-   admin MFA-ready architecture
-   immutable/security audit stream
-   secret management via environment/KMS/Vault abstraction
-   database migrations
-   backup/restore procedure
-   dependency vulnerability scanning

------------------------------------------------------------------------

# 49. ADMIN PANEL --- PRODUCT OPERATIONS

The Admin panel is an internal operations console, not a customer-facing
Company Workspace.

Its purpose is to operate the Free product efficiently.

Recommended navigation:

``` text
Dashboard
Users
Sessions / Devices
Agent Suggestions
Error Reports
Feedback
Agent Catalog
Feature Flags
Remote Config
Releases
Provider Health Metadata
Usage Analytics
Security
Audit
System
```

## 49.1 Dashboard

Useful cards:

-   total registered users
-   daily active users
-   weekly active users
-   new registrations
-   verified users
-   active desktop versions
-   OS/platform distribution
-   agent executions reported as aggregate metrics
-   top agent types
-   top departments
-   error rate
-   crash/error reports
-   pending agent suggestions
-   unresolved feedback
-   provider error-rate aggregates
-   Free → Paid interest signals

Do not ingest user business content to build these metrics.

## 49.2 User Management

Admin can:

-   search user by ID/email
-   view safe profile
-   view account state
-   see verification status
-   see registration date
-   see last account API activity
-   see app versions/devices
-   disable account
-   re-enable account
-   revoke sessions
-   trigger password reset workflow
-   view user-submitted suggestions/feedback/errors

Admin must never see provider API keys.

## 49.3 Agent Suggestion Management

Admin board:

``` text
NEW
REVIEWING
PLANNED
IN_PROGRESS
SHIPPED
REJECTED
```

Features:

-   filter by department
-   duplicate detection
-   vote/count similar requests
-   admin notes
-   priority
-   link suggestion to roadmap item
-   mark shipped version
-   reply/status surfaced to user later

This becomes a direct product discovery channel.

## 49.4 Error Report Center

Features:

-   group by error fingerprint
-   app version
-   OS
-   architecture
-   component
-   provider
-   model
-   first seen
-   last seen
-   occurrence count
-   affected user count
-   severity
-   status
-   assignment
-   internal note

Statuses:

``` text
NEW
TRIAGED
IN_PROGRESS
RESOLVED
IGNORED
```

Important:

Error aggregation should use safe fingerprinted technical metadata.

## 49.5 Feedback Center

Allow tagging:

-   UX
-   bug
-   model quality
-   agent request
-   performance
-   privacy
-   billing/future-paid
-   other

## 49.6 Agent Catalog Management

Admin may manage product metadata, not executable arbitrary code in MVP.

Fields:

``` text
agent_id
name
department
description
icon
tier
status
minimum_app_version
feature_flag
documentation_url?
release_notes?
```

Executable agent packages should remain signed/versioned release
artifacts.

## 49.7 Feature Flags

Examples:

-   enable new onboarding
-   enable beta agent
-   enable local RAG v2
-   enable smart routing
-   enable experimental provider adapter
-   hide problematic feature

Feature flags must not be used to bypass Free/Paid entitlement security.

## 49.8 Remote Config

Safe operational config examples:

-   support URL
-   max default error attachment size
-   feature announcement
-   minimum app version
-   recommended app version
-   maintenance message
-   provider capability metadata override

Remote config must be schema-validated and signed/versioned where
appropriate.

It must never remotely enable unrestricted file upload or bypass Egress
Guard.

## 49.9 Release Management

Admin can manage:

-   stable/beta channels
-   version
-   platform/architecture
-   minimum supported version
-   mandatory security update flag
-   release notes
-   artifact hash/signature metadata
-   staged rollout percentage
-   rollback marker

## 49.10 Provider Health Metadata

The central API should not proxy customer LLM traffic.

Admin can still track aggregate, opt-in safe metadata:

-   provider adapter failures
-   connection-validation failures
-   API compatibility issues
-   model discovery errors
-   app-version-specific adapter problems

Never collect API keys.

## 49.11 Product Analytics

Useful privacy-preserving events:

-   onboarding completed
-   department selected
-   agent started
-   agent completed
-   agent failed
-   workflow saved
-   workflow replayed
-   knowledge workspace created
-   RAG query completed
-   suggestion submitted
-   upgrade-interest clicked

Payload must exclude business content.

## 49.12 Security Admin

Admin features:

-   admin users
-   admin roles
-   permission matrix
-   admin session revoke
-   audit history
-   suspicious login summary
-   rate-limit events
-   blocked IP/account indicators

High-risk admin operations require step-up/MFA once MFA is implemented.

------------------------------------------------------------------------

# 50. ADMIN RBAC

Recommended roles:

## Super Admin

Full platform control.

## Product Admin

Agent catalog, suggestions, feature flags, release metadata.

## Support Admin

Users, sessions, error reports, feedback.

## Security Admin

Security events, session revoke, admin access controls.

## Read Only

Dashboard and reports.

Permissions must be capability-based, e.g.:

``` text
admin.users.read
admin.users.disable
admin.sessions.revoke
admin.suggestions.manage
admin.errors.manage
admin.feature_flags.write
admin.releases.publish
admin.audit.read
```

Do not gate admin access with frontend-only role checks.

------------------------------------------------------------------------

# 51. ADMIN AUDIT

Every privileged admin operation must record:

``` text
admin_user_id
permission
action
target_type
target_id
before_summary
after_summary
request_id
ip
user_agent
created_at
```

Sensitive values must be redacted.

------------------------------------------------------------------------

# 52. UPDATED MONOREPO STRUCTURE

Target structure becomes:

``` text
/
├── apps/
│   ├── desktop/                 # Desktop UI
│   ├── control-api/             # Go API
│   ├── admin-web/               # Internal Admin UI
│   └── company-server/          # Paid server, later phase
├── crates/                      # Rust local agent/runtime libraries
│   ├── agent-runtime/
│   ├── orchestrator/
│   ├── skill-sdk/
│   ├── tool-runtime/
│   ├── workflow-engine/
│   ├── llm-gateway/
│   ├── provider-openai/
│   ├── provider-anthropic/
│   ├── provider-gemini/
│   ├── provider-openai-compatible/
│   ├── provider-ollama/
│   ├── egress-guard/
│   ├── rag-core/
│   ├── cache-core/
│   ├── secure-store/
│   ├── audit-core/
│   └── common/
├── go/
│   └── control-api/
│       ├── cmd/
│       ├── internal/
│       ├── migrations/
│       └── tests/
├── packages/
│   ├── schemas/
│   ├── agent-packs/
│   └── skill-packs/
├── docs/
│   ├── adr/
│   ├── architecture/
│   ├── api/
│   └── IMPLEMENTATION_STATUS.md
└── tests/
```

If a more idiomatic Go workspace layout is chosen, document it in ADR
and keep the architectural boundaries.

------------------------------------------------------------------------

# 53. NEW IMPLEMENTATION PHASES --- CONTROL PLANE

Insert these phases after the Rust foundation and before broad product
expansion.

## PHASE-01A --- Go Control API Foundation

### Goal

Create the smallest secure central API required by Free Desktop.

### TASK-01A01 --- Go module and Clean Architecture skeleton

Acceptance: - build passes, - domain has no DB/HTTP dependency, -
dependency direction tests/review pass.

### TASK-01A02 --- PostgreSQL

-   connection
-   migrations
-   transaction abstraction
-   health checks

### TASK-01A03 --- API baseline

-   router
-   middleware
-   request IDs
-   structured logging
-   stable response envelope
-   validation
-   panic recovery

### TASK-01A04 --- Auth domain

-   User
-   Session
-   verification token
-   password reset token

### TASK-01A05 --- Register

### TASK-01A06 --- Login

### TASK-01A07 --- Refresh token rotation

### TASK-01A08 --- Logout / logout all

### TASK-01A09 --- Verify email

### TASK-01A10 --- Forgot/reset password

### Exit criteria

Desktop can register/login securely against a local/staging Go API
backed by PostgreSQL.

------------------------------------------------------------------------

## PHASE-01B --- Desktop Account Integration

### Goal

Free Desktop communicates with Go only for central product/account
functions.

Tasks:

-   login screen
-   register screen
-   forgot password
-   account session persistence
-   refresh handling
-   logout
-   offline/degraded account state
-   account page

Acceptance: Local agent runtime remains decoupled from HTTP auth client.

------------------------------------------------------------------------

## PHASE-01C --- Suggestions, Error Reports, Feedback

### TASK-01C01 --- Agent suggestion API

### TASK-01C02 --- Agent suggestion UI

### TASK-01C03 --- Error report sanitizer

### TASK-01C04 --- Error report preview UI

### TASK-01C05 --- Error report API

### TASK-01C06 --- Feedback API/UI

Security acceptance: A fixture containing API keys, local business data
and private file paths is sanitized before submission.

------------------------------------------------------------------------

## PHASE-01D --- Admin Foundation

### Goal

Provide an internal operational console from day one.

Tasks:

-   admin auth boundary
-   admin RBAC
-   dashboard
-   users
-   sessions/devices
-   agent suggestions
-   error reports
-   feedback
-   audit

Exit criteria: Support/product staff can operate the Free release
without direct DB access.

------------------------------------------------------------------------

## PHASE-01E --- Release & Remote Operations

Tasks:

-   feature flags
-   remote config
-   release metadata
-   stable/beta channels
-   minimum version
-   staged rollout metadata
-   admin screens
-   audit

Exit criteria: Desktop release behavior can be safely managed without
shipping configuration secrets or bypassing security boundaries.

------------------------------------------------------------------------

# 54. CONTROL API DEFINITION OF DONE

Before the Free public release:

-   [ ] Go API follows Clean Architecture
-   [ ] SOLID/DRY/YAGNI rules are documented and code-reviewed
-   [ ] PostgreSQL migrations are reproducible
-   [ ] Register/login works
-   [ ] email verification path exists
-   [ ] forgot/reset password works
-   [ ] refresh token rotation works
-   [ ] session revoke works
-   [ ] rate limiting works
-   [ ] account delete workflow exists
-   [ ] agent suggestion works
-   [ ] error report preview/sanitization works
-   [ ] feedback works
-   [ ] admin RBAC works
-   [ ] admin audit works
-   [ ] feature flags work
-   [ ] release metadata works
-   [ ] local business documents are not stored in central DB
-   [ ] BYOK keys never enter central Go API
-   [ ] Rust local runtime still works independently of central business
    logic

------------------------------------------------------------------------

# 55. ADDITIONAL REQUIRED ADRs

Add:

-   ADR-017 Go HTTP Router
-   ADR-018 Go PostgreSQL Access Layer
-   ADR-019 Database Migration Strategy
-   ADR-020 Auth Token Strategy
-   ADR-021 Password Hashing
-   ADR-022 Email Delivery Provider
-   ADR-023 Admin Frontend Stack
-   ADR-024 Feature Flag Strategy
-   ADR-025 Release Metadata / Desktop Update Trust
-   ADR-026 Product Telemetry Privacy Model

------------------------------------------------------------------------

# 56. CODING AGENT --- ARCHITECTURE ENFORCEMENT CHECKLIST

Before completing any feature, verify:

``` text
[ ] Is the domain independent of frameworks?
[ ] Is this responsibility in the correct service/language?
[ ] Did I reuse an existing abstraction instead of duplicate business knowledge?
[ ] Did I avoid creating an abstraction that the current requirement does not need?
[ ] Is the smallest viable solution implemented?
[ ] Is Free/Paid enforced outside the UI?
[ ] Can central API failure break local agent execution unnecessarily?
[ ] Can business data leak to the Go API?
[ ] Can provider API keys leak to the Go API?
[ ] Are side effects audited?
[ ] Are public errors safe?
[ ] Are tests present?
[ ] Are acceptance criteria actually verified?
```

If any answer indicates a boundary violation, fix it before moving to
the next task.
