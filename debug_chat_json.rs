use coze_mcp_server::api::chat_models::ChatRequest;

fn main() {
    // 模拟用户的实际请求参数
    let bot_id = "7409830408747073570";
    let message = "请详细介绍kVPAC IDE的功能和特点，包括PLC编程支持、界面特性、调试功能等";
    
    // 创建正确的ChatRequest（包含必需的user_id）
    let request = ChatRequest::new(bot_id.to_string(), message.to_string())
        .with_user_id("default_user".to_string());  // 添加必需的user_id
    
    let json_value = serde_json::to_value(&request).unwrap();
    println!("正确的ChatRequest JSON格式:");
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap());
    
    println!("\n错误分析:");
    println!("1. user_id是Coze API的必选参数，但在原始请求中缺失");
    println!("2. 原始请求使用'message'字段，但应该使用'additional_messages'数组");
    println!("3. additional_messages中的每条消息需要role、content、content_type字段");
}
