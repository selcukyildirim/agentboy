use agent_common::error::AppResult;
use agent_common::types::DataClassification;
use async_trait::async_trait;
use egress_guard::guard::EgressGuard;
use egress_guard::manifest::EgressManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<LlmMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<LlmUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn provider_id(&self) -> &str;
    async fn complete(&self, request: LlmCompletionRequest) -> AppResult<LlmCompletionResponse>;
    async fn validate_credentials(&self) -> AppResult<bool> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_tokens: u32,
    pub temperature: f64,
    /// When true, LLM calls are skipped and agents fall back to deterministic-only
    /// analysis (privacy-first / air-gapped operation).
    #[serde(default)]
    pub offline: bool,
    /// Per-agent overrides. Recognized keys: `model` (string), `temperature`
    /// (number), `max_tokens` (number). Unknown keys are ignored.
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            temperature: 0.3,
            offline: false,
            custom: HashMap::new(),
        }
    }
}

impl AgentConfig {
    pub fn model_override(&self) -> Option<String> {
        self.custom
            .get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn effective_temperature(&self) -> f64 {
        self.custom
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(self.temperature)
    }

    pub fn effective_max_tokens(&self) -> u32 {
        self.custom
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(self.max_tokens)
    }
}

#[async_trait]
pub trait AgentContext: Send + Sync {
    fn llm(&self) -> &dyn LlmProvider;
    fn config(&self) -> &AgentConfig;
    fn last_egress_manifest(&self) -> Option<EgressManifest>;

    async fn call_llm(&self, system_prompt: &str, user_prompt: &str) -> AppResult<String> {
        if self.config().offline {
            tracing::debug!("Offline mode: skipping LLM call, returning deterministic placeholder");
            return Ok(
                "[offline] LLM analysis skipped (offline mode enabled); deterministic results only."
                    .to_string(),
            );
        }

        let (sanitized_system, sanitized_user) = self.sanitize_for_egress(system_prompt, user_prompt)?;

        let request = LlmCompletionRequest {
            model: self.config().model_override(),
            messages: vec![
                LlmMessage {
                    role: "system".to_string(),
                    content: sanitized_system,
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: sanitized_user,
                },
            ],
            max_tokens: Some(self.config().effective_max_tokens()),
            temperature: Some(self.config().effective_temperature()),
        };

        let response = self.llm().complete(request).await?;
        Ok(response.content)
    }

    async fn call_llm_with_context(
        &self,
        system_prompt: &str,
        context: &str,
        user_prompt: &str,
    ) -> AppResult<String> {
        let full_user = format!("{}\n\n{}", context, user_prompt);
        self.call_llm(system_prompt, &full_user).await
    }

    fn sanitize_for_egress(&self, system_prompt: &str, user_prompt: &str) -> AppResult<(String, String)> {
        Ok((system_prompt.to_string(), user_prompt.to_string()))
    }
}

pub struct DefaultAgentContext {
    llm: Box<dyn LlmProvider>,
    config: AgentConfig,
    egress_guard: Mutex<EgressGuard>,
    last_egress_manifest: Mutex<Option<EgressManifest>>,
}

impl DefaultAgentContext {
    pub fn new(llm: Box<dyn LlmProvider>, config: AgentConfig) -> Self {
        Self {
            llm,
            config,
            egress_guard: Mutex::new(EgressGuard::default()),
            last_egress_manifest: Mutex::new(None),
        }
    }

    pub fn with_config(llm: Box<dyn LlmProvider>) -> Self {
        Self::new(llm, AgentConfig::default())
    }

    pub fn with_egress_classification(llm: Box<dyn LlmProvider>, classification: DataClassification) -> Self {
        Self {
            llm,
            config: AgentConfig::default(),
            egress_guard: Mutex::new(EgressGuard::new(classification)),
            last_egress_manifest: Mutex::new(None),
        }
    }
}

impl AgentContext for DefaultAgentContext {
    fn llm(&self) -> &dyn LlmProvider {
        self.llm.as_ref()
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    fn last_egress_manifest(&self) -> Option<EgressManifest> {
        self.last_egress_manifest.lock().unwrap().clone()
    }

    fn sanitize_for_egress(&self, system_prompt: &str, user_prompt: &str) -> AppResult<(String, String)> {
        let mut guard = self.egress_guard.lock().unwrap();
        let provider_id = self.llm.provider_id();

        let (sanitized_user, classification) = guard
            .check_and_sanitize(user_prompt, provider_id)
            .map_err(|e| agent_common::error::AppError::EgressBlocked {
                reason: e.to_string(),
            })?;

        let sanitized_system = guard.classifier().sanitize(system_prompt);

        let pii_detected = guard.classifier().detect_pii(user_prompt).is_some();
        let secrets_detected = guard.classifier().detect_secrets(user_prompt).is_some();

        let manifest = guard.build_manifest(
            provider_id,
            classification,
            0,
            vec![],
            pii_detected,
            secrets_detected,
            "agent-llm-call",
        );

        *self.last_egress_manifest.lock().unwrap() = Some(manifest);

        Ok((sanitized_system, sanitized_user))
    }
}

pub struct MockLlmProvider {
    pub responses: Vec<String>,
    pub call_count: std::sync::atomic::AtomicUsize,
}

impl MockLlmProvider {
    pub fn with_response(response: &str) -> Self {
        Self {
            responses: vec![response.to_string()],
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses,
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn times_called(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, _request: LlmCompletionRequest) -> AppResult<LlmCompletionResponse> {
        let idx = self.call_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let content = self.responses.get(idx % self.responses.len())
            .cloned()
            .unwrap_or_default();

        Ok(LlmCompletionResponse {
            content,
            model: "mock-model".to_string(),
            usage: Some(LlmUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
        })
    }

    fn model_name(&self) -> &str {
        "mock-model"
    }

    fn provider_id(&self) -> &str {
        "mock"
    }
}

pub struct MockAgentContext {
    llm: MockLlmProvider,
    config: AgentConfig,
}

impl MockAgentContext {
    pub fn new(llm: MockLlmProvider) -> Self {
        Self {
            llm,
            config: AgentConfig::default(),
        }
    }

    pub fn with_config(llm: MockLlmProvider, config: AgentConfig) -> Self {
        Self { llm, config }
    }

    pub fn provider(&self) -> &MockLlmProvider {
        &self.llm
    }
}

impl AgentContext for MockAgentContext {
    fn llm(&self) -> &dyn LlmProvider {
        &self.llm
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    fn last_egress_manifest(&self) -> Option<EgressManifest> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_provider() {
        let provider = MockLlmProvider::with_response("Hello!");
        let request = LlmCompletionRequest {
            model: None,
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            max_tokens: None,
            temperature: None,
        };

        let response = provider.complete(request).await.unwrap();
        assert_eq!(response.content, "Hello!");
        assert_eq!(provider.times_called(), 1);
    }

    #[tokio::test]
    async fn test_mock_agent_context() {
        let llm = MockLlmProvider::with_response("Analysis result");
        let ctx = MockAgentContext::new(llm);

        let result = ctx.call_llm("You are an analyst", "Analyze this data").await.unwrap();
        assert_eq!(result, "Analysis result");
    }

    #[tokio::test]
    async fn test_mock_agent_context_with_context() {
        let llm = MockLlmProvider::with_response("Contextual result");
        let ctx = MockAgentContext::new(llm);

        let result = ctx.call_llm_with_context(
            "You are an analyst",
            "Context: revenue=1000",
            "What is the revenue?"
        ).await.unwrap();
        assert_eq!(result, "Contextual result");
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_tokens, 2048);
        assert!((config.temperature - 0.3).abs() < 0.01);
        assert!(!config.offline);
    }

    #[tokio::test]
    async fn test_offline_mode_skips_llm() {
        let llm = MockLlmProvider::with_response("should not be called");
        let mut config = AgentConfig::default();
        config.offline = true;
        let ctx = MockAgentContext::with_config(llm, config);

        let result = ctx.call_llm("system", "user").await.unwrap();
        assert!(result.contains("[offline]"));
        assert_eq!(ctx.provider().times_called(), 0);
    }

    #[test]
    fn test_custom_overrides() {
        let mut config = AgentConfig::default();
        config.custom.insert("model".to_string(), serde_json::json!("gpt-4o"));
        config.custom.insert("temperature".to_string(), serde_json::json!(0.9));
        config.custom.insert("max_tokens".to_string(), serde_json::json!(512));

        assert_eq!(config.model_override().as_deref(), Some("gpt-4o"));
        assert!((config.effective_temperature() - 0.9).abs() < 0.001);
        assert_eq!(config.effective_max_tokens(), 512);
    }

    #[test]
    fn test_custom_overrides_default_when_absent() {
        let config = AgentConfig::default();
        assert!(config.model_override().is_none());
        assert!((config.effective_temperature() - 0.3).abs() < 0.001);
        assert_eq!(config.effective_max_tokens(), 2048);
    }

    #[tokio::test]
    async fn test_mock_llm_multiple_responses() {
        let provider = MockLlmProvider::with_responses(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);

        let request = LlmCompletionRequest {
            model: None,
            messages: vec![],
            max_tokens: None,
            temperature: None,
        };

        let r1 = provider.complete(request.clone()).await.unwrap();
        let r2 = provider.complete(request.clone()).await.unwrap();
        let r3 = provider.complete(request).await.unwrap();

        assert_eq!(r1.content, "First");
        assert_eq!(r2.content, "Second");
        assert_eq!(r3.content, "Third");
        assert_eq!(provider.times_called(), 3);
    }

    #[tokio::test]
    async fn test_egress_guard_sanitize_pii() {
        let llm = MockLlmProvider::with_response("Analysis result");
        let ctx = DefaultAgentContext::with_egress_classification(
            Box::new(llm),
            DataClassification::L3MinimumRequired,
        );

        let result = ctx.call_llm(
            "You are an analyst",
            "Contact user@example.com for details",
        ).await.unwrap();
        assert_eq!(result, "Analysis result");

        let manifest = ctx.last_egress_manifest().unwrap();
        assert!(manifest.pii_removed);
        assert_eq!(manifest.provider, "mock");
    }

    #[tokio::test]
    async fn test_egress_guard_sanitize_secrets() {
        let llm = MockLlmProvider::with_response("Analysis result");
        let ctx = DefaultAgentContext::with_egress_classification(
            Box::new(llm),
            DataClassification::L3MinimumRequired,
        );

        let result = ctx.call_llm(
            "You are an analyst",
            "Use api_key=sk-abc123def456ghi789jkl0 to authenticate",
        ).await.unwrap();
        assert_eq!(result, "Analysis result");

        let manifest = ctx.last_egress_manifest().unwrap();
        assert!(manifest.secrets_removed);
    }

    #[tokio::test]
    async fn test_egress_guard_blocks_high_classification() {
        let llm = MockLlmProvider::with_response("Analysis result");
        let ctx = DefaultAgentContext::with_egress_classification(
            Box::new(llm),
            DataClassification::L0LocalOnly,
        );

        let result = ctx.call_llm(
            "You are an analyst",
            "This is a long message that exceeds one hundred characters and should be classified as L3MinimumRequired by the data classifier for testing purposes",
        ).await;
        assert!(result.is_err());
    }
}
