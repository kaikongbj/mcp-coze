use serde::{Deserialize, Serialize};

// NOTE:
// 当前文件包含未来可能开放/工具层尚未调用的模型结构体与辅助 new 方法。
// 为减少编译警告噪音并保持后续扩展便利，暂时允许 dead_code。
// 当相关功能落地后，可逐步移除 allow 并删除未使用结构。
// (HashMap usage removed after pruning chat models)

// Pruned unused chat & ancillary model structs to reduce warnings.

// ================= CN Spec Aligned Structures (upload v2) =================
// According to API_upload.md (document_bases -> name + source_info{ file_base64, file_type } ...)
// We keep them separate to avoid breaking existing internal DocumentBase usage.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_source: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file_id: Option<String>,
}

impl SourceInfo {
    pub fn file_base64(file_base64: String, file_type: String) -> Self {
        let mut fp = file_type.to_lowercase();
        if file_type == "md" {
            fp = "txt".to_string();
        }
        Self {
            file_base64: Some(file_base64),
            file_type: Some(fp),
            web_url: None,
            document_source: None,
            source_file_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBaseCn {
    pub name: String,
    pub source_info: SourceInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_rule: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStrategyCn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_extra_spaces: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_urls_emails: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption_type: Option<i32>,
}

impl ChunkStrategyCn {
    pub fn text(separator: String, max_tokens: i64, chunk_type: i32) -> Self {
        Self {
            chunk_type: Some(chunk_type),
            separator: Some(separator),
            max_tokens: Some(max_tokens),
            remove_extra_spaces: Some(false),
            remove_urls_emails: Some(false),
            caption_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentUploadRequestCn {
    pub dataset_id: String,
    pub document_bases: Vec<DocumentBaseCn>,
    pub chunk_strategy: ChunkStrategyCn,
    pub format_type: i32, // 0 text, 2 image
}

impl KnowledgeDocumentUploadRequestCn {
    /// Sanitize request to comply with API rules:
    /// - If chunk_type == 0 (auto), omit manual chunk fields.
    /// - Remove any unexpected extraneous fields in document_bases (serde already restricts).
    pub fn sanitized(mut self) -> Self {
        if matches!(self.chunk_strategy.chunk_type, Some(0)) {
            // Ensure manual fields are cleared
            self.chunk_strategy.separator = None;
            self.chunk_strategy.max_tokens = None;
            self.chunk_strategy.remove_extra_spaces = None;
            self.chunk_strategy.remove_urls_emails = None;
            self.chunk_strategy.caption_type = None; // only for images / manual caption
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentUploadResponseCn {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_infos: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

// === 创建知识库 API 相关模型 (基于 v1/datasets 规范) ===

/// 创建知识库请求 (符合 POST /v1/datasets API 文档)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    /// 知识库名称，长度不超过 100 个字符
    pub name: String,
    /// 知识库所在空间的唯一标识
    pub space_id: String,
    /// 知识库类型：0-文本类型，2-图片类型
    pub format_type: i32,
    /// 知识库描述信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 知识库图标（可选），需传入【上传文件】API 返回的 file_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

impl CreateDatasetRequest {
    /// 创建文本类型知识库请求
    #[allow(dead_code)]
    pub fn new_text(name: String, space_id: String, description: Option<String>) -> Self {
        Self {
            name,
            space_id,
            format_type: 0, // 文本类型
            description,
            file_id: None,
        }
    }

    /// 创建图片类型知识库请求
    #[allow(dead_code)]
    pub fn new_image(name: String, space_id: String, description: Option<String>) -> Self {
        Self {
            name,
            space_id,
            format_type: 2, // 图片类型
            description,
            file_id: None,
        }
    }

    /// 设置知识库图标
    #[allow(dead_code)]
    pub fn with_icon(mut self, file_id: String) -> Self {
        self.file_id = Some(file_id);
        self
    }
}

/// 创建知识库响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetOpenApiData {
    /// 新知识库的唯一标识
    pub dataset_id: String,
}

/// 响应详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDetail {
    /// 请求日志 ID，用于错误排查
    pub logid: String,
}

/// 创建知识库响应 (符合 POST /v1/datasets API 文档)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetResponse {
    /// 状态码，0 表示调用成功
    pub code: i64,
    /// 状态信息，失败时返回错误详情
    pub msg: String,
    /// 返回内容，包含新知识库的 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<CreateDatasetOpenApiData>,
    /// 本次请求的日志 ID，用于异常排查
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ResponseDetail>,
}
