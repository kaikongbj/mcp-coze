// 日志和监控改进示例
use tracing::{info, warn, error, debug, instrument, Span};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};
use serde_json::json;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

// 性能指标收集器
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    request_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    response_times: Arc<RwLock<Vec<Duration>>>,
    endpoint_stats: Arc<RwLock<HashMap<String, EndpointStats>>>,
}

#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub total_requests: u64,
    pub error_count: u64,
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            response_times: Arc::new(RwLock::new(Vec::new())),
            endpoint_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub async fn record_response_time(&self, endpoint: &str, duration: Duration) {
        // 记录全局响应时间
        {
            let mut times = self.response_times.write().await;
            times.push(duration);
            
            // 保持最近1000个记录
            if times.len() > 1000 {
                times.drain(0..500);
            }
        }
        
        // 记录端点特定统计
        {
            let mut stats = self.endpoint_stats.write().await;
            let entry = stats.entry(endpoint.to_string()).or_insert(EndpointStats {
                total_requests: 0,
                error_count: 0,
                avg_response_time: Duration::from_millis(0),
                min_response_time: Duration::from_secs(999),
                max_response_time: Duration::from_millis(0),
            });
            
            entry.total_requests += 1;
            entry.min_response_time = entry.min_response_time.min(duration);
            entry.max_response_time = entry.max_response_time.max(duration);
            
            // 计算移动平均
            let total_time = entry.avg_response_time.as_millis() as u64 * (entry.total_requests - 1) + duration.as_millis() as u64;
            entry.avg_response_time = Duration::from_millis(total_time / entry.total_requests);
        }
    }
    
    pub async fn record_error(&self, endpoint: &str) {
        self.increment_errors();
        
        let mut stats = self.endpoint_stats.write().await;
        if let Some(entry) = stats.get_mut(endpoint) {
            entry.error_count += 1;
        }
    }
    
    pub async fn get_stats(&self) -> MetricsSnapshot {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);
        
        let response_times = self.response_times.read().await;
        let avg_response_time = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: u128 = response_times.iter().map(|d| d.as_millis()).sum();
            Duration::from_millis((total / response_times.len() as u128) as u64)
        };
        
        let endpoint_stats = self.endpoint_stats.read().await.clone();
        
        MetricsSnapshot {
            total_requests: request_count,
            total_errors: error_count,
            error_rate: if request_count > 0 { 
                (error_count as f64 / request_count as f64) * 100.0 
            } else { 
                0.0 
            },
            avg_response_time,
            endpoint_stats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub avg_response_time: Duration,
    pub endpoint_stats: HashMap<String, EndpointStats>,
}

// 日志配置和初始化
pub fn init_logging(log_level: &str, json_format: bool) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))?;
    
    let fmt_layer = if json_format {
        fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .boxed()
    } else {
        fmt::layer()
            .pretty()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .boxed()
    };
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
    
    info!("Logging initialized with level: {}", log_level);
    Ok(())
}

// 带监控的 API 客户端装饰器
#[derive(Debug, Clone)]
pub struct MonitoredApiClient {
    inner: crate::api::CozeApiClient,
    metrics: MetricsCollector,
}

impl MonitoredApiClient {
    pub fn new(client: crate::api::CozeApiClient) -> Self {
        Self {
            inner: client,
            metrics: MetricsCollector::new(),
        }
    }
    
    #[instrument(skip(self, req), fields(endpoint = %req.endpoint, method = ?req.method))]
    pub async fn execute_request_monitored(
        &self,
        req: crate::models::CozeApiRequest,
    ) -> Result<crate::models::CozeApiResponse, crate::api::error::ApiError> {
        let endpoint = req.endpoint.clone();
        let start = Instant::now();
        
        self.metrics.increment_requests();
        
        debug!("Starting API request to {}", endpoint);
        
        let result = self.inner.execute_request(req).await;
        let duration = start.elapsed();
        
        match &result {
            Ok(response) => {
                if response.success {
                    info!(
                        endpoint = %endpoint,
                        duration_ms = duration.as_millis(),
                        status = response.status_code,
                        "API request completed successfully"
                    );
                } else {
                    warn!(
                        endpoint = %endpoint,
                        duration_ms = duration.as_millis(),
                        status = response.status_code,
                        "API request completed with error status"
                    );
                    self.metrics.record_error(&endpoint).await;
                }
                self.metrics.record_response_time(&endpoint, duration).await;
            }
            Err(e) => {
                error!(
                    endpoint = %endpoint,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "API request failed"
                );
                self.metrics.record_error(&endpoint).await;
            }
        }
        
        result
    }
    
    pub async fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.get_stats().await
    }
    
    pub async fn log_metrics_summary(&self) {
        let stats = self.get_metrics().await;
        
        info!(
            total_requests = stats.total_requests,
            total_errors = stats.total_errors,
            error_rate = format!("{:.2}%", stats.error_rate),
            avg_response_time_ms = stats.avg_response_time.as_millis(),
            "API metrics summary"
        );
        
        // 记录每个端点的详细统计
        for (endpoint, endpoint_stats) in &stats.endpoint_stats {
            debug!(
                endpoint = %endpoint,
                requests = endpoint_stats.total_requests,
                errors = endpoint_stats.error_count,
                avg_time_ms = endpoint_stats.avg_response_time.as_millis(),
                min_time_ms = endpoint_stats.min_response_time.as_millis(),
                max_time_ms = endpoint_stats.max_response_time.as_millis(),
                "Endpoint statistics"
            );
        }
    }
}

// 工具调用监控装饰器
#[derive(Debug, Clone)]
pub struct MonitoredCozeTools {
    inner: crate::tools::coze_tools::CozeTools,
    metrics: MetricsCollector,
}

impl MonitoredCozeTools {
    pub fn new(tools: crate::tools::coze_tools::CozeTools) -> Self {
        Self {
            inner: tools,
            metrics: MetricsCollector::new(),
        }
    }
    
    #[instrument(skip(self, args), fields(tool = %tool_name))]
    pub async fn call_tool_monitored(
        &self,
        tool_name: &str,
        args: Option<serde_json::Value>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let start = Instant::now();
        self.metrics.increment_requests();
        
        debug!(tool = %tool_name, args = ?args, "Calling MCP tool");
        
        let result = match tool_name {
            "list_workspaces" => self.inner.list_workspaces(args).await,
            "list_bots" => self.inner.list_bots(args).await,
            "list_knowledge_bases" => self.inner.list_knowledge_bases(args).await,
            "create_knowledge_base_v2" => self.inner.create_knowledge_base_v2(args).await,
            "upload_document_to_knowledge_base" => self.inner.upload_document_to_knowledge_base(args).await,
            "list_conversations" => self.inner.list_conversations(args).await,
            _ => Err(rmcp::ErrorData::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            )),
        };
        
        let duration = start.elapsed();
        
        match &result {
            Ok(call_result) => {
                let is_error = call_result.is_error.unwrap_or(false);
                if is_error {
                    warn!(
                        tool = %tool_name,
                        duration_ms = duration.as_millis(),
                        "Tool call completed with error"
                    );
                    self.metrics.record_error(tool_name).await;
                } else {
                    info!(
                        tool = %tool_name,
                        duration_ms = duration.as_millis(),
                        "Tool call completed successfully"
                    );
                }
                self.metrics.record_response_time(tool_name, duration).await;
            }
            Err(e) => {
                error!(
                    tool = %tool_name,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "Tool call failed"
                );
                self.metrics.record_error(tool_name).await;
            }
        }
        
        result
    }
    
    pub async fn get_tool_metrics(&self) -> MetricsSnapshot {
        self.metrics.get_stats().await
    }
}

// 健康检查和监控端点
pub struct HealthChecker {
    api_client: MonitoredApiClient,
    tools: MonitoredCozeTools,
}

impl HealthChecker {
    pub fn new(api_client: MonitoredApiClient, tools: MonitoredCozeTools) -> Self {
        Self { api_client, tools }
    }
    
    #[instrument(skip(self))]
    pub async fn check_health(&self) -> HealthStatus {
        let mut status = HealthStatus {
            overall: "healthy".to_string(),
            api_connectivity: false,
            tool_functionality: false,
            metrics: None,
            timestamp: chrono::Utc::now(),
        };
        
        // 检查 API 连接性
        let api_check = self.check_api_connectivity().await;
        status.api_connectivity = api_check;
        
        // 检查工具功能
        let tool_check = self.check_tool_functionality().await;
        status.tool_functionality = tool_check;
        
        // 获取指标
        status.metrics = Some(self.api_client.get_metrics().await);
        
        // 确定整体状态
        if !api_check || !tool_check {
            status.overall = "unhealthy".to_string();
        }
        
        info!(
            overall_status = %status.overall,
            api_connectivity = status.api_connectivity,
            tool_functionality = status.tool_functionality,
            "Health check completed"
        );
        
        status
    }
    
    async fn check_api_connectivity(&self) -> bool {
        // 尝试调用一个简单的 API 端点
        let req = crate::models::CozeApiRequest {
            endpoint: "/v1/workspaces".to_string(),
            method: crate::models::HttpMethod::Get,
            headers: Default::default(),
            params: Default::default(),
            body: None,
        };
        
        match self.api_client.execute_request_monitored(req).await {
            Ok(response) => response.success,
            Err(e) => {
                warn!(error = %e, "API connectivity check failed");
                false
            }
        }
    }
    
    async fn check_tool_functionality(&self) -> bool {
        // 尝试调用一个简单的工具
        match self.tools.call_tool_monitored("list_workspaces", None).await {
            Ok(result) => !result.is_error.unwrap_or(true),
            Err(e) => {
                warn!(error = %e, "Tool functionality check failed");
                false
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub overall: String,
    pub api_connectivity: bool,
    pub tool_functionality: bool,
    pub metrics: Option<MetricsSnapshot>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// 定期指标报告任务
pub async fn start_metrics_reporter(
    api_client: MonitoredApiClient,
    tools: MonitoredCozeTools,
    interval: Duration,
) {
    let mut interval_timer = tokio::time::interval(interval);
    
    loop {
        interval_timer.tick().await;
        
        info!("=== Periodic Metrics Report ===");
        
        // API 客户端指标
        api_client.log_metrics_summary().await;
        
        // 工具指标
        let tool_stats = tools.get_tool_metrics().await;
        info!(
            tool_requests = tool_stats.total_requests,
            tool_errors = tool_stats.total_errors,
            tool_error_rate = format!("{:.2}%", tool_stats.error_rate),
            "Tool metrics summary"
        );
        
        info!("=== End Metrics Report ===");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // 记录一些指标
        collector.increment_requests();
        collector.increment_requests();
        collector.increment_errors();
        
        collector.record_response_time("test_endpoint", Duration::from_millis(100)).await;
        collector.record_response_time("test_endpoint", Duration::from_millis(200)).await;
        collector.record_error("test_endpoint").await;
        
        let stats = collector.get_stats().await;
        
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_errors, 2); // 1 from increment_errors + 1 from record_error
        assert_eq!(stats.error_rate, 100.0);
        
        let endpoint_stats = stats.endpoint_stats.get("test_endpoint").unwrap();
        assert_eq!(endpoint_stats.total_requests, 2);
        assert_eq!(endpoint_stats.error_count, 1);
        assert_eq!(endpoint_stats.avg_response_time, Duration::from_millis(150));
    }
    
    #[test]
    fn test_logging_init() {
        let result = init_logging("debug", false);
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_health_status_serialization() {
        let status = HealthStatus {
            overall: "healthy".to_string(),
            api_connectivity: true,
            tool_functionality: true,
            metrics: None,
            timestamp: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&status);
        assert!(json.is_ok());
    }
}