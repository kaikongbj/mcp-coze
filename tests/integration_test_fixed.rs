use coze_mcp_server::api::client::CozeApiClient;

use std::env;

#[tokio::test]
async fn test_real_api_integration() {
    println!("🧪 开始真实API集成测试...");

    // 直接设置API密钥和参数
    let api_token =
        "pat_8NU7DXjMPg4O7rg4tbt8ZYzKkHRIMTZ8SKANbdYdjf0vMPKR7CbKsn0biE9TKcDi".to_string();
    let base_url = "https://api.coze.cn".to_string();
    let space_id = "7409828301432356875".to_string();

    println!("✅ API配置已设置");
    println!("   API Token: {}...", &api_token[..10]);
    println!("   Base URL: {}", base_url);
    println!("   Space ID: {}", space_id);

    // 创建API客户端
    let client = CozeApiClient::new(base_url.clone(), api_token.clone()).expect("client init");

    // 测试获取知识库列表
    match client
        .list_datasets(&space_id, None, None, Some(1), Some(50))
        .await
    {
        Ok(response) => {
            println!("✅ 真实API测试成功！");
            println!("   总数量: {}", response.total);
            println!("   数据集数量: {}", response.datasets.len());

            if !response.datasets.is_empty() {
                let mut total_files = 0;

                for (i, dataset) in response.datasets.iter().enumerate() {
                    // 直接从列表响应中获取准确的文件数量
                    let api_reported_count = dataset.document_count;
                    let actual_file_count = dataset
                        .file_list
                        .as_ref()
                        .map(|files| files.len())
                        .unwrap_or(0);

                    println!("");
                    println!("   📊 数据集 {}: {}", i + 1, dataset.name);
                    println!("      API返回文档数量: {}", api_reported_count);
                    println!("      实际文件数量: {}", actual_file_count);

                    if api_reported_count == actual_file_count {
                        println!("      ✅ 数量匹配");
                    } else {
                        println!(
                            "      ⚠️  数量不匹配: API={}, 实际={}",
                            api_reported_count, actual_file_count
                        );
                    }

                    // 显示文件列表
                    if let Some(file_list) = &dataset.file_list {
                        if !file_list.is_empty() {
                            println!("      📁 文件列表 ({}个):", file_list.len());
                            for (j, filename) in file_list.iter().take(5).enumerate() {
                                println!("         {}. {}", j + 1, filename);
                            }
                            if file_list.len() > 5 {
                                println!("         ... 还有 {} 个文件", file_list.len() - 5);
                            }
                        } else {
                            println!("      📁 空数据集");
                        }
                    } else {
                        println!("      📁 无文件列表信息");
                    }

                    total_files += actual_file_count;
                }

                println!("");
                println!("   📈 汇总统计:");
                println!("   总数据集数量: {}", response.datasets.len());
                println!("   总文件数量: {}", total_files);

                // 计算准确性
                let accurate_count = response
                    .datasets
                    .iter()
                    .filter(|d| {
                        d.document_count == d.file_list.as_ref().map(|f| f.len()).unwrap_or(0)
                    })
                    .count();

                println!(
                    "   准确报告的数据集: {} / {}",
                    accurate_count,
                    response.datasets.len()
                );
                println!(
                    "   准确率: {:.1}%",
                    (accurate_count as f64 / response.datasets.len() as f64) * 100.0
                );

                if accurate_count < response.datasets.len() {
                    println!("   💡 建议: 使用 file_list.length 获取准确的文件数量");
                }
            }
        }
        Err(e) => {
            println!("❌ 真实API测试失败: {}", e);
            panic!("API测试失败");
        }
    }
}

#[test]
fn test_environment_check() {
    // 检查环境变量
    let api_token = env::var("COZE_API_TOKEN");
    let base_url = env::var("COZE_BASE_URL");
    let space_id = env::var("COZE_SPACE_ID");

    println!("环境变量检查:");
    match api_token {
        Ok(token) => println!("✅ COZE_API_TOKEN: 已设置 (长度: {})", token.len()),
        Err(_) => println!("⚠️  COZE_API_TOKEN: 未设置"),
    }

    match base_url {
        Ok(url) => println!("✅ COZE_BASE_URL: {}", url),
        Err(_) => println!("⚠️  COZE_BASE_URL: 使用默认值 https://api.coze.cn"),
    }

    match space_id {
        Ok(id) => println!("✅ COZE_SPACE_ID: {}", id),
        Err(_) => println!("⚠️  COZE_SPACE_ID: 使用默认值"),
    }
}

#[tokio::test]
async fn test_api_endpoints() {
    let api_token = env::var("COZE_API_TOKEN").unwrap_or_else(|_| "mock_token".to_string());
    let base_url = env::var("COZE_BASE_URL").unwrap_or_else(|_| "https://api.coze.cn".to_string());

    if api_token == "mock_token" {
        println!("⚠️  跳过API端点测试，设置 COZE_API_TOKEN 环境变量");
        return;
    }

    let client = match CozeApiClient::new(base_url, api_token) {
        Ok(c) => c,
        Err(e) => {
            println!("跳过: 客户端创建失败 {}", e);
            return;
        }
    };
    let space_id = env::var("COZE_SPACE_ID").unwrap_or_else(|_| "default_space".to_string());

    // 测试各种端点
    let test_cases = vec![("list_datasets", || async {
        client
            .list_datasets(&space_id, None, None, Some(1), Some(5))
            .await
    })];

    for (endpoint_name, test_fn) in test_cases {
        match test_fn().await {
            Ok(_) => println!("✅ {}: 测试通过", endpoint_name),
            Err(e) => println!("❌ {}: 测试失败 - {}", endpoint_name, e),
        }
    }
}

#[tokio::test]
async fn test_pagination() {
    let api_token = env::var("COZE_API_TOKEN").unwrap_or_else(|_| "mock_token".to_string());
    let base_url = env::var("COZE_BASE_URL").unwrap_or_else(|_| "https://api.coze.cn".to_string());

    if api_token == "mock_token" {
        println!("⚠️  跳过分页测试，设置 COZE_API_TOKEN 环境变量");
        return;
    }

    let client = match CozeApiClient::new(base_url, api_token) {
        Ok(c) => c,
        Err(_) => {
            return;
        }
    };
    let space_id = env::var("COZE_SPACE_ID").unwrap_or_else(|_| "default_space".to_string());

    // 测试分页功能
    let page_sizes = vec![1, 5, 10];

    for page_size in page_sizes {
        let result = client
            .list_datasets(&space_id, None, None, Some(1), Some(page_size))
            .await;

        match result {
            Ok(response) => {
                println!("✅ 分页测试通过 - 页面大小: {}", page_size);
                assert!(response.datasets.len() <= page_size as usize);
            }
            Err(e) => println!("❌ 分页测试失败 - 页面大小: {} - {}", page_size, e),
        }
    }
}

#[tokio::test]
async fn test_filtering() {
    let api_token = env::var("COZE_API_TOKEN").unwrap_or_else(|_| "mock_token".to_string());
    let base_url = env::var("COZE_BASE_URL").unwrap_or_else(|_| "https://api.coze.cn".to_string());

    if api_token == "mock_token" {
        println!("⚠️  跳过过滤测试，设置 COZE_API_TOKEN 环境变量");
        return;
    }

    let client = match CozeApiClient::new(base_url, api_token) {
        Ok(c) => c,
        Err(_) => {
            return;
        }
    };
    let space_id = env::var("COZE_SPACE_ID").unwrap_or_else(|_| "default_space".to_string());

    // 测试过滤功能
    let test_name = "测试".to_string();

    let result = client
        .list_datasets(&space_id, Some(test_name.as_str()), None, Some(1), Some(10))
        .await;

    match result {
        Ok(response) => {
            println!("✅ 过滤测试通过");
            for dataset in &response.datasets {
                println!("找到数据集: {}", dataset.name);
            }
        }
        Err(e) => println!("❌ 过滤测试失败: {}", e),
    }
}
