use crate::api::CozeApiClient;
use crate::knowledge::KnowledgeManager;
// Removed obsolete imports: chat/knowledge modules not present in models.
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
// Import request/response model types
use crate::models::{CozeApiRequest, HttpMethod};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CozeTools {
    coze_client: Arc<CozeApiClient>,
    knowledge_manager: Arc<KnowledgeManager>,
    default_space_id: String,
}

impl CozeTools {
    fn html_escape(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                _ => out.push(ch),
            }
        }
        out
    }
    pub fn new(
        coze_client: Arc<CozeApiClient>,
        knowledge_manager: Arc<KnowledgeManager>,
        default_space_id: String,
    ) -> Self {
        Self {
            coze_client,
            knowledge_manager,
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
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let format_type = args
            .get("format_type")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32);
        let page_num = args
            .get("page_num")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .or(Some(1));
        let page_size = args
            .get("page_size")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);
        let accurate_counts = args
            .get("accurate_counts")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let detailed = args
            .get("detailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    // 使用统一 /v1/datasets 接口
    match self.coze_client.list_datasets(&space_id, name.as_deref(), format_type, page_num, page_size).await {
            Ok(mut result) => {
                // Optional: refine document_count by fetching dataset detail (limited to first 50 to avoid many requests)
                if accurate_counts {
                    let limit = result.datasets.len().min(50);
                    for i in 0..limit {
                        let ds_id = result.datasets[i].dataset_id.clone();
                        if let Ok(detail) = self.coze_client.get_dataset_cn(&ds_id).await {
                            let data = detail.get("data").cloned().unwrap_or(detail);
                            if let Some(obj) = data.as_object() {
                                let doc_count = obj
                                    .get("file_list")
                                    .and_then(|v| v.as_array().map(|a| a.len() as u64))
                                    .or_else(|| obj.get("doc_count").and_then(|v| v.as_u64()))
                                    .or_else(|| obj.get("document_count").and_then(|v| v.as_u64()))
                                    .or_else(|| obj.get("file_count").and_then(|v| v.as_u64()))
                                    .unwrap_or(result.datasets[i].document_count as u64);
                                result.datasets[i].document_count = doc_count as usize;
                            }
                        }
                    }
                }
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
                let sc_items: Vec<Value> = if detailed {
                    result.datasets.iter().map(|kb| {
                        serde_json::json!({
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
                    }).collect()
                } else {
                    result.datasets.iter().map(|kb| {
                        serde_json::json!({
                            "dataset_id": kb.dataset_id,
                            "name": kb.name,
                            "description": kb.description,
                            "document_count": kb.document_count,
                            "created_at": kb.created_at,
                        })
                    }).collect()
                };
                let structured = serde_json::json!({ "total": result.total, "detailed": detailed, "items": sc_items });

                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(content)]),
                    is_error: Some(false),
                    structured_content: Some(structured),
                })
            }
            Err(e) => Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(format!(
                    "获取知识库列表失败: {}",
                    e
                ))]),
                is_error: Some(true),
                structured_content: None,
            }),
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
        let page = args.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
        let page_size = args.get("page_size").and_then(|v| v.as_u64()).unwrap_or(20);

        let endpoint = format!(
            "/v1/bots?workspace_id={}&page={}&page_size={}&status=draft_published",
            urlencoding::encode(workspace_id),
            page,
            page_size
        );
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
                let mut out = format!("找到 {} 个 Bot:\n\n", total);
                let mut sc_items: Vec<Value> = Vec::new();
                for (i, it) in items.iter().take(5).enumerate() {
                    if let Some(obj) = it.as_object() {
                        let id = Self::get_str(obj, "bot_id");
                        let name = Self::get_str(obj, "name");
                        let status = Self::get_str(obj, "status");
                        out.push_str(&format!(
                            "{}. {} (id: {}, status: {})\n",
                            i + 1,
                            name,
                            id,
                            status
                        ));
                        sc_items.push(serde_json::json!({
                            "bot_id": id,
                            "name": name,
                            "status": status,
                        }));
                    }
                }
                let structured = serde_json::json!({ "total": total, "items": sc_items });
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(out)]),
                    is_error: Some(false),
                    structured_content: Some(structured),
                })
            }
            Err(e) => Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(format!(
                    "[Bots] 请求失败: {}",
                    e
                ))]),
                is_error: Some(true),
                structured_content: None,
            }),
        }
    }


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
                let mut out = format!("找到 {} 个 Workspace:\n\n", total);
                let mut sc_items: Vec<Value> = Vec::new();
                for (i, it) in items.iter().take(5).enumerate() {
                    if let Some(obj) = it.as_object() {
                        // (Removed many conversation export/analysis helpers for minimal surface.)
        let mut items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if direction.eq_ignore_ascii_case("desc") { items.reverse(); }
        let items: Vec<Value> = if let Some(l) = limit { items.into_iter().take(l).collect() } else { items };
        let mut text = String::new();
        for it in items.iter() {
            if let Some(o) = it.as_object() {
                let role = o.get("role").and_then(|v| v.as_str()).unwrap_or("");
                let content = o.get("content").and_then(|v| v.as_str()).unwrap_or("");
                text.push_str(&format!("[{}] {}\n", role, content));
            }
        }
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("导出会话 {} 的纯文本成功（方向: {}，行数: {}）", conversation_id, direction, text.lines().count()))]), is_error: Some(false), structured_content: Some(serde_json::json!({"conversation_id": conversation_id, "direction": direction, "text": text})) })
    }

    // 获取某条消息的上下文窗口（前后若干条）
    pub async fn get_message_context(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let message_id = args.get("message_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing message_id parameter", None))?;
        let before = args.get("before").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let after = args.get("after").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Message Context] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let pos = items.iter().position(|it| it.get("message_id").and_then(|v| v.as_str()) == Some(message_id));
        if let Some(idx) = pos {
            let start = idx.saturating_sub(before);
            let end = (idx + 1 + after).min(items.len());
            let window = &items[start..end];
            let sc_items: Vec<Value> = window.iter().map(|v| v.clone()).collect();
            let content = format!("消息上下文窗口：index={}，范围=[{}, {})，总计 {} 条", idx, start, end, sc_items.len());
            Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(content)]), is_error: Some(false), structured_content: Some(serde_json::json!({"conversation_id": conversation_id, "center_message_id": message_id, "start_index": start, "end_index": end, "messages": sc_items})) })
        } else {
            Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("未找到指定 message_id")]), is_error: Some(true), structured_content: None })
        }
    }

    // 获取某条消息在会话中的索引（0=最旧，-1=最新），便于定位
    pub async fn get_message_index_by_id(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let message_id = args.get("message_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing message_id parameter", None))?;
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Message Index] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("未找到任何消息")]), is_error: Some(true), structured_content: None });
        }
        let idx_opt = items.iter().position(|it| it.get("message_id").and_then(|v| v.as_str()) == Some(message_id));
        match idx_opt {
            Some(idx) => {
                let last_idx = items.len() as isize - 1;
                let neg_idx = idx as isize - (last_idx as isize);
                let obj = items[idx].as_object().cloned().unwrap_or_default();
                let role = obj.get("role").and_then(|v| v.as_str()).unwrap_or("");
                let content = obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let create_time = obj.get("create_time").cloned().unwrap_or(Value::Null);
                let summary = format!("message_id={} at index={} (neg_index={}), role={}, create_time={}", message_id, idx, neg_idx, role, create_time);
                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(summary)]),
                    is_error: Some(false),
                    structured_content: Some(serde_json::json!({
                        "conversation_id": conversation_id,
                        "message_id": message_id,
                        "index": idx,
                        "neg_index": neg_idx,
                        "role": role,
                        "create_time": create_time,
                        "content_preview": content.chars().take(120).collect::<String>()
                    }))
                })
            }
            None => Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("未找到指定 message_id")]), is_error: Some(true), structured_content: None }),
        }
    }

    // 获取会话的时间线简表（index, message_id, role, create_time），便于概览
    pub async fn get_conversation_timeline(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let direction = args.get("direction").and_then(|v| v.as_str()).unwrap_or("asc");
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Timeline] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let mut items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if direction.eq_ignore_ascii_case("desc") { items.reverse(); }
        let items: Vec<Value> = if let Some(l) = limit { items.into_iter().take(l).collect() } else { items };
        let mut rows: Vec<Value> = Vec::with_capacity(items.len());
        for (i, it) in items.iter().enumerate() {
            if let Some(o) = it.as_object() {
                let message_id = o.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
                let role = o.get("role").and_then(|v| v.as_str()).unwrap_or("");
                let create_time = o.get("create_time").cloned().unwrap_or(Value::Null);
                rows.push(serde_json::json!({
                    "index": i,
                    "message_id": message_id,
                    "role": role,
                    "create_time": create_time
                }));
            }
        }
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("时间线生成成功：{} 条", rows.len()))]), is_error: Some(false), structured_content: Some(serde_json::json!({
            "conversation_id": conversation_id,
            "direction": direction,
            "rows": rows
        })) })
    }

    // 导出会话为 CSV 文本（只读，本地拼接），列：index,message_id,role,create_time,content
    pub async fn export_conversation_csv(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let direction = args.get("direction").and_then(|v| v.as_str()).unwrap_or("asc");
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Export CSV] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let mut items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if direction.eq_ignore_ascii_case("desc") { items.reverse(); }
        let items: Vec<Value> = if let Some(l) = limit { items.into_iter().take(l).collect() } else { items };
        let mut csv = String::from("index,message_id,role,create_time,content\n");
        for (i, it) in items.iter().enumerate() {
            if let Some(o) = it.as_object() {
                let message_id = o.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
                let role = o.get("role").and_then(|v| v.as_str()).unwrap_or("");
                let create_time = o.get("create_time").map(|v| v.to_string()).unwrap_or("".to_string());
                let mut content = o.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                // 简易转义：替换双引号并包裹
                content = content.replace('"', "\"");
                let content = format!("\"{}\"", content);
                csv.push_str(&format!("{},{}{},{}{},{}{},{}{},{}\n",
                    i,
                    "", message_id,
                    "", role,
                    "", create_time,
                    "", content,
                    ""));
            }
        }
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("导出会话 {} 的 CSV 成功（方向: {}，行数: {}）", conversation_id, direction, csv.lines().count() - 1))]), is_error: Some(false), structured_content: Some(serde_json::json!({
            "conversation_id": conversation_id,
            "direction": direction,
            "csv": csv
        })) })
    }

    // 过滤指定角色的消息
    pub async fn filter_messages_by_role(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let role = args.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(50);
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Filter By Role] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let mut out: Vec<Value> = Vec::new();
        for it in items.iter() {
            if let Some(o) = it.as_object() {
                let r = o.get("role").and_then(|v| v.as_str()).unwrap_or("");
                if role.is_empty() || r.eq_ignore_ascii_case(role) {
                    out.push(Value::Object(o.clone()));
                    if out.len() >= limit { break; }
                }
            }
        }
        let content = format!("筛选角色 '{}' 的消息，共 {} 条（limit={}）", role, out.len(), limit);
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(content)]), is_error: Some(false), structured_content: Some(serde_json::json!({
            "conversation_id": conversation_id,
            "role": role,
            "limit": limit,
            "messages": out
        })) })
    }

    // 计算会话时长（首尾 create_time 的差值）
    pub async fn get_conversation_duration(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Duration] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text("没有消息，无法计算时长")]), is_error: Some(true), structured_content: None });
        }
        let first_ct = items.first().and_then(|x| x.get("create_time")).cloned();
        let last_ct = items.last().and_then(|x| x.get("create_time")).cloned();
        // 兼容数字或字符串时间戳（毫秒/秒），这里仅以原值返回并尝试做简单差值（若均为整数）
        let duration_ms = match (first_ct.as_ref().and_then(|v| v.as_i64()), last_ct.as_ref().and_then(|v| v.as_i64())) {
            (Some(f), Some(l)) => Some(l.saturating_sub(f)),
            _ => None,
        };
        let content = match duration_ms {
            Some(d) => format!("会话时长：{} (单位取决于 create_time 的粒度，通常为毫秒)", d),
            None => "会话时长：无法计算（时间戳类型不统一或缺失）".to_string(),
        };
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(content.clone())]), is_error: Some(false), structured_content: Some(serde_json::json!({
            "conversation_id": conversation_id,
            "first_create_time": first_ct,
            "last_create_time": last_ct,
            "duration": duration_ms
        })) })
    }

    // 导出会话为 NDJSON（每行一个 JSON 对象）
    pub async fn export_conversation_ndjson(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let direction = args.get("direction").and_then(|v| v.as_str()).unwrap_or("asc");
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Export NDJSON] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let mut items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if direction.eq_ignore_ascii_case("desc") { items.reverse(); }
        let items: Vec<Value> = if let Some(l) = limit { items.into_iter().take(l).collect() } else { items };
        let mut out = String::new();
        for it in items.iter() {
            out.push_str(&serde_json::to_string(it).unwrap_or("{}".to_string()));
            out.push('\n');
        }
        Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("导出会话 {} 的 NDJSON 成功（方向: {}，行数: {}）", conversation_id, direction, out.lines().count()))]), is_error: Some(false), structured_content: Some(serde_json::json!({
            "conversation_id": conversation_id,
            "direction": direction,
            "ndjson": out
        })) })
    }

    // 按时间范围过滤消息（闭区间），create_time 支持数值字符串
    pub async fn filter_messages_by_time_range(
        &self,
        args: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let args = args.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
        let conversation_id = args.get("conversation_id").and_then(|v| v.as_str()).ok_or_else(|| McpError::invalid_params("Missing conversation_id parameter", None))?;
        let start = args.get("start").and_then(|v| v.as_i64()).or_else(|| args.get("start").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok())).ok_or_else(|| McpError::invalid_params("Missing or invalid start", None))?;
        let end = args.get("end").and_then(|v| v.as_i64()).or_else(|| args.get("end").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok())).ok_or_else(|| McpError::invalid_params("Missing or invalid end", None))?;
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(200);
        let body = match self.coze_client.get_conversation_messages_v3(conversation_id).await { Ok(b) => b, Err(e) => { return Ok(CallToolResult { content: Some(vec![rmcp::model::Content::text(format!("[Filter By Time] 请求失败: {}", e))]), is_error: Some(true), structured_content: None }) } };
        let data = body.get("data").cloned().unwrap_or(body);
        let items = data.get("messages").or_else(|| data.get("list")).or_else(|| data.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let mut out: Vec<Value> = Vec::new();
        for it in items.iter() {
            if let Some(o) = it.as_object() {
                let ct = o.get("create_time").and_then(|v| v.as_i64()).or_else(|| o.get("create_time").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok()));
                if let Some(ts) = ct {
                    if ts >= start && ts <= end {
                        out.push(Value::Object(o.clone()));
                        if out.len() >= limit { break; }
                    }
                }
                let structured = serde_json::json!({
                    "dataset_id": dataset_id,
                    "name": name,
                    "description": description,
                    "space_id": space_id,
                    "permission": permission
                });

                Ok(CallToolResult {
                    content: Some(vec![rmcp::model::Content::text(content)]),
                    is_error: Some(false),
                    structured_content: Some(structured),
                })
            }
            Err(e) => Ok(CallToolResult {
                content: Some(vec![rmcp::model::Content::text(format!(
                    "知识库创建失败: {}",
                    e
                ))]),
                is_error: Some(true),
                structured_content: None,
            }),
        }
    }
}
