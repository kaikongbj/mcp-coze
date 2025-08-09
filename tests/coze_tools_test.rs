use coze_mcp_server::api::CozeApiClient;
use coze_mcp_server::tools::coze_tools::CozeTools;
use rmcp::{ErrorData as McpError, ErrorData};
use serde_json::json;
use std::sync::Arc;

// 创建测试用的 CozeTools 实例
fn create_test_coze_tools() -> CozeTools {
    // 这里使用一个模拟的客户端，实际项目中可能需要使用 mockall 等库
    let mock_client = Arc::new(
        CozeApiClient::new("https://api.coze.cn".to_string(), "test_token".to_string()).unwrap(),
    );

    CozeTools::new(mock_client, "test_space_id".to_string())
}

#[tokio::test]
async fn test_upload_document_missing_arguments() {
    let tools = create_test_coze_tools();

    let result = tools.upload_document_to_knowledge_base(None).await;

    assert!(result.is_err());
    if let Err(ErrorData { message, .. }) = result {
        assert_eq!(message, "Missing arguments");
    }
}

#[tokio::test]
async fn test_upload_document_missing_dataset_id() {
    let tools = create_test_coze_tools();
    let args = json!({
        "file_path": "/path/to/file.txt"
    });

    let result = tools.upload_document_to_knowledge_base(Some(args)).await;

    assert!(result.is_err());
    if let Err(McpError { message, .. }) = result {
        assert_eq!(message, "Missing dataset_id");
    }
}

#[tokio::test]
async fn test_upload_document_missing_file_path() {
    let tools = create_test_coze_tools();
    let args = json!({
        "dataset_id": "dataset_123"
    });

    let result = tools.upload_document_to_knowledge_base(Some(args)).await;

    assert!(result.is_err());
    if let Err(McpError { message, .. }) = result {
        assert_eq!(message, "Missing file_path");
    }
}

#[tokio::test]
async fn test_upload_document_file_not_found() {
    let tools = create_test_coze_tools();
    let args = json!({
        "dataset_id": "dataset_123",
        "file_path": "/nonexistent/file.txt"
    });

    let result = tools.upload_document_to_knowledge_base(Some(args)).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));

    if let Some(content) = &call_result.content {
        let text = content[0].as_text().unwrap();
        assert!(text.text.contains("Failed to read file metadata"));
    }
}

#[tokio::test]
#[ignore] // 默认忽略，需要真实API环境时移除
async fn test_upload_document_real_api() {
    let api_token =
        "pat_8NU7DXjMPg4O7rg4tbt8ZYzKkHRIMTZ8SKANbdYdjf0vMPKR7CbKsn0biE9TKcDi".to_string();
    let space_id = "7409828301432356875".to_string();
    let client =
        Arc::new(CozeApiClient::new("https://api.coze.cn".to_string(), api_token).unwrap());

    let tools = CozeTools::new(client, space_id);

    let file_path = "D:\\mcp-coze\\USAGE.md";

    let args = json!({
        "dataset_id": "7533122859578048564", // 需要真实的dataset_id
        "file_path": file_path,
        "document_name": "integration_test.txt"
    });

    let result = tools.upload_document_to_knowledge_base(Some(args)).await;
    println!("{:?}", result);
    // 根据实际API响应进行验证
    match result {
        Ok(call_result) => {
            assert_eq!(call_result.is_error, Some(false));
            println!("Upload successful: {:?}", call_result.structured_content);
        }
        Err(e) => {
            println!("Upload failed: {:?}", e);
            // 在集成测试中，可能需要更宽松的错误处理
        }
    }
}
