use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub coze: CozeConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CozeConfig {
    pub api_token: String,
    pub base_url: String,
    pub timeout: u64,
    pub retry_attempts: u32,
}

impl Default for CozeConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            base_url: "https://api.coze.com".to_string(),
            timeout: 30,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub transport: TransportType,
    pub listen_addr: String,
    pub max_connections: u32,
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: TransportType::Stdio,
            listen_addr: "127.0.0.1:8080".to_string(),
            max_connections: 100,
            cors_origins: vec!["*".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    Http,
}

impl std::str::FromStr for TransportType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdio" => Ok(TransportType::Stdio),
            "sse" => Ok(TransportType::Sse),
            "http" => Ok(TransportType::Http),
            _ => Err(format!("invalid transport type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            file_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 尝试从配置文件加载
        let config_path = std::env::var("COZE_MCP_CONFIG")
            .unwrap_or_else(|_| "config.toml".to_string());

        if std::path::Path::new(&config_path).exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            // 使用环境变量或默认值
            let mut config = Config::default();
            
            if let Ok(token) = std::env::var("COZE_API_TOKEN") {
                config.coze.api_token = token;
            }
            
            if let Ok(base_url) = std::env::var("COZE_API_BASE_URL") {
                config.coze.base_url = base_url;
            }
            
            if let Ok(listen_addr) = std::env::var("LISTEN_ADDR") {
                config.server.listen_addr = listen_addr;
            }
            
            if let Ok(transport) = std::env::var("TRANSPORT") {
                config.server.transport = transport.parse()?;
            }
            
            if let Ok(log_level) = std::env::var("LOG_LEVEL") {
                config.logging.level = log_level;
            }

            Ok(config)
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(path, config_str)?;
        Ok(())
    }
}

