# RMCP (Rust Model Context Protocol) 服务器实现文档

## 概述

RMCP 是 Model Context Protocol (MCP) 的官方 Rust SDK 实现，提供了构建符合 MCP 规范的服务器所需的核心功能。本文档专注于服务器端实现，基于最新的 0.3.x API。

## 安装

在你的 `Cargo.toml` 文件中添加以下依赖：

```toml
[dependencies]
rmcp = { version = "0.2.0", features = ["server"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
```

或者使用开发版本：

```toml
[dependencies]
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main", features = ["server"] }
```

## 快速开始

### 基本服务器实现

以下是一个使用最新 API 的 RMCP 服务器实现：

```rust
use rmcp::{
    ErrorData as McpError, ServiceExt, model::*, tool, tool_router, 
    transport::stdio, ServerHandler
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl Counter {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

// 实现服务器处理器
#[tool_handler]
impl ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple counter server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
```

### 启动标准输入输出服务器

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建并运行使用 STDIO 传输的服务器
    let service = Counter::new().serve(stdio()).await.inspect_err(|e| {
        println!("Error starting server: {}", e);
    })?;
    service.waiting().await?;

    Ok(())
}
```

## 核心概念

### 服务器架构

RMCP 服务器基于异步 Rust 构建，使用 Tokio 作为异步运行时。服务器通过实现 `ServerHandler` trait 来处理来自客户端的请求。

### 协议特性

- **异步优先**：所有操作都是异步的，充分利用 Rust 的异步生态系统
- **传输无关**：支持多种传输机制，包括标准输入输出、SSE、Streamable HTTP 等
- **类型安全**：使用 Rust 的类型系统确保协议正确性
- **宏驱动**：使用 `#[tool]` 和 `#[tool_router]` 宏简化工具定义

## 传输层实现

RMCP 支持多种传输机制，每种都有其特定的用途：

### 支持的传输类型

| 传输类型 | 客户端 | 服务器 | 用途 |
|:--------:|:------:|:------:|:----:|
| 标准输入输出 | `TokioChildProcess` | `stdio` | 子进程通信 |
| Streamable HTTP | `StreamableHttpClientTransport` | `create_session` | HTTP 流式通信 |
| SSE | `SseClientTransport` | `SseServer` | 服务器推送事件 |

### 标准输入输出传输

```rust
use rmcp::transport::stdio;
use tokio::io::{stdin, stdout};

// 使用 stdio 传输
let transport = stdio();
// 或者手动构建
let transport = (stdin(), stdout());

let service = Counter::new().serve(transport).await?;
```

### SSE (Server-Sent Events) 传输

```rust
use rmcp::transport::sse_server::SseServer;

// 启动 SSE 服务器
let server = SseServer::new("127.0.0.1:8000".parse()?);
let service = Counter::new().serve(server).await?;
println!("SSE server started on http://127.0.0.1:8000/sse");
```

### Streamable HTTP 传输

```rust
use rmcp::transport::streamable_http_server;

// 使用 Axum 框架的 HTTP 传输
let service = Counter::new().serve(
    streamable_http_server::session::create_session("127.0.0.1:3000".parse()?)
).await?;
```

## ServerHandler Trait 实现

### 基本 ServerHandler 结构

```rust
use rmcp::{ServerHandler, model::*};

struct BasicServer;

impl ServerHandler for BasicServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A basic MCP server".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    // 可选：处理进度通知
    async fn on_progress(
        &self,
        notification: ProgressNotificationParam,
        context: NotificationContext<RoleServer>,
    ) {
        let peer = context.peer;
        let _ = peer
            .notify_logging_message(LoggingMessageNotificationParam {
                level: LoggingLevel::Info,
                logger: None,
                data: serde_json::json!({
                    "message": format!("Progress: {}", notification.progress),
                }),
            })
            .await;
    }
}
```

### 服务器能力配置

```rust
use rmcp::model::ServerCapabilities;

impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("My custom MCP server".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()           // 启用工具支持
                .enable_prompts()         // 启用提示支持
                .enable_resources()       // 启用资源支持
                .enable_logging()         // 启用日志支持
                .enable_tool_list_changed() // 启用工具列表变更通知
                .build(),
            ..Default::default()
        }
    }
}

## 工具定义和实现

### 使用 `#[tool]` 宏定义工具

```rust
use rmcp::{tool, tool_router, ServerHandler, model::*, schemars};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "左操作数")]
    pub a: i32,
    #[schemars(description = "右操作数")]
    pub b: i32,
}

#[derive(Debug, Clone)]
pub struct Calculator;

#[tool_router]
impl Calculator {
    fn new() -> Self {
        Self
    }

    // 异步工具函数
    #[tool(description = "计算两个数的和")]
    async fn sum(&self, #[tool(aggr)] SumRequest { a, b }: SumRequest) -> String {
        (a + b).to_string()
    }

    // 同步工具函数
    #[tool(description = "计算两个数的差")]
    fn subtract(
        &self,
        #[tool(param)]
        #[schemars(description = "被减数")]
        a: i32,
        #[tool(param)]
        #[schemars(description = "减数")]
        b: i32,
    ) -> String {
        (a - b).to_string()
    }

    // 返回 Result 类型的工具
    #[tool(description = "计算两个数的商")]
    fn divide(&self, a: f64, b: f64) -> Result<String, String> {
        if b == 0.0 {
            Err("除数不能为零".to_string())
        } else {
            Ok((a / b).to_string())
        }
    }
}

// 使用 tool_handler 宏自动实现 call_tool 和 list_tools
#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("一个简单的计算器".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
```

### 手动实现工具处理

如果不使用宏，可以手动实现 `call_tool` 和 `list_tools` 方法：

```rust
use rmcp::{ServerHandler, model::*, handler::server::tool::ToolCallContext};

struct ManualToolServer {
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl ServerHandler for ManualToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("手动实现的工具服务器".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match request.name.as_str() {
            "echo" => {
                let message = request.arguments
                    .and_then(|args| args.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Hello");
                
                Ok(CallToolResult::success(vec![Content::text(message.to_string())]))
            },
            _ => Err(rmcp::ErrorData::method_not_found()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let tools = vec![
            Tool {
                name: "echo".to_string(),
                description: "回显输入的消息".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string", "description": "要回显的消息"}
                    },
                    "required": ["message"]
                }),
            }
        ];
        
        Ok(ListToolsResult::with_all_items(tools))
    }
}
```

## 错误处理

### 错误类型

RMCP 使用 `ErrorData` 类型来表示错误：

```rust
use rmcp::ErrorData;

// 创建不同类型的错误
let invalid_params_error = ErrorData::invalid_params("参数无效");
let method_not_found_error = ErrorData::method_not_found();
let internal_error = ErrorData::internal_error("内部服务器错误");
```

### 在工具中处理错误

```rust
#[tool(description = "可能失败的工具")]
fn risky_operation(&self, value: i32) -> Result<String, ErrorData> {
    if value < 0 {
        Err(ErrorData::invalid_params("值不能为负数"))
    } else if value > 100 {
        Err(ErrorData::internal_error("值超出处理范围"))
    } else {
        Ok(format!("处理结果: {}", value * 2))
    }
}
```

### 错误日志记录

## Coze MCP 工具清单与示例

本项目当前在 MCP 服务器中暴露以下工具（中国区 api.coze.cn 已验证端点）：

- list_bots：列出 Bots（workspace_id/space_id 必填；page/page_size 可选）
- list_workflows：列出 Workflows（workspace_id/space_id 必填；page/page_size 可选；内部优先 page_num，失败回退 page）
- list_workspaces：列出 Workspaces（无参）
- list_conversations：列出 Conversations（workspace_id/space_id 与 bot_id 必填；page/page_size 可选；page/page_num 兼容）
- get_bot：获取单个 Bot 详情（bot_id 必填）
- list_knowledge_bases：列出知识库（space_id 必填）
- create_knowledge_base：创建知识库（name/description/space_id 必填）
- upload_document：上传文档到知识库（knowledge_base_id/file_path/document_name 必填）
- set_api_key / get_config_status / test_connection：配置类工具

工具调用示例（Call Tool 消息负载片段，仅示意）：

```json
{
    "method": "tools/call",
    "params": {
        "name": "get_bot",
        "arguments": {
            "bot_id": "<your_bot_id>"
        }
    }
}
```

```json
{
    "method": "tools/call",
    "params": {
        "name": "list_bots",
        "arguments": {
            "workspace_id": "<your_space_id>",
            "page": 1,
            "page_size": 20
        }
    }
}
```

```json
{
    "method": "tools/call",
    "params": {
        "name": "list_conversations",
        "arguments": {
            "workspace_id": "<your_space_id>",
            "bot_id": "<your_bot_id>",
            "page": 1,
            "page_size": 20
        }
    }
}
```

```json
{
    "method": "tools/call",
    "params": {
        "name": "list_knowledge_bases",
        "arguments": {
            "space_id": "<your_space_id>"
        }
    }
}
```

输出风格说明：

- 工具返回文本摘要（总数 + 前若干条简要信息），structured_content 为空；错误时 is_error=true 并返回错误文本。
- 分页字段在不同端点/版本间可能为 page 或 page_num，工具内部已做好回退兼容。

注意事项（中国区）：

- Conversations 列表需要 bot_id 参数；缺失将返回 400。
- Workspaces 详情（/v1/workspaces/{workspace_id}）在本项目环境返回 404，未纳入工具；列表端点可用。
- 始终使用 <https://api.coze.cn> 作为基础域名。

```rust
use tracing::{error, warn, info};

impl ServerHandler for LoggingServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("调用工具: {}", request.name);
        
        match self.handle_tool_call(request).await {
            Ok(result) => {
                info!("工具调用成功");
                Ok(result)
            },
            Err(e) => {
                error!("工具调用失败: {:?}", e);
                Err(e)
            }
        }
    }
}
```

## 最佳实践

### 1. 功能特性配置

根据需要启用相应的功能特性：

```toml
[dependencies]
rmcp = { 
    version = "0.2.0", 
    features = [
        "server",                    # 服务器功能
        "macros",                    # 宏支持（默认启用）
        "transport-io",              # 标准输入输出传输
        "transport-sse-server",      # SSE 服务器传输
        "transport-streamable-http-server", # HTTP 流式传输
        "schemars",                  # JSON Schema 生成
        "auth"                       # OAuth2 认证支持
    ] 
}
```

### 2. 性能优化

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct OptimizedServer {
    // 使用 Arc<RwLock> 来共享状态
    shared_data: Arc<RwLock<HashMap<String, String>>>,
}

impl OptimizedServer {
    fn new() -> Self {
        Self {
            shared_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tool_router]
impl OptimizedServer {
    #[tool(description = "高效的数据访问")]
    async fn get_data(&self, key: String) -> Option<String> {
        let data = self.shared_data.read().await;
        data.get(&key).cloned()
    }
}
```

### 3. 日志记录

```rust
use tracing::{info, error, debug};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("启动 RMCP 服务器");
    
    let service = MyServer::new().serve(stdio()).await?;
    service.waiting().await?;
    
    Ok(())
}
```

### 4. 优雅关闭

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = MyServer::new().serve(stdio()).await?;
    
    // 等待关闭信号或服务自然结束
    tokio::select! {
        result = service.waiting() => {
            info!("服务正常结束: {:?}", result);
            result
        },
        _ = signal::ctrl_c() => {
            info!("收到中断信号，正在关闭服务器");
            service.cancel().await
        }
    }
}
```

## 故障排除

### 常见问题

1. **工具宏编译错误**

    ```rust
   // 确保启用了 macros 功能
   [dependencies]
   rmcp = { version = "0.2.0", features = ["server", "macros"] }
   
   // 确保正确导入宏
   use rmcp::{tool, tool_router, tool_handler};
   ```

2. **传输连接失败**

    ```rust
   // 检查传输配置
   use rmcp::transport::stdio;
   
   let service = match MyServer::new().serve(stdio()).await {
       Ok(s) => s,
       Err(e) => {
           error!("传输连接失败: {}", e);
           return Err(e.into());
       }
   };
   ```

3. **工具参数解析错误**

    ```rust
   // 使用 schemars 确保正确的 JSON Schema
   #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
   struct MyParams {
       #[schemars(description = "参数描述")]
       value: String,
   }
   
   #[tool(description = "示例工具")]
   fn my_tool(&self, #[tool(aggr)] params: MyParams) -> String {
       params.value
   }
   ```

### 调试技巧

```rust
use tracing::{debug, info, error};

#[tool_router]
impl DebugServer {
    #[tool(description = "调试工具")]
    async fn debug_tool(&self, input: String) -> String {
        debug!("收到输入: {}", input);
        
        let result = format!("处理结果: {}", input.to_uppercase());
        info!("返回结果: {}", result);
        
        result
    }
}

// 启用详细日志
fn setup_debug_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}
```

## 完整示例

### 功能完整的服务器实现

```rust
use rmcp::{
    ServerHandler, ServiceExt, model::*, tool, tool_router, tool_handler,
    transport::stdio, schemars
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

// 定义请求参数结构
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CalculateRequest {
    #[schemars(description = "第一个操作数")]
    a: f64,
    #[schemars(description = "第二个操作数")]
    b: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct StoreRequest {
    #[schemars(description = "存储的键")]
    key: String,
    #[schemars(description = "存储的值")]
    value: String,
}

// 完整的服务器实现
#[derive(Clone)]
struct CompleteServer {
    storage: Arc<RwLock<HashMap<String, String>>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl CompleteServer {
    fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            tool_router: Self::tool_router(),
        }
    }

    // 数学计算工具
    #[tool(description = "计算两个数的和")]
    async fn add(&self, #[tool(aggr)] req: CalculateRequest) -> String {
        (req.a + req.b).to_string()
    }

    #[tool(description = "计算两个数的乘积")]
    async fn multiply(&self, #[tool(aggr)] req: CalculateRequest) -> String {
        (req.a * req.b).to_string()
    }

    #[tool(description = "计算两个数的商")]
    async fn divide(&self, #[tool(aggr)] req: CalculateRequest) -> Result<String, String> {
        if req.b == 0.0 {
            Err("除数不能为零".to_string())
        } else {
            Ok((req.a / req.b).to_string())
        }
    }

    // 存储工具
    #[tool(description = "存储键值对")]
    async fn store(&self, #[tool(aggr)] req: StoreRequest) -> String {
        let mut storage = self.storage.write().await;
        storage.insert(req.key.clone(), req.value.clone());
        format!("已存储: {} = {}", req.key, req.value)
    }

    #[tool(description = "获取存储的值")]
    async fn get(&self, key: String) -> Option<String> {
        let storage = self.storage.read().await;
        storage.get(&key).cloned()
    }

    #[tool(description = "列出所有存储的键")]
    async fn list_keys(&self) -> Vec<String> {
        let storage = self.storage.read().await;
        storage.keys().cloned().collect()
    }

    // 实用工具
    #[tool(description = "回显输入的消息")]
    fn echo(&self, message: String) -> String {
        format!("回显: {}", message)
    }

    #[tool(description = "获取当前时间戳")]
    fn timestamp(&self) -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

// 实现服务器处理器
#[tool_handler]
impl ServerHandler for CompleteServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("一个功能完整的 MCP 服务器，提供计算、存储和实用工具".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_logging()
                .build(),
            ..Default::default()
        }
    }

    // 处理进度通知
    async fn on_progress(
        &self,
        notification: ProgressNotificationParam,
        context: NotificationContext<RoleServer>,
    ) {
        info!("进度更新: {}", notification.progress);
        
        let _ = context.peer
            .notify_logging_message(LoggingMessageNotificationParam {
                level: LoggingLevel::Info,
                logger: Some("progress".to_string()),
                data: serde_json::json!({
                    "message": format!("进度: {}%", notification.progress),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            })
            .await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("启动完整的 RMCP 服务器");

    // 创建并启动服务器
    let service = CompleteServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            error!("服务器启动失败: {}", e);
        })?;

    info!("服务器已启动，等待连接...");
    
    // 等待服务器结束
    let quit_reason = service.waiting().await?;
    info!("服务器已关闭，原因: {:?}", quit_reason);

    Ok(())
}
```

### 运行服务器

```bash
# 编译并运行
cargo run

# 或者构建发布版本
cargo build --release
./target/release/your-server-name
```

## 高级特性

### OAuth 2.1 认证支持

RMCP 支持 OAuth 2.1 认证，用于安全的服务器访问：

```toml
[dependencies]
rmcp = { version = "0.2.0", features = ["server", "auth", "transport-sse-server"] }
```

```rust
use rmcp::auth::OAuthState;

// 初始化 OAuth 状态
let mut oauth_state = OAuthState::new("https://auth.example.com", None).await?;
oauth_state.start_authorization(&["mcp", "profile"], "http://localhost:8080/callback").await?;

// 获取授权 URL
let auth_url = oauth_state.get_authorization_url().await?;
println!("请访问: {}", auth_url);
```

### 多路由器组合

可以组合多个工具路由器：

```rust
mod calculator {
    use super::*;
    
    #[derive(Clone)]
    pub struct Calculator;
    
    #[tool_router(router = calculator_router, vis = pub)]
    impl Calculator {
        #[tool(description = "加法")]
        fn add(&self, a: i32, b: i32) -> i32 { a + b }
    }
}

mod storage {
    use super::*;
    
    #[derive(Clone)]
    pub struct Storage;
    
    #[tool_router(router = storage_router, vis = pub)]
    impl Storage {
        #[tool(description = "存储")]
        fn store(&self, key: String, value: String) -> String {
            format!("存储: {} = {}", key, value)
        }
    }
}

// 组合路由器
impl CombinedServer {
    fn new() -> Self {
        Self {
            tool_router: calculator::calculator_router() + storage::storage_router(),
        }
    }
}
```

### 动态服务管理

```rust
// 将服务转换为动态类型以便管理
let service = CompleteServer::new().serve(stdio()).await?;
let dyn_service = service.into_dyn();

// 可以将多个服务放入集合中管理
let mut services = Vec::new();
services.push(dyn_service);
```

## 总结

本文档提供了基于 RMCP 0.3.x 的完整服务器端实现指南，包括：

1. **现代化 API**：基于最新的 `ServerHandler` trait 和宏系统
2. **传输层支持**：标准输入输出、SSE、Streamable HTTP 等多种传输方式
3. **工具系统**：使用 `#[tool]` 和 `#[tool_router]` 宏简化工具定义
4. **错误处理**：完整的 `ErrorData` 错误处理机制
5. **最佳实践**：性能优化、日志记录、优雅关闭
6. **高级特性**：OAuth 认证、多路由器组合、动态服务管理
7. **完整示例**：可直接运行的功能完整服务器实现

通过本文档，你可以快速构建功能强大、性能优秀的 RMCP 服务器应用程序。

## 参考资源

- [MCP 规范](https://spec.modelcontextprotocol.io/specification/2024-11-05/)
- [RMCP GitHub 仓库](https://github.com/modelcontextprotocol/rust-sdk)
- [示例代码](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples)
