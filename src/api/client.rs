use crate::api::endpoints::{
    CHAT_COMPLETION_CN, CHAT_COMPLETION_URL,
    KNOWLEDGE_DOCUMENT_CREATE_URL,
};
use crate::api::error::ApiError;
use crate::api::knowledge_models::{
    ChatCompletionRequest, ChatCompletionResponse,
};
use crate::models::{CozeApiRequest, CozeApiResponse, HttpMethod};
use reqwest::{Client, Response};
use serde_json::json;
use std::time::Duration;
use urlencoding::encode;

#[derive(Debug, Clone)]
pub struct CozeApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl CozeApiClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            api_key,
            // Per-request timeout already configured in reqwest client; no extra field needed
        })
    }

    async fn send_raw_request(
        &self,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response, ApiError> {
        let request = self.client
            .request(method.parse().unwrap(), url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        let request = match body {
            Some(b) => request.json(&b),
            None => request,
        };

        let response = request.send().await?;
        Ok(response)
    }

    async fn process_response<T>(&self, response: Response) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(ApiError::from_response(status, body));
        }

        serde_json::from_str(&body).map_err(ApiError::from)
    }

    // Removed deprecated create_knowledge_base* variants (single public path retained at tool layer if needed)

    // Removed deprecated upload_document_to_knowledge_base* variants
    pub async fn upload_document(
        &self,
        dataset_id: &str,
        documents: Vec<crate::api::knowledge_models::DocumentBase>,
        chunk_strategy: Option<crate::api::knowledge_models::ChunkStrategy>,
    ) -> Result<crate::api::knowledge_models::KnowledgeDocumentCreateResponse, ApiError> {
        use crate::api::knowledge_models::KnowledgeDocumentCreateRequest;
        use crate::api::endpoints::KNOWLEDGE_DOCUMENT_CREATE_URL;
        let url = format!("{}/{}", self.base_url, KNOWLEDGE_DOCUMENT_CREATE_URL);
        let req = KnowledgeDocumentCreateRequest {
            dataset_id: dataset_id.to_string(),
            document_bases: documents,
            chunk_strategy,
            format_type: None,
        };
        let payload = req.into_json()?;
        let resp = self.send_raw_request("POST", &url, Some(payload)).await?;
        self.process_response(resp).await
    }

    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ApiError> {
        let url = format!("{}/{}", self.base_url, CHAT_COMPLETION_URL);
        
        let response = self.send_raw_request("POST", &url, Some(serde_json::to_value(&request)?)).await?;
        self.process_response(response).await
    }

    pub async fn chat_completion_cn(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ApiError> {
        let url = format!("{}/{}", self.base_url, CHAT_COMPLETION_CN);
        
        let response = self.send_raw_request("POST", &url, Some(serde_json::to_value(&request)?)).await?;
        self.process_response(response).await
    }

    // ---- Public high-level generic request used by CozeTools ----
    pub async fn execute_request(&self, req: CozeApiRequest) -> Result<CozeApiResponse, ApiError> {
        let method_str = match req.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        };
        let mut url = format!("{}{}", self.base_url, req.endpoint);
        if !req.params.is_empty() {
            let mut pairs: Vec<String> = Vec::new();
            for (k, v) in &req.params {
                if let Some(s) = v.as_str() {
                    pairs.push(format!("{}={}", encode(k), encode(s)));
                } else if v.is_number() || v.is_boolean() {
                    pairs.push(format!("{}={}", encode(k), encode(&v.to_string())));
                }
            }
            if !pairs.is_empty() {
                if url.contains('?') {
                    url.push_str("&");
                } else {
                    url.push('?');
                }
                url.push_str(&pairs.join("&"));
            }
        }

        let mut request_builder = self
            .client
            .request(method_str.parse().unwrap(), &url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        for (k, v) in &req.headers {
            request_builder = request_builder.header(k, v);
        }
        if let Some(body) = &req.body {
            request_builder = request_builder.json(body);
        }

        let resp = request_builder.send().await?;
        let status_code = resp.status().as_u16();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect::<std::collections::HashMap<String, String>>();
        let text = resp.text().await?;
        let body_json: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw": text}));
        Ok(CozeApiResponse {
            status_code,
            headers,
            success: status_code < 400,
            body: body_json,
        })
    }

    // ---- Additional API helpers required by tools ----
    pub async fn get_dataset_cn(&self, dataset_id: &str) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::datasets_cn::GET_KNOWLEDGE_BASE;
        let url = format!("{}{}?dataset_id={}", self.base_url, GET_KNOWLEDGE_BASE, encode(dataset_id));
        let resp = self.send_raw_request("GET", &url, None).await?;
        self.process_response(resp).await
    }

    pub async fn retrieve_conversation_v1(&self, conversation_id: &str) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::conversation::GET_CONVERSATION;
        let url = format!("{}{}?conversation_id={}", self.base_url, GET_CONVERSATION, encode(conversation_id));
        let resp = self.send_raw_request("GET", &url, None).await?;
        self.process_response(resp).await
    }

    pub async fn get_conversation_messages_v3(&self, conversation_id: &str) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::conversation::GET_MESSAGES;
        let path = GET_MESSAGES.replace("{conversation_id}", conversation_id);
        let url = format!("{}{}", self.base_url, path);
        let resp = self.send_raw_request("GET", &url, None).await?;
        self.process_response(resp).await
    }

    pub async fn list_conversations_v1(
        &self,
        bot_id: &str,
        workspace_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::conversation::LIST_CONVERSATIONS;
        let mut url = format!("{}{}?bot_id={}", self.base_url, LIST_CONVERSATIONS, encode(bot_id));
        if let Some(ws) = workspace_id { url.push_str(&format!("&workspace_id={}", encode(ws))); }
        if let Some(p) = page { url.push_str(&format!("&page={}", p)); }
        if let Some(ps) = page_size { url.push_str(&format!("&page_size={}", ps)); }
        let resp = self.send_raw_request("GET", &url, None).await?;
        self.process_response(resp).await
    }

    pub async fn create_knowledge_base_with_permission(
        &self,
        name: String,
        description: Option<String>,
        space_id: Option<String>,
        permission: Option<i32>,
    ) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::datasets_cn::CREATE_KNOWLEDGE_BASE;
        let url = format!("{}{}", self.base_url, CREATE_KNOWLEDGE_BASE);
        let payload = json!({
            "name": name,
            "description": description.unwrap_or_default(),
            "space_id": space_id,
            "permission": permission,
        });
        let resp = self.send_raw_request("POST", &url, Some(payload)).await?;
        self.process_response(resp).await
    }


    /// Official /v1/datasets list API (as per public documentation)
    pub async fn list_datasets(
        &self,
        space_id: &str,
        name: Option<&str>,
        format_type: Option<i32>,
        page_num: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<crate::api::ListKnowledgeBasesResponse, ApiError> {
        use crate::api::endpoints::datasets_v1::LIST_DATASETS;
        // Basic validation according to doc: page_num >=1, page_size 1..=300
        if let Some(pn) = page_num { if pn == 0 { return Err(ApiError::BadRequest("page_num must be >= 1".into())); } }
        if let Some(ps) = page_size { if ps == 0 || ps > 300 { return Err(ApiError::BadRequest("page_size must be in 1..=300".into())); } }
        let mut url = format!("{}{}?space_id={}", self.base_url, LIST_DATASETS, encode(space_id));
        if let Some(n) = name { if !n.is_empty() { url.push_str(&format!("&name={}", encode(n))); } }
        if let Some(f) = format_type { url.push_str(&format!("&format_type={}", f)); }
        if let Some(pn) = page_num { url.push_str(&format!("&page_num={}", pn)); }
        if let Some(ps) = page_size { url.push_str(&format!("&page_size={}", ps)); }
        let resp = self.send_raw_request("GET", &url, None).await?;
        let raw_text_status = resp.status();
        let text = resp.text().await?;
        if !raw_text_status.is_success() { return Err(ApiError::from_response(raw_text_status, text)); }
        // Try direct deserialize first
        let parsed: Result<crate::api::ListDatasetsApiResponse, _> = serde_json::from_str(&text);
        if let Ok(r) = parsed {
            return Ok(r.into_internal());
        }
        // Fallback: tolerant mapping similar to CN version
        let raw: serde_json::Value = serde_json::from_str(&text).map_err(ApiError::from)?;
        let data = raw.get("data").unwrap_or(&raw);
        let list = data.get("dataset_list").or_else(|| data.get("datasets")).cloned().unwrap_or(serde_json::Value::Null);
        let total = data.get("total_count").or_else(|| data.get("total")).and_then(|v| v.as_u64()).unwrap_or_else(|| list.as_array().map(|a| a.len() as u64).unwrap_or(0)) as usize;
        let mut datasets: Vec<crate::api::KnowledgeBaseInfo> = Vec::new();
        if let Some(arr) = list.as_array() { for item in arr { if let Some(obj) = item.as_object() {
            let dataset_id = obj.get("dataset_id").or_else(|| obj.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name_v = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let description = obj.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let created_at = obj.get("create_time").or_else(|| obj.get("created_at")).and_then(|v| v.as_i64()).unwrap_or(0);
            let doc_count = obj.get("doc_count").or_else(|| obj.get("document_count")).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            datasets.push(crate::api::KnowledgeBaseInfo {
                dataset_id,
                name: name_v,
                description,
                created_at,
                document_count: doc_count,
                update_time: obj.get("update_time").and_then(|v| v.as_i64()),
                status: obj.get("status").and_then(|v| v.as_i64()).map(|v| v as i32),
                format_type: obj.get("format_type").and_then(|v| v.as_i64()).map(|v| v as i32),
                slice_count: obj.get("slice_count").and_then(|v| v.as_u64()).map(|v| v as usize),
                space_id: obj.get("space_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                dataset_type: obj.get("dataset_type").and_then(|v| v.as_i64()).map(|v| v as i32),
                can_edit: obj.get("can_edit").and_then(|v| v.as_bool()),
                icon_url: obj.get("icon_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                icon_uri: obj.get("icon_uri").and_then(|v| v.as_str()).map(|s| s.to_string()),
                avatar_url: obj.get("avatar_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                creator_id: obj.get("creator_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                creator_name: obj.get("creator_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                hit_count: obj.get("hit_count").and_then(|v| v.as_u64()).map(|v| v as usize),
                all_file_size: obj.get("all_file_size").and_then(|v| if v.is_u64() { v.as_u64() } else { v.as_str().and_then(|s| s.parse::<u64>().ok()) }),
                bot_used_count: obj.get("bot_used_count").and_then(|v| v.as_u64()).map(|v| v as usize),
                file_list: obj.get("file_list").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()),
                failed_file_list: obj.get("failed_file_list").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()),
                processing_file_list: obj.get("processing_file_list").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()),
                processing_file_id_list: obj.get("processing_file_id_list").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()),
                chunk_strategy: obj.get("chunk_strategy").cloned(),
                storage_config: obj.get("storage_config").cloned(),
                project_id: obj.get("project_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                raw_extra: None,
            }); } }
        }
        Ok(crate::api::ListKnowledgeBasesResponse { datasets, total })
    }

}


