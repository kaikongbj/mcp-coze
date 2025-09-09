use crate::api::CozeApiClient;
use crate::models::{CozeApiRequest, HttpMethod};
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid;

#[derive(Debug, Clone)]
pub struct CozeTools {
    coze_client: Arc<CozeApiClient>,
    default_space_id: String,
}

impl CozeTools {
    pub fn new(coze_client: Arc<CozeApiClient>, default_space_id: String) -> Self {
        Self {
            coze_client,
            default_space_id,
        }
    }

    // Helper: tolerant list + total extractor for various CN shapes
    fn extract_list_and_total(data: &Value) -> (Vec<Value>, usize) {
        let items = data
            .get("datasets")
            .or_else(|| data.get("dataset_list"))
            .or_else(|| data.get("list"))
            .or_else(|| data.get("items"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let total = data
            .get("total")
            .or_else(|| data.get("total_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(items.len() as u64) as usize;
        (items, total)
    }

    fn get_str<'a>(obj: &'a serde_json::Map<String, Value>, k: &str) -> &'a str {
        obj.get(k).and_then(|v| v.as_str()).unwrap_or("")
    }

    // CN: write operations for datasets/files are not exposed via MCP tools by default.

    pub async fn list_knowledge_bases(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let space_id = args
            .get("space_id")
            .and_then(|v| v.as_str())
            .or_else(|| {
                if !self.default_space_id.is_empty() {
                    Some(&self.default_space_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| McpError::invalid_params("Missing space_id parameter", None))?
            .to_string();
        // 使用统一 /v1/datasets 接口
        match self
            .coze_client
            .list_datasets(&space_id, None, None, None, None)
            .await
        {
            Ok(result) => {
                // Optional: refine document_count by fetching dataset detail (limited to first 50 to avoid many requests)
                let content = if result.datasets.is_empty() {
                    "没有找到知识库".to_string()
                } else {
                    let mut response = format!("找到 {} 个知识库:\n\n", result.total);
                    for (i, kb) in result.datasets.iter().enumerate() {
                        response.push_str(&format!(
                            "{}. ID: {}\n   名称: {}\n   描述: {}\n   文档数量: {}\n   创建时间: {}\n\n",
                            i + 1,
                            kb.dataset_id,
                            kb.name,
                            kb.description,
                            kb.document_count,
                            kb.created_at
                        ));
                    }
                    response
                };
                let sc_items: Vec<Value> = result
                    .datasets
                    .iter()
                    .map(|kb| {
                        json!({
                            "dataset_id": kb.dataset_id,
                            "name": kb.name,
                            "description": kb.description,
                            "document_count": kb.document_count,
                            "created_at": kb.created_at,
                            "update_time": kb.update_time,
                            "status": kb.status,
                            "format_type": kb.format_type,
                            "slice_count": kb.slice_count,
                            "space_id": kb.space_id,
                            "dataset_type": kb.dataset_type,
                            "can_edit": kb.can_edit,
                            "icon_url": kb.icon_url,
                            "icon_uri": kb.icon_uri,
                            "avatar_url": kb.avatar_url,
                            "creator_id": kb.creator_id,
                            "creator_name": kb.creator_name,
                            "hit_count": kb.hit_count,
                            "all_file_size": kb.all_file_size,
                            "bot_used_count": kb.bot_used_count,
                            "file_list": kb.file_list,
                            "failed_file_list": kb.failed_file_list,
                            "processing_file_list": kb.processing_file_list,
                            "processing_file_id_list": kb.processing_file_id_list,
                            "chunk_strategy": kb.chunk_strategy,
                            "storage_config": kb.storage_config,
                            "project_id": kb.project_id,
                            "raw_extra": kb.raw_extra,
                        })
                    })
                    .collect();
                let structured = json!({ "total": result.total,  "items": sc_items });

                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(content)]),
                    is_error: Some(false),
                    structured_content: Some(structured),
                })
            }
            Err(e) => {
                let serialized =
                    serde_json::to_value(&e).unwrap_or(json!({"error": e.to_string()}));
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "获取知识库列表失败: {e}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": serialized})),
                })
            }
        }
    }

    pub async fn list_bots(&self, args: Option<Value>) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;

        // Accept either workspace_id or space_id
        let workspace_id = args
            .get("workspace_id")
            .or_else(|| args.get("space_id"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                if !self.default_space_id.is_empty() {
                    Some(&self.default_space_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                McpError::invalid_params("Missing workspace_id (or space_id) parameter", None)
            })?;

        // 解析可选参数
        let page_num = args
            .get("page")
            .and_then(|v| v.as_u64())
            .map(|p| p as u32)
            .unwrap_or(1);
        let page_size = args
            .get("page_size")
            .and_then(|v| v.as_u64())
            .map(|p| p as u32)
            .unwrap_or(20);

        // 解析发布状态
        let publish_status = args
            .get("publish_status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "all" => crate::api::bot_models::BotPublishStatus::All,
                "published_online" => crate::api::bot_models::BotPublishStatus::PublishedOnline,
                "published_draft" => crate::api::bot_models::BotPublishStatus::PublishedDraft,
                "unpublished_draft" => crate::api::bot_models::BotPublishStatus::UnpublishedDraft,
                _ => crate::api::bot_models::BotPublishStatus::PublishedOnline,
            })
            .unwrap_or(crate::api::bot_models::BotPublishStatus::PublishedOnline);

        let connector_id = args
            .get("connector_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "1024".to_string());

        // 构建请求
        let request = crate::api::bot_models::ListBotsRequest::new(workspace_id.to_string())
            .with_publish_status(publish_status)
            .with_connector_id(connector_id)
            .with_page(page_num, page_size);

        match self.coze_client.list_bots_typed(&request).await {
            Ok(response) => {
                let total = response.data.total;
                let mut out = format!("找到 {total} 个 Bot:\n\n");
                let mut sc_items: Vec<Value> = Vec::new();

                for (i, bot) in response.data.items.iter().take(5).enumerate() {
                    let status = if bot.is_published.unwrap_or(false) {
                        "published"
                    } else {
                        "draft"
                    };
                    out.push_str(&format!(
                        "{}. {} (id: {}, status: {})\n",
                        i + 1,
                        bot.name,
                        bot.id,
                        status
                    ));
                    sc_items.push(json!({
                        "bot_id": bot.id,
                        "name": bot.name,
                        "status": status,
                        "description": bot.description,
                        "icon_url": bot.icon_url,
                        "updated_at": bot.updated_at,
                        "owner_user_id": bot.owner_user_id,
                    }));
                }

                let structured = json!({
                    "total": total,
                    "items": sc_items,
                    "page_num": page_num,
                    "page_size": page_size,
                });

                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(out)]),
                    is_error: Some(false),
                    structured_content: Some(structured),
                })
            }
            Err(e) => {
                let serialized =
                    serde_json::to_value(&e).unwrap_or(json!({"error": e.to_string()}));
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "[Bots] 请求失败: {e}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": serialized})),
                })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn list_workspaces(&self, _args: Option<Value>) -> Result<CallToolResult, McpError> {
        let endpoint = "/v1/workspaces".to_string();
        let req = CozeApiRequest {
            endpoint,
            method: HttpMethod::Get,
            headers: Default::default(),
            params: Default::default(),
            body: None,
        };
        match self.coze_client.execute_request(req).await {
            Ok(resp) => {
                let data = resp.body.get("data").cloned().unwrap_or(resp.body);
                let (items, total) = Self::extract_list_and_total(&data);
                let mut out = format!("找到 {total} 个 Workspace:\n\n");
                let mut sc_items: Vec<Value> = Vec::new();
                for (i, it) in items.iter().take(5).enumerate() {
                    if let Some(obj) = it.as_object() {
                        let ws_id = Self::get_str(obj, "workspace_id");
                        let name = Self::get_str(obj, "name");
                        out.push_str(&format!("{}. {} (id: {})\n", i + 1, name, ws_id));
                        sc_items.push(json!({"workspace_id": ws_id, "name": name}));
                    }
                }
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(out)]),
                    is_error: Some(false),
                    structured_content: Some(json!({"total": total, "items": sc_items})),
                })
            }
            Err(e) => {
                let serialized =
                    serde_json::to_value(&e).unwrap_or(json!({"error": e.to_string()}));
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "[Workspaces] 请求失败: {e}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": serialized})),
                })
            }
        }
    }

    // ===== 保留最小工具面 =====
    // ===== 仅保留: list_workspaces, list_bots, list_knowledge_bases, create_dataset, upload_document_to_knowledge_base, list_conversations =====

    /// 创建知识库 (使用标准 v1/datasets API，符合官方文档规范)
    ///
    /// 支持创建文本或图片类型的知识库
    ///
    /// 参数:
    /// - name: 知识库名称 (必需，长度不超过100字符)
    /// - space_id: 空间ID (必需)
    /// - format_type: 知识库类型 (必需，0-文本，2-图片)
    /// - description: 描述信息 (可选)
    /// - file_id: 图标文件ID (可选)
    pub async fn create_dataset(&self, args: Option<Value>) -> Result<CallToolResult, McpError> {
        let args = match args {
            Some(args) => args,
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text("Missing arguments")]),
                    is_error: Some(true),
                    structured_content: Some(json!({
                        "success": false,
                        "error": "Missing arguments"
                    })),
                });
            }
        };

        let name = match args.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text("Missing name parameter")]),
                    is_error: Some(true),
                    structured_content: Some(json!({
                        "success": false,
                        "error": "Missing name parameter"
                    })),
                });
            }
        };

        if name.len() > 100 {
            return Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(
                    "Name length cannot exceed 100 characters",
                )]),
                is_error: Some(true),
                structured_content: Some(json!({
                    "success": false,
                    "error": "Name length cannot exceed 100 characters"
                })),
            });
        }

        let space_id = args.get("space_id").and_then(|v| v.as_str()).or_else(|| {
            if !self.default_space_id.is_empty() {
                Some(&self.default_space_id)
            } else {
                None
            }
        });

        let space_id = match space_id {
            Some(space_id) => space_id,
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        "Missing space_id parameter",
                    )]),
                    is_error: Some(true),
                    structured_content: Some(json!({
                        "success": false,
                        "error": "Missing space_id parameter"
                    })),
                });
            }
        };

        let format_type = match args
            .get("format_type")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
        {
            Some(format_type) => format_type,
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        "Missing format_type parameter (0 for text, 2 for image)",
                    )]),
                    is_error: Some(true),
                    structured_content: Some(json!({
                        "success": false,
                        "error": "Missing format_type parameter (0 for text, 2 for image)"
                    })),
                });
            }
        };

        if format_type != 0 && format_type != 2 {
            return Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(
                    "Invalid format_type, must be 0 (text) or 2 (image)",
                )]),
                is_error: Some(true),
                structured_content: Some(json!({
                    "success": false,
                    "error": "Invalid format_type, must be 0 (text) or 2 (image)"
                })),
            });
        }

        let description = args.get("description").and_then(|v| v.as_str());

        let file_id = args.get("file_id").and_then(|v| v.as_str());

        let request = crate::api::knowledge_models::CreateDatasetRequest {
            name: name.to_string(),
            space_id: space_id.to_string(),
            format_type,
            description: description.map(|s| s.to_string()),
            file_id: file_id.map(|s| s.to_string()),
        };

        match self.coze_client.create_dataset(request).await {
            Ok(response) => {
                if response.code == 0 {
                    let dataset_id = response
                        .data
                        .as_ref()
                        .map(|d| d.dataset_id.as_str())
                        .unwrap_or("unknown");

                    let format_type_str = match format_type {
                        0 => "文本",
                        2 => "图片",
                        _ => "未知",
                    };

                    let content = format!(
                        "知识库创建成功:\n- 知识库ID: {}\n- 名称: {}\n- 类型: {} ({})\n- 空间ID: {}{}{}{}",
                        dataset_id,
                        name,
                        format_type_str,
                        format_type,
                        space_id,
                        description.map(|d| format!("\n- 描述: {d}")).unwrap_or_default(),
                        file_id.map(|f| format!("\n- 图标文件ID: {f}")).unwrap_or_default(),
                        response.detail.as_ref().map(|d| format!("\n- 日志ID: {}", d.logid)).unwrap_or_default()
                    );

                    let structured = json!({
                        "success": true,
                        "dataset_id": dataset_id,
                        "name": name,
                        "format_type": format_type,
                        "format_type_name": format_type_str,
                        "space_id": space_id,
                        "description": description,
                        "file_id": file_id,
                        "logid": response.detail.as_ref().map(|d| &d.logid)
                    });

                    Ok(CallToolResult {
                        content: Some(vec![rmcp::model::Content::text(content)]),
                        is_error: Some(false),
                        structured_content: Some(structured),
                    })
                } else {
                    // API 返回错误
                    let error_msg = if response.msg.is_empty() {
                        "创建知识库失败".to_string()
                    } else {
                        response.msg
                    };

                    let content = format!(
                        "创建知识库失败:\n- 错误码: {}\n- 错误信息: {}{}",
                        response.code,
                        error_msg,
                        response
                            .detail
                            .as_ref()
                            .map(|d| format!("\n- 日志ID: {}", d.logid))
                            .unwrap_or_default()
                    );

                    let structured = json!({
                        "success": false,
                        "error_code": response.code,
                        "error_message": error_msg,
                        "logid": response.detail.as_ref().map(|d| &d.logid)
                    });

                    Ok(CallToolResult {
                        content: Some(vec![rmcp::model::Content::text(content)]),
                        is_error: Some(true),
                        structured_content: Some(structured),
                    })
                }
            }
            Err(e) => {
                let error_msg = format!("API 调用失败: {e}");
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(error_msg.clone())]),
                    is_error: Some(true),
                    structured_content: Some(json!({
                        "success": false,
                        "error": error_msg
                    })),
                })
            }
        }
    }

    /// 上传文档到知识库（本地文件）
    pub async fn upload_document_to_knowledge_base(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        use crate::api::knowledge_models::{
            ChunkStrategyCn, DocumentBaseCn, KnowledgeDocumentUploadRequestCn, SourceInfo,
        };
        use tokio::fs;

        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let dataset_id = args
            .get("dataset_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing dataset_id", None))?;
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing file_path", None))?;
        let document_name = args
            .get("document_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("document")
                    .to_string()
            });
        let chunk_size = args
            .get("chunk_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(800);

        let metadata = match fs::metadata(file_path).await {
            Ok(metadata) => metadata,
            Err(e) => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "Failed to read file metadata: {e}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(serde_json::json!({
                        "error": "file_not_found",
                        "message": format!("Failed to read file metadata: {}", e)
                    })),
                });
            }
        };
        let file_size = metadata.len();
        if file_size == 0 {
            return Err(McpError::invalid_params("File is empty", None));
        }
        if file_size > 10 * 1024 * 1024 {
            return Err(McpError::invalid_params(
                "File exceeds 10MB size limit for this example",
                None,
            ));
        }

        let bytes = fs::read(file_path)
            .await
            .map_err(|e| McpError::invalid_params(format!("Failed to read file: {e}"), None))?;
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        // MIME 类型目前不直接发送（服务器依据 file_type 推断），保留扩展判断仅用于潜在后续扩展
        // let mime_type = match ext.to_lowercase().as_str() { "txt" => "text/plain", "md" => "text/markdown", "pdf" => "application/pdf", "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document", _ => "application/octet-stream", };

        let content_base64 = {
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD.encode(&bytes)
        };
        // CN spec: document_bases: [{ name, source_info{ file_base64, file_type } }]
        let source_info = SourceInfo::file_base64(content_base64, ext.to_string());
        let document_cn = DocumentBaseCn {
            name: document_name.clone(),
            source_info,
            caption: None,
            update_rule: None,
        };
        // chunk_strategy: choose custom (chunk_type=1) with separator and max_tokens
        let separator = args
            .get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or("\n\n")
            .to_string();
        let chunk_type = 0; // custom
        let max_tokens = chunk_size as i64; // reuse user chunk_size param as max_tokens
        let chunk_strategy_cn = ChunkStrategyCn::text(separator, max_tokens, chunk_type);
        let format_type = args
            .get("format_type")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32; // default 0 text
        let request = KnowledgeDocumentUploadRequestCn {
            dataset_id: dataset_id.to_string(),
            document_bases: vec![document_cn],
            chunk_strategy: chunk_strategy_cn,
            format_type,
        };
        match self.coze_client.upload_document_cn(request).await {
            Ok(resp) => {
                let infos_len = resp.document_infos.as_ref().map(|v| v.len()).unwrap_or(0);
                let content = format!(
                    "文档上传成功: dataset_id={dataset_id}, 文件='{document_name}', size={file_size} bytes, documents_returned={infos_len}"
                );
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(content)]),
                    is_error: Some(false),
                    structured_content: Some(json!({
                        "dataset_id": dataset_id,
                        "file_name": document_name,
                        "file_size": file_size,
                        "returned_count": infos_len,
                        "code": resp.code,
                        "msg": resp.msg,
                    })),
                })
            }
            Err(e) => {
                println!("Failed to upload document: {e:?}");
                let serialized =
                    serde_json::to_value(&e).unwrap_or(json!({"error": e.to_string()}));
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "文档上传失败: {e:?}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": serialized})),
                })
            }
        }
    }

    /// 列出会话（最小实现）
    pub async fn list_conversations(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let workspace_id = args
            .get("workspace_id")
            .or_else(|| args.get("space_id"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                if !self.default_space_id.is_empty() {
                    Some(&self.default_space_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                McpError::invalid_params("Missing workspace_id (or space_id) parameter", None)
            })?;
        let bot_id = args
            .get("bot_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing bot_id parameter", None))?;
        let page = args.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
        let page_size = args.get("page_size").and_then(|v| v.as_u64()).unwrap_or(20);
        match self
            .coze_client
            .list_conversations_v1(
                bot_id,
                Some(workspace_id),
                Some(page as u32),
                Some(page_size as u32),
            )
            .await
        {
            Ok(body) => {
                let data = body.get("data").cloned().unwrap_or(body);
                let (items, total) = Self::extract_list_and_total(&data);
                let mut out = format!("{total} 条会话，page={page}, page_size={page_size}:\n\n");
                let mut sc: Vec<Value> = Vec::new();
                for (i, it) in items.iter().take(5).enumerate() {
                    if let Some(obj) = it.as_object() {
                        let cid = Self::get_str(obj, "conversation_id");
                        let title = Self::get_str(obj, "title");
                        out.push_str(&format!("{}. {} (id: {})\n", i + 1, title, cid));
                        sc.push(json!({"conversation_id": cid, "title": title}));
                    }
                }
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(out)]),
                    is_error: Some(false),
                    structured_content: Some(json!({"total": total, "items": sc})),
                })
            }
            Err(e) => {
                let serialized =
                    serde_json::to_value(&e).unwrap_or(json!({"error": e.to_string()}));
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(format!(
                        "[List Conversations] 请求失败: {e}"
                    ))]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": serialized})),
                })
            }
        }
    }

    // ===== 聊天功能 =====

    /// 发送聊天消息（非流式）
    pub async fn chat(&self, args: Option<Value>) -> Result<CallToolResult, McpError> {
        let args = args.unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        let bot_id = match args.get("bot_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        "错误: 缺少必需的 bot_id 参数",
                    )]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": "Missing bot_id parameter"})),
                });
            }
        };

        let message = match args.get("message").and_then(|v| v.as_str()) {
            Some(msg) => msg.to_string(),
            None => {
                return Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(
                        "错误: 缺少必需的 message 参数",
                    )]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": "Missing message parameter"})),
                });
            }
        };

        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // 如果用户没有提供user_id，自动生成一个随机UUID
                uuid::Uuid::new_v4().to_string()
            });
        let conversation_id = args
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 构建聊天请求（确保包含user_id，自动生成或用户提供）
        let mut chat_request = crate::api::chat_models::ChatRequest::new(bot_id, message)
            .with_stream(false)
            .with_user_id(user_id.clone()); // user_id是必选参数，自动生成或用户提供

        if let Some(cid) = conversation_id {
            chat_request = chat_request.with_conversation_id(cid);
        }

        // 处理自定义变量
        if let Some(variables_obj) = args.get("custom_variables") {
            if let Some(variables_map) = variables_obj.as_object() {
                let mut custom_vars = std::collections::HashMap::new();
                for (k, v) in variables_map {
                    if let Some(s) = v.as_str() {
                        custom_vars.insert(k.clone(), s.to_string());
                    }
                }
                if !custom_vars.is_empty() {
                    chat_request = chat_request.with_custom_variables(custom_vars);
                }
            }
        }

        match self.coze_client.chat(chat_request).await {
            Ok(response) => {
                let was_user_id_generated = args.get("user_id").and_then(|v| v.as_str()).is_none();

                let user_id_info = if was_user_id_generated {
                    format!("user_id: {user_id} (自动生成)\n")
                } else {
                    format!("user_id: {user_id} (用户提供)\n")
                };

                // 如果状态是in_progress，等待完成并获取最终消息
                if response.status.as_deref() == Some("in_progress")
                    || response.status.as_deref() == Some("created")
                {
                    // 等待对话完成
                    let mut final_status = response.status.clone();
                    let mut attempts = 0;
                    const MAX_ATTEMPTS: u32 = 30; // 最多等待30次，每次2秒

                    while (final_status.as_deref() == Some("in_progress")
                        || final_status.as_deref() == Some("created"))
                        && attempts < MAX_ATTEMPTS
                    {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        attempts += 1;

                        match self
                            .coze_client
                            .get_chat_detail(&response.conversation_id, &response.id)
                            .await
                        {
                            Ok(detail) => {
                                final_status = detail.status.clone();
                                if final_status.as_deref() == Some("completed") {
                                    // 获取对话消息
                                    match self
                                        .coze_client
                                        .get_chat_messages(&response.conversation_id, &response.id)
                                        .await
                                    {
                                        Ok(messages) => {
                                            // 找到助手的回复
                                            let assistant_reply = messages.iter()
                                                .filter(|msg| msg.role == crate::api::chat_models::MessageRole::Assistant)
                                                .filter_map(|msg| msg.content.as_ref())
                                                .map(|s| s.as_str())
                                                .collect::<Vec<_>>()
                                                .join("\n");

                                            let output = format!(
                                                "{}对话ID: {}\n消息ID: {}\n状态: {}\n\n🤖 助手回复:\n{}\n",
                                                user_id_info,
                                                response.conversation_id,
                                                response.id,
                                                final_status.as_deref().unwrap_or("completed"),
                                                if assistant_reply.is_empty() { "暂无回复内容" } else { &assistant_reply }
                                            );

                                            return Ok(CallToolResult {
                                                content: Some(vec![rmcp::model::Content::text(
                                                    output,
                                                )]),
                                                is_error: Some(false),
                                                structured_content: Some(json!({
                                                    "conversation_id": response.conversation_id,
                                                    "message_id": response.id,
                                                    "status": final_status,
                                                    "user_id": user_id,
                                                    "user_id_generated": was_user_id_generated,
                                                    "assistant_reply": assistant_reply,
                                                    "messages": messages
                                                })),
                                            });
                                        }
                                        Err(e) => {
                                            let output = format!(
                                                "{}对话ID: {}\n消息ID: {}\n状态: {}\n\n⚠️ 获取消息失败: {}",
                                                user_id_info,
                                                response.conversation_id,
                                                response.id,
                                                final_status.as_deref().unwrap_or("completed"),
                                                e
                                            );

                                            return Ok(CallToolResult {
                                                content: Some(vec![rmcp::model::Content::text(
                                                    output,
                                                )]),
                                                is_error: Some(true),
                                                structured_content: Some(json!({
                                                    "conversation_id": response.conversation_id,
                                                    "message_id": response.id,
                                                    "status": final_status,
                                                    "error": format!("Failed to get messages: {}", e)
                                                })),
                                            });
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                // 无法获取详情，继续等待
                                println!(
                                    "等待对话完成... (尝试 {attempts}/{MAX_ATTEMPTS}，错误: {e})"
                                );
                            }
                        }
                    }

                    // 如果超时或失败
                    let output = format!(
                        "{}对话ID: {}\n消息ID: {}\n状态: {}\n\n⏰ 等待超时或对话未完成，请稍后手动查询结果",
                        user_id_info,
                        response.conversation_id,
                        response.id,
                        final_status.as_deref().unwrap_or("timeout")
                    );

                    Ok(CallToolResult {
                        content: Some(vec![rmcp::model::Content::text(output)]),
                        is_error: Some(false),
                        structured_content: Some(json!({
                            "conversation_id": response.conversation_id,
                            "message_id": response.id,
                            "status": final_status,
                            "timeout": true
                        })),
                    })
                } else {
                    // 对话已经完成或有其他状态
                    let output = format!(
                        "{}对话ID: {}\n消息ID: {}\n状态: {}\n",
                        user_id_info,
                        response.conversation_id,
                        response.id,
                        response.status.as_deref().unwrap_or("unknown")
                    );

                    Ok(CallToolResult {
                        content: Some(vec![rmcp::model::Content::text(output)]),
                        is_error: Some(false),
                        structured_content: Some(
                            serde_json::to_value(&response).unwrap_or_default(),
                        ),
                    })
                }
            }
            Err(e) => {
                let error_msg = format!("[Chat] 聊天失败: {e}");
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(error_msg)]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": e.to_string()})),
                })
            }
        }
    }

    /// 发送流式聊天消息
    pub async fn chat_stream(&self, args: Option<Value>) -> Result<CallToolResult, McpError> {
        let args = args.unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        let bot_id = args
            .get("bot_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing bot_id parameter", None))?
            .to_string();

        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing message parameter", None))?
            .to_string();

        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // 如果用户没有提供user_id，自动生成一个随机UUID
                uuid::Uuid::new_v4().to_string()
            });
        let conversation_id = args
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 构建聊天请求（确保包含user_id，自动生成或用户提供）
        let mut chat_request = crate::api::chat_models::ChatRequest::new(bot_id, message)
            .with_stream(true)
            .with_user_id(user_id.clone()); // user_id是必选参数，自动生成或用户提供

        if let Some(cid) = conversation_id {
            chat_request = chat_request.with_conversation_id(cid);
        }

        // 处理自定义变量
        if let Some(variables_obj) = args.get("custom_variables") {
            if let Some(variables_map) = variables_obj.as_object() {
                let mut custom_vars = std::collections::HashMap::new();
                for (k, v) in variables_map {
                    if let Some(s) = v.as_str() {
                        custom_vars.insert(k.clone(), s.to_string());
                    }
                }
                if !custom_vars.is_empty() {
                    chat_request = chat_request.with_custom_variables(custom_vars);
                }
            }
        }

        match self.coze_client.chat_stream(chat_request).await {
            Ok(stream) => {
                use futures::StreamExt;

                let mut full_content = String::new();
                let mut conversation_id = String::new();
                let mut message_id = String::new();
                let mut final_usage: Option<crate::api::chat_models::ChatUsage> = None;
                let mut events = Vec::new();

                // Pin the stream to make it ready for iteration
                tokio::pin!(stream);

                // 收集流式响应
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(response) => {
                            events.push(serde_json::to_value(&response).unwrap_or_default());

                            // 更新会话信息
                            if let Some(cid) = &response.conversation_id {
                                conversation_id = cid.clone();
                            }
                            if let Some(mid) = &response.id {
                                message_id = mid.clone();
                            }

                            // 累积内容
                            if let Some(delta) = &response.delta {
                                if let Some(content) = &delta.content {
                                    full_content.push_str(content);
                                }
                            }

                            // 保存最终使用情况
                            if let Some(usage) = &response.usage {
                                final_usage = Some(usage.clone());
                            }

                            // 检查是否完成
                            match response.event {
                                crate::api::chat_models::StreamEventType::Done |
                                crate::api::chat_models::StreamEventType::ConversationChatCompleted => {
                                    break;
                                }
                                crate::api::chat_models::StreamEventType::ConversationChatFailed => {
                                    return Ok(CallToolResult {
                                        content: Some(vec![rmcp::model::Content::text(
                                            format!("[Chat Stream] 聊天失败: {:?}", response.last_error)
                                        )]),
                                        is_error: Some(true),
                                        structured_content: Some(json!({
                                            "error": "chat_failed",
                                            "last_error": response.last_error,
                                            "events": events
                                        })),
                                    });
                                }
                                _ => continue,
                            }
                        }
                        Err(e) => {
                            return Ok(CallToolResult {
                                content: Some(vec![rmcp::model::Content::text(format!(
                                    "[Chat Stream] 流式响应错误: {e}"
                                ))]),
                                is_error: Some(true),
                                structured_content: Some(json!({
                                    "error": e.to_string(),
                                    "events": events
                                })),
                            });
                        }
                    }
                }

                let output = format!(
                    "对话ID: {conversation_id}\n消息ID: {message_id}\n完整回复:\n{full_content}\n\n使用情况: {final_usage:?}"
                );

                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(output)]),
                    is_error: Some(false),
                    structured_content: Some(json!({
                        "conversation_id": conversation_id,
                        "message_id": message_id,
                        "content": full_content,
                        "usage": final_usage,
                        "events": events
                    })),
                })
            }
            Err(e) => {
                let error_msg = format!("[Chat Stream] 流式聊天失败: {e}");
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(error_msg)]),
                    is_error: Some(true),
                    structured_content: Some(json!({"error": e.to_string()})),
                })
            }
        }
    }
}
