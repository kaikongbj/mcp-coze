/// Base URL for Coze API
pub const COZE_BASE_URL: &str = "https://api.coze.cn";

// NOTE: Slimmed to only actively used endpoint groups to reduce warnings.

pub mod conversation {
    pub const LIST_CONVERSATIONS: &str = "/v1/conversations"; // used by list_conversations_v1
    pub const GET_CONVERSATION: &str = "/v1/conversation/retrieve"; // used by retrieve_conversation_v1
    pub const GET_MESSAGES: &str = "/v3/chat/conversations/{conversation_id}/messages"; // used by get_conversation_messages_v3
}

pub mod datasets_v1 {
    pub const LIST_DATASETS: &str = "/v1/datasets"; // canonical dataset listing
}

pub mod datasets_cn {
    pub const GET_KNOWLEDGE_BASE: &str = "/open_api/knowledge/dataset"; // still used by get_dataset_cn for detail fetch
    pub const CREATE_KNOWLEDGE_BASE: &str = "/open_api/knowledge/datasets"; // used by create_knowledge_base_with_permission
}

pub const CHAT_COMPLETION_URL: &str = "/v3/chat"; // used by chat_completion
pub const CHAT_COMPLETION_CN: &str = "/v3/chat"; // used by chat_completion_cn
pub const KNOWLEDGE_DOCUMENT_CREATE_URL: &str = "/open_api/knowledge/document/create"; // (legacy upload removed; retain if tool layer still references)

/// Common request/response structures
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateKnowledgeRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadDocumentRequest {
    pub dataset_id: String,
    pub document_bases: Vec<crate::api::knowledge_models::DocumentBase>,
    pub chunk_strategy: Option<crate::api::knowledge_models::ChunkStrategy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}
