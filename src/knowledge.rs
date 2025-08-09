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

    /// Create knowledge base with permission
    pub async fn create_knowledge_base_with_permission(
        &self,
        name: &str,
        description: Option<&str>,
        space_id: Option<&str>,
        permission: Option<i32>,
    ) -> Result<serde_json::Value, ApiError> {
        let request = crate::api::endpoints::CreateKnowledgeRequest {
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            space_id: space_id.map(|s| s.to_string()),
            permission,
        };

        self.client
            .create_knowledge_base_with_permission(
                request.name,
                request.description,
                request.space_id,
                request.permission,
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
    let client = CozeApiClient::new("http://localhost".to_string(), "test-token".to_string()).unwrap();
        let config = KnowledgeConfig::default();
        let manager = KnowledgeManager::new(client, config);
        assert_eq!(manager.get_config().chunk_size, 800);
    }
}