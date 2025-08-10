use serde::{Deserialize, Serialize};

/// Bot 发布状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BotPublishStatus {
    /// 全部状态
    All,
    /// 已发布正式版
    #[default]
    PublishedOnline,
    /// 已发布草稿
    PublishedDraft,
    /// 未发布
    UnpublishedDraft,
}

/// Bot 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    /// Bot ID
    pub id: String,
    /// Bot 名称
    pub name: String,
    /// Bot 图标 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// 更新时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
    /// Bot 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 是否已发布
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,
    /// 所有者用户 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,
}

/// Bot 列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBotsRequest {
    /// 工作空间 ID（必选）
    pub workspace_id: String,
    /// 发布状态筛选（可选，默认为 published_online）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_status: Option<BotPublishStatus>,
    /// 渠道 ID（可选，默认 1024 为 API 渠道）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<String>,
    /// 分页页码（可选，默认 1）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_num: Option<u32>,
    /// 每页数据量（可选，默认 20）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

/// Bot 列表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotListData {
    /// Bot 列表
    pub items: Vec<BotInfo>,
    /// 总数量
    pub total: u32,
}

/// Bot 列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBotsResponse {
    /// Bot 列表数据
    pub data: BotListData,
    /// 状态码（0 表示成功）
    pub code: i64,
    /// 状态描述
    pub msg: String,
    /// 调试信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

impl ListBotsRequest {
    /// 创建新的 Bot 列表查询请求
    pub fn new(workspace_id: String) -> Self {
        Self {
            workspace_id,
            publish_status: Some(BotPublishStatus::PublishedOnline),
            connector_id: Some("1024".to_string()),
            page_num: Some(1),
            page_size: Some(20),
        }
    }

    /// 设置发布状态
    pub fn with_publish_status(mut self, status: BotPublishStatus) -> Self {
        self.publish_status = Some(status);
        self
    }

    /// 设置渠道 ID
    pub fn with_connector_id(mut self, connector_id: String) -> Self {
        self.connector_id = Some(connector_id);
        self
    }

    /// 设置分页参数
    pub fn with_page(mut self, page_num: u32, page_size: u32) -> Self {
        self.page_num = Some(page_num);
        self.page_size = Some(page_size);
        self
    }

    /// 构建查询参数字符串
    pub fn to_query_params(&self) -> String {
        let mut params = vec![format!("workspace_id={}", urlencoding::encode(&self.workspace_id))];
        
        if let Some(ref status) = self.publish_status {
            let status_str = match status {
                BotPublishStatus::All => "all",
                BotPublishStatus::PublishedOnline => "published_online",
                BotPublishStatus::PublishedDraft => "published_draft",
                BotPublishStatus::UnpublishedDraft => "unpublished_draft",
            };
            params.push(format!("publish_status={status_str}"));
        }
        
        if let Some(ref connector_id) = self.connector_id {
            params.push(format!("connector_id={}", urlencoding::encode(connector_id)));
        }
        
        if let Some(page_num) = self.page_num {
            params.push(format!("page_num={page_num}"));
        }
        
        if let Some(page_size) = self.page_size {
            params.push(format!("page_size={page_size}"));
        }
        
        params.join("&")
    }
}
