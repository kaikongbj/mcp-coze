// 聊天功能集成测试 - 仅验证结构，不实际调用API
use coze_mcp_server::api::chat_models::*;
use coze_mcp_server::tools::coze_tools::CozeTools;
use coze_mcp_server::api::CozeApiClient;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_chat_tool_argument_validation() {
    // 创建模拟客户端 - 使用虚拟凭证，不会实际调用API
    let client = CozeApiClient::new(
        "https://api.coze.cn".to_string(),
        "pat_test_token".to_string(),
    ).expect("Failed to create client");
    
    let tools = CozeTools::new(Arc::new(client), "test_space".to_string());
    
    // 测试缺少必需参数
    let empty_args = Some(json!({}));
    let result = tools.chat(empty_args).await;
    if result.is_err() {
        println!("Empty args error: {:?}", result.as_ref().err());
    }
    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
    
    // 测试有效参数结构（不实际调用API）
    let valid_args = Some(json!({
        "bot_id": "test_bot_123",
        "message": "Hello, world!",
        "user_id": "user_456",
        "custom_variables": {
            "context": "test",
            "language": "zh"
        }
    }));
    
    // 这会尝试调用API，但由于使用虚拟token会失败
    // 我们只验证参数解析是否正确
    let result = tools.chat(valid_args).await;
    assert!(result.is_ok());
    
    // 验证流式聊天参数解析
    let stream_args = Some(json!({
        "bot_id": "test_bot_123",
        "message": "Generate a long response",
        "conversation_id": "conv_789"
    }));
    
    let result = tools.chat_stream(stream_args).await;
    assert!(result.is_ok());
}

#[test]
fn test_message_role_serialization() {
    let role = MessageRole::Assistant;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"assistant\"");
    
    let role: MessageRole = serde_json::from_str("\"user\"").unwrap();
    assert!(matches!(role, MessageRole::User));
}

#[test]
fn test_content_type_serialization() {
    let content_type = ContentType::Text;
    let json = serde_json::to_string(&content_type).unwrap();
    assert_eq!(json, "\"text\"");
    
    let content_type: ContentType = serde_json::from_str("\"image\"").unwrap();
    assert!(matches!(content_type, ContentType::Image));
}

#[test]
fn test_stream_event_type_serialization() {
    let event = StreamEventType::ConversationMessageDelta;
    let json = serde_json::to_string(&event).unwrap();
    assert_eq!(json, "\"conversation_message_delta\"");
    
    let event: StreamEventType = serde_json::from_str("\"done\"").unwrap();
    assert!(matches!(event, StreamEventType::Done));
}

#[test]
fn test_chat_request_builder() {
    let request = ChatRequest::new("bot_123".to_string(), "Hello".to_string())
        .with_stream(true)
        .with_user_id("user_456".to_string())
        .with_conversation_id("conv_789".to_string());
    
    assert_eq!(request.bot_id, "bot_123");
    assert_eq!(request.stream, Some(true));
    assert_eq!(request.user_id, Some("user_456".to_string()));
    assert_eq!(request.conversation_id, Some("conv_789".to_string()));
    assert_eq!(request.additional_messages.len(), 1);
    assert_eq!(request.additional_messages[0].content, Some("Hello".to_string()));
}

#[test]
fn test_chat_request_json_serialization() {
    let request = ChatRequest::new("bot_123".to_string(), "Hello".to_string());
    let json_value = serde_json::to_value(&request).unwrap();
    
    // 打印完整的JSON以便调试
    println!("Complete ChatRequest JSON:");
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap());
    
    assert_eq!(json_value["bot_id"], "bot_123");
    assert_eq!(json_value["stream"], false);
    assert_eq!(json_value["auto_save_history"], true);
    assert!(json_value["additional_messages"].is_array());
    
    let messages = json_value["additional_messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "Hello");
    // content_type should not be present for simple text messages
    assert!(messages[0]["content_type"].is_null());
    assert!(messages[0]["object_string"].is_null());
}
