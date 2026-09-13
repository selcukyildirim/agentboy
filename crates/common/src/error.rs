use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("User facing error: {code} - {message}")]
    UserFacing { code: ErrorCode, message: String },

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },

    #[error("Entitlement error: {0}")]
    Entitlement(String),

    #[error("Policy denied: {0}")]
    PolicyDenied(String),

    #[error("Egress blocked: {reason}")]
    EgressBlocked { reason: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Tool permission denied: {tool} - {reason}")]
    ToolPermissionDenied { tool: String, reason: String },

    #[error("Tool execution failed: {tool} - {reason}")]
    ToolExecutionFailed { tool: String, reason: String },

    #[error("Execution limit exceeded: max {limit} steps")]
    ExecutionLimitExceeded { limit: u32 },

    #[error("Execution cancelled: {reason}")]
    ExecutionCancelled { reason: String },

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ErrorCode {
    AUTH_INVALID_CREDENTIALS,
    AUTH_TOKEN_EXPIRED,
    AUTH_TOKEN_INVALID,
    PROVIDER_AUTH_FAILED,
    PROVIDER_RATE_LIMITED,
    PROVIDER_UNAVAILABLE,
    PROVIDER_UNSUPPORTED_CAPABILITY,
    ENTITLEMENT_DENIED,
    POLICY_DENIED,
    EGRESS_BLOCKED,
    TOOL_PATH_TRAVERSAL,
    TOOL_ACCESS_DENIED,
    TOOL_DESTRUCTIVE_REQUIRES_APPROVAL,
    DOCUMENT_PARSE_FAILED,
    CACHE_ERROR,
    DATABASE_ERROR,
    INTERNAL_ERROR,
    VALIDATION_ERROR,
    EXECUTION_LIMIT_EXCEEDED,
    EXECUTION_CANCELLED,
    INVALID_STATE_TRANSITION,
    NOT_FOUND,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type AppResult<T> = Result<T, AppError>;
