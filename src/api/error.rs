use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorData {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    NetworkError(ApiErrorData),
    TimeoutError(ApiErrorData),
    AuthenticationError(ApiErrorData),
    AuthorizationError(ApiErrorData),
    BadRequest(ApiErrorData),
    NotFound(ApiErrorData),
    RateLimitExceeded(ApiErrorData),
    ServerError(ApiErrorData),
    InvalidResponseFormat(ApiErrorData),
    SerializationError(ApiErrorData),
    ConfigError(ApiErrorData),
}

impl ApiErrorData {
    pub fn new(kind: &str, message: String, status: Option<u16>, raw_body: Option<String>) -> Self {
        Self {
            kind: kind.to_string(),
            message,
            status,
            raw_body,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(d) => write!(f, "Network error: {}", d.message),
            ApiError::TimeoutError(d) => write!(f, "Timeout error: {}", d.message),
            ApiError::AuthenticationError(d) => write!(f, "Authentication error: {}", d.message),
            ApiError::AuthorizationError(d) => write!(f, "Authorization error: {}", d.message),
            ApiError::BadRequest(d) => write!(f, "Bad request: {}", d.message),
            ApiError::NotFound(d) => write!(f, "Resource not found: {}", d.message),
            ApiError::RateLimitExceeded(d) => write!(f, "Rate limit exceeded: {}", d.message),
            ApiError::ServerError(d) => write!(f, "Server error: {}", d.message),
            ApiError::InvalidResponseFormat(d) => {
                write!(f, "Invalid response format: {}", d.message)
            }
            ApiError::SerializationError(d) => write!(f, "Serialization error: {}", d.message),
            ApiError::ConfigError(d) => write!(f, "Configuration error: {}", d.message),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        let msg = err.to_string();
        if err.is_timeout() {
            return ApiError::TimeoutError(ApiErrorData::new("timeout", msg, None, None));
        }
        if let Some(status) = err.status() {
            let code = status.as_u16();
            let data = |kind: &str| ApiErrorData::new(kind, msg.clone(), Some(code), None);
            return match code {
                400 => ApiError::BadRequest(data("bad_request")),
                401 => ApiError::AuthenticationError(data("authentication")),
                403 => ApiError::AuthorizationError(data("authorization")),
                404 => ApiError::NotFound(data("not_found")),
                429 => ApiError::RateLimitExceeded(data("rate_limited")),
                500..=599 => ApiError::ServerError(data("server_error")),
                _ => ApiError::NetworkError(data("network")),
            };
        }
        ApiError::NetworkError(ApiErrorData::new("network", msg, None, None))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::SerializationError(ApiErrorData::new(
            "serialization",
            err.to_string(),
            None,
            None,
        ))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::NetworkError(ApiErrorData::new("io", err.to_string(), None, None))
    }
}

impl ApiError {
    pub fn from_response(status: reqwest::StatusCode, body: String) -> Self {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => json
                .get("msg")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("message").and_then(|v| v.as_str()))
                .unwrap_or(&body)
                .to_string(),
            Err(_) => body.clone(),
        };

        let code = status.as_u16();
        let data = |kind: &str| {
            ApiErrorData::new(kind, error_message.clone(), Some(code), Some(body.clone()))
        };
        match code {
            400 => ApiError::BadRequest(data("bad_request")),
            401 => ApiError::AuthenticationError(data("authentication")),
            403 => ApiError::AuthorizationError(data("authorization")),
            404 => ApiError::NotFound(data("not_found")),
            429 => ApiError::RateLimitExceeded(data("rate_limited")),
            500..=599 => ApiError::ServerError(data("server_error")),
            _ => ApiError::NetworkError(data("network")),
        }
    }

    // Removed unused helper methods (error_code, is_retryable, retry_delay_ms, user_friendly_message)
}
