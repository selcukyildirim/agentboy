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

impl AppError {
    /// Stable error code for API responses and client handling.
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::UserFacing { code, .. } => code.clone(),
            AppError::Internal(_) => ErrorCode::INTERNAL_ERROR,
            AppError::Config(_) => ErrorCode::INTERNAL_ERROR,
            AppError::Database(_) => ErrorCode::DATABASE_ERROR,
            AppError::Provider { message, .. } => {
                if message.contains("rate") || message.contains("429") {
                    ErrorCode::PROVIDER_RATE_LIMITED
                } else if message.contains("auth") || message.contains("401") {
                    ErrorCode::PROVIDER_AUTH_FAILED
                } else {
                    ErrorCode::PROVIDER_UNAVAILABLE
                }
            }
            AppError::Entitlement(_) => ErrorCode::ENTITLEMENT_DENIED,
            AppError::PolicyDenied(_) => ErrorCode::POLICY_DENIED,
            AppError::EgressBlocked { .. } => ErrorCode::EGRESS_BLOCKED,
            AppError::Validation(_) => ErrorCode::VALIDATION_ERROR,
            AppError::ToolPermissionDenied { .. } => ErrorCode::TOOL_ACCESS_DENIED,
            AppError::ToolExecutionFailed { .. } => ErrorCode::INTERNAL_ERROR,
            AppError::ExecutionLimitExceeded { .. } => ErrorCode::EXECUTION_LIMIT_EXCEEDED,
            AppError::ExecutionCancelled { .. } => ErrorCode::EXECUTION_CANCELLED,
            AppError::InvalidStateTransition { .. } => ErrorCode::INVALID_STATE_TRANSITION,
            AppError::NotFound(_) => ErrorCode::NOT_FOUND,
        }
    }
}

/// Stable, serializable error envelope: `{ "code": "...", "message": "..." }`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self {
            code: err.error_code(),
            message: err.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Generic API response envelope used by UI/API boundaries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        assert_eq!(
            AppError::Entitlement("x".into()).error_code(),
            ErrorCode::ENTITLEMENT_DENIED
        );
        assert_eq!(
            AppError::EgressBlocked { reason: "x".into() }.error_code(),
            ErrorCode::EGRESS_BLOCKED
        );
        assert_eq!(
            AppError::Validation("x".into()).error_code(),
            ErrorCode::VALIDATION_ERROR
        );
        assert_eq!(
            AppError::NotFound("x".into()).error_code(),
            ErrorCode::NOT_FOUND
        );
    }

    #[test]
    fn test_api_error_from_app_error() {
        let api: ApiError = AppError::Validation("bad input".into()).into();
        assert_eq!(api.code, ErrorCode::VALIDATION_ERROR);
        assert!(api.message.contains("bad input"));
    }

    #[test]
    fn test_api_response_ok() {
        let resp = ApiResponse::ok(42);
        assert!(resp.success);
        assert_eq!(resp.data, Some(42));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_err() {
        let resp: ApiResponse<()> = ApiResponse::err(ApiError::new(ErrorCode::POLICY_DENIED, "no"));
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error.unwrap().code, ErrorCode::POLICY_DENIED);
    }

    #[test]
    fn test_api_error_serializes_with_code_field() {
        let api = ApiError::new(ErrorCode::NOT_FOUND, "missing");
        let json = serde_json::to_value(&api).unwrap();
        assert_eq!(json["code"], "NOT_FOUND");
        assert_eq!(json["message"], "missing");
    }
}

