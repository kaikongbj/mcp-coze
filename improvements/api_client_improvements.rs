// API 客户端改进示例
use reqwest::{Client, Response, RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },
    #[error("Authentication failed: {message}")]
    Authentication { message: String },
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

// 通用 API 端点 trait
pub trait ApiEndpoint {
    type Request: Serialize;
    type Response: DeserializeOwned;
    
    fn endpoint(&self) -> String;
    fn method(&self) -> reqwest::Method;
    fn requires_auth(&self) -> bool { true }
}

// 缓存项
#[derive(Debug, Clone)]
struct CacheItem {
    data: serde_json::Value,
    expires_at: Instant,
}

// 改进的 API 客户端
#[derive(Debug, Clone)]
pub struct CozeApiClient {
    client: Client,
    base_url: String,
    api_key: String,
    cache: Arc<RwLock<HashMap<String, CacheItem>>>,
    cache_ttl: Duration,
    cache_enabled: bool,
}

impl CozeApiClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| ApiError::Config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            api_key,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes default
            cache_enabled: true,
        })
    }
    
    pub fn with_cache_config(mut self, enabled: bool, ttl: Duration) -> Self {
        self.cache_enabled = enabled;
        self.cache_ttl = ttl;
        self
    }
    
    // 通用 API 调用方法
    #[instrument(skip(self, request), fields(endpoint = %endpoint.endpoint()))]
    pub async fn call<T: ApiEndpoint>(&self, endpoint: T, request: T::Request) -> Result<T::Response, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint.endpoint());
        let method = endpoint.method();
        
        debug!("Making API call to: {} {}", method, url);
        
        // 检查缓存（仅对 GET 请求）
        if method == reqwest::Method::GET && self.cache_enabled {
            if let Some(cached) = self.get_from_cache(&url).await {
                debug!("Cache hit for: {}", url);
                return serde_json::from_value(cached)
                    .map_err(ApiError::Serialization);
            }
        }
        
        let mut request_builder = self.client.request(method, &url);
        
        // 添加认证头
        if endpoint.requires_auth() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // 添加请求体
        request_builder = request_builder
            .header("Content-Type", "application/json")
            .json(&request);
        
        let start = Instant::now();
        let response = request_builder.send().await?;
        let duration = start.elapsed();
        
        info!("API call completed in {:?}", duration);
        
        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            return Err(self.handle_error_response(status.as_u16(), &response_text));
        }
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        
        // 缓存响应（仅对 GET 请求）
        if method == reqwest::Method::GET && self.cache_enabled {
            self.set_cache(&url, response_json.clone()).await;
        }
        
        serde_json::from_value(response_json).map_err(ApiError::Serialization)
    }
    
    // 带重试的 API 调用
    #[instrument(skip(self, request), fields(endpoint = %endpoint.endpoint()))]
    pub async fn call_with_retry<T: ApiEndpoint>(
        &self, 
        endpoint: T, 
        request: T::Request,
        max_retries: u32,
        initial_delay: Duration,
    ) -> Result<T::Response, ApiError> {
        let mut delay = initial_delay;
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.call(&endpoint, &request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // 不重试某些错误类型
                    match &e {
                        ApiError::Authentication { .. } => return Err(e),
                        ApiError::Api { status, .. } if *status == 400 || *status == 404 => return Err(e),
                        _ => {}
                    }
                    
                    if attempt < max_retries {
                        warn!("API call failed (attempt {}/{}), retrying in {:?}: {}", 
                              attempt + 1, max_retries + 1, delay, e);
                        tokio::time::sleep(delay).await;
                        delay = delay.mul_f32(2.0).min(Duration::from_secs(60));
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
    
    // 批量 API 调用
    pub async fn call_batch<T: ApiEndpoint + Clone>(
        &self,
        requests: Vec<(T, T::Request)>,
        max_concurrent: usize,
    ) -> Vec<Result<T::Response, ApiError>> {
        use futures::stream::{self, StreamExt};
        
        stream::iter(requests)
            .map(|(endpoint, request)| async move {
                self.call(endpoint, request).await
            })
            .buffer_unordered(max_concurrent)
            .collect()
            .await
    }
    
    // 缓存管理
    async fn get_from_cache(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.cache.read().await;
        if let Some(item) = cache.get(key) {
            if item.expires_at > Instant::now() {
                return Some(item.data.clone());
            }
        }
        None
    }
    
    async fn set_cache(&self, key: &str, value: serde_json::Value) {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), CacheItem {
            data: value,
            expires_at: Instant::now() + self.cache_ttl,
        });
        
        // 清理过期缓存
        let now = Instant::now();
        cache.retain(|_, item| item.expires_at > now);
    }
    
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let now = Instant::now();
        let total = cache.len();
        let expired = cache.values().filter(|item| item.expires_at <= now).count();
        (total, expired)
    }
    
    // 错误处理
    fn handle_error_response(&self, status: u16, body: &str) -> ApiError {
        let error_message = match serde_json::from_str::<serde_json::Value>(body) {
            Ok(json) => {
                json.get("msg")
                    .or_else(|| json.get("message"))
                    .or_else(|| json.get("error"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(body)
                    .to_string()
            }
            Err(_) => body.to_string(),
        };

        match status {
            401 => ApiError::Authentication { message: error_message },
            429 => ApiError::RateLimit { message: error_message },
            _ => ApiError::Api { status, message: error_message },
        }
    }
}

// 具体的 API 端点实现示例
#[derive(Debug, Clone)]
pub struct ListKnowledgeBases {
    pub space_id: String,
    pub name: Option<String>,
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
}

impl ApiEndpoint for ListKnowledgeBases {
    type Request = ();
    type Response = crate::api::ListKnowledgeBasesResponse;
    
    fn endpoint(&self) -> String {
        let mut url = format!("/v1/datasets?space_id={}", urlencoding::encode(&self.space_id));
        
        if let Some(name) = &self.name {
            url.push_str(&format!("&name={}", urlencoding::encode(name)));
        }
        
        if let Some(page_num) = self.page_num {
            url.push_str(&format!("&page_num={}", page_num));
        }
        
        if let Some(page_size) = self.page_size {
            url.push_str(&format!("&page_size={}", page_size));
        }
        
        url
    }
    
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateKnowledgeBase {
    pub name: String,
    pub description: Option<String>,
    pub space_id: Option<String>,
    pub permission: Option<i32>,
}

impl ApiEndpoint for CreateKnowledgeBase {
    type Request = Self;
    type Response = serde_json::Value;
    
    fn endpoint(&self) -> String {
        "/v1/datasets".to_string()
    }
    
    fn method(&self) -> reqwest::Method {
        reqwest::Method::POST
    }
}

// 使用示例
impl CozeApiClient {
    pub async fn list_knowledge_bases_improved(
        &self,
        space_id: String,
        name: Option<String>,
        page_num: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<crate::api::ListKnowledgeBasesResponse, ApiError> {
        let endpoint = ListKnowledgeBases {
            space_id,
            name,
            page_num,
            page_size,
        };
        
        self.call_with_retry(
            endpoint,
            (),
            3, // max retries
            Duration::from_millis(1000), // initial delay
        ).await
    }
    
    pub async fn create_knowledge_base_improved(
        &self,
        name: String,
        description: Option<String>,
        space_id: Option<String>,
        permission: Option<i32>,
    ) -> Result<serde_json::Value, ApiError> {
        let endpoint = CreateKnowledgeBase {
            name: name.clone(),
            description,
            space_id,
            permission,
        };
        
        self.call(endpoint.clone(), endpoint).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use tokio;
    
    #[tokio::test]
    async fn test_api_client_creation() {
        let client = CozeApiClient::new(
            "https://api.coze.cn".to_string(),
            "pat_test_token".to_string(),
        );
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_cache_functionality() {
        let client = CozeApiClient::new(
            "https://api.coze.cn".to_string(),
            "pat_test_token".to_string(),
        ).unwrap();
        
        // 测试缓存设置和获取
        let test_data = serde_json::json!({"test": "data"});
        client.set_cache("test_key", test_data.clone()).await;
        
        let cached = client.get_from_cache("test_key").await;
        assert_eq!(cached, Some(test_data));
        
        // 测试缓存统计
        let (total, expired) = client.cache_stats().await;
        assert_eq!(total, 1);
        assert_eq!(expired, 0);
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/v1/test")
            .with_status(401)
            .with_body(r#"{"msg": "Unauthorized"}"#)
            .create_async().await;
        
        let client = CozeApiClient::new(
            server.url(),
            "invalid_token".to_string(),
        ).unwrap();
        
        // 这里需要实现一个测试端点
        // let result = client.call(test_endpoint, ()).await;
        // assert!(matches!(result, Err(ApiError::Authentication { .. })));
        
        mock.assert_async().await;
    }
}