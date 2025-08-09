use serde::{Deserialize, Serialize};

// NOTE:
// 当前文件包含未来可能开放/工具层尚未调用的模型结构体与辅助 new 方法。
// 为减少编译警告噪音并保持后续扩展便利，暂时允许 dead_code。
// 当相关功能落地后，可逐步移除 allow 并删除未使用结构。
#[allow(dead_code)]
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBase {
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub content: String,
    pub url: Option<String>,
    pub format_type: Option<String>,
}

impl DocumentBase {
    pub fn new(
        file_name: String,
        file_type: String,
        file_size: u64,
        content: String,
        url: Option<String>,
        format_type: Option<String>,
    ) -> Self {
        Self {
            file_name,
            file_type,
            file_size,
            content,
            url,
            format_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub source_type: String,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
}

impl SourceInfo {
    pub fn new(source_type: String) -> Self {
        Self {
            source_type,
            source_url: None,
            source_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStrategy {
    pub chunk_type: i32,
    pub max_tokens: usize,
    pub chunk_overlap: usize,
}

impl ChunkStrategy {
    pub fn new(chunk_type: i32, max_tokens: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_type,
            max_tokens,
            chunk_overlap,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentCreateRequest {
    pub dataset_id: String,
    pub document_bases: Vec<DocumentBase>,
    pub chunk_strategy: Option<ChunkStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_type: Option<String>,
}

impl KnowledgeDocumentCreateRequest {
    pub fn new(dataset_id: String, document_bases: Vec<DocumentBase>) -> Self {
        Self {
            dataset_id,
            document_bases,
            chunk_strategy: None,
            format_type: None,
        }
    }

    pub fn into_json(&self) -> Result<serde_json::Value, crate::api::error::ApiError> {
        let mut map = serde_json::Map::new();
        map.insert("dataset_id".to_string(), serde_json::Value::String(self.dataset_id.clone()));
        
        let document_bases_json: Vec<serde_json::Value> = self.document_bases
            .iter()
            .map(|doc| {
                let mut doc_map = serde_json::Map::new();
                doc_map.insert("file_name".to_string(), serde_json::Value::String(doc.file_name.clone()));
                doc_map.insert("file_type".to_string(), serde_json::Value::String(doc.file_type.clone()));
                doc_map.insert("file_size".to_string(), serde_json::Value::Number(doc.file_size.into()));
                doc_map.insert("content".to_string(), serde_json::Value::String(doc.content.clone()));
                
                if let Some(url) = &doc.url {
                    doc_map.insert("url".to_string(), serde_json::Value::String(url.clone()));
                }
                
                if let Some(format_type) = &doc.format_type {
                    doc_map.insert("format_type".to_string(), serde_json::Value::String(format_type.clone()));
                }
                
                serde_json::Value::Object(doc_map)
            })
            .collect();
        
        map.insert("document_bases".to_string(), serde_json::Value::Array(document_bases_json));
        
        if let Some(chunk_strategy) = &self.chunk_strategy {
            let mut chunk_map = serde_json::Map::new();
            chunk_map.insert("chunk_type".to_string(), serde_json::Value::Number(chunk_strategy.chunk_type.into()));
            chunk_map.insert("max_tokens".to_string(), serde_json::Value::Number((chunk_strategy.max_tokens as i64).into()));
            chunk_map.insert("chunk_overlap".to_string(), serde_json::Value::Number((chunk_strategy.chunk_overlap as i64).into()));
            map.insert("chunk_strategy".to_string(), serde_json::Value::Object(chunk_map));
        }
        
        if let Some(format_type) = &self.format_type {
            map.insert("format_type".to_string(), serde_json::Value::String(format_type.clone()));
        }
        
        Ok(serde_json::Value::Object(map))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentCreateResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<KnowledgeDocumentData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentData {
    pub document_ids: Vec<String>,
    pub dataset_id: String,
    pub total_documents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub bot_id: String,
    pub user_id: String,
    pub additional_messages: Option<Vec<ChatMessage>>,
    pub stream: Option<bool>,
    pub custom_variables: Option<HashMap<String, String>>,
}

impl ChatCompletionRequest {
    pub fn new(bot_id: String, user_id: String) -> Self {
        Self {
            bot_id,
            user_id,
            additional_messages: None,
            stream: Some(false),
            custom_variables: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<ChatCompletionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionData {
    pub messages: Vec<ChatMessage>,
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub bot_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub create_time: i64,
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub model_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub supported_formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<KnowledgeBaseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseData {
    pub dataset_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<DocumentUploadData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUploadData {
    pub document_ids: Vec<String>,
    pub dataset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListKnowledgeBasesResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<Vec<KnowledgeBase>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

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
