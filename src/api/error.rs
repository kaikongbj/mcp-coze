use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    NetworkError(String),
    TimeoutError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    BadRequest(String),
    NotFound(String),
    RateLimitExceeded(String),
    ServerError(String),
    InvalidResponseFormat(String),
    SerializationError(String),
    ConfigError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            ApiError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            ApiError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Resource not found: {}", msg),
            ApiError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
            ApiError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ApiError::InvalidResponseFormat(msg) => write!(f, "Invalid response format: {}", msg),
            ApiError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ApiError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::TimeoutError(err.to_string())
        } else if err.is_status() {
            match err.status() {
                Some(status) => match status.as_u16() {
                    400 => ApiError::BadRequest(err.to_string()),
                    401 => ApiError::AuthenticationError(err.to_string()),
                    403 => ApiError::AuthorizationError(err.to_string()),
                    404 => ApiError::NotFound(err.to_string()),
                    429 => ApiError::RateLimitExceeded(err.to_string()),
                    500..=599 => ApiError::ServerError(err.to_string()),
                    _ => ApiError::NetworkError(err.to_string()),
                },
                None => ApiError::NetworkError(err.to_string()),
            }
        } else {
            ApiError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::NetworkError(err.to_string())
    }
}

impl ApiError {
    pub fn from_response(status: reqwest::StatusCode, body: String) -> Self {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => {
                json.get("msg")
                    .and_then(|v| v.as_str())
                    .or_else(|| json.get("message").and_then(|v| v.as_str()))
                    .unwrap_or(&body)
                    .to_string()
            }
            Err(_) => body,
        };

        match status.as_u16() {
            400 => ApiError::BadRequest(error_message),
            401 => ApiError::AuthenticationError(error_message),
            403 => ApiError::AuthorizationError(error_message),
            404 => ApiError::NotFound(error_message),
            429 => ApiError::RateLimitExceeded(error_message),
            500..=599 => ApiError::ServerError(error_message),
            _ => ApiError::NetworkError(error_message),
        }
    }

    pub fn error_code(&self) -> i32 {
        match self {
            ApiError::NetworkError(_) => 1000,
            ApiError::TimeoutError(_) => 1001,
            ApiError::AuthenticationError(_) => 1002,
            ApiError::AuthorizationError(_) => 1003,
            ApiError::BadRequest(_) => 1004,
            ApiError::NotFound(_) => 1005,
            ApiError::RateLimitExceeded(_) => 1006,
            ApiError::ServerError(_) => 1007,
            ApiError::InvalidResponseFormat(_) => 1008,
            ApiError::SerializationError(_) => 1009,
            ApiError::ConfigError(_) => 1010,
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::NetworkError(_) => true,
            ApiError::TimeoutError(_) => true,
            ApiError::RateLimitExceeded(_) => true,
            ApiError::ServerError(_) => true,
            _ => false,
        }
    }

    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            ApiError::RateLimitExceeded(_) => 5000,
            ApiError::NetworkError(_) => 1000,
            ApiError::TimeoutError(_) => 2000,
            ApiError::ServerError(_) => 3000,
            _ => 0,
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            ApiError::NetworkError(_) => "网络连接失败，请检查网络后重试".to_string(),
            ApiError::TimeoutError(_) => "请求超时，请稍后重试".to_string(),
            ApiError::AuthenticationError(_) => "认证失败，请检查API密钥".to_string(),
            ApiError::AuthorizationError(_) => "权限不足，请检查权限设置".to_string(),
            ApiError::BadRequest(_) => "请求参数错误，请检查输入".to_string(),
            ApiError::NotFound(_) => "请求的资源不存在".to_string(),
            ApiError::RateLimitExceeded(_) => "请求频率过高，请稍后再试".to_string(),
            ApiError::ServerError(_) => "服务器内部错误，请稍后重试".to_string(),
            ApiError::InvalidResponseFormat(_) => "响应格式错误，请检查API文档".to_string(),
            ApiError::SerializationError(_) => "数据序列化失败".to_string(),
            ApiError::ConfigError(_) => "配置错误，请检查配置".to_string(),
        }
    }
}
