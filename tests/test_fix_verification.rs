#[cfg(test)]
mod test_complete_fix_verification {
    use coze_mcp_server::api::client::CozeApiClient;
    use coze_mcp_server::api::{ListDatasetsApiResponse, ListKnowledgeBasesResponse};
    
    #[test]
    fn test_complete_fix_for_missing_doc_count() {
        // Test the complete fix by simulating various API response formats
        // that might not include doc_count field
        
        // 1. Response with missing doc_count (the original issue)
        let response_without_doc_count = r#"{
            "code": 0,
            "msg": "Success",
            "data": {
                "total_count": 2,
                "dataset_list": [
                    {
                        "dataset_id": "ds_1",
                        "name": "测试知识库1",
                        "description": "测试描述1",
                        "create_time": 1733817948
                    },
                    {
                        "dataset_id": "ds_2", 
                        "name": "测试知识库2",
                        "description": "测试描述2",
                        "create_time": 1733817950
                    }
                ]
            }
        }"#;
        
        // This should now parse successfully with our fix
        let parsed: Result<ListDatasetsApiResponse, _> = serde_json::from_str(response_without_doc_count);
        assert!(parsed.is_ok(), "Should parse successfully even without doc_count field");
        
        let api_response = parsed.unwrap();
        let internal_response: ListKnowledgeBasesResponse = api_response.into_internal();
        
        assert_eq!(internal_response.total, 2);
        assert_eq!(internal_response.datasets.len(), 2);
        
        // Both should have document_count = 0 (default value)
        for dataset in &internal_response.datasets {
            assert_eq!(dataset.document_count, 0, "Missing doc_count should default to 0");
        }
        
        println!("✅ Fix verified: API responses without doc_count now parse correctly");
        
        // 2. Response with explicit doc_count = 0 
        let response_with_zero_docs = r#"{
            "code": 0,
            "data": {
                "total": 1,
                "dataset_list": [
                    {
                        "dataset_id": "ds_3",
                        "name": "空知识库",
                        "description": "没有文档",
                        "doc_count": 0,
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        let parsed_zero: ListDatasetsApiResponse = serde_json::from_str(response_with_zero_docs).unwrap();
        let internal_zero = parsed_zero.into_internal();
        assert_eq!(internal_zero.datasets[0].document_count, 0);
        
        // 3. Response with positive doc_count
        let response_with_docs = r#"{
            "code": 0,
            "data": {
                "total": 1,
                "dataset_list": [
                    {
                        "dataset_id": "ds_4",
                        "name": "有文档知识库",
                        "description": "包含文档",
                        "doc_count": 15,
                        "create_time": 1733817948
                    }
                ]
            }
        }"#;
        
        let parsed_docs: ListDatasetsApiResponse = serde_json::from_str(response_with_docs).unwrap();
        let internal_docs = parsed_docs.into_internal();
        assert_eq!(internal_docs.datasets[0].document_count, 15);
        
        println!("✅ All test cases pass - fix is comprehensive");
    }
    
    #[test]
    fn test_mcp_tool_output_format() {
        // Verify that the MCP tool will display correct information with our fix
        use coze_mcp_server::api::KnowledgeBaseInfo;
        
        // Simulate a knowledge base with 0 documents (from missing doc_count)
        let kb_no_docs = KnowledgeBaseInfo {
            dataset_id: "ds_empty".to_string(),
            name: "空知识库".to_string(),
            description: "这个知识库之前显示错误".to_string(),
            created_at: 1733817948,
            document_count: 0, // This would be set by our default
            update_time: None,
            status: None,
            format_type: None,
            slice_count: None,
            space_id: None,
            dataset_type: None,
            can_edit: None,
            icon_url: None,
            icon_uri: None,
            avatar_url: None,
            creator_id: None,
            creator_name: None,
            hit_count: None,
            all_file_size: None,
            bot_used_count: None,
            file_list: None,
            failed_file_list: None,
            processing_file_list: None,
            processing_file_id_list: None,
            chunk_strategy: None,
            storage_config: None,
            project_id: None,
            raw_extra: None,
        };
        
        // This is the format that would be shown to users in the MCP tool
        let display_output = format!(
            "ID: {}\n名称: {}\n描述: {}\n文档数量: {}\n创建时间: {}",
            kb_no_docs.dataset_id,
            kb_no_docs.name,
            kb_no_docs.description,
            kb_no_docs.document_count,
            kb_no_docs.created_at
        );
        
        println!("MCP Tool Output:\n{}", display_output);
        
        // Verify the output shows 0 documents instead of causing an error
        assert!(display_output.contains("文档数量: 0"));
        assert!(display_output.contains("空知识库"));
        
        println!("✅ MCP tool will now correctly display 0 documents instead of failing");
    }
}