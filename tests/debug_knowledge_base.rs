#[cfg(test)]
mod test_knowledge_base_debug {
    use coze_mcp_server::api::{CozeApiClient, ListDatasetsApiResponse};
    
    #[test]
    fn test_document_count_parsing() {
        // Test the parsing with different response formats
        
        // Format 1: Using doc_count (as in current test)
        let test_json_1 = r#"{
            "code": 0,
            "msg": "Success",
            "data": {
                "total_count": 1,
                "dataset_list": [
                    {
                        "dataset_id": "ds_1",
                        "name": "kb1",
                        "description": "desc",
                        "doc_count": 2,
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        let structured: ListDatasetsApiResponse = serde_json::from_str(test_json_1).expect("parse format 1");
        let internal = structured.into_internal();
        assert_eq!(internal.total, 1);
        assert_eq!(internal.datasets.len(), 1);
        assert_eq!(internal.datasets[0].document_count, 2);
        println!("Format 1 (doc_count): Document count = {}", internal.datasets[0].document_count);
        
        // Format 2: Using document_count
        let test_json_2 = r#"{
            "code": 0,
            "data": {
                "total": 1,
                "datasets": [
                    {
                        "dataset_id": "ds_2",
                        "name": "kb2",
                        "description": "desc2",
                        "document_count": 3,
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        // This should be parsed by the fallback logic in client.rs
        // Let's check if it works correctly
        let parsed_2: serde_json::Value = serde_json::from_str(test_json_2).expect("parse format 2 as json");
        println!("Format 2 raw data: {:?}", parsed_2);
        
        // Format 3: Zero document count
        let test_json_3 = r#"{
            "code": 0,
            "data": {
                "total": 1,
                "dataset_list": [
                    {
                        "dataset_id": "ds_3",
                        "name": "kb3",
                        "description": "desc3",
                        "doc_count": 0,
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        let structured_3: ListDatasetsApiResponse = serde_json::from_str(test_json_3).expect("parse format 3");
        let internal_3 = structured_3.into_internal();
        assert_eq!(internal_3.datasets[0].document_count, 0);
        println!("Format 3 (zero docs): Document count = {}", internal_3.datasets[0].document_count);
        
        // Format 4: Missing document count
        let test_json_4 = r#"{
            "code": 0,
            "data": {
                "total": 1,
                "dataset_list": [
                    {
                        "dataset_id": "ds_4",
                        "name": "kb4",
                        "description": "desc4",
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        let structured_4: ListDatasetsApiResponse = serde_json::from_str(test_json_4).expect("parse format 4");
        let internal_4 = structured_4.into_internal();
        assert_eq!(internal_4.datasets[0].document_count, 0);
        println!("Format 4 (missing docs): Document count = {}", internal_4.datasets[0].document_count);
    }
    
    #[tokio::test]
    async fn test_api_client_with_mock_response() {
        // Test if the client fallback parsing works correctly
        use std::env;
        
        let api_token = env::var("COZE_API_TOKEN").unwrap_or_else(|_| "test_token".to_string());
        let space_id = env::var("COZE_SPACE_ID").unwrap_or_else(|_| "test_space".to_string());
        
        if api_token.starts_with("pat_") && !space_id.is_empty() && space_id != "test_space" {
            println!("Testing with real API...");
            let client = CozeApiClient::new("https://api.coze.cn".to_string(), api_token).expect("create client");
            
            match client.list_datasets(&space_id, None, None, None, None).await {
                Ok(response) => {
                    println!("API call successful!");
                    println!("Total datasets: {}", response.total);
                    for (i, dataset) in response.datasets.iter().enumerate() {
                        println!("Dataset {}: ID={}, Name={}, Document Count={}", 
                                 i + 1, dataset.dataset_id, dataset.name, dataset.document_count);
                    }
                }
                Err(e) => {
                    println!("API call failed: {:?}", e);
                }
            }
        } else {
            println!("Skipping real API test - no valid credentials");
        }
    }
}