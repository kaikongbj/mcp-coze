use coze_mcp_server::api::chat_models::ChatRequest;

#[test]
fn test_random_user_id_generation() {
    // 测试不提供user_id时，ChatRequest应该能够工作
    let request = ChatRequest::new(
        "7409830408747073570".to_string(),
        "请详细介绍kVPAC IDE的功能和特点".to_string(),
    );

    // 如果没有手动设置user_id，我们应该在工具层生成它
    let json_value = serde_json::to_value(&request).unwrap();
    let json_string = serde_json::to_string_pretty(&json_value).unwrap();

    println!("基础ChatRequest（工具层将自动生成user_id）:");
    println!("{}", json_string);

    // 验证基本结构正确
    assert!(json_string.contains("\"bot_id\": \"7409830408747073570\""));
    assert!(json_string.contains("\"additional_messages\""));
    assert!(json_string.contains("\"role\": \"user\""));
    assert!(json_string.contains("\"stream\": false"));
    assert!(json_string.contains("\"auto_save_history\": true"));
}

#[test]
fn test_uuid_generation() {
    // 测试UUID生成功能
    let uuid1 = uuid::Uuid::new_v4().to_string();
    let uuid2 = uuid::Uuid::new_v4().to_string();

    println!("生成的UUID示例:");
    println!("UUID1: {}", uuid1);
    println!("UUID2: {}", uuid2);

    // 验证UUID格式和唯一性
    assert_ne!(uuid1, uuid2, "生成的UUID应该是唯一的");
    assert_eq!(uuid1.len(), 36, "UUID应该是36个字符");
    assert!(uuid1.contains("-"), "UUID应该包含连字符");

    // 验证UUID格式（8-4-4-4-12）
    let parts: Vec<&str> = uuid1.split('-').collect();
    assert_eq!(parts.len(), 5, "UUID应该有5个部分");
    assert_eq!(parts[0].len(), 8, "第一部分应该是8个字符");
    assert_eq!(parts[1].len(), 4, "第二部分应该是4个字符");
    assert_eq!(parts[2].len(), 4, "第三部分应该是4个字符");
    assert_eq!(parts[3].len(), 4, "第四部分应该是4个字符");
    assert_eq!(parts[4].len(), 12, "第五部分应该是12个字符");
}

#[test]
fn test_chat_request_with_manual_user_id() {
    // 测试手动提供user_id的情况
    let request = ChatRequest::new("7409830408747073570".to_string(), "测试消息".to_string())
        .with_user_id("custom_user_123".to_string());

    let json_value = serde_json::to_value(&request).unwrap();
    let json_string = serde_json::to_string_pretty(&json_value).unwrap();

    println!("手动指定user_id的ChatRequest:");
    println!("{}", json_string);

    // 验证包含手动指定的user_id
    assert!(json_string.contains("\"user_id\": \"custom_user_123\""));
}
