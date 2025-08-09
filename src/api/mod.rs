pub mod client;
pub mod endpoints;
pub mod error;
pub mod knowledge_models;
pub mod chat_models;
pub mod bot_models;

pub use client::CozeApiClient;
// pub use knowledge_models::*; // 注释掉未使用的导入

// ---- Coze API typed models used by knowledge layer ----
// search-related types removed (CN does not support knowledge search API)

// Custom deserializer for u64 from string or number
fn deserialize_optional_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct U64Visitor;

    impl<'de> Visitor<'de> for U64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 as number or string, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v >= 0 {
                Ok(Some(v as u64))
            } else {
                Ok(None)
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v.parse::<u64>() {
                Ok(val) => Ok(Some(val)),
                Err(_) => Ok(None), // If parsing fails, treat as None
            }
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&v)
        }
    }

    deserializer.deserialize_any(U64Visitor)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeBaseInfo {
    pub dataset_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "create_time")]
    pub created_at: i64,
    #[serde(rename = "doc_count")]
    pub document_count: usize,
    // ---- Extended optional fields from official API (list_dataset) ----
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
    #[serde(rename = "all_file_size", skip_serializing_if = "Option::is_none", default, deserialize_with = "deserialize_optional_u64_from_string_or_number")]
    pub all_file_size: Option<u64>,
    #[serde(rename = "bot_used_count", skip_serializing_if = "Option::is_none")]
    pub bot_used_count: Option<usize>,
    #[serde(rename = "file_list", skip_serializing_if = "Option::is_none")]
    pub file_list: Option<Vec<String>>,
    #[serde(rename = "failed_file_list", skip_serializing_if = "Option::is_none")]
    pub failed_file_list: Option<Vec<String>>,
    #[serde(rename = "processing_file_list", skip_serializing_if = "Option::is_none")]
    pub processing_file_list: Option<Vec<String>>,
    #[serde(rename = "processing_file_id_list", skip_serializing_if = "Option::is_none")]
    pub processing_file_id_list: Option<Vec<String>>,
    #[serde(rename = "chunk_strategy", skip_serializing_if = "Option::is_none")]
    pub chunk_strategy: Option<serde_json::Value>,
    #[serde(rename = "storage_config", skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    #[serde(rename = "project_id", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    // Any unknown / not yet mapped extra fields retained for forward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListKnowledgeBasesResponse {
    #[serde(rename = "datasets")]
    pub datasets: Vec<KnowledgeBaseInfo>,
    #[serde(rename = "total")]
    pub total: usize,
}

// ---- /v1/datasets API 文档直通结构 ----
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDatasetsApiData {
    #[serde(rename = "total_count", alias = "total")]
    pub total_count: usize,
    #[serde(rename = "dataset_list", alias = "datasets")]
    pub dataset_list: Vec<KnowledgeBaseInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDatasetsApiResponse {
    pub code: i32,
    #[serde(default)]
    pub msg: Option<String>,
    pub data: Option<ListDatasetsApiData>,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
}

impl ListDatasetsApiResponse {
    pub fn into_internal(self) -> ListKnowledgeBasesResponse {
        if let Some(d) = self.data {
            ListKnowledgeBasesResponse {
                datasets: d.dataset_list,
                total: d.total_count,
            }
        } else {
            ListKnowledgeBasesResponse { datasets: vec![], total: 0 }
        }
    }
}

// Removed unused CreateKnowledgeBaseResponse / UploadDocumentResponse to reduce warnings.
