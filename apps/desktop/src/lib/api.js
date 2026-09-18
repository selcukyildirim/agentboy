import { invoke } from "@tauri-apps/api/core";

/** Normalized error thrown by all backend calls. */
export class ApiError extends Error {
  constructor(code, message) {
    super(message || "Unknown error");
    this.name = "ApiError";
    this.code = code || "INTERNAL_ERROR";
  }
}

/**
 * Call a Tauri command and normalize errors into {@link ApiError}.
 * Backend commands return `{ code, message }` on failure.
 */
export async function call(command, args = {}) {
  try {
    return await invoke(command, args);
  } catch (err) {
    if (err && typeof err === "object" && err.code && err.message) {
      throw new ApiError(err.code, err.message);
    }
    if (typeof err === "string") {
      throw new ApiError("INTERNAL_ERROR", err);
    }
    throw new ApiError(err?.code || "INTERNAL_ERROR", err?.message || String(err));
  }
}

export const api = {
  // system / providers
  health: () => call("get_health"),
  metrics: () => call("get_metrics"),
  appVersion: () => call("get_app_version"),
  providers: () => call("list_providers"),
  providerStatus: () => call("get_provider_status"),
  configureProvider: (config) => call("configure_provider", { config }),
  testProvider: (provider, apiKey, baseUrl) =>
    call("test_provider", { provider, apiKey, baseUrl }),
  removeProvider: (provider) => call("remove_provider_credential", { provider }),
  providerModels: (provider) => call("list_provider_models", { provider }),
  setActiveProvider: (provider) => call("set_active_provider", { provider }),
  activeProvider: () => call("get_active_provider"),

  // agents
  agents: () => call("list_agents"),
  agentManifest: (agentId) => call("get_agent_manifest", { agentId }),
  executeAgent: (agentId, input, offline = false) =>
    call("execute_agent", { agentId, input, offline }),

  // workflows
  workflows: () => call("list_workflows"),
  createWorkflow: (name, description) =>
    call("create_workflow", { name, description }),
  addWorkflowStep: (workflowId, agentId, name) =>
    call("add_workflow_step", { workflowId, agentId, name }),
  deleteWorkflow: (workflowId) => call("delete_workflow", { workflowId }),
  executeWorkflow: (workflowId, input) =>
    call("execute_workflow", { workflowId, input }),

  // documents / RAG
  documents: () => call("list_documents"),
  uploadDocument: (name, content, contentType) =>
    call("upload_document", { name, content, contentType }),
  queryRag: (query, topK) => call("query_rag", { query, topK }),

  // executions
  executions: () => call("list_executions"),
  execution: (executionId) => call("get_execution", { executionId }),
  executionSteps: (executionId) => call("get_execution_steps", { executionId }),
  deleteExecution: (executionId) => call("delete_execution", { executionId }),
  executionStats: (days) => call("get_execution_stats", { days }),

  // decisions
  decisionTypes: () => call("list_decision_types"),
  decisionContext: (decisionTypeId) =>
    call("get_decision_context", { decisionTypeId }),
  recommendation: (decisionTypeId, context) =>
    call("get_recommendation", { decisionTypeId, context }),

  // cache
  cacheStats: () => call("get_cache_stats"),
  clearCache: () => call("clear_cache"),
};
