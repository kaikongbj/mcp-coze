use coze_mcp_server::api::chat_models::*;
use serde_json::json;

#[tokio::test]
async fn test_chat_models() {
    // 测试创建聊天请求
    let request = ChatRequest::new("test_bot_id".to_string(), "Hello, world!".to_string());

    assert_eq!(request.bot_id, "test_bot_id");
    assert_eq!(request.additional_messages.len(), 1);
    assert_eq!(
        request.additional_messages[0].content,
        Some("Hello, world!".to_string())
    );
    assert_eq!(request.stream, Some(false));

    // 测试链式配置
    let request = request
        .with_stream(true)
        .with_conversation_id("conv_123".to_string())
        .with_user_id("user_456".to_string());

    assert_eq!(request.stream, Some(true));
    assert_eq!(request.conversation_id, Some("conv_123".to_string()));
    assert_eq!(request.user_id, Some("user_456".to_string()));

    // 测试序列化
    let json_value = serde_json::to_value(&request).unwrap();
    assert!(json_value.get("bot_id").is_some());
    assert!(json_value.get("additional_messages").is_some());

    println!("Chat models test passed!");
}

#[tokio::test]
async fn test_stream_event_serialization() {
    let stream_response = StreamChatResponse {
        event: StreamEventType::ConversationMessageDelta,
        conversation_id: Some("conv_123".to_string()),
        id: Some("msg_456".to_string()),
        created_at: Some(1234567890),
        delta: Some(StreamDelta {
            content: Some("Hello".to_string()),
            role: Some(MessageRole::Assistant),
            content_type: Some(ContentType::Text),
        }),
        usage: None,
        last_error: None,
    };

    let json_value = serde_json::to_value(&stream_response).unwrap();
    assert_eq!(json_value["event"], "conversation_message_delta");
    assert_eq!(json_value["conversation_id"], "conv_123");

    println!("Stream event serialization test passed!");
}
