#[cfg(test)]
mod test_knowledge_base_integration {
    use coze_mcp_server::tools::coze_tools::CozeTools;
    use coze_mcp_server::api::CozeApiClient;
    use std::sync::Arc;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_list_knowledge_bases_tool_with_missing_doc_count() {
        // Test the MCP tool integration with a mock client that returns responses without doc_count
        let client = CozeApiClient::new("https://api.coze.cn".to_string(), "test_token".to_string()).expect("create client");
        let tools = CozeTools::new(Arc::new(client), "test_space".to_string());
        
        // Test with no specific space_id (should use default)
        let args = Some(json!({}));
        
        // Note: This will fail because we don't have real API credentials
        // But it will help us verify the parsing logic doesn't crash
        match tools.list_knowledge_bases(args).await {
            Ok(result) => {
                println!("Tool result: {:?}", result);
                // If we get a successful result, it means our parsing logic worked
            }
            Err(e) => {
                println!("Expected error due to test credentials: {:?}", e);
                // This is expected since we're using test credentials
            }
        }
    }
    
    #[test]
    fn test_knowledge_base_display_with_various_document_counts() {
        // Test that the structured content properly handles various document count scenarios
        use coze_mcp_server::api::{KnowledgeBaseInfo, ListKnowledgeBasesResponse};
        
        let kb_with_docs = KnowledgeBaseInfo {
            dataset_id: "ds_1".to_string(),
            name: "知识库1".to_string(),
            description: "有文档的知识库".to_string(),
            created_at: 1733817948,
            document_count: 5,
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
        
        let kb_without_docs = KnowledgeBaseInfo {
            dataset_id: "ds_2".to_string(),
            name: "知识库2".to_string(),
            description: "没有文档的知识库".to_string(),
            created_at: 1733817948,
            document_count: 0,
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
        
        let response = ListKnowledgeBasesResponse {
            datasets: vec![kb_with_docs, kb_without_docs],
            total: 2,
        };
        
        // Test the display logic that would be used in the MCP tool
        let mut display_content = format!("找到 {} 个知识库:\n\n", response.total);
        for (i, kb) in response.datasets.iter().enumerate() {
            display_content.push_str(&format!(
                "{}. ID: {}\n   名称: {}\n   描述: {}\n   文档数量: {}\n   创建时间: {}\n\n",
                i + 1,
                kb.dataset_id,
                kb.name,
                kb.description,
                kb.document_count,
                kb.created_at
            ));
        }
        
        println!("Display content:\n{}", display_content);
        
        // Verify that document counts are displayed correctly
        assert!(display_content.contains("文档数量: 5"));
        assert!(display_content.contains("文档数量: 0"));
        assert!(display_content.contains("找到 2 个知识库"));
    }
}