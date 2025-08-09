// 配置管理改进示例
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub api: ApiConfig,
    pub server: ServerSettings,
    pub cache: CacheConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub token: String,
    pub timeout_seconds: u64,
    pub rate_limit: RateLimit,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub default_space_id: Option<String>,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File { path: String },
    Both { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            server: ServerSettings::default(),
            cache: CacheConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.coze.cn".to_string(),
            token: String::new(),
            timeout_seconds: 30,
            rate_limit: RateLimit::default(),
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            default_space_id: None,
            max_concurrent_requests: 100,
            request_timeout_seconds: 30,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 300, // 5 minutes
            max_entries: 1000,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Stdout,
        }
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            burst_limit: 10,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

impl ServerConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // API 配置
        if let Ok(token) = std::env::var("COZE_API_TOKEN") {
            config.api.token = token;
        }
        
        if let Ok(base_url) = std::env::var("COZE_API_BASE_URL") {
            config.api.base_url = base_url;
        }
        
        if let Ok(timeout) = std::env::var("COZE_API_TIMEOUT") {
            config.api.timeout_seconds = timeout.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "COZE_API_TIMEOUT".to_string(),
                    value: timeout,
                })?;
        }
        
        // 服务器配置
        if let Ok(space_id) = std::env::var("COZE_DEFAULT_SPACE_ID") {
            config.server.default_space_id = Some(space_id);
        }
        
        if let Ok(max_concurrent) = std::env::var("COZE_MAX_CONCURRENT_REQUESTS") {
            config.server.max_concurrent_requests = max_concurrent.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "COZE_MAX_CONCURRENT_REQUESTS".to_string(),
                    value: max_concurrent,
                })?;
        }
        
        // 缓存配置
        if let Ok(cache_enabled) = std::env::var("COZE_CACHE_ENABLED") {
            config.cache.enabled = cache_enabled.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "COZE_CACHE_ENABLED".to_string(),
                    value: cache_enabled,
                })?;
        }
        
        if let Ok(cache_ttl) = std::env::var("COZE_CACHE_TTL_SECONDS") {
            config.cache.ttl_seconds = cache_ttl.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "COZE_CACHE_TTL_SECONDS".to_string(),
                    value: cache_ttl,
                })?;
        }
        
        // 日志配置
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.logging.level = log_level;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证 API token
        if self.api.token.is_empty() {
            return Err(ConfigError::MissingEnvVar("COZE_API_TOKEN".to_string()));
        }
        
        if !self.api.token.starts_with("pat_") {
            return Err(ConfigError::ValidationFailed(
                "API token must start with 'pat_'".to_string()
            ));
        }
        
        // 验证 URL
        if !self.api.base_url.starts_with("http") {
            return Err(ConfigError::ValidationFailed(
                "Base URL must start with http:// or https://".to_string()
            ));
        }
        
        // 验证超时时间
        if self.api.timeout_seconds == 0 || self.api.timeout_seconds > 300 {
            return Err(ConfigError::ValidationFailed(
                "Timeout must be between 1 and 300 seconds".to_string()
            ));
        }
        
        // 验证并发请求数
        if self.server.max_concurrent_requests == 0 || self.server.max_concurrent_requests > 1000 {
            return Err(ConfigError::ValidationFailed(
                "Max concurrent requests must be between 1 and 1000".to_string()
            ));
        }
        
        // 验证缓存配置
        if self.cache.enabled && self.cache.max_entries == 0 {
            return Err(ConfigError::ValidationFailed(
                "Cache max entries must be greater than 0 when cache is enabled".to_string()
            ));
        }
        
        // 验证日志级别
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationFailed(
                format!("Invalid log level: {}. Valid levels: {:?}", self.logging.level, valid_levels)
            ));
        }
        
        Ok(())
    }
    
    /// 获取超时时间
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.api.timeout_seconds)
    }
    
    /// 获取缓存 TTL
    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.ttl_seconds)
    }
    
    /// 是否启用缓存
    pub fn is_cache_enabled(&self) -> bool {
        self.cache.enabled
    }
    
    /// 获取速率限制配置
    pub fn rate_limit(&self) -> &RateLimit {
        &self.api.rate_limit
    }
    
    /// 获取重试配置
    pub fn retry_config(&self) -> &RetryConfig {
        &self.api.retry_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.api.base_url, "https://api.coze.cn");
        assert_eq!(config.api.timeout_seconds, 30);
        assert!(config.cache.enabled);
    }
    
    #[test]
    fn test_config_validation_missing_token() {
        let config = ServerConfig::default();
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_validation_invalid_token() {
        let mut config = ServerConfig::default();
        config.api.token = "invalid_token".to_string();
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_validation_valid() {
        let mut config = ServerConfig::default();
        config.api.token = "pat_valid_token_here".to_string();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_from_env() {
        env::set_var("COZE_API_TOKEN", "pat_test_token");
        env::set_var("COZE_DEFAULT_SPACE_ID", "test_space");
        
        let config = ServerConfig::from_env().unwrap();
        assert_eq!(config.api.token, "pat_test_token");
        assert_eq!(config.server.default_space_id, Some("test_space".to_string()));
        
        env::remove_var("COZE_API_TOKEN");
        env::remove_var("COZE_DEFAULT_SPACE_ID");
    }
}