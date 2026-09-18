use crate::gateway::LlmProvider;
use crate::types::{CompletionRequest, CompletionResponse, ProviderCapabilities, RoutingMode};
use agent_common::error::{AppError, AppResult};
use std::collections::HashMap;
use std::sync::Arc;

pub struct GatewayRouter {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    default_provider: Option<String>,
    mode: RoutingMode,
    fallback_order: Vec<String>,
}

impl GatewayRouter {
    #[must_use]
    pub fn new(mode: RoutingMode) -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            mode,
            fallback_order: Vec::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn LlmProvider>) {
        let id = provider.id().to_string();
        self.providers.insert(id.clone(), provider);
        if self.default_provider.is_none() {
            self.default_provider = Some(id);
        }
    }

    pub fn set_default(&mut self, provider_id: &str) {
        if self.providers.contains_key(provider_id) {
            self.default_provider = Some(provider_id.to_string());
        }
    }

    pub fn set_fallback_order(&mut self, order: Vec<String>) {
        self.fallback_order = order;
    }

    #[must_use]
    pub fn get_provider(&self, id: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(id).cloned()
    }

    #[must_use]
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    #[must_use]
    pub fn default_provider_id(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    pub fn mode(&self) -> &RoutingMode {
        &self.mode
    }

    pub async fn complete(
        &self,
        provider_id: Option<&str>,
        req: CompletionRequest,
    ) -> AppResult<CompletionResponse> {
        let target = provider_id
            .or(self.default_provider.as_deref())
            .ok_or_else(|| AppError::Provider {
                provider: "none".to_string(),
                message: "No provider configured".to_string(),
            })?;

        if let Some(provider) = self.providers.get(target) {
            return provider.complete(req).await;
        }

        Err(AppError::Provider {
            provider: target.to_string(),
            message: "Provider not found".to_string(),
        })
    }

    pub async fn complete_with_fallback(
        &self,
        provider_id: Option<&str>,
        req: CompletionRequest,
    ) -> AppResult<CompletionResponse> {
        let primary = provider_id
            .or(self.default_provider.as_deref())
            .ok_or_else(|| AppError::Provider {
                provider: "none".to_string(),
                message: "No provider configured".to_string(),
            })?;

        if let Some(provider) = self.providers.get(primary) {
            match provider.complete(req.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if !self.is_infrastructure_error(&e) {
                        return Err(e);
                    }
                    tracing::warn!(
                        provider = primary,
                        error = %e,
                        "Provider failed, trying fallback"
                    );
                }
            }
        }

        for fallback_id in &self.fallback_order {
            if fallback_id == primary {
                continue;
            }
            if let Some(provider) = self.providers.get(fallback_id) {
                match provider.complete(req.clone()).await {
                    Ok(response) => {
                        tracing::info!(
                            primary = primary,
                            fallback = fallback_id,
                            "Fallback succeeded"
                        );
                        return Ok(response);
                    }
                    Err(e) => {
                        if !self.is_infrastructure_error(&e) {
                            return Err(e);
                        }
                        tracing::warn!(
                            provider = fallback_id,
                            error = %e,
                            "Fallback also failed"
                        );
                    }
                }
            }
        }

        Err(AppError::Provider {
            provider: primary.to_string(),
            message: "All providers failed".to_string(),
        })
    }

    fn is_infrastructure_error(&self, error: &AppError) -> bool {
        match error {
            AppError::Provider { message, .. } => {
                message.contains("timeout")
                    || message.contains("rate limit")
                    || message.contains("Rate limited")
                    || message.contains("500")
                    || message.contains("502")
                    || message.contains("503")
                    || message.contains("network")
            }
            _ => false,
        }
    }
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new(RoutingMode::Manual)
    }
}

pub struct SmartRouter;

impl SmartRouter {
    #[must_use]
    pub fn select_provider(
        _task_type: &str,
        required_capabilities: &ProviderCapabilities,
        providers: &[(String, ProviderCapabilities)],
    ) -> Option<String> {
        providers
            .iter()
            .find(|(_, caps)| {
                caps.text
                    && (!required_capabilities.vision || caps.vision)
                    && (!required_capabilities.tools || caps.tools)
                    && (!required_capabilities.structured_output || caps.structured_output)
            })
            .map(|(id, _)| id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_default() {
        let router = GatewayRouter::new(RoutingMode::Manual);
        assert!(router.default_provider_id().is_none());
        assert!(router.list_providers().is_empty());
    }
}
