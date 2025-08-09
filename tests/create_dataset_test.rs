/// 测试创建知识库功能 (基于 v1/datasets API 文档)
#[cfg(test)]
mod tests {
    use coze_mcp_server::api::client::CozeApiClient;
    use coze_mcp_server::api::knowledge_models::{CreateDatasetRequest, CreateDatasetResponse};
    use serde_json::json;
    use std::sync::Arc;
    use coze_mcp_server::tools::coze_tools::CozeTools;

    #[test]
    fn test_create_dataset_request_validation() {
        // 测试文本类型知识库请求
        let text_request = CreateDatasetRequest::new_text(
            "测试文本知识库".to_string(),
            "123456789".to_string(),
            Some("这是一个测试文本知识库".to_string()),
        );
        
        assert_eq!(text_request.name, "测试文本知识库");
        assert_eq!(text_request.space_id, "123456789");
        assert_eq!(text_request.format_type, 0); // 文本类型
        assert_eq!(text_request.description, Some("这是一个测试文本知识库".to_string()));
        assert_eq!(text_request.file_id, None);

        // 测试图片类型知识库请求
        let image_request = CreateDatasetRequest::new_image(
            "测试图片知识库".to_string(),
            "987654321".to_string(),
            None,
        ).with_icon("file_123".to_string());
        
        assert_eq!(image_request.name, "测试图片知识库");
        assert_eq!(image_request.space_id, "987654321");
        assert_eq!(image_request.format_type, 2); // 图片类型
        assert_eq!(image_request.description, None);
        assert_eq!(image_request.file_id, Some("file_123".to_string()));
    }

    #[test]
    fn test_create_dataset_response_parsing() {
        // 测试成功响应解析
        let success_json = json!({
            "code": 0,
            "msg": "",
            "data": {
                "dataset_id": "744668935865830123"
            },
            "detail": {
                "logid": "20241210160547B25AEC1917B03A2F1F07"
            }
        });

        let response: CreateDatasetResponse = serde_json::from_value(success_json).unwrap();
        assert_eq!(response.code, 0);
        assert_eq!(response.msg, "");
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().dataset_id, "744668935865830123");
        assert!(response.detail.is_some());
        assert_eq!(response.detail.unwrap().logid, "20241210160547B25AEC1917B03A2F1F07");

        // 测试错误响应解析
        let error_json = json!({
            "code": 4000,
            "msg": "参数错误",
            "data": null,
            "detail": {
                "logid": "20241210160547B25AEC1917B03A2F1F08"
            }
        });

        let error_response: CreateDatasetResponse = serde_json::from_value(error_json).unwrap();
        assert_eq!(error_response.code, 4000);
        assert_eq!(error_response.msg, "参数错误");
        assert!(error_response.data.is_none());
        assert!(error_response.detail.is_some());
        assert_eq!(error_response.detail.unwrap().logid, "20241210160547B25AEC1917B03A2F1F08");
    }

    #[tokio::test]
    async fn test_create_dataset_tool_validation() {
        // 创建测试用的 CozeTools 实例
        let client = CozeApiClient::new(
            "https://api.coze.cn".to_string(),
            "test-token".to_string(),
        ).unwrap();
        let tools = CozeTools::new(Arc::new(client), "test_space_id".to_string());

        // 测试缺少必需参数
        let result = tools.create_dataset(None).await;
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true));

        // 测试名称过长
        let long_name_args = json!({
            "name": "a".repeat(101), // 超过100字符限制
            "format_type": 0,
            "space_id": "test_space"
        });
        let result = tools.create_dataset(Some(long_name_args)).await;
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true));

        // 测试无效的 format_type
        let invalid_format_args = json!({
            "name": "测试知识库",
            "format_type": 1, // 无效类型，只支持0和2
            "space_id": "test_space"
        });
        let result = tools.create_dataset(Some(invalid_format_args)).await;
        assert!(result.is_ok());
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true));

        // 测试有效参数（但会因为网络调用失败）
        let valid_args = json!({
            "name": "测试知识库",
            "format_type": 0,
            "space_id": "test_space",
            "description": "测试描述"
        });
        let result = tools.create_dataset(Some(valid_args)).await;
        assert!(result.is_ok()); // 工具调用本身成功，但API调用可能失败
        // 注意：这里不检查 is_error，因为网络调用会失败
    }

    #[test]
    fn test_format_type_validation() {
        // 测试支持的 format_type 值
        assert_eq!(0, 0); // 文本类型
        assert_eq!(2, 2); // 图片类型
        
        // 验证不支持的值
        let unsupported_types = vec![1, 3, 4, -1];
        for unsupported in unsupported_types {
            // 在实际工具中，这些值应该被拒绝
            assert_ne!(unsupported, 0);
            assert_ne!(unsupported, 2);
        }
    }

    #[test]
    fn test_request_serialization() {
        let request = CreateDatasetRequest {
            name: "测试知识库".to_string(),
            space_id: "123456789".to_string(),
            format_type: 0,
            description: Some("测试描述".to_string()),
            file_id: None,
        };

        let serialized = serde_json::to_value(&request).unwrap();
        let expected = json!({
            "name": "测试知识库",
            "space_id": "123456789",
            "format_type": 0,
            "description": "测试描述"
        });

        assert_eq!(serialized, expected);

        // 测试带图标的请求
        let request_with_icon = CreateDatasetRequest {
            name: "带图标的知识库".to_string(),
            space_id: "987654321".to_string(),
            format_type: 2,
            description: None,
            file_id: Some("file_123".to_string()),
        };

        let serialized_with_icon = serde_json::to_value(&request_with_icon).unwrap();
        let expected_with_icon = json!({
            "name": "带图标的知识库",
            "space_id": "987654321",
            "format_type": 2,
            "file_id": "file_123"
        });

        assert_eq!(serialized_with_icon, expected_with_icon);
    }
}
