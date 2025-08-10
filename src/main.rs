mod api;
// knowledge module removed after pruning
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

use crate::api::endpoints::COZE_BASE_URL;
use crate::api::CozeApiClient;
use crate::tools::coze_tools::CozeTools;

#[derive(Clone)]
pub struct CozeServer {
    _coze_client: Arc<CozeApiClient>,
    tools: Arc<CozeTools>,
    _default_space_id: String,
}

impl CozeServer {
    pub fn new(
        api_base_url: String,
        api_token: String,
        default_space_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let coze_client_instance = CozeApiClient::new(api_base_url, api_token)?;
        let coze_client = Arc::new(coze_client_instance);
        let tools = Arc::new(CozeTools::new(
            coze_client.clone(),
            default_space_id.clone(),
        ));

        Ok(Self {
            _coze_client: coze_client,
            tools,
            _default_space_id: default_space_id,
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
            "list_bots" => self.tools.list_bots(args_value.clone()).await,
            "list_knowledge_bases" => self.tools.list_knowledge_bases(args_value.clone()).await,
            "create_dataset" => self.tools.create_dataset(args_value.clone()).await,
            "upload_document_to_knowledge_base" => {
                self.tools
                    .upload_document_to_knowledge_base(args_value.clone())
                    .await
            }
            "list_conversations" => self.tools.list_conversations(args_value.clone()).await,
            "chat" => self.tools.chat(args_value.clone()).await,
            "chat_stream" => self.tools.chat_stream(args_value.clone()).await,
            "ping" => Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text("pong")]),
                is_error: Some(false),
                structured_content: Some(serde_json::json!({"ok":true})),
            }),
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: {tool_name}"),
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
        // 精选7个最常用的工具
        let tools = vec![
            // 1. Bot管理 - 核心功能
            Tool {
                name: "list_bots".into(),
                description: Some("列出智能体列表 - 支持按发布状态、分页等条件筛选".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": { 
                            "type": "string", 
                            "description": "工作区ID (必填，或使用默认space_id)" 
                        },
                        "publish_status": { 
                            "type": "string", 
                            "enum": ["all", "published_online", "published_draft", "unpublished_draft"],
                            "description": "发布状态筛选：all(全部)、published_online(已发布正式版)、published_draft(已发布草稿)、unpublished_draft(未发布)，默认published_online" 
                        },
                        "connector_id": { 
                            "type": "string", 
                            "description": "渠道ID，默认1024(API渠道)" 
                        },
                        "page": { 
                            "type": "number", 
                            "description": "页码，默认1" 
                        },
                        "page_size": { 
                            "type": "number", 
                            "description": "每页数量，默认20" 
                        }
                    },
                    "required": []
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 2. 知识库管理 - 核心功能
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
            // 3. 标准知识库创建 API - 符合官方文档规范
            Tool {
                name: "create_dataset".into(),
                description: Some("创建知识库（使用标准 v1/datasets API，符合官方文档规范）".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { 
                            "type": "string", 
                            "description": "知识库名称，长度不超过100个字符" 
                        },
                        "space_id": { 
                            "type": "string", 
                            "description": "知识库所在空间的唯一标识（可选，使用默认space_id）" 
                        },
                        "format_type": { 
                            "type": "number", 
                            "enum": [0, 2],
                            "description": "知识库类型：0-文本类型，2-图片类型" 
                        },
                        "description": { 
                            "type": "string", 
                            "description": "知识库描述信息（可选）" 
                        },
                        "file_id": { 
                            "type": "string", 
                            "description": "知识库图标文件ID（可选），需通过【上传文件】API获取" 
                        }
                    },
                    "required": ["name", "format_type"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 4. 文档上传 - 重要功能
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
            // 5. 会话管理 - 核心功能
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
            // 6. 聊天对话 - 重要功能
            Tool {
                name: "chat".into(),
                description: Some("发送聊天消息（非流式）".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "bot_id": { "type": "string", "description": "Bot ID（必填）" },
                        "message": { "type": "string", "description": "要发送的消息内容（必填）" },
                        "user_id": { "type": "string", "description": "用户ID（可选）" },
                        "conversation_id": { "type": "string", "description": "对话ID（可选，不提供则创建新对话）" },
                        "custom_variables": { 
                            "type": "object", 
                            "description": "自定义变量（可选）",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["bot_id", "message"]
                }).as_object().unwrap().clone()),
                annotations: None,
                output_schema: None,
            },
            // 7. 流式聊天对话 - 重要功能
            Tool {
                name: "chat_stream".into(),
                description: Some("发送流式聊天消息".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "bot_id": { "type": "string", "description": "Bot ID（必填）" },
                        "message": { "type": "string", "description": "要发送的消息内容（必填）" },
                        "user_id": { "type": "string", "description": "用户ID（可选）" },
                        "conversation_id": { "type": "string", "description": "对话ID（可选，不提供则创建新对话）" },
                        "custom_variables": { 
                            "type": "object", 
                            "description": "自定义变量（可选）",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["bot_id", "message"]
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
                version: "0.2.2".into(),
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
    let running_service = serve_server(server, rmcp::transport::stdio()).await?;
    running_service.waiting().await?;
    Ok(())
}
