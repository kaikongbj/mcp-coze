use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息角色类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// 消息内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Image,
    File,
    Audio,
}

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetaData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_info: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_info: Option<serde_json::Value>,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<MessageMetaData>,
}

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub bot_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    pub additional_messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_variables: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_save_history: Option<bool>,
}

/// 聊天响应中的使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

/// 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_action: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatUsage>,
}

/// 流式聊天事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    ConversationMessageDelta,
    ConversationChatCompleted,
    ConversationChatInProgress,
    ConversationChatFailed,
    ConversationChatRequiresAction,
    Done,
    Error,
}

/// 流式聊天增量内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
}

/// 流式聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChatResponse {
    pub event: StreamEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<StreamDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<serde_json::Value>,
}

impl ChatMessage {
    /// 创建纯文本消息
    pub fn text(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content: Some(content),
            content_type: None,
            object_string: None,
            meta_data: None,
        }
    }
    
    /// 创建包含文件/图片的消息
    pub fn object_string(role: MessageRole, object_string: String) -> Self {
        Self {
            role,
            content: None,
            content_type: None,
            object_string: Some(object_string),
            meta_data: None,
        }
    }
}

impl ChatRequest {
    pub fn new(bot_id: String, message: String) -> Self {
        Self {
            bot_id,
            user_id: None,
            conversation_id: None,
            additional_messages: vec![ChatMessage::text(MessageRole::User, message)],
            stream: Some(false),
            custom_variables: None,
            auto_save_history: Some(true),
        }
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn with_conversation_id(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_custom_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.custom_variables = Some(variables);
        self
    }
}
