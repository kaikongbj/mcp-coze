/// Base URL for Coze API
pub const COZE_BASE_URL: &str = "https://api.coze.cn";

// NOTE: Slimmed to only actively used endpoint groups to reduce warnings.

pub mod conversation {
    pub const LIST_CONVERSATIONS: &str = "/v1/conversations"; // used by list_conversations_v1
}

pub mod datasets_v1 {
    pub const LIST_DATASETS: &str = "/v1/datasets"; // canonical dataset listing
    pub const CREATE_DATASETS: &str = "/v1/datasets"; // 创建知识库 API
}

pub mod datasets_cn {
    pub const GET_KNOWLEDGE_BASE: &str = "/open_api/knowledge/dataset"; // still used by get_dataset_cn for detail fetch
}

pub mod bots {
    pub const LIST_BOTS: &str = "/v1/bots"; // used by list_bots
}

pub mod chat {
    pub const CHAT_V3: &str = "/v3/chat";
    pub const CHAT_V3_STREAM: &str = "/v3/chat";
    pub const GET_CHAT_DETAIL: &str = "/v3/chat/retrieve"; // 获取对话详情
    pub const GET_CHAT_MESSAGES: &str = "/v3/chat/message/list"; // 获取对话消息列表
}

// Chat completion endpoints removed (unused)
pub const KNOWLEDGE_DOCUMENT_CREATE_URL: &str = "/open_api/knowledge/document/create"; // (legacy upload removed; retain if tool layer still references)

// Removed unused request/response structs
