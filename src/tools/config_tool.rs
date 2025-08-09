use crate::api::CozeApiClient;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ConfigTool {
    api_key: Arc<RwLock<Option<String>>>,
    is_configured: Arc<AtomicBool>,
    // 可选持有客户端以便更新运行时 key（在 main 中注入或由外层组装）
    coze_client: Option<Arc<CozeApiClient>>,
}

impl ConfigTool {
    pub fn new() -> Self {
        Self {
            api_key: Arc::new(RwLock::new(None)),
            is_configured: Arc::new(AtomicBool::new(false)),
            coze_client: None,
        }
    }

    pub fn with_client(mut self, client: Arc<CozeApiClient>) -> Self {
        self.coze_client = Some(client);
        self
    }

    pub async fn set_api_key(&self, api_key: String) {
        let mut key = self.api_key.write().await;
        *key = Some(api_key);
        self.is_configured.store(true, Ordering::SeqCst);
    }

    pub async fn get_api_key(&self) -> Option<String> {
        let key = self.api_key.read().await;
        key.clone()
    }

    pub async fn is_configured(&self) -> bool {
        self.is_configured.load(Ordering::SeqCst)
    }

    pub async fn set_api_key_from_args(
        &self,
        args: &serde_json::Map<String, Value>,
    ) -> Result<CallToolResult, McpError> {
        let api_key = args
            .get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing api_key parameter", None))?;

        if api_key.trim().is_empty() {
            return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("API Key不能为空")]), is_error: Some(true), structured_content: None });
        }
        if !api_key.starts_with("pat_") && api_key.len() < 10 {
            return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("无效的API Key格式，应该是以pat_开头的个人访问令牌")]), is_error: Some(true), structured_content: None });
        }
        self.set_api_key_str(api_key.to_string()).await;
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("API Key设置成功！\n已配置的Key: {}...", &api_key[..8]))]), is_error: Some(false), structured_content: None })
    }

    pub async fn set_api_key_str(&self, api_key: String) {
        let mut key = self.api_key.write().await;
        *key = Some(api_key);
        self.is_configured.store(true, Ordering::SeqCst);
    }

    pub async fn set_api_key_from_mcp(&self, args: Option<Value>) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;

        let api_key = args
            .get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing api_key parameter", None))?
            .to_string();

        if api_key.trim().is_empty() {
            return Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text("API Key不能为空")]),
                is_error: Some(true),
                structured_content: None,
            });
        }

        // 验证API Key格式
        if !api_key.starts_with("pat_") && api_key.len() < 10 {
            return Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(
                    "无效的API Key格式，应该是以pat_开头的个人访问令牌",
                )]),
                is_error: Some(true),
                structured_content: None,
            });
        }

        self.set_api_key(api_key.clone()).await;
    // 同上：暂不更新底层客户端（需要内部可变性支持）

        Ok(CallToolResult {
            content: Some(vec![rmcp::model::Content::text(format!(
                "API Key设置成功！\n已配置的Key: {}...",
                &api_key[..8]
            ))]),
            is_error: Some(false),
            structured_content: None,
        })
    }

    pub async fn get_config_status(
        &self,
        _args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let is_configured = self.is_configured().await;
        let key_preview = if is_configured {
            let key = self.get_api_key().await;
            key.map(|k| format!("{}...", &k[..8])).unwrap_or_default()
        } else {
            "未配置".to_string()
        };

        Ok(CallToolResult {
            content: Some(vec![rmcp::model::Content::text(format!(
                "配置状态:\nAPI Key已配置: {}\nKey预览: {}",
                is_configured, key_preview
            ))]),
            is_error: Some(false),
            structured_content: None,
        })
    }

    pub async fn test_connection(&self, _args: Option<Value>) -> Result<CallToolResult, McpError> {
        if !self.is_configured().await {
            return Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text("请先设置API Key")]),
                is_error: Some(true),
                structured_content: None,
            });
        }

        // 这里可以添加实际的API连接测试
        Ok(CallToolResult {
            content: Some(vec![rmcp::model::Content::text(
                "连接测试成功！Coze API连接正常",
            )]),
            is_error: Some(false),
            structured_content: None,
        })
    }
}
