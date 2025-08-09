# RMCP (Rust Model Context Protocol) 模块使用方法调研报告

## 项目概述

RMCP 是 Model Context Protocol (MCP) 的官方 Rust SDK 实现，提供了构建符合 MCP 规范的服务器所需的核心功能。本项目基于 rmcp 0.5.0 版本实现了一个 Coze 平台的 MCP 服务器。

## 核心特性

### 1. 异步优先架构
- 基于 Tokio 异步运行时
- 所有操作都是异步的，充分利用 Rust 的异步生态系统

### 2. 传输层抽象
- 支持多种传输机制：
  - 标准输入输出 (stdio)
  - Server-Sent Events (SSE)
  - Streamable HTTP
  - WebSocket

### 3. 类型安全
- 使用 Rust 的类型系统确保协议正确性
- 提供完整的类型定义和验证

### 4. 宏驱动开发
- 使用 `#[tool]` 和 `#[tool_router]` 宏简化工具定义
- 自动生成 JSON Schema 和参数验证

## 安装配置

### Cargo.toml 依赖配置

```toml
[dependencies]
rmcp = { 
    version = "0.5.0", 
    features = [
        "server",           # 服务器功能
        "macros",           # 宏支持
        "transport-io",     # 标准输入输出传输
        "schemars"          # JSON Schema 生成
    ] 
}
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
```

## 基本使用模式

### 1. 定义服务器结构体

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

#[derive(Clone)]
pub struct MyServer {
    // 服务器状态
    shared_data: Arc<RwLock<HashMap<String, String>>>,
}

impl MyServer {
    fn new() -> Self {
        Self {
            shared_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
```

### 2. 实现 ServerHandler trait

```rust
impl ServerHandler for MyServer {
    async fn call_tool(
        &self,
        params: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = &params.name;
        let args = params.arguments;
        
        match tool_name.as_str() {
            "my_tool" => self.handle_my_tool(args).await,
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
                name: "my_tool".into(),
                description: Some("工具描述".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": { "type": "string" },
                        "param2": { "type": "number" }
                    },
                    "required": ["param1"]
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
                name: "my-mcp-server".into(),
                version: "0.1.0".into(),
            },
            instructions: Some("My MCP Server instructions".into()),
        }
    }
}
```

### 3. 启动服务器

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let server = MyServer::new();
    
    // 使用标准输入输出传输
    let running_service = serve_server(server, rmcp::transport::stdio()).await?;
    running_service.waiting().await?;
    
    Ok(())
}
```

## 高级功能

### 1. 使用宏简化工具定义

```rust
use rmcp::{tool, tool_router};

#[tool_router]
impl MyServer {
    #[tool(description = "计算两个数的和")]
    async fn add(&self, a: i32, b: i32) -> String {
        (a + b).to_string()
    }

    #[tool(description = "存储键值对")]
    async fn store(&self, key: String, value: String) -> String {
        let mut data = self.shared_data.write().await;
        data.insert(key.clone(), value.clone());
        format!("Stored: {} = {}", key, value)
    }
}
```

### 2. 复杂参数处理

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ComplexRequest {
    #[schemars(description = "用户ID")]
    pub user_id: String,
    #[schemars(description = "操作类型")]
    pub action: String,
    #[schemars(description = "附加数据")]
    pub metadata: Option<serde_json::Value>,
}

#[tool(description = "处理复杂请求")]
async fn process_request(&self, #[tool(aggr)] req: ComplexRequest) -> Result<String, String> {
    // 处理逻辑
    Ok(format!("Processed request for user: {}", req.user_id))
}
```

### 3. 错误处理

```rust
use rmcp::ErrorData as McpError;

impl MyServer {
    async fn handle_tool_call(&self, args: Option<serde_json::Value>) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        
        // 参数验证
        let user_id = args.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing user_id", None))?;
            
        // 业务逻辑
        let result = self.process_user_request(user_id).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
            
        Ok(CallToolResult {
            content: Some(vec![rmcp::model::Content::text(result)]),
            is_error: Some(false),
            structured_content: Some(serde_json::json!({"status": "success"})),
        })
    }
}
```

## 实际应用示例

### Coze MCP 服务器实现

本项目实现了以下功能：

1. **Bot管理工具**
   - `list_bots`: 列出智能体
   - `get_bot`: 获取单个Bot详情

2. **知识库管理**
   - `list_knowledge_bases`: 列出知识库
   - `create_dataset`: 创建知识库
   - `upload_document_to_knowledge_base`: 上传文档

3. **对话管理**
   - `list_conversations`: 列出对话
   - `chat`: 发送聊天消息
   - `chat_stream`: 流式聊天

### 配置管理

```rust
// 支持多种配置方式
let api_base_url = cli_base_url
    .or_else(|| env::var("COZE_API_BASE_URL").ok())
    .unwrap_or_else(|| "https://api.coze.cn".to_string());

let api_token = cli_api_key
    .or_else(|| env::var("COZE_API_TOKEN").ok())
    .or_else(|| env::var("COZE_API_KEY").ok())
    .unwrap_or_default();
```

## 最佳实践

### 1. 状态管理
- 使用 `Arc<RwLock<T>>` 实现线程安全的共享状态
- 避免在工具函数中持有锁过长时间

### 2. 日志记录
```rust
use tracing::{info, error, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Starting RMCP server");
    // ...
}
```

### 3. 错误处理
- 使用 `McpError` 类型表示协议级别的错误
- 提供清晰的错误消息
- 使用结构化日志记录错误详情

### 4. 性能优化
- 合理使用异步操作
- 避免不必要的克隆操作
- 使用连接池管理外部服务连接

## 调试技巧

### 1. 启用详细日志
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

### 2. 工具测试
```rust
// 使用 MCP Inspector 进行测试
// 或使用简单的命令行测试
cargo run -- --api-key YOUR_API_KEY --space-id YOUR_SPACE_ID
```

### 3. 错误排查
- 检查传输层连接状态
- 验证工具参数格式
- 确认外部服务可用性

## 总结

RMCP 提供了一个强大而灵活的框架，用于构建符合 MCP 规范的服务器。通过其异步架构、类型安全和宏支持，开发者可以快速构建功能丰富的 MCP 服务器。本项目展示了如何使用 RMCP 构建一个完整的 Coze 平台集成服务器，涵盖了从基本配置到高级功能的所有方面。

## 参考资料

- [RMCP 官方文档](https://docs.rs/rmcp)
- [MCP 协议规范](https://modelcontextprotocol.io)
- [Coze API 文档](https://www.coze.cn/docs/developer_guides/api_overview)