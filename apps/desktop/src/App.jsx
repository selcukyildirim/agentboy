import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const PROVIDER_CONFIGS = {
  openai: { name: "OpenAI", icon: "🤖", models: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"] },
  anthropic: { name: "Anthropic", icon: "🧠", models: ["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"] },
  gemini: { name: "Google Gemini", icon: "✨", models: ["gemini-2.0-flash", "gemini-1.5-pro"] },
  "openai-compatible": { name: "OpenAI Compatible", icon: "🔗", models: [] },
  ollama: { name: "Ollama (Local)", icon: "🦙", models: [] },
};

function App() {
  const [activeTab, setActiveTab] = useState("dashboard");
  const [agents, setAgents] = useState([]);
  const [workflows, setWorkflows] = useState([]);
  const [documents, setDocuments] = useState([]);
  const [cacheStats, setCacheStats] = useState(null);
  const [health, setHealth] = useState(null);
  const [executions, setExecutions] = useState([]);
  const [configuringProvider, setConfiguringProvider] = useState(null);
  const [apiKey, setApiKey] = useState("");
  const [selectedModel, setSelectedModel] = useState("");
  const [testStatus, setTestStatus] = useState(null);
  const [configuredProviders, setConfiguredProviders] = useState({});

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    try {
      const [agentsData, workflowsData, docsData, cacheData, healthData, execsData] = await Promise.all([
        invoke("list_agents"),
        invoke("list_workflows"),
        invoke("list_documents"),
        invoke("get_cache_stats"),
        invoke("get_health"),
        invoke("list_executions"),
      ]);
      setAgents(agentsData);
      setWorkflows(workflowsData);
      setDocuments(docsData);
      setCacheStats(cacheData);
      setHealth(healthData);
      setExecutions(execsData);
    } catch (err) {
      console.error("Failed to load dashboard data:", err);
    }
  };

  const handleConfigureProvider = (provider) => {
    setConfiguringProvider(provider);
    setApiKey("");
    setSelectedModel("");
    setTestStatus(null);
  };

  const handleTestProvider = async () => {
    setTestStatus("testing");
    try {
      await invoke("test_provider", { provider: configuringProvider });
      setTestStatus("success");
    } catch {
      setTestStatus("error");
    }
  };

  const handleSaveProvider = async () => {
    try {
      await invoke("configure_provider", {
        config: {
          provider: configuringProvider,
          api_key: apiKey,
          model: selectedModel || null,
        },
      });
      setConfiguredProviders((prev) => ({ ...prev, [configuringProvider]: true }));
      setConfiguringProvider(null);
    } catch (err) {
      console.error("Failed to save provider:", err);
    }
  };

  const handleExecuteAgent = async (agentId) => {
    try {
      const result = await invoke("execute_agent", { agentId, input: {} });
      setExecutions((prev) => [...prev, result]);
      setActiveTab("executions");
    } catch (err) {
      console.error("Failed to execute agent:", err);
    }
  };

  const handleClearCache = async () => {
    try {
      await invoke("clear_cache");
      setCacheStats({ l1_entries: 0, l1_size_bytes: 0, l2_entries: 0, l2_size_bytes: 0, hit_rate: 0, total_requests: 0, total_hits: 0 });
    } catch (err) {
      console.error("Failed to clear cache:", err);
    }
  };

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="logo">
          <h1>AgentBoy</h1>
          <span className="version">v{health?.version || "0.1.0"}</span>
        </div>
        <nav>
          <button className={activeTab === "dashboard" ? "active" : ""} onClick={() => setActiveTab("dashboard")}>
            Dashboard
          </button>
          <button className={activeTab === "agents" ? "active" : ""} onClick={() => setActiveTab("agents")}>
            Agents
          </button>
          <button className={activeTab === "workflows" ? "active" : ""} onClick={() => setActiveTab("workflows")}>
            Workflows
          </button>
          <button className={activeTab === "knowledge" ? "active" : ""} onClick={() => setActiveTab("knowledge")}>
            Knowledge
          </button>
          <button className={activeTab === "providers" ? "active" : ""} onClick={() => setActiveTab("providers")}>
            AI Providers
          </button>
          <button className={activeTab === "executions" ? "active" : ""} onClick={() => setActiveTab("executions")}>
            Executions
          </button>
          <button className={activeTab === "decisions" ? "active" : ""} onClick={() => setActiveTab("decisions")}>
            Decisions
          </button>
          <button className={activeTab === "settings" ? "active" : ""} onClick={() => setActiveTab("settings")}>
            Settings
          </button>
        </nav>
        <div className="sidebar-footer">
          <div className="status-indicator">
            <span className={`status-dot ${health?.status === "healthy" ? "healthy" : "warning"}`} />
            {health?.status === "healthy" ? "System Healthy" : "Checking..."}
          </div>
        </div>
      </aside>

      <main className="content">
        {activeTab === "dashboard" && (
          <DashboardTab agents={agents} executions={executions} documents={documents} cacheStats={cacheStats} />
        )}
        {activeTab === "agents" && (
          <AgentsTab agents={agents} onExecute={handleExecuteAgent} />
        )}
        {activeTab === "workflows" && (
          <WorkflowsTab workflows={workflows} />
        )}
        {activeTab === "knowledge" && (
          <KnowledgeTab documents={documents} />
        )}
        {activeTab === "providers" && (
          <ProvidersTab
            configuringProvider={configuringProvider}
            configuredProviders={configuredProviders}
            apiKey={apiKey}
            selectedModel={selectedModel}
            testStatus={testStatus}
            onConfigure={handleConfigureProvider}
            onApiKeyChange={setApiKey}
            onModelChange={setSelectedModel}
            onTest={handleTestProvider}
            onSave={handleSaveProvider}
            onCancel={() => setConfiguringProvider(null)}
          />
        )}
        {activeTab === "executions" && (
          <ExecutionsTab executions={executions} />
        )}
        {activeTab === "decisions" && (
          <DecisionsTab />
        )}
        {activeTab === "settings" && (
          <SettingsTab cacheStats={cacheStats} onClearCache={handleClearCache} />
        )}
      </main>
    </div>
  );
}

function DashboardTab({ agents, executions, documents, cacheStats }) {
  return (
    <div className="tab-content">
      <h2>Dashboard</h2>
      <div className="cards">
        <div className="card">
          <h3>Agents</h3>
          <p className="card-value">{agents.length}</p>
          <p className="card-label">Available</p>
        </div>
        <div className="card">
          <h3>Executions</h3>
          <p className="card-value">{executions.length}</p>
          <p className="card-label">Total</p>
        </div>
        <div className="card">
          <h3>Documents</h3>
          <p className="card-value">{documents.length}</p>
          <p className="card-label">Indexed</p>
        </div>
        <div className="card">
          <h3>Cache</h3>
          <p className="card-value">{cacheStats ? `${(cacheStats.hit_rate * 100).toFixed(0)}%` : "0%"}</p>
          <p className="card-label">Hit Rate</p>
        </div>
      </div>

      <div className="section">
        <h3>Recent Executions</h3>
        {executions.length === 0 ? (
          <p className="empty-hint">No executions yet. Run an agent to get started.</p>
        ) : (
          <div className="execution-list">
            {executions.slice(0, 5).map((exec) => (
              <div key={exec.execution_id} className="execution-item">
                <span className="exec-agent">{exec.agent_id}</span>
                <span className={`exec-status ${exec.status}`}>{exec.status}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function AgentsTab({ agents, onExecute }) {
  const departments = [...new Set(agents.map((a) => a.department))];
  return (
    <div className="tab-content">
      <h2>Department Agents</h2>
      <p className="subtitle">{agents.length} production-quality agents ready</p>
      {departments.map((dept) => (
        <div key={dept} className="agent-group">
          <h3>{dept}</h3>
          <div className="agent-grid">
            {agents
              .filter((a) => a.department === dept)
              .map((agent) => (
                <div key={agent.id} className="agent-card">
                  <h4>{agent.name}</h4>
                  <p className="agent-desc">{agent.description}</p>
                  <div className="agent-meta">
                    <span className="agent-tier">{agent.tier}</span>
                    <span className="agent-id">{agent.id}</span>
                  </div>
                  <button className="btn-primary" onClick={() => onExecute(agent.id)}>
                    Run
                  </button>
                </div>
              ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function WorkflowsTab({ workflows }) {
  return (
    <div className="tab-content">
      <h2>Workflows</h2>
      <p className="subtitle">Reusable automation sequences</p>
      {workflows.length === 0 ? (
        <div className="empty-state">
          <p>No workflows saved yet.</p>
          <p className="empty-hint">Complete an agent execution and save it as a workflow.</p>
        </div>
      ) : (
        <div className="workflow-list">
          {workflows.map((wf) => (
            <div key={wf.id} className="workflow-item">
              <div className="workflow-info">
                <h4>{wf.name}</h4>
                <p>{wf.description}</p>
                <span className="workflow-meta">
                  v{wf.version} • {wf.step_count} steps
                </span>
              </div>
              <button className="btn-secondary">Execute</button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function KnowledgeTab({ documents }) {
  return (
    <div className="tab-content">
      <h2>Knowledge Workspace</h2>
      <p className="subtitle">Your local document index</p>
      <div className="upload-area">
        <div className="upload-icon">📁</div>
        <p>Drag & drop files here or click to browse</p>
        <p className="supported">Supported: PDF, DOCX, XLSX, CSV, TXT, MD, JSON, XML</p>
      </div>
      <div className="stats-row">
        <div className="stat">
          <span className="stat-value">{documents.length}</span>
          <span className="stat-label">Documents</span>
        </div>
        <div className="stat">
          <span className="stat-value">{documents.reduce((acc, d) => acc + d.chunk_count, 0)}</span>
          <span className="stat-label">Chunks</span>
        </div>
        <div className="stat">
          <span className="stat-value">
            {(documents.reduce((acc, d) => acc + d.size_bytes, 0) / 1024 / 1024).toFixed(1)} MB
          </span>
          <span className="stat-label">Size</span>
        </div>
      </div>
      {documents.length > 0 && (
        <div className="document-list">
          {documents.map((doc) => (
            <div key={doc.id} className="document-item">
              <span className="doc-name">{doc.name}</span>
              <span className="doc-type">{doc.content_type}</span>
              <span className="doc-chunks">{doc.chunk_count} chunks</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ProvidersTab({ configuringProvider, configuredProviders, apiKey, selectedModel, testStatus, onConfigure, onApiKeyChange, onModelChange, onTest, onSave, onCancel }) {
  return (
    <div className="tab-content">
      <h2>AI Providers</h2>
      <p className="subtitle">Configure your LLM providers (BYOK)</p>
      {configuringProvider ? (
        <div className="config-panel">
          <div className="config-header">
            <h3>
              {PROVIDER_CONFIGS[configuringProvider]?.icon} {PROVIDER_CONFIGS[configuringProvider]?.name}
            </h3>
            <button className="btn-close" onClick={onCancel}>x</button>
          </div>
          <div className="config-field">
            <label>API Key</label>
            <input type="password" placeholder="Enter API Key" value={apiKey} onChange={(e) => onApiKeyChange(e.target.value)} />
          </div>
          {PROVIDER_CONFIGS[configuringProvider]?.models?.length > 0 && (
            <div className="config-field">
              <label>Default Model</label>
              <select value={selectedModel} onChange={(e) => onModelChange(e.target.value)}>
                <option value="">Select model</option>
                {PROVIDER_CONFIGS[configuringProvider].models.map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            </div>
          )}
          <div className="config-actions">
            <button onClick={onTest} className="btn-secondary">
              {testStatus === "testing" ? "Testing..." : "Test Connection"}
            </button>
            <button onClick={onSave}>Save</button>
          </div>
          {testStatus === "success" && <p className="test-success">Connection successful!</p>}
          {testStatus === "error" && <p className="test-error">Connection failed. Check your key.</p>}
        </div>
      ) : (
        <div className="provider-list">
          {Object.entries(PROVIDER_CONFIGS).map(([key, config]) => (
            <div key={key} className="provider-item">
              <span className="provider-icon">{config.icon}</span>
              <div className="provider-info">
                <span className="provider-name">{config.name}</span>
                <span className="provider-models">{config.models.slice(0, 2).join(", ") || "Custom endpoint"}</span>
              </div>
              <span className={`provider-badge ${configuredProviders[key] ? "configured" : "not-configured"}`}>
                {configuredProviders[key] ? "Configured" : "Not configured"}
              </span>
              <button className="btn-small" onClick={() => onConfigure(key)}>Configure</button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ExecutionsTab({ executions }) {
  return (
    <div className="tab-content">
      <h2>Executions</h2>
      <p className="subtitle">Agent execution history</p>
      {executions.length === 0 ? (
        <div className="empty-state">
          <p>No executions yet.</p>
          <p className="empty-hint">Run an agent to see execution history here.</p>
        </div>
      ) : (
        <div className="execution-table">
          <div className="exec-header">
            <span>ID</span>
            <span>Agent</span>
            <span>Status</span>
            <span>Started</span>
          </div>
          {executions.map((exec) => (
            <div key={exec.execution_id} className="exec-row">
              <span className="exec-id">{exec.execution_id?.slice(0, 8) || "N/A"}</span>
              <span className="exec-agent">{exec.agent_id}</span>
              <span className={`exec-status ${exec.status}`}>{exec.status}</span>
              <span className="exec-time">{exec.started_at || new Date().toISOString()}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function DecisionsTab() {
  const [decisionTypes, setDecisionTypes] = useState([]);
  const [selectedType, setSelectedType] = useState(null);

  useEffect(() => {
    invoke("list_decision_types").then(setDecisionTypes).catch(console.error);
  }, []);

  return (
    <div className="tab-content">
      <h2>Decision Intelligence</h2>
      <p className="subtitle">Evidence-backed decision support</p>
      <div className="decision-list">
        {decisionTypes.map((dt) => (
          <div key={dt.id} className={`decision-item ${selectedType === dt.id ? "selected" : ""}`} onClick={() => setSelectedType(dt.id)}>
            <h4>{dt.name}</h4>
            <p>Category: {dt.category}</p>
            <p>Risk: {dt.risk_level}</p>
            <p className="decision-facts">Required: {dt.required_facts.join(", ")}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function SettingsTab({ cacheStats, onClearCache }) {
  return (
    <div className="tab-content">
      <h2>Settings</h2>
      <div className="settings-grid">
        <div className="settings-section">
          <h3>General</h3>
          <div className="setting-row">
            <label>Theme</label>
            <select>
              <option>System</option>
              <option>Light</option>
              <option>Dark</option>
            </select>
          </div>
          <div className="setting-row">
            <label>Language</label>
            <select>
              <option>English</option>
              <option>Turkish</option>
            </select>
          </div>
        </div>
        <div className="settings-section">
          <h3>Privacy</h3>
          <div className="setting-row">
            <label>Data Egress Guard</label>
            <span className="setting-badge active">Active</span>
          </div>
          <div className="setting-row">
            <label>Local Processing</label>
            <span className="setting-badge active">Enabled</span>
          </div>
        </div>
        <div className="settings-section">
          <h3>Cache</h3>
          <div className="setting-row">
            <label>L1 In-Memory</label>
            <span className="setting-badge active">Active</span>
          </div>
          <div className="setting-row">
            <label>L2 Persistent</label>
            <span className="setting-badge active">Active</span>
          </div>
          <div className="setting-row">
            <label>Hit Rate</label>
            <span className="setting-value">{cacheStats ? `${(cacheStats.hit_rate * 100).toFixed(0)}%` : "0%"}</span>
          </div>
          <button className="btn-danger" onClick={onClearCache}>Clear Cache</button>
        </div>
      </div>
    </div>
  );
}

export default App;