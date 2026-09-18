use agent_runtime::context::LlmUsage;

/// Price in USD per 1M tokens.
#[derive(Debug, Clone, Copy)]
pub struct ModelPrice {
    pub input_per_m: f64,
    pub output_per_m: f64,
}

impl ModelPrice {
    const fn new(input_per_m: f64, output_per_m: f64) -> Self {
        Self {
            input_per_m,
            output_per_m,
        }
    }
}

/// Built-in price table. These are published list prices (USD / 1M tokens)
/// used only for a local estimate; they are intentionally not user-editable.
/// Local models (Ollama) are free.
const TABLE: &[(&str, ModelPrice)] = &[
    ("gpt-4o-mini", ModelPrice::new(0.15, 0.60)),
    ("gpt-4o", ModelPrice::new(2.50, 10.00)),
    ("gpt-4.1-mini", ModelPrice::new(0.40, 1.60)),
    ("gpt-4.1", ModelPrice::new(2.00, 8.00)),
    ("gpt-4-turbo", ModelPrice::new(10.00, 30.00)),
    ("o1-mini", ModelPrice::new(1.10, 4.40)),
    ("o1", ModelPrice::new(15.00, 60.00)),
    ("claude-3-5-haiku", ModelPrice::new(0.80, 4.00)),
    ("claude-3-5-sonnet", ModelPrice::new(3.00, 15.00)),
    ("claude-3-opus", ModelPrice::new(15.00, 75.00)),
    ("claude-sonnet-4", ModelPrice::new(3.00, 15.00)),
    ("claude-opus-4", ModelPrice::new(15.00, 75.00)),
    ("gemini-2.0-flash", ModelPrice::new(0.10, 0.40)),
    ("gemini-1.5-pro", ModelPrice::new(1.25, 5.00)),
    ("gemini-1.5-flash", ModelPrice::new(0.075, 0.30)),
    ("llama", ModelPrice::new(0.0, 0.0)),
    ("mistral", ModelPrice::new(0.0, 0.0)),
    ("qwen", ModelPrice::new(0.0, 0.0)),
];

/// Resolve a model's price. `None` means the model is unknown (no estimate).
pub fn price_for(model: &str) -> Option<ModelPrice> {
    let m = model.to_lowercase();
    // Match most specific first (longer keys are more specific).
    TABLE
        .iter()
        .filter(|(key, _)| m.contains(*key))
        .max_by_key(|(key, _)| key.len())
        .map(|(_, price)| *price)
}

/// Estimate the cost of a call in USD. `None` when the model is unknown.
pub fn estimate_cost_usd(model: &str, usage: &LlmUsage) -> Option<f64> {
    let price = price_for(model)?;
    let input = usage.prompt_tokens as f64 / 1_000_000.0 * price.input_per_m;
    let output = usage.completion_tokens as f64 / 1_000_000.0 * price.output_per_m;
    Some(input + output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(p: u32, c: u32) -> LlmUsage {
        LlmUsage {
            prompt_tokens: p,
            completion_tokens: c,
            total_tokens: p + c,
        }
    }

    #[test]
    fn test_price_lookup() {
        assert!(price_for("gpt-4o-mini").is_some());
        assert!(price_for("openai/gpt-4o").is_some());
        assert!(price_for("claude-3-5-haiku-20241022").is_some());
        assert!(price_for("gemini-2.0-flash").is_some());
        assert!(price_for("unknown-model-xyz").is_none());
    }

    #[test]
    fn test_most_specific_match() {
        let mini = price_for("gpt-4o-mini").unwrap();
        assert!((mini.input_per_m - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_estimate() {
        let cost = estimate_cost_usd("gpt-4o-mini", &usage(1_000_000, 1_000_000)).unwrap();
        assert!((cost - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_small() {
        let cost = estimate_cost_usd("gpt-4o-mini", &usage(1000, 500)).unwrap();
        assert!(cost < 0.001);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_local_model_is_free() {
        let cost = estimate_cost_usd("llama3.1", &usage(1_000_000, 1_000_000)).unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_unknown_model_no_estimate() {
        assert!(estimate_cost_usd("nope", &usage(10, 10)).is_none());
    }
}
