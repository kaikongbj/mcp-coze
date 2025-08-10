use crate::api::endpoints::{
    KNOWLEDGE_DOCUMENT_CREATE_URL,
};
use crate::api::error::{ApiError, ApiErrorData};
// Chat completion models removed (unused)
use reqwest::{Client, Response};
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
            .map_err(|e| ApiError::ConfigError(ApiErrorData::new("config", format!("Failed to build HTTP client: {}", e), None, None)))?;

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
            .header("Content-Type", "application/json")
            // Per official upload spec: include Agw-Js-Conv to preserve numeric precision (harmless elsewhere)
            .header("Agw-Js-Conv", "str");

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

    /// Execute a generic API request
    pub async fn execute_request(
        &self,
        req: crate::models::CozeApiRequest,
    ) -> Result<crate::models::CozeApiResponse, ApiError> {
        use crate::models::HttpMethod;
        
        let method_str = match req.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        };

        let mut url = format!("{}{}", self.base_url, req.endpoint);
        
        // Add query parameters for GET requests
        if !req.params.is_empty() && matches!(req.method, HttpMethod::Get) {
            let params: Vec<String> = req.params
                .iter()
                .map(|(k, v)| {
                    let value_str = match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string().trim_matches('"').to_string(),
                    };
                    format!("{}={}", urlencoding::encode(k), urlencoding::encode(&value_str))
                })
                .collect();
            if !params.is_empty() {
                url.push('?');
                url.push_str(&params.join("&"));
            }
        }

        let response = self.send_raw_request(method_str, &url, req.body).await?;
        let status_code = response.status().as_u16();
        let headers = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body_text = response.text().await?;
        let body: serde_json::Value = serde_json::from_str(&body_text)
            .unwrap_or(serde_json::Value::String(body_text));

        Ok(crate::models::CozeApiResponse {
            status_code,
            headers,
            body,
            success: (200..300).contains(&status_code),
        })
    }

    /// Create knowledge base with permission (legacy compatibility method)
    /// This method wraps the standard create_dataset API with additional parameters
    pub async fn create_knowledge_base_with_permission(
        &self,
        name: String,
        description: Option<String>,
        space_id: Option<String>,
        _permission: Option<i32>, // Note: permission parameter not used in current API
    ) -> Result<serde_json::Value, ApiError> {
        use crate::api::knowledge_models::CreateDatasetRequest;
        
        // Default space_id if not provided (this should be configured properly)
        let space_id = space_id.unwrap_or_else(|| "default_space".to_string());
        
        let request = CreateDatasetRequest {
            name,
            space_id,
            format_type: 0, // Default to text type
            description,
            file_id: None,
        };
        
        let response = self.create_dataset(request).await?;
        
        // Convert to generic JSON value for compatibility
        serde_json::to_value(response).map_err(ApiError::from)
    }

    // Removed deprecated create_knowledge_base* variants (single public path retained at tool layer if needed)

    // Removed deprecated upload_document_to_knowledge_base* variants
    /// CN Spec aligned upload (document_bases -> name + source_info)
    pub async fn upload_document_cn(
        &self,
        req: crate::api::knowledge_models::KnowledgeDocumentUploadRequestCn,
    ) -> Result<crate::api::knowledge_models::KnowledgeDocumentUploadResponseCn, ApiError> {
        let url = format!("{}{}", self.base_url, KNOWLEDGE_DOCUMENT_CREATE_URL);
    let sanitized = req.sanitized();
    let payload = serde_json::to_value(&sanitized).map_err(ApiError::from)?;
        println!("Upload payload: {:?}", payload);
        let resp = self.send_raw_request("POST", &url, Some(payload)).await?;
        println!("Upload response: {:?}", resp);
        self.process_response(resp).await
    }

    // chat_completion methods removed (tool layer does not expose)

    // ---- Additional API helpers required by tools ----
    pub async fn get_dataset_cn(&self, dataset_id: &str) -> Result<serde_json::Value, ApiError> {
        use crate::api::endpoints::datasets_cn::GET_KNOWLEDGE_BASE;
        let url = format!("{}{}?dataset_id={}", self.base_url, GET_KNOWLEDGE_BASE, encode(dataset_id));
        let resp = self.send_raw_request("GET", &url, None).await?;
        self.process_response(resp).await
    }

    // retrieve_conversation_v1 & get_conversation_messages_v3 removed (not used)

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

    /// 创建知识库 (符合 POST /v1/datasets API 文档规范)
    pub async fn create_dataset(
        &self,
        request: crate::api::knowledge_models::CreateDatasetRequest,
    ) -> Result<crate::api::knowledge_models::CreateDatasetResponse, ApiError> {
        use crate::api::endpoints::datasets_v1::CREATE_DATASETS;
        let url = format!("{}{}", self.base_url, CREATE_DATASETS);
        
        // 将请求转换为 JSON 格式
        let payload = serde_json::to_value(&request).map_err(ApiError::from)?;
        
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
    if let Some(pn) = page_num { if pn == 0 { return Err(ApiError::BadRequest(ApiErrorData::new("bad_request", "page_num must be >= 1".to_string(), None, None))); } }
    if let Some(ps) = page_size { if ps == 0 || ps > 300 { return Err(ApiError::BadRequest(ApiErrorData::new("bad_request", "page_size must be in 1..=300".to_string(), None, None))); } }
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

    // ---- Chat API methods ----
    
    /// 发送聊天请求（非流式）
    pub async fn chat(
        &self,
        request: crate::api::chat_models::ChatRequest,
    ) -> Result<crate::api::chat_models::ChatResponse, ApiError> {
        use crate::api::endpoints::chat::CHAT_V3;
        let url = format!("{}{}", self.base_url, CHAT_V3);
        
        let mut req = request;
        req.stream = Some(false); // 确保非流式
        
        let payload = serde_json::to_value(&req).map_err(ApiError::from)?;
        let resp = self.send_raw_request("POST", &url, Some(payload)).await?;
        
        // 解析响应
        let status = resp.status();
        let body = resp.text().await?;
        
        if !status.is_success() {
            return Err(ApiError::from_response(status, body));
        }
        
        // 尝试解析为标准响应格式
        let response_value: serde_json::Value = serde_json::from_str(&body)
            .map_err(ApiError::from)?;
        
        // 检查是否有业务错误
        if let Some(code) = response_value.get("code") {
            if let Some(code_num) = code.as_i64() {
                if code_num != 0 {
                    let msg = response_value.get("msg")
                        .or_else(|| response_value.get("message"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    return Err(ApiError::BadRequest(ApiErrorData::new(
                        "api_error",
                        format!("API returned error code {}: {}", code_num, msg),
                        Some(status.as_u16()),
                        Some(body),
                    )));
                }
            }
        }
        
        // 提取data字段或使用整个响应
        let data = response_value.get("data").unwrap_or(&response_value);
        
        serde_json::from_value(data.clone()).map_err(ApiError::from)
    }
    
    /// 发送流式聊天请求
    pub async fn chat_stream(
        &self,
        request: crate::api::chat_models::ChatRequest,
    ) -> Result<impl futures::Stream<Item = Result<crate::api::chat_models::StreamChatResponse, ApiError>>, ApiError> {
        use crate::api::endpoints::chat::CHAT_V3_STREAM;
        use futures::stream::StreamExt;
        
        let url = format!("{}{}", self.base_url, CHAT_V3_STREAM);
        
        let mut req = request;
        req.stream = Some(true); // 确保流式
        
        let payload = serde_json::to_value(&req).map_err(ApiError::from)?;
        
        let request_builder = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .json(&payload);
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(ApiError::from_response(status, body));
        }
        
        let stream = response.bytes_stream()
            .map(|chunk_result| {
                chunk_result
                    .map_err(ApiError::from)
                    .and_then(|chunk| {
                        let text = String::from_utf8_lossy(&chunk);
                        Self::parse_sse_chunk(&text)
                    })
            })
            .filter_map(|result| async move {
                match result {
                    Ok(Some(response)) => Some(Ok(response)),
                    Ok(None) => None, // 跳过空块或注释
                    Err(e) => Some(Err(e)),
                }
            });
        
        Ok(stream)
    }
    
    /// 解析SSE (Server-Sent Events) 数据块
    fn parse_sse_chunk(text: &str) -> Result<Option<crate::api::chat_models::StreamChatResponse>, ApiError> {
        for line in text.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            
            // 解析data字段
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    return Ok(Some(crate::api::chat_models::StreamChatResponse {
                        event: crate::api::chat_models::StreamEventType::Done,
                        conversation_id: None,
                        id: None,
                        created_at: None,
                        delta: None,
                        usage: None,
                        last_error: None,
                    }));
                }
                
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .map_err(ApiError::from)?;
                
                // 检查是否有业务错误
                if let Some(code) = parsed.get("code") {
                    if let Some(code_num) = code.as_i64() {
                        if code_num != 0 {
                            let msg = parsed.get("msg")
                                .or_else(|| parsed.get("message"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error");
                            return Err(ApiError::BadRequest(ApiErrorData::new(
                                "stream_error",
                                format!("Stream returned error code {}: {}", code_num, msg),
                                None,
                                Some(data.to_string()),
                            )));
                        }
                    }
                }
                
                // 提取data字段或使用整个响应
                let data_field = parsed.get("data").unwrap_or(&parsed);
                
                let response: crate::api::chat_models::StreamChatResponse = 
                    serde_json::from_value(data_field.clone()).map_err(ApiError::from)?;
                
                return Ok(Some(response));
            }
        }
        
        Ok(None)
    }

    /// 使用类型化模型获取智能体列表
    pub async fn list_bots_typed(
        &self,
        request: &crate::api::bot_models::ListBotsRequest,
    ) -> Result<crate::api::bot_models::ListBotsResponse, ApiError> {
        use crate::api::endpoints::bots::LIST_BOTS;
        
        let url = format!("{}{}?{}", self.base_url, LIST_BOTS, request.to_query_params());
        
        let response = self.send_raw_request("GET", &url, None).await?;
        let text = response.text().await.map_err(ApiError::from)?;
        
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(ApiError::from)?;
        
        // 检查业务错误
        if let Some(code) = parsed.get("code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = parsed.get("msg")
                    .or_else(|| parsed.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(ApiError::BadRequest(ApiErrorData::new(
                    "list_bots",
                    format!("API returned error code {}: {}", code, msg),
                    None,
                    Some(text),
                )));
            }
        }
        
        // 解析响应
        let response: crate::api::bot_models::ListBotsResponse = 
            serde_json::from_str(&text).map_err(ApiError::from)?;
        
        Ok(response)
    }

    /// 获取对话详情
    pub async fn get_chat_detail(
        &self,
        conversation_id: &str,
        chat_id: &str,
    ) -> Result<crate::api::chat_models::ChatResponse, ApiError> {
        use crate::api::endpoints::chat::GET_CHAT_DETAIL;
        
        let url = format!("{}{}?conversation_id={}&chat_id={}", 
            self.base_url, GET_CHAT_DETAIL, encode(conversation_id), encode(chat_id));
        
        let response = self.send_raw_request("GET", &url, None).await?;
        let text = response.text().await.map_err(ApiError::from)?;
        
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(ApiError::from)?;
        
        // 检查业务错误
        if let Some(code) = parsed.get("code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = parsed.get("msg")
                    .or_else(|| parsed.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(ApiError::BadRequest(ApiErrorData::new(
                    "get_chat_detail",
                    format!("API returned error code {}: {}", code, msg),
                    None,
                    Some(text),
                )));
            }
        }
        
        // 提取data字段或使用整个响应
        let data = parsed.get("data").unwrap_or(&parsed);
        serde_json::from_value(data.clone()).map_err(ApiError::from)
    }

    /// 获取对话消息列表
    pub async fn get_chat_messages(
        &self,
        conversation_id: &str,
        chat_id: &str,
    ) -> Result<Vec<crate::api::chat_models::ChatMessage>, ApiError> {
        use crate::api::endpoints::chat::GET_CHAT_MESSAGES;
        
        let url = format!("{}{}?conversation_id={}&chat_id={}", 
            self.base_url, GET_CHAT_MESSAGES, encode(conversation_id), encode(chat_id));
        
        let response = self.send_raw_request("GET", &url, None).await?;
        let text = response.text().await.map_err(ApiError::from)?;
        
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(ApiError::from)?;
        
        // 检查业务错误
        if let Some(code) = parsed.get("code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = parsed.get("msg")
                    .or_else(|| parsed.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(ApiError::BadRequest(ApiErrorData::new(
                    "get_chat_messages",
                    format!("API returned error code {}: {}", code, msg),
                    None,
                    Some(text),
                )));
            }
        }
        
        // 提取messages字段
        let empty_array = serde_json::Value::Array(vec![]);
        let messages = parsed.get("data")
            .and_then(|d| d.get("data"))
            .or_else(|| parsed.get("data"))
            .or_else(|| parsed.get("messages"))
            .unwrap_or(&empty_array);
            
        serde_json::from_value(messages.clone()).map_err(ApiError::from)
    }

}


