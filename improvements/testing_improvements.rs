// 测试改进示例
use crate::api::{CozeApiClient, ApiError};
use crate::tools::coze_tools::CozeTools;
use crate::knowledge::KnowledgeManager;
use mockito::{Server, Mock};
use serde_json::json;
use std::sync::Arc;
use tokio;

// 测试工具和辅助函数
pub struct TestHelper {
    pub mock_server: Server,
    pub client: CozeApiClient,
    pub tools: CozeTools,
}

impl TestHelper {
    pub async fn new() -> Self {
        let mock_server = Server::new_async().await;
        let client = CozeApiClient::new(
            mock_server.url(),
            "pat_test_token".to_string(),
        ).unwrap();
        
        let knowledge_manager = Arc::new(KnowledgeManager::new(
            client.clone(),
            crate::knowledge::KnowledgeConfig::default(),
        ));
        
        let tools = CozeTools::new(
            Arc::new(client.clone()),
            knowledge_manager,
            "test_space_id".to_string(),
        );
        
        Self {
            mock_server,
            client,
            tools,
        }
    }
    
    pub async fn mock_success_response(&mut self, endpoint: &str, response_body: serde_json::Value) -> Mock {
        self.mock_server
            .mock("GET", endpoint)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await
    }
    
    pub async fn mock_error_response(&mut self, endpoint: &str, status: usize, error_msg: &str) -> Mock {
        self.mock_server
            .mock("GET", endpoint)
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(json!({"msg": error_msg}).to_string())
            .create_async()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolResult;
    
    #[tokio::test]
    async fn test_list_workspaces_success() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "list": [
                    {
                        "workspace_id": "ws_123",
                        "name": "Test Workspace",
                        "description": "A test workspace"
                    }
                ],
                "total": 1
            }
        });
        
        let mock = helper.mock_success_response("/v1/workspaces", mock_response).await;
        
        let result = helper.tools.list_workspaces(None).await;
        
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(call_result.content.is_some());
        assert!(call_result.structured_content.is_some());
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_list_workspaces_error() {
        let mut helper = TestHelper::new().await;
        
        let mock = helper.mock_error_response("/v1/workspaces", 401, "Unauthorized").await;
        
        let result = helper.tools.list_workspaces(None).await;
        
        assert!(result.is_ok()); // 工具调用本身成功，但返回错误结果
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true));
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_list_knowledge_bases_with_params() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "datasets": [
                    {
                        "dataset_id": "kb_123",
                        "name": "Test Knowledge Base",
                        "description": "A test knowledge base",
                        "doc_count": 5,
                        "create_time": 1640995200
                    }
                ],
                "total": 1
            }
        });
        
        let mock = helper.mock_success_response("/v1/datasets", mock_response).await;
        
        let args = json!({
            "space_id": "test_space_id",
            "name": "Test",
            "page_num": 1,
            "page_size": 10
        });
        
        let result = helper.tools.list_knowledge_bases(Some(args)).await;
        
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        
        // 验证结构化内容
        if let Some(structured) = call_result.structured_content {
            assert_eq!(structured["total"], 1);
            assert!(structured["items"].is_array());
            let items = structured["items"].as_array().unwrap();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0]["dataset_id"], "kb_123");
        }
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_create_knowledge_base() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "dataset_id": "kb_new_123",
                "name": "New Knowledge Base",
                "description": "A newly created knowledge base"
            }
        });
        
        let mock = helper.mock_server
            .mock("POST", "/v1/datasets")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;
        
        let args = json!({
            "name": "New Knowledge Base",
            "description": "A newly created knowledge base",
            "space_id": "test_space_id",
            "permission": "private"
        });
        
        let result = helper.tools.create_knowledge_base_v2(Some(args)).await;
        
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_list_bots_with_pagination() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "list": [
                    {
                        "bot_id": "bot_123",
                        "name": "Test Bot",
                        "status": "published"
                    },
                    {
                        "bot_id": "bot_456",
                        "name": "Another Bot",
                        "status": "draft"
                    }
                ],
                "total": 2
            }
        });
        
        let mock = helper.mock_success_response("/v1/bots", mock_response).await;
        
        let args = json!({
            "workspace_id": "test_space_id",
            "page": 1,
            "page_size": 20
        });
        
        let result = helper.tools.list_bots(Some(args)).await;
        
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        
        // 验证内容包含预期信息
        if let Some(content) = &call_result.content {
            let text = &content[0];
            if let rmcp::model::Content::Text { text } = text {
                assert!(text.contains("找到 2 个 Bot"));
                assert!(text.contains("Test Bot"));
                assert!(text.contains("Another Bot"));
            }
        }
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_api_client_error_handling() {
        let mut helper = TestHelper::new().await;
        
        // 测试 401 错误
        let mock = helper.mock_error_response("/v1/test", 401, "Invalid API key").await;
        
        let result = helper.client.execute_request(crate::models::CozeApiRequest {
            endpoint: "/v1/test".to_string(),
            method: crate::models::HttpMethod::Get,
            headers: Default::default(),
            params: Default::default(),
            body: None,
        }).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.success);
        assert_eq!(response.status_code, 401);
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_api_client_timeout() {
        let mut helper = TestHelper::new().await;
        
        // 模拟超时响应
        let mock = helper.mock_server
            .mock("GET", "/v1/slow")
            .with_status(200)
            .with_delay(std::time::Duration::from_secs(35)) // 超过默认30秒超时
            .with_body("{}")
            .create_async()
            .await;
        
        let result = helper.client.execute_request(crate::models::CozeApiRequest {
            endpoint: "/v1/slow".to_string(),
            method: crate::models::HttpMethod::Get,
            headers: Default::default(),
            params: Default::default(),
            body: None,
        }).await;
        
        // 应该返回超时错误
        assert!(result.is_err());
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_knowledge_manager_upload_document() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "document_id": "doc_123",
                "status": "processing"
            }
        });
        
        let mock = helper.mock_server
            .mock("POST", "/v1/datasets/kb_123/documents")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;
        
        let documents = vec![
            crate::api::knowledge_models::DocumentBase {
                document_name: "test.pdf".to_string(),
                document_source: crate::api::knowledge_models::DocumentSource::LocalFile {
                    file_path: "/path/to/test.pdf".to_string(),
                },
            }
        ];
        
        let knowledge_manager = KnowledgeManager::new(
            helper.client.clone(),
            crate::knowledge::KnowledgeConfig::default(),
        );
        
        let result = knowledge_manager.upload_document("kb_123", documents).await;
        
        assert!(result.is_ok());
        
        mock.assert_async().await;
    }
}

// 集成测试
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::CozeServer;
    use rmcp::model::{CallToolRequestParam, ListToolsResult};
    use rmcp::service::RequestContext;
    use rmcp::handler::server::ServerHandler;
    
    async fn create_test_server() -> CozeServer {
        CozeServer::new(
            "https://api.coze.cn".to_string(),
            "pat_test_token".to_string(),
            "test_space_id".to_string(),
        ).unwrap()
    }
    
    #[tokio::test]
    async fn test_server_list_tools() {
        let server = create_test_server().await;
        let context = RequestContext::default();
        
        let result = server.list_tools(None, context).await;
        
        assert!(result.is_ok());
        let tools_result = result.unwrap();
        assert!(!tools_result.tools.is_empty());
        
        // 验证包含核心工具
        let tool_names: Vec<&str> = tools_result.tools.iter()
            .map(|t| t.name.as_str())
            .collect();
        
        assert!(tool_names.contains(&"set_api_key"));
        assert!(tool_names.contains(&"list_workspaces"));
        assert!(tool_names.contains(&"list_knowledge_bases"));
    }
    
    #[tokio::test]
    async fn test_server_call_ping_tool() {
        let server = create_test_server().await;
        let context = RequestContext::default();
        
        let params = CallToolRequestParam {
            name: "ping".to_string(),
            arguments: None,
        };
        
        let result = server.call_tool(params, context).await;
        
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        
        if let Some(structured) = call_result.structured_content {
            assert_eq!(structured["ok"], true);
        }
    }
}

// 性能测试
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_concurrent_api_calls() {
        let mut helper = TestHelper::new().await;
        
        let mock_response = json!({
            "data": {
                "list": [],
                "total": 0
            }
        });
        
        let mock = helper.mock_success_response("/v1/workspaces", mock_response).await;
        
        let start = Instant::now();
        
        // 并发调用10次
        let tasks: Vec<_> = (0..10).map(|_| {
            let tools = helper.tools.clone();
            tokio::spawn(async move {
                tools.list_workspaces(None).await
            })
        }).collect();
        
        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();
        
        // 验证所有调用都成功
        for result in results {
            assert!(result.is_ok());
            let call_result = result.unwrap().unwrap();
            assert_eq!(call_result.is_error, Some(false));
        }
        
        // 性能断言（应该在合理时间内完成）
        assert!(duration < std::time::Duration::from_secs(5));
        
        mock.assert_async().await;
    }
    
    #[tokio::test]
    async fn test_cache_performance() {
        let client = CozeApiClient::new(
            "https://api.coze.cn".to_string(),
            "pat_test_token".to_string(),
        ).unwrap().with_cache_config(true, std::time::Duration::from_secs(60));
        
        let test_data = json!({"large_data": vec![0; 1000]});
        
        let start = Instant::now();
        
        // 设置1000个缓存项
        for i in 0..1000 {
            client.set_cache(&format!("key_{}", i), test_data.clone()).await;
        }
        
        let set_duration = start.elapsed();
        
        let start = Instant::now();
        
        // 读取1000个缓存项
        for i in 0..1000 {
            let _ = client.get_from_cache(&format!("key_{}", i)).await;
        }
        
        let get_duration = start.elapsed();
        
        // 性能断言
        assert!(set_duration < std::time::Duration::from_secs(1));
        assert!(get_duration < std::time::Duration::from_millis(100));
        
        let (total, expired) = client.cache_stats().await;
        assert_eq!(total, 1000);
        assert_eq!(expired, 0);
    }
}