use serde::{Deserialize, Serialize};
use std::collections::HashMap; // retained for potential header/maps usage

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

// Pruned unused high-level MCP tool metadata structs to minimize warnings.
