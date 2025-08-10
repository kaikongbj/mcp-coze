use coze_mcp_server::api::chat_models::ChatRequest;
use coze_mcp_server::api::CozeApiClient;

#[tokio::test]
async fn debug_chat_request_format() {
    // 创建一个包含必需user_id的聊天请求
    let request = ChatRequest::new(
        "7409830408747073570".to_string(),
        "请详细介绍kVPAC IDE的功能和特点，包括PLC编程相关的所有功能".to_string(),
    )
    .with_user_id("default_user".to_string()); // 添加必需的user_id

    let json_value = serde_json::to_value(&request).unwrap();
    println!("修复后的ChatRequest JSON:");
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap());

    // 模拟发送请求（使用虚假的API密钥，不会真正调用）
    let _client = CozeApiClient::new(
        "https://api.coze.cn".to_string(),
        "pat_fake_token_for_debug".to_string(),
    )
    .expect("Failed to create client");

    // 将请求序列化为JSON，看看实际会发送什么
    let payload = serde_json::to_value(&request)
        .map_err(|e| format!("Serialization error: {}", e))
        .unwrap();
    println!("\n修复后的API请求格式:");
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    // 验证JSON是否有效并包含必需字段
    let json_str = serde_json::to_string(&payload).unwrap();
    println!("\n错误分析:");
    println!("原始请求缺少必需的user_id字段，导致API返回4000错误");
    println!("正确的请求必须包含: bot_id, user_id, additional_messages");

    // 验证必需字段存在
    assert!(json_str.contains("\"user_id\""), "user_id字段缺失");
    assert!(json_str.contains("\"bot_id\""), "bot_id字段缺失");
    assert!(
        json_str.contains("\"additional_messages\""),
        "additional_messages字段缺失"
    );
}
