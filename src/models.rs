use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ParameterDefinition>,
    pub returns: Vec<ReturnDefinition>,
    pub metadata: ToolMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ParameterType,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ReturnType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReturnType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRuleType {
    MinLength,
    MaxLength,
    MinValue,
    MaxValue,
    Pattern,
    EnumValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub user_id: String,
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CozeApiRequest {
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, serde_json::Value>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CozeApiResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub version: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub total_tools: usize,
}
