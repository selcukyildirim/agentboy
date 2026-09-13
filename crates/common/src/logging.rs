pub mod logging {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};

    pub fn init(log_level: &str) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_filter(env_filter);

        let subscriber = Registry::default().with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");
    }

    pub fn correlation_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

pub mod secret_filter {
    use std::sync::OnceLock;

    struct SecretPattern {
        pattern: regex::Regex,
    }

    fn patterns() -> &'static Vec<SecretPattern> {
        static PATTERNS: OnceLock<Vec<SecretPattern>> = OnceLock::new();
        PATTERNS.get_or_init(|| {
            vec![
                SecretPattern {
                    pattern: regex::Regex::new(
                        r#"(?i)(api[_-]?key|secret|password|token|credential)\s*[:=]\s*['"]?([^\s'"]+)['"]?"#
                    ).unwrap(),
                },
                SecretPattern {
                    pattern: regex::Regex::new(
                        r#"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*"#
                    ).unwrap(),
                },
                SecretPattern {
                    pattern: regex::Regex::new(
                        r#"(?i)sk-[A-Za-z0-9]{20,}"#
                    ).unwrap(),
                },
                SecretPattern {
                    pattern: regex::Regex::new(
                        r#"(?i)ghp_[A-Za-z0-9]{36}"#
                    ).unwrap(),
                },
            ]
        })
    }

    pub fn redact(input: &str) -> String {
        let mut result = input.to_string();
        for p in patterns() {
            result = p.pattern.replace_all(&result, "[REDACTED]").to_string();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id() {
        let id = logging::correlation_id();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn test_secret_redaction() {
        let input = "api_key=sk-abc123def456ghi789jkl012mno";
        let redacted = secret_filter::redact(input);
        assert!(!redacted.contains("sk-abc123"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_bearer_redaction() {
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted = secret_filter::redact(input);
        assert!(!redacted.contains("eyJhbGci"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
