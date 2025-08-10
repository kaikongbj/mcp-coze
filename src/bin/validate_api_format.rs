use coze_mcp_server::api::{KnowledgeBaseInfo, ListKnowledgeBasesResponse};

fn main() {
    println!("🔍 验证API数据格式兼容性...");

    // 模拟API响应数据
    let mock_response = r#"{
        "data": {
            "datasets": [
                {
                    "dataset_id": "kb_123456",
                    "name": "测试知识库",
                    "description": "这是一个测试知识库",
                    "doc_count": 15,
                    "create_time": 1700000000,
                    "update_time": 1700000000,
                    "status": 1,
                    "format_type": 0,
                    "slice_count": 200,
                    "space_id": "space_123",
                    "dataset_type": 0,
                    "can_edit": true,
                    "icon_url": "https://example.com/icon.png",
                    "creator_id": "user_123",
                    "creator_name": "测试用户",
                    "hit_count": 100,
                    "all_file_size": 204800
                }
            ],
            "total": 1,
            "total_count": 1
        }
    }"#;

    let mock_response_v2 = r#"{
        "datasets": [
            {
                "dataset_id": "kb_789",
                "name": "新版API测试",
                "description": "新版API格式测试",
                "doc_count": 25,
                "create_time": 1700000000,
                "update_time": 1700000000,
                "status": 1,
                "format_type": 0,
                "slice_count": 300,
                "space_id": "space_456",
                "dataset_type": 0,
                "can_edit": true,
                "creator_id": "user_456",
                "creator_name": "测试用户2",
                "hit_count": 200,
                "all_file_size": 409600
            }
        ],
        "total": 1
    }"#;

    // 测试格式1：嵌套data结构
    println!("📋 测试格式1：嵌套data结构");
    let response: Result<serde_json::Value, _> = serde_json::from_str(mock_response);
    match response {
        Ok(data) => {
            if let Some(dataset_list) = data.get("data").and_then(|d| d.get("datasets")) {
                if let Ok(datasets) =
                    serde_json::from_value::<Vec<KnowledgeBaseInfo>>(dataset_list.clone())
                {
                    println!("   ✅ 格式1解析成功！数据集数量: {}", datasets.len());
                    if let Some(dataset) = datasets.first() {
                        println!("      数据集ID: {}", dataset.dataset_id);
                        println!("      名称: {}", dataset.name);
                        println!("      文档数量: {}", dataset.document_count);
                    }
                } else {
                    println!("   ❌ 格式1解析失败");
                }
            }
        }
        Err(e) => println!("   ❌ 格式1JSON解析失败: {e}"),
    }

    // 测试格式2：直接datasets结构
    println!("📋 测试格式2：直接datasets结构");
    let response_v2: Result<ListKnowledgeBasesResponse, _> = serde_json::from_str(mock_response_v2);
    match response_v2 {
        Ok(data) => {
            println!("   ✅ 格式2解析成功！");
            println!("   总数量: {}", data.total);
            println!("   数据集数量: {}", data.datasets.len());

            if let Some(dataset) = data.datasets.first() {
                println!("   第一个数据集:");
                println!("     ID: {}", dataset.dataset_id);
                println!("     名称: {}", dataset.name);
                println!("     描述: {}", dataset.description);
                println!("     文档数量: {}", dataset.document_count);
                println!("     创建时间: {}", dataset.created_at);

                // 验证所有必需字段
                assert!(!dataset.dataset_id.is_empty());
                assert!(!dataset.name.is_empty());
                // document_count is usize; non-negative by definition
                assert!(dataset.created_at > 0);

                println!("   ✅ 所有必需字段验证通过！");
            }
        }
        Err(e) => println!("   ❌ 格式2解析失败: {e}"),
    }

    // 测试字段映射
    println!("📋 测试字段映射:");
    let test_json = r#"{
        "dataset_id": "test_id",
        "name": "测试名称",
        "description": "测试描述",
        "create_time": 1234567890,
        "doc_count": 42
    }"#;

    let test_result: Result<KnowledgeBaseInfo, _> = serde_json::from_str(test_json);
    match test_result {
        Ok(info) => {
            println!("   ✅ 字段映射验证通过！");
            println!("   dataset_id -> {}", info.dataset_id);
            println!("   name -> {}", info.name);
            println!("   description -> {}", info.description);
            println!("   create_time -> {}", info.created_at);
            println!("   doc_count -> {}", info.document_count);
        }
        Err(e) => println!("   ❌ 字段映射验证失败: {e}"),
    }

    println!("🎉 所有API格式验证完成！");
}
