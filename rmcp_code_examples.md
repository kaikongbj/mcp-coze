# RMCP 代码示例集合

## 基础服务器示例

### 1. 最小可运行的RMCP服务器

```rust
use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo, Tool,
    },
    service::{serve_server, RequestContext, RoleServer},
    ErrorData as McpError,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SimpleServer {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl SimpleServer {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ServerHandler for SimpleServer {
    async fn call_tool(
        &self,
        params: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match params.name.as_str() {
            "store" => {
                let args = params.arguments.unwrap_or_default();
                let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("default");
                let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
                
                let mut data = self.data.write().await;
                data.insert(key.to_string(), value.to_string());
                
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        format!("Stored: {} = {}", key, value)
                    )]),
                    is_error: Some(false),
                    structured_content: Some(serde_json::json!({
                        "key": key,
                        "value": value,
                        "status": "stored"
                    })),
                })
            }
            "get" => {
                let args = params.arguments.unwrap_or_default();
                let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("default");
                
                let data = self.data.read().await;
                let value = data.get(key).map(|s| s.as_str()).unwrap_or("Not found");
                
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        format!("Value for '{}': {}", key, value)
                    )]),
                    is_error: Some(false),
                    structured_content: Some(serde_json::json!({
                        "key": key,
                        "value": value
                    })),
                })
            }
            _ => Err(McpError::method_not_found()),
        }
    }

    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = vec![
            Tool {
                name: "store".into(),
                description: Some("Store a key-value pair".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "The key to store"},
                        "value": {"type": "string", "description": "The value to store"}
                    },
                    "required": ["key", "value"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            Tool {
                name: "get".into(),
                description: Some("Get a value by key".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "The key to retrieve"}
                    },
                    "required": ["key"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
        ];
        
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: rmcp::model::Implementation {
                name: "simple-mcp-server".into(),
                version: "0.1.0".into(),
            },
            instructions: Some("A simple key-value store MCP server".into()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let server = SimpleServer::new();
    let running_service = serve_server(server, rmcp::transport::stdio()).await?;
    running_service.waiting().await?;
    
    Ok(())
}
```

### 2. 使用宏的简化版本

```rust
use rmcp::{tool, tool_router, ServiceExt};
use rmcp::model::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct MacroServer {
    data: Arc<RwLock<HashMap<String, String>>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl MacroServer {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Add two numbers")]
    async fn add(&self, a: i32, b: i32) -> String {
        (a + b).to_string()
    }

    #[tool(description = "Store a key-value pair")]
    async fn store(&self, key: String, value: String) -> String {
        let mut data = self.data.write().await;
        data.insert(key.clone(), value.clone());
        format!("Stored: {} = {}", key, value)
    }

    #[tool(description = "Get a value by key")]
    async fn get(&self, key: String) -> Option<String> {
        let data = self.data.read().await;
        data.get(&key).cloned()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = MacroServer::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

## 高级特性示例

### 1. 复杂参数验证

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UserProfile {
    #[schemars(description = "用户唯一标识符")]
    pub user_id: String,
    #[schemars(description = "用户姓名")]
    pub name: String,
    #[schemars(description = "用户邮箱")]
    pub email: Option<String>,
    #[schemars(description = "用户年龄", minimum = 0, maximum = 150)]
    pub age: Option<u8>,
}

#[tool(description = "创建用户资料")]
async fn create_user(&self, #[tool(aggr)] profile: UserProfile) -> Result<String, String> {
    // 验证邮箱格式
    if let Some(ref email) = profile.email {
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
    }
    
    // 验证年龄
    if let Some(age) = profile.age {
        if age < 18 {
            return Err("User must be 18 or older".to_string());
        }
    }
    
    Ok(format!("Created user: {}", profile.user_id))
}
```

### 2. 异步外部API调用

```rust
use reqwest;
use serde_json::Value;

#[derive(Clone)]
struct WeatherServer {
    http_client: reqwest::Client,
    api_key: String,
}

#[tool(description = "获取天气信息")]
async fn get_weather(&self, city: String) -> Result<String, String> {
    let url = format!(
        "http://api.weatherapi.com/v1/current.json?key={}&q={}",
        self.api_key, city
    );
    
    let response = self.http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    let weather_data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let temp = weather_data["current"]["temp_c"]
        .as_f64()
        .ok_or("Failed to get temperature")?;
    
    Ok(format!("Current temperature in {}: {}°C", city, temp))
}
```

### 3. 错误处理和重试

```rust
use tokio::time::{sleep, Duration};

#[tool(description = "执行可能失败的操作")]
async fn risky_operation(&self, input: String) -> Result<String, McpError> {
    let mut retries = 0;
    const MAX_RETRIES: u32 = 3;
    
    loop {
        match self.perform_operation(&input).await {
            Ok(result) => return Ok(result),
            Err(e) if retries < MAX_RETRIES => {
                retries += 1;
                tracing::warn!("Operation failed, retrying {}/{}: {}", retries, MAX_RETRIES, e);
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(McpError::internal_error(format!("Operation failed: {}", e)));
            }
        }
    }
}

async fn perform_operation(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 实际的操作逻辑
    if input.is_empty() {
        return Err("Input cannot be empty".into());
    }
    Ok(format!("Processed: {}", input.to_uppercase()))
}
```

## 测试示例

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolRequestParam;
    use serde_json::json;

    #[tokio::test]
    async fn test_store_and_get() {
        let server = SimpleServer::new();
        
        // 测试存储
        let store_request = CallToolRequestParam {
            name: "store".to_string(),
            arguments: Some(json!({
                "key": "test_key",
                "value": "test_value"
            }).as_object().unwrap().clone()),
        };
        
        let result = server.call_tool(store_request, Default::default()).await;
        assert!(result.is_ok());
        
        // 测试获取
        let get_request = CallToolRequestParam {
            name: "get".to_string(),
            arguments: Some(json!({
                "key": "test_key"
            }).as_object().unwrap().clone()),
        };
        
        let result = server.call_tool(get_request, Default::default()).await;
        assert!(result.is_ok());
        
        if let Ok(call_result) = result {
            if let Some(content) = call_result.content {
                assert!(content[0].text().unwrap().contains("test_value"));
            }
        }
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = SimpleServer::new();
        let tools = server.list_tools(None, Default::default()).await.unwrap();
        
        assert_eq!(tools.tools.len(), 2);
        assert!(tools.tools.iter().any(|t| t.name == "store"));
        assert!(tools.tools.iter().any(|t| t.name == "get"));
    }
}
```

### 2. 集成测试

```rust
#[tokio::test]
async fn test_full_integration() {
    use rmcp::transport::stdio;
    use tokio::io::{stdin, stdout};
    
    let server = SimpleServer::new();
    let transport = (stdin(), stdout());
    
    let service = serve_server(server, transport).await.unwrap();
    
    // 在实际测试中，这里会模拟客户端请求
    // service.waiting().await.unwrap();
}
```

## 部署和配置

### 1. 环境变量配置

```bash
# .env 文件示例
COZE_API_KEY=your_api_key_here
COZE_DEFAULT_SPACE_ID=your_space_id
COZE_API_BASE_URL=https://api.coze.cn
RUST_LOG=info
```

### 2. 日志配置

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();
}
```

### 3. Docker部署

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/my-mcp-server /app/server
ENTRYPOINT ["./server"]
```

这些示例展示了RMCP模块的完整使用方法，从基础配置到高级特性，涵盖了实际开发中的各种场景。