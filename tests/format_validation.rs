use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseInfo {
    pub dataset_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "create_time")]
    pub created_at: i64,
    #[serde(rename = "doc_count")]
    pub document_count: usize,
    #[serde(rename = "update_time", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(rename = "format_type", skip_serializing_if = "Option::is_none")]
    pub format_type: Option<i32>,
    #[serde(rename = "slice_count", skip_serializing_if = "Option::is_none")]
    pub slice_count: Option<usize>,
    #[serde(rename = "space_id", skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(rename = "dataset_type", skip_serializing_if = "Option::is_none")]
    pub dataset_type: Option<i32>,
    #[serde(rename = "can_edit", skip_serializing_if = "Option::is_none")]
    pub can_edit: Option<bool>,
    #[serde(rename = "icon_url", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(rename = "icon_uri", skip_serializing_if = "Option::is_none")]
    pub icon_uri: Option<String>,
    #[serde(rename = "avatar_url", skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(rename = "creator_id", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    #[serde(rename = "creator_name", skip_serializing_if = "Option::is_none")]
    pub creator_name: Option<String>,
    #[serde(rename = "hit_count", skip_serializing_if = "Option::is_none")]
    pub hit_count: Option<usize>,
    #[serde(rename = "all_file_size", skip_serializing_if = "Option::is_none")]
    pub all_file_size: Option<u64>,
    #[serde(rename = "bot_used_count", skip_serializing_if = "Option::is_none")]
    pub bot_used_count: Option<usize>,
    #[serde(rename = "file_list", skip_serializing_if = "Option::is_none")]
    pub file_list: Option<Vec<String>>,
    #[serde(rename = "failed_file_list", skip_serializing_if = "Option::is_none")]
    pub failed_file_list: Option<Vec<String>>,
    #[serde(
        rename = "processing_file_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub processing_file_list: Option<Vec<String>>,
    #[serde(
        rename = "processing_file_id_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub processing_file_id_list: Option<Vec<String>>,
    #[serde(rename = "chunk_strategy", skip_serializing_if = "Option::is_none")]
    pub chunk_strategy: Option<serde_json::Value>,
    #[serde(rename = "storage_config", skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    #[serde(rename = "project_id", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListKnowledgeBasesResponse {
    #[serde(rename = "datasets")]
    pub datasets: Vec<KnowledgeBaseInfo>,
    #[serde(rename = "total")]
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_format_compatibility() {
        let mock_response = r#"{
            "datasets": [
                {
                    "dataset_id": "test_dataset_123",
                    "name": "测试知识库",
                    "description": "这是一个测试知识库",
                    "doc_count": 5,
                    "create_time": 1700000000,
                    "update_time": 1700000000,
                    "status": 1,
                    "format_type": 0,
                    "slice_count": 100,
                    "space_id": "test_space",
                    "dataset_type": 0,
                    "can_edit": true,
                    "icon_url": "https://example.com/icon.png",
                    "creator_id": "user_123",
                    "creator_name": "测试用户",
                    "hit_count": 50,
                    "all_file_size": 102400
                }
            ],
            "total": 1
        }"#;

        let response: ListKnowledgeBasesResponse = serde_json::from_str(mock_response).unwrap();

        assert_eq!(response.datasets.len(), 1);
        assert_eq!(response.total, 1);

        let dataset = &response.datasets[0];
        assert_eq!(dataset.dataset_id, "test_dataset_123");
        assert_eq!(dataset.name, "测试知识库");
        assert_eq!(dataset.description, "这是一个测试知识库");
        assert_eq!(dataset.document_count, 5);
        assert_eq!(dataset.created_at, 1700000000);
        assert_eq!(dataset.update_time.unwrap(), 1700000000);
        assert_eq!(dataset.status.unwrap(), 1);
        assert_eq!(dataset.format_type.unwrap(), 0);
        assert_eq!(dataset.slice_count.unwrap(), 100);
        assert_eq!(dataset.space_id.as_ref().unwrap(), "test_space");
        assert_eq!(dataset.can_edit.unwrap(), true);
        assert_eq!(dataset.creator_id.as_ref().unwrap(), "user_123");
        assert_eq!(dataset.creator_name.as_ref().unwrap(), "测试用户");
        assert_eq!(dataset.hit_count.unwrap(), 50);
        assert_eq!(dataset.all_file_size.unwrap(), 102400);

        println!("✅ 所有API格式兼容性测试通过！");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = ListKnowledgeBasesResponse {
            datasets: vec![KnowledgeBaseInfo {
                dataset_id: "test_123".to_string(),
                name: "测试".to_string(),
                description: "描述".to_string(),
                created_at: 1234567890,
                document_count: 10,
                update_time: Some(1234567890),
                status: Some(1),
                format_type: Some(0),
                slice_count: Some(100),
                space_id: Some("space_123".to_string()),
                dataset_type: Some(0),
                can_edit: Some(true),
                icon_url: Some("https://example.com/icon.png".to_string()),
                icon_uri: None,
                avatar_url: None,
                creator_id: Some("user_123".to_string()),
                creator_name: Some("用户".to_string()),
                hit_count: Some(50),
                all_file_size: Some(102400),
                bot_used_count: None,
                file_list: None,
                failed_file_list: None,
                processing_file_list: None,
                processing_file_id_list: None,
                chunk_strategy: None,
                storage_config: None,
                project_id: None,
                raw_extra: None,
            }],
            total: 1,
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ListKnowledgeBasesResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.datasets.len(), deserialized.datasets.len());
        assert_eq!(original.total, deserialized.total);
        assert_eq!(
            original.datasets[0].dataset_id,
            deserialized.datasets[0].dataset_id
        );
        assert_eq!(
            original.datasets[0].document_count,
            deserialized.datasets[0].document_count
        );

        println!("✅ 序列化/反序列化测试通过！");
        println!("序列化结果: {}", serialized);
    }
}
