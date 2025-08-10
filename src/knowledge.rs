use crate::api::client::CozeApiClient;
use crate::api::error::ApiError;
// Upload-related types removed; keep ChunkStrategy if reintroduced later

/// Configuration for knowledge management
#[derive(Debug, Clone)]
pub struct KnowledgeConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    // Removed max_file_size (no enforcement logic present)
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            chunk_size: 800,
            chunk_overlap: 100,
            // max_file_size removed
        }
    }
}

/// Knowledge manager for handling knowledge base operations
#[derive(Debug, Clone)]
pub struct KnowledgeManager {
    client: CozeApiClient,
    config: KnowledgeConfig,
}

impl KnowledgeManager {
    /// Create a new knowledge manager
    pub fn new(client: CozeApiClient, config: KnowledgeConfig) -> Self {
        Self { client, config }
    }

    // Upload document methods removed (deprecated)

    /// 创建知识库 (使用标准 v1/datasets API)
    ///
    /// 根据 Coze API 文档创建知识库，支持文本和图片类型
    ///
    /// # 参数
    /// - `name`: 知识库名称，长度不超过 100 个字符
    /// - `space_id`: 知识库所在空间的唯一标识
    /// - `format_type`: 知识库类型，0-文本类型，2-图片类型
    /// - `description`: 知识库描述信息（可选）
    /// - `file_id`: 知识库图标（可选），需传入【上传文件】API 返回的 file_id
    pub async fn create_dataset(
        &self,
        name: &str,
        space_id: &str,
        format_type: i32,
        description: Option<&str>,
        file_id: Option<&str>,
    ) -> Result<crate::api::knowledge_models::CreateDatasetResponse, ApiError> {
        use crate::api::knowledge_models::CreateDatasetRequest;

        let request = CreateDatasetRequest {
            name: name.to_string(),
            space_id: space_id.to_string(),
            format_type,
            description: description.map(|d| d.to_string()),
            file_id: file_id.map(|f| f.to_string()),
        };

        self.client.create_dataset(request).await
    }

    /// 创建文本类型知识库
    pub async fn create_text_dataset(
        &self,
        name: &str,
        space_id: &str,
        description: Option<&str>,
        file_id: Option<&str>,
    ) -> Result<crate::api::knowledge_models::CreateDatasetResponse, ApiError> {
        self.create_dataset(name, space_id, 0, description, file_id)
            .await
    }

    /// 创建图片类型知识库
    pub async fn create_image_dataset(
        &self,
        name: &str,
        space_id: &str,
        description: Option<&str>,
        file_id: Option<&str>,
    ) -> Result<crate::api::knowledge_models::CreateDatasetResponse, ApiError> {
        self.create_dataset(name, space_id, 2, description, file_id)
            .await
    }

    /// Create knowledge base with permission
    pub async fn create_knowledge_base_with_permission(
        &self,
        name: &str,
        description: Option<&str>,
        space_id: Option<&str>,
        permission: Option<i32>,
    ) -> Result<serde_json::Value, ApiError> {
        self.client
            .create_knowledge_base_with_permission(
                name.to_string(),
                description.map(|d| d.to_string()),
                space_id.map(|s| s.to_string()),
                permission,
            )
            .await
    }

    /// Get current configuration
    pub fn get_config(&self) -> &KnowledgeConfig {
        &self.config
    }

    // update_config removed (unused)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::CozeApiClient;

    #[test]
    fn test_knowledge_config_default() {
        let config = KnowledgeConfig::default();
        assert_eq!(config.chunk_size, 800);
        assert_eq!(config.chunk_overlap, 100);
        // max_file_size assertion removed
    }

    #[test]
    fn test_knowledge_manager_creation() {
        let client =
            CozeApiClient::new("http://localhost".to_string(), "test-token".to_string()).unwrap();
        let config = KnowledgeConfig::default();
        let manager = KnowledgeManager::new(client, config);
        assert_eq!(manager.get_config().chunk_size, 800);
    }
}
