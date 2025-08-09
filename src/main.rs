mod api;
mod knowledge;
mod models;
mod tools;

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::{serve_server, RequestContext, RoleServer},
    ErrorData as McpError,
};
use serde_json::Value;
use std::env;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

use crate::api::endpoints::COZE_BASE_URL;
use crate::api::CozeApiClient;
use crate::knowledge::KnowledgeManager;
use crate::tools::config_tool::ConfigTool;
use crate::tools::coze_tools::CozeTools;

#[derive(Clone)]
pub struct CozeServer {
    coze_client: Arc<CozeApiClient>,
    knowledge_manager: Arc<KnowledgeManager>,
    tools: Arc<CozeTools>,
    config_tool: Arc<ConfigTool>,
    default_space_id: String,
}

impl CozeServer {
    pub fn new(
        api_base_url: String,
        api_token: String,
        default_space_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let coze_client_instance = CozeApiClient::new(api_base_url, api_token)?;
        let coze_client = Arc::new(coze_client_instance);
        let knowledge_manager = Arc::new(KnowledgeManager::new(
            (*coze_client).clone(),
            knowledge::KnowledgeConfig::default(),
        ));
        let tools = Arc::new(CozeTools::new(
            coze_client.clone(),
            knowledge_manager.clone(),
            default_space_id.clone(),
        ));
        // 将客户端注入到配置工具中，以便运行时更新 API Key 能影响后续请求
        let config_tool = Arc::new(ConfigTool::new().with_client(coze_client.clone()));

        Ok(Self {
            coze_client,
            knowledge_manager,
            tools,
            config_tool,
            default_space_id,
        })
    }
}

impl ServerHandler for CozeServer {
    async fn call_tool(
        &self,
        params: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = &params.name;
        let args_value: Option<Value> = params.arguments.as_ref().map(|m| Value::Object(m.clone()));

        info!("Calling tool: {}", tool_name);

        let result: Result<CallToolResult, McpError> = match &tool_name[..] {
            "set_api_key" => {
                if let Some(map) = params.arguments.as_ref() {
                    self.config_tool.set_api_key_from_args(map).await
                } else {
                    Err(McpError::invalid_params("Missing arguments", None))
                }
            }
            "list_workspaces" => self.tools.list_workspaces(args_value.clone()).await,
            "list_bots" => self.tools.list_bots(args_value.clone()).await,
            "list_knowledge_bases" => self.tools.list_knowledge_bases(args_value.clone()).await,
            "create_knowledge_base_v2" => {
                self.tools
                    .create_knowledge_base_v2(args_value.clone())
                    .await
            }
            "upload_document_to_knowledge_base" => {
                self.tools
                    .upload_document_to_knowledge_base(args_value.clone())
                    .await
            }
            "list_conversations" => self.tools.list_conversations(args_value.clone()).await,
            "ping" => Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text("pong")]),
                is_error: Some(false),
                structured_content: Some(serde_json::json!({"ok":true})),
            }),
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            )),
        };

        result
    }
    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        // 精选10个最常用的工具
        let tools = vec![
            // 1. 配置管理 - 必需
            Tool {
                name: "set_api_key".into(),
                description: Some("设置Coze API Key".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "api_key": {
                            "type": "string",
                            "description": "Coze个人访问令牌 (以pat_开头)"
                        }
                    },
                    "required": ["api_key"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 2. 工作空间管理 - 基础功能
            Tool {
                name: "list_workspaces".into(),
                description: Some("列出所有工作空间".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 3. Bot管理 - 核心功能
            Tool {
                name: "list_bots".into(),
                description: Some("列出Bots".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": { "type": "string", "description": "工作区ID (可选，使用默认space_id)" },
                        "page": { "type": "number", "description": "页码，默认1" },
                        "page_size": { "type": "number", "description": "每页数量，默认20" }
                    },
                    "required": []
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 4. 知识库管理 - 核心功能
            Tool {
                name: "list_knowledge_bases".into(),
                description: Some("列出所有知识库".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "space_id": { "type": "string", "description": "空间ID (可选，使用默认space_id)" },
                        "name": { "type": "string", "description": "按名称模糊搜索（可选）" },
                        "page_num": { "type": "number", "description": "页码，默认1（可选）" },
                        "page_size": { "type": "number", "description": "每页数量（可选）" }
                    },
                    "required": []
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 5. 知识库创建 - 重要功能
            Tool {
                name: "create_knowledge_base_v2".into(),
                description: Some("创建知识库".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "知识库名称" },
                        "description": { "type": "string", "description": "知识库描述（可选）" },
                        "space_id": { "type": "string", "description": "空间ID（可选，使用默认space_id）" },
                        "permission": { "type": "string", "enum": ["private", "public"], "description": "权限类型（可选，默认private）" }
                    },
                    "required": ["name"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 6. 文档上传 - 重要功能
            Tool {
                name: "upload_document_to_knowledge_base".into(),
                description: Some("上传本地文档到知识库".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "dataset_id": { "type": "string", "description": "知识库ID" },
                        "file_path": { "type": "string", "description": "本地文件路径" },
                        "document_name": { "type": "string", "description": "文档名称（可选）" },
                        "chunk_size": { "type": "number", "description": "分片大小（可选，默认800）" },
                        "chunk_overlap": { "type": "number", "description": "分片重叠（可选，默认100）" }
                    },
                    "required": ["dataset_id", "file_path"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 7. 会话管理 - 核心功能
            Tool {
                name: "list_conversations".into(),
                description: Some("列出对话".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": { "type": "string", "description": "工作区ID (可选，使用默认space_id)" },
                        "bot_id": { "type": "string", "description": "Bot ID（必填）" },
                        "page": { "type": "number", "description": "页码，默认1" },
                        "page_size": { "type": "number", "description": "每页数量，默认20" }
                    },
                    "required": ["bot_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
        ];
        info!("list_tools invoked, returning {} tools", tools.len());
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "coze-mcp-server".into(),
                version: "0.2.0".into(),
            },
            // 在 instructions 中嵌入工具名称，便于客户端未调用 list_tools 时人工确认
            instructions: Some("Coze MCP Server ready.".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    // ---- CLI 参数解析（优先级: CLI > 环境变量 > 默认） ----
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("Coze MCP Server\n\n用法: coze-mcp-server [--api-key <KEY>] [--space-id <SPACE>] [--base-url <URL>]\n\n优先级: CLI > 环境变量 > 默认\n\n环境变量: COZE_API_KEY / COZE_API_TOKEN, COZE_DEFAULT_SPACE_ID, COZE_API_BASE_URL\n");
        return Ok(());
    }
    let mut cli_api_key: Option<String> = None;
    let mut cli_space_id: Option<String> = None;
    let mut cli_base_url: Option<String> = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--api-key" => {
                if let Some(v) = iter.next() {
                    cli_api_key = Some(v.to_string());
                }
            }
            s if s.starts_with("--api-key=") => {
                cli_api_key = Some(s[10..].to_string());
            }
            "--space-id" => {
                if let Some(v) = iter.next() {
                    cli_space_id = Some(v.to_string());
                }
            }
            s if s.starts_with("--space-id=") => {
                cli_space_id = Some(s[11..].to_string());
            }
            "--base-url" => {
                if let Some(v) = iter.next() {
                    cli_base_url = Some(v.to_string());
                }
            }
            s if s.starts_with("--base-url=") => {
                cli_base_url = Some(s[11..].to_string());
            }
            _ => {}
        }
    }

    let api_base_url = cli_base_url
        .or_else(|| env::var("COZE_API_BASE_URL").ok())
        .unwrap_or_else(|| COZE_BASE_URL.to_string());
    let api_token = cli_api_key
        .or_else(|| env::var("COZE_API_TOKEN").ok())
        .or_else(|| env::var("COZE_API_KEY").ok())
        .unwrap_or_default();
    let default_space_id = cli_space_id
        .or_else(|| env::var("COZE_DEFAULT_SPACE_ID").ok())
        .unwrap_or_else(|| "default".to_string());

    info!("Starting Coze MCP Server...");
    info!("API Base URL: {}", api_base_url);
    info!("Default Space ID: {}", default_space_id);

    let server = CozeServer::new(api_base_url, api_token, default_space_id)?;

    info!("Server initialized successfully");

    // 使用 IO 传输特性：当前 rmcp crate 使用 transport-io 特征，提供 default io 传输构建器
    // 使用 rmcp 提供的 stdio 传输实现（transport-io 特性）
    let running_service =   serve_server(server, rmcp::transport::stdio()).await?;
    running_service.waiting().await?;
    Ok(())
}
