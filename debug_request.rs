use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // 测试序列化一个简单的ChatRequest
    let mut additional_messages = Vec::new();
    additional_messages.push(serde_json::json!({
        "role": "user",
        "content": "请详细介绍kVPAC IDE的功能和特点，包括PLC编程相关的所有功能",
        "content_type": "text"
    }));

    let request = serde_json::json!({
        "bot_id": "7409830408747073570",
        "additional_messages": additional_messages,
        "stream": false,
        "auto_save_history": true
    });

    println!("Complete request JSON:");
    println!("{}", serde_json::to_string_pretty(&request).unwrap());
    
    // 测试是否是有效的JSON
    let json_str = serde_json::to_string(&request).unwrap();
    println!("\nCompact JSON:");
    println!("{}", json_str);
    
    // 验证可以重新解析
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    println!("\nSuccessfully parsed back to Value");
}
