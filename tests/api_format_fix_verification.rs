use coze_mcp_server::api::chat_models::ChatRequest;
use serde_json::json;

#[test]
fn test_correct_api_format_with_user_id() {
    // 测试包含必需user_id的正确格式
    let request = ChatRequest::new(
        "7409830408747073570".to_string(),
        "请详细介绍kVPAC IDE的功能和特点，包括PLC编程支持、界面特性、调试功能等".to_string(),
    )
    .with_user_id("default_user".to_string());

    let json_value = serde_json::to_value(&request).unwrap();
    let json_string = serde_json::to_string_pretty(&json_value).unwrap();

    println!("✅ 修复后的正确API请求格式:");
    println!("{}", json_string);

    // 验证必需字段存在
    assert!(json_string.contains("\"bot_id\": \"7409830408747073570\""));
    assert!(json_string.contains("\"user_id\": \"default_user\""));
    assert!(json_string.contains("\"additional_messages\""));
    assert!(json_string.contains("\"role\": \"user\""));
    assert!(json_string.contains("\"stream\": false"));
    assert!(json_string.contains("\"auto_save_history\": true"));
}

#[test]
fn test_user_request_format_comparison() {
    println!("\n🔍 错误分析 - 原始请求 vs 正确格式:");
    println!("❌ 原始请求（导致4000错误）:");
    let original_request = json!({
        "bot_id": "7409830408747073570",
        "message": "请详细介绍kVPAC IDE的功能和特点，包括PLC编程支持、界面特性、调试功能等"
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&original_request).unwrap()
    );

    println!("\n✅ 正确请求格式:");
    let correct_request = ChatRequest::new(
        "7409830408747073570".to_string(),
        "请详细介绍kVPAC IDE的功能和特点，包括PLC编程支持、界面特性、调试功能等".to_string(),
    )
    .with_user_id("default_user".to_string());

    let correct_json = serde_json::to_value(&correct_request).unwrap();
    println!("{}", serde_json::to_string_pretty(&correct_json).unwrap());

    println!("\n📋 关键修复点:");
    println!("1. ✅ 添加了必需的user_id字段");
    println!("2. ✅ 将message字段转换为additional_messages数组格式");
    println!("3. ✅ 为消息添加了正确的role和content_type");
    println!("4. ✅ 设置了stream和auto_save_history参数");
}
