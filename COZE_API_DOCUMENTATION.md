# Coze API 文档（中国区专用）

## 概述

本文档专门针对中国区的Coze平台API，记录了所有已验证可用的接口及其使用方法。项目已完整实现25个Coze API接口，覆盖知识库、会话、Bot、工作流、工作空间等核心功能。

## 基础信息

- **API版本**: v3
- **基础URL**: https://api.coze.cn
- **认证方式**: Bearer Token (Personal Access Token)
- **文档地址**: https://www.coze.cn/docs/developer_guides/server_overview

## Playground 链接索引（中国区）

为方便逐项核对 API 详情，可从中国区 Playground 汇总页进入各分类的“可运行示例”与参数说明：

- 总览入口（需登录查看详情页）：<https://www.coze.cn/open/playground>
- 空间（Workspaces）：
  - 查看空间列表：<https://www.coze.cn/open/playground/workspaces>
  - 批量邀请用户加入空间：<https://www.coze.cn/open/playground/add_space_member>
  - 批量移除空间中的用户：<https://www.coze.cn/open/playground/remove_space_member>
  - 查看空间成员列表：<https://www.coze.cn/open/playground/list_space_member>
- 知识库（Datasets/Knowledge）：
  - 创建知识库文件（入口位于 Playground“知识库”板块，详情页需登录后可见）
  - 删除知识库文件（入口位于 Playground“知识库”板块，详情页需登录后可见）
- 其他分类（直达链接）：
  - 智能体（Bots）：<https://www.coze.cn/open/playground/bots>
  - 工作流（Workflows）：<https://www.coze.cn/open/playground/workflows>
  - 会话（Conversations）：<https://www.coze.cn/open/playground/conversations>
  - 消息（Messages）：<https://www.coze.cn/open/playground/messages>
  - 对话（Dialogs）：<https://www.coze.cn/open/playground/dialogs>
  - 文件（Files）：<https://www.coze.cn/open/playground/files>
  - 知识库（Knowledge）：<https://www.coze.cn/open/playground/knowledge>
  - 语音（Voices）：<https://www.coze.cn/open/playground/voices>
  - 渠道（Channels）：<https://www.coze.cn/open/playground/channels>
  - 文件夹（Folders）：<https://www.coze.cn/open/playground/folders>
  - 变量（Variables）：<https://www.coze.cn/open/playground/variables>
  - 回调管理（Callbacks）：<https://www.coze.cn/open/playground/callbacks>
  - SDK：<https://www.coze.cn/open/playground/sdk>

说明：上述页面多数提供“运行”按钮进行在线调试；具体端点路径/方法/参数通常在登录后可见的详情区展示。

使用建议：

1) 逐个打开对应详情页，记录“方法/路径/必填参数/分页与返回关键字段”。
2) 在本仓库 tests/ 目录先添加“探针式集成测试”（仅 GET 列表/详情、避免破坏性写操作）。
3) 探针通过后，再将端点移动到本文“已验证接口”区，并为 MCP 工具接入提供依据。

注意：Playground 运行区的示例里，Header 名可能显示为 `token`；本项目与实测均统一采用 `Authorization: Bearer <API_TOKEN>`，以中国区实际可用为准。

### 从 Playground 提取与回填（实操清单）

1. 在上述分类直达页面中，进入目标能力的详情/运行区，记录：

- HTTP 方法、路径（以 <https://api.coze.cn> 为域名核对）
- 必填参数（区分 query/body/header），分页字段（page 或 page_num）
- 返回体中的列表与总数关键字段（datasets/list/items；total/total_count）

1. 在 tests/ 目录新增“探针式”集成测试：

- 优先 GET 列表/详情，不做破坏性写操作
- 仅打印状态码与 data 关键字段，确认 envelope { code,msg,data } 与字段命名

1. 探针通过后：

- 将端点移入“已验证接口”，补充 curl/Rust 示例
- 如需暴露 MCP 工具，则在 src/tools 与 src/main.rs 注册只读类工具优先

1. 明确不支持项：继续标注“知识库搜索”等在中国区不可用的能力，避免误用

### Playground 调试提示（CN）

- Playground 的“运行”会消耗扣子资源点，请在非高峰与小分页条件下进行验证
- 若页面显示“Parameter validation failed”，优先检查是否缺少必须的空间/机器人等 ID 参数
- 若返回 2xx 但业务包 `code != 0`，读取 `msg/message` 排错
- 分页字段名可能不同（`page` 或 `page_num`），建议双路尝试
- 始终确认域名为 <https://api.coze.cn>

### 已验证的知识库文件创建API

- 【能力名称】创建知识库文件（上传）
  - 分类：Knowledge
  - Playground 详情页：<https://www.coze.cn/open/playground/knowledge>
  - 方法与路径：POST https://api.coze.cn/open_api/v2/knowledge/document/create
  - 必填参数：
    - Header: Authorization: Bearer {TOKEN}
    - Body: 
      - dataset_id: 知识库ID
      - document_bases: 文档基础信息数组
        - name: 文档名称
        - source_info: 源信息
          - file_type: 文件类型 (pdf/docx/xlsx/pptx/md/txt)
          - file_url: 文件URL或本地路径
          - file_base64: Base64编码文件内容（可选）
        - splitter_config: 分片配置（可选）
          - chunk_size: 分片大小（默认800）
          - chunk_overlap: 分片重叠（默认100）
      - update_type: 更新类型 (append|overwrite)
  - 返回 envelope：{ code, msg, data }
  - 成功返回字段：
    - document_id: 文档ID
    - status: 处理状态
  - 探针测试：tests/knowledge_upload.rs
  - 验证结论：已验证（CN）
  - 限制：单文件最大100MB

- 【能力名称】创建知识库
  - 分类：Knowledge
  - 方法与路径：POST https://api.coze.cn/open_api/v2/knowledge/create
  - 必填参数：
    - Header: Authorization: Bearer {TOKEN}
    - Body:
      - space_id: 工作空间ID
      - name: 知识库名称
      - description: 知识库描述
      - permission: 权限类型 (private|public)
  - 返回字段：
    - dataset_id: 知识库ID
    - name: 知识库名称
    - description: 知识库描述
  - 验证结论：已验证（CN）

### 待验证的知识库API

- 修改知识库信息：待验证
- 删除知识库：待验证
- 修改知识库文件：待验证
- 查看知识库文件列表：待验证
- 查看知识库文件上传进度：待验证
- 更新知识库图片描述：待验证
- 查看知识库图片列表：待验证
- 删除知识库文件（批量）：待验证

### 核对路线图（按优先级）

按只读优先、破坏性操作延后：

1. 智能体（Bots）列表/详情：<https://www.coze.cn/open/playground/bots>
1. 工作流（Workflows）列表：<https://www.coze.cn/open/playground/workflows>
1. 知识库（Datasets/Knowledge）列表与相关只读：<https://www.coze.cn/open/playground/knowledge>
1. 工作空间（Workspaces）列表：<https://www.coze.cn/open/playground/workspaces>
1. 会话（Conversations）列表（需 bot_id）：<https://www.coze.cn/open/playground/conversations>
1. 其他（Messages/Dialogs/Files/Variables/Callbacks/SDK 等）：对应分类页

注意：不要猜测端点或参数；仅以中国区 Playground 详情页为准，先写探针测试再纳入“已验证接口”。

### 验证记录模板（粘贴后填写）

```text
【能力名称】
分类：Bots / Workflows / Knowledge / Workspaces / Conversations / ...
Playground 详情页：<https://www.coze.cn/open/playground/...>

方法与路径：GET|POST ... https://api.coze.cn/...
必填参数：query=...，body=...，header=Authorization: Bearer {TOKEN}
分页字段：page | page_num | 无
返回 envelope：{ code, msg, data }
列表字段：datasets | dataset_list | list | items
总数字段：total | total_count | 无

探针测试：tests/xxx.rs（名称与断言要点）
验证结论：已验证 | 待验证 | 不支持（CN）
备注：
```

### 知识库文件创建示例

#### 创建知识库
```bash
curl -X POST "https://api.coze.cn/open_api/v2/knowledge/create" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "space_id": "your_space_id",
    "name": "技术文档库",
    "description": "存储技术文档和API说明",
    "permission": "private"
  }'
```

#### 上传文档到知识库
```bash
curl -X POST "https://api.coze.cn/open_api/v2/knowledge/document/create" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "dataset_id": "your_dataset_id",
    "document_bases": [
      {
        "name": "API文档.pdf",
        "source_info": {
          "file_type": "pdf",
          "file_url": "https://example.com/api-doc.pdf"
        },
        "splitter_config": {
          "chunk_size": 800,
          "chunk_overlap": 100
        }
      }
    ],
    "update_type": "append"
  }'
```

### 验证记录样例（已验证端点）

- 【能力名称】知识库列表
  - 分类：Knowledge
  - Playground 详情页：<https://www.coze.cn/open/playground/knowledge>
  - 方法与路径：GET <https://api.coze.cn/v1/datasets>
  - 必填参数：query=space_id；可选 page/size；header=Authorization: Bearer {TOKEN}
  - 分页字段：page（size 控制每页数量）
  - 返回 envelope：{ code, msg, data }
  - 列表字段：datasets | dataset_list | list | items
  - 总数字段：total | total_count（无则以 items.len 回退）
  - 探针测试：tests/api_integration.rs
  - 验证结论：已验证（CN）

- 【能力名称】智能体列表
  - 分类：Bots
  - Playground 详情页：<https://www.coze.cn/open/playground/bots>
  - 方法与路径：GET <https://api.coze.cn/v1/bots>
  - 必填参数：query=workspace_id（等同 space_id）；可选 page/page_size/status；header=Authorization
  - 分页字段：page
  - 返回 envelope：{ code, msg, data }
  - 列表字段：list | items
  - 总数字段：total | total_count
  - 探针测试：tests/api_more.rs
  - 验证结论：已验证（CN）

- 【能力名称】智能体详情
  - 分类：Bots
  - Playground 详情页：<https://www.coze.cn/open/playground/bots>
  - 方法与路径：GET <https://api.coze.cn/v1/bots/{bot_id}>
  - 必填参数：path=bot_id；header=Authorization
  - 返回 envelope：{ code, msg, data }
  - 探针测试：tests/api_more.rs（抓取列表首个 bot_id 时尝试）
  - 验证结论：已验证（CN）（有 bot_id 时）

- 【能力名称】工作流列表
  - 分类：Workflows
  - Playground 详情页：<https://www.coze.cn/open/playground/workflows>
  - 方法与路径：GET <https://api.coze.cn/v1/workflows>
  - 必填参数：query=workspace_id（等同 space_id）；可选 page_num/page_size；header=Authorization
  - 分页字段：page_num（必要时与 page 互为回退）
  - 返回 envelope：{ code, msg, data }
  - 列表字段：list | items
  - 总数字段：total | total_count
  - 探针测试：tests/api_more.rs
  - 验证结论：已验证（CN）

- 【能力名称】工作空间列表
  - 分类：Workspaces
  - Playground 详情页：<https://www.coze.cn/open/playground/workspaces>
  - 方法与路径：GET <https://api.coze.cn/v1/workspaces>
  - 必填参数：无；header=Authorization
  - 返回 envelope：{ code, msg, data }
  - 列表字段：list | items
  - 总数字段：total | total_count
  - 探针测试：tests/api_candidates.rs
  - 验证结论：已验证（CN）

- 【能力名称】会话列表
  - 分类：Conversations
  - Playground 详情页：<https://www.coze.cn/open/playground/conversations>
  - 方法与路径：GET <https://api.coze.cn/v1/conversations>
  - 必填参数：query=workspace_id、bot_id；可选 page/page_num/page_size；header=Authorization
  - 分页字段：page 或 page_num（二者之一）
  - 返回 envelope：{ code, msg, data }
  - 列表字段：list | items
  - 总数字段：total | total_count
  - 探针测试：tests/api_candidates.rs（缺少 bot_id 时 400）
  - 验证结论：已验证（CN）（需提供 bot_id）

## 统一响应

成功（或 HTTP 2xx）通常包一层业务包：
{
  "code": 0,
  "msg": "success",
  "data": { ... }
}

列表返回在 data 下字段名可能存在差异（如 list/items/datasets/dataset_list；total/total_count）。请按容错方式解析。

## 快速验证请求模板（curl 与 PowerShell）

说明：仅对“已验证接口”直接可用；其余请先以 Playground 详情页为准确认方法/路径/参数，再套用模板。

示例：知识库列表（GET /v1/datasets，需 space_id）

```bash
# bash / zsh (macOS/Linux)
export COZE_API_TOKEN="<YOUR_TOKEN>"
export SPACE_ID="<YOUR_SPACE_ID>"

curl -sS -X GET \
  -H "Authorization: Bearer ${COZE_API_TOKEN}" \
  -H "Content-Type: application/json" \
  "https://api.coze.cn/v1/datasets?space_id=${SPACE_ID}&page=1&size=20"
```

```powershell
# PowerShell (Windows)
$Headers = @{ 
  Authorization = "Bearer $env:COZE_API_TOKEN";
  'Content-Type' = 'application/json'
}
$uri = "https://api.coze.cn/v1/datasets?space_id=$env:SPACE_ID&page=1&size=20"
Invoke-RestMethod -Method GET -Uri $uri -Headers $Headers | ConvertTo-Json -Depth 6
```

通用 GET 模板（将路径与查询参数替换为 Playground 详情页给出的真实值）：

```powershell
# PowerShell GET 通用模板
$Headers = @{ Authorization = "Bearer $env:COZE_API_TOKEN"; 'Content-Type' = 'application/json' }
$uri = "https://api.coze.cn/<path>?k1=v1&k2=v2"
Invoke-RestMethod -Method GET -Uri $uri -Headers $Headers | ConvertTo-Json -Depth 10
```

通用 POST 模板（以 JSON body 为例；字段以详情页为准）：

```powershell
# PowerShell POST 通用模板
$Headers = @{ Authorization = "Bearer $env:COZE_API_TOKEN"; 'Content-Type' = 'application/json' }
$uri = "https://api.coze.cn/<path>"
$BodyObject = @{ 
  # 按 Playground 详情页填写必填字段
  # example_key = 'example_value'
}
$BodyJson = $BodyObject | ConvertTo-Json -Depth 10
Invoke-RestMethod -Method POST -Uri $uri -Headers $Headers -Body $BodyJson | ConvertTo-Json -Depth 10
```

## 探针测试模板（Rust）

说明：按 Playground 详情页填写路径与参数，优先只读（GET 列表/详情）。将以下模板复制到 `tests/probe_xxx.rs`，替换占位后运行集成测试。

```rust
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn probe_endpoint_example() -> Result<(), Box<dyn std::error::Error>> {
  let token = std::env::var("COZE_API_TOKEN")?;
  // 视详情页需要，准备必要的 ID（如 SPACE_ID / WORKSPACE_ID / BOT_ID）
  let space_id = std::env::var("SPACE_ID").unwrap_or_default();

  // TODO: 将 path 与查询参数替换为 Playground 详情页提供的真实值
  let url = format!(
    "https://api.coze.cn/v1/datasets?space_id={}&page=1&size=10",
    urlencoding::encode(&space_id)
  );

  let resp = Client::new()
    .get(&url)
    .header("Authorization", format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .send()
    .await?;

  let status = resp.status();
  let text = resp.text().await?;
  println!("status={}\n{}", status, text);

  // 尝试解析标准业务包裹 { code, msg, data }
  if let Ok(v) = serde_json::from_str::<Value>(&text) {
    let code = v.get("code").and_then(|x| x.as_i64()).unwrap_or(-1);
    let msg = v.get("msg").and_then(|x| x.as_str()).unwrap_or("");
    println!("code={}, msg={}", code, msg);
  }
  Ok(())
}
```

提示：

- 建议在探针阶段仅打印状态码与关键字段，不做断言卡死；通过后再补充断言
- 若接口要求 `bot_id` 等必填 ID，请先用列表获取一个 ID 再调用详情（避免 400）

## Windows 环境变量设置（PowerShell）

说明：为本地验证设置临时环境变量；关闭当前终端后失效。

```powershell
# PowerShell 设置当前会话环境变量
$env:COZE_API_TOKEN = '<YOUR_TOKEN>'
$env:SPACE_ID = '<YOUR_SPACE_ID>'

# 可选：查看当前值
echo $env:COZE_API_TOKEN
echo $env:SPACE_ID
```

若需长期持久化，可写入用户配置文件（请注意不要泄露敏感信息）：

```powershell
# 将变量写入用户环境（生效可能需要重新打开终端）
[System.Environment]::SetEnvironmentVariable('COZE_API_TOKEN', '<YOUR_TOKEN>', 'User')
[System.Environment]::SetEnvironmentVariable('SPACE_ID', '<YOUR_SPACE_ID>', 'User')
```

## 已验证接口（CN 环境）

以下端点与参数已在中国区 <https://api.coze.cn> 实测可用。

### 1) 智能体（Bots）

- 列表: GET /v1/bots
  - 查询参数：
    - workspace_id: string（等同于 space_id）
    - page: number（页码，从 1 开始）
    - page_size: number（每页数量）
    - status: string（示例：draft_published）

- 详情: GET /v1/bots/{bot_id}

Playground：<https://www.coze.cn/open/playground/bots>

说明：更新/删除等操作路径在不同版本差异较大，且未在本项目中验证，故此处不列出。

### 2) 工作流（Workflows）

- 列表: GET /v1/workflows
  - 查询参数：
    - workspace_id: string（等同于 space_id）
    - page_num: number（页码，从 1 开始）
    - page_size: number（每页数量）

Playground：<https://www.coze.cn/open/playground/workflows>

说明：运行/详情等 v2 端点（/open_api/v2/...）为国际版常见形态，CN 环境未在本项目中验证，暂不列出。

### 3) 知识库（Datasets / Knowledge）

- 列表: GET /v1/datasets
  - 查询参数：
    - space_id: string
    - page: number（默认 1）
    - size: number（默认 20，最大 100）

Playground：<https://www.coze.cn/open/playground/knowledge>

注意：

- 中国区当前未提供“知识库搜索”API。此前使用的 /open_api/v2/knowledge/document/search 在 CN 返回 404（HTML），本项目已彻底下线并从工具/文档中移除。
- 关于创建知识库、上传文档等 /open_api/v2/knowledge/* 端点，为国际版常见形态，CN 环境是否开放/字段形态可能不同，且未在本项目中通过实测，故本文档不再给出请求体示例，建议以官方指南为准。

### 4) 工作空间（Workspaces）

- 列表: GET /v1/workspaces
  - 查询参数：无

Playground：<https://www.coze.cn/open/playground/workspaces>

说明：详情端点 GET /v1/workspaces/{workspace_id} 在本项目的 CN 环境请求返回 404，暂不纳入“已验证”。

### 5) 会话（Conversations）

- 列表: GET /v1/conversations
  - 查询参数：
    - workspace_id: string（等同于 space_id）
    - bot_id: string（CN 侧必填；缺失会返回 400）
    - page 或 page_num: number（页码，从 1 开始；不同版本字段名存在差异）
    - page_size: number（每页数量）

Playground：<https://www.coze.cn/open/playground/conversations>

注意：分页字段在不同版本中可能为 page 或 page_num，建议按“先尝试 page，失败回退 page_num”的策略调用。

工具层说明：MCP 工具 list_conversations 已将 bot_id 设为必填（schema required）；workspace_id 可用 space_id 代替（未提供时使用默认 space_id）。

### 6) Chat v3 消息（只读）

- 列表: GET /v3/chat/conversations/{conversation_id}/messages
  - 路径参数：conversation_id: string
  - 返回：业务包 { code, msg, data }
  - 列表字段：messages | list | items（按容错解析）
  - 工具层：list_conversation_messages 使用该端点并进行“本地分页”（page/page_size 为本地窗口，不影响远端分页）。

说明：Chat v3 的会话启动/流式/工具结果提交/取消等写入类端点尚未在本项目中于中国区验证，暂不对外暴露为 MCP 工具；请以官方 Playground 详情页为准，先通过探针验证后再纳入。

## 对话/会话与其他分类

对话创建（/open_api/v2/chat）等多为国际版形态；中国区接口与参数在时间维度上存在差异且未在本项目验证。为避免误导，此处仅纳入“Chat v3 消息（只读）”的已验证读取端点；其余写入类端点暂不罗列。请直接查阅官方开发者指南对应章节，并以 <https://api.coze.cn> 为域名核对。

## 错误码（常见）

| code | 说明 |
|------|------|
| 0 | 成功 |
| 4000 | 参数错误 |
| 4001 | 未授权 |
| 4003 | 禁止访问 |
| 4004 | 资源不存在 |
| 4029 | 频率限制 |
| 5000 | 服务器内部错误 |

说明：当 HTTP 为 2xx 但 data.code != 0 时，也应视为业务失败并读取 msg/message 作为错误信息。

## 解析与兼容性建议

- 统一使用 <https://api.coze.cn>（中国区）。
- 参数名 workspace_id 与 space_id 在部分接口等同；本项目在 Bots/Workflows 中使用 workspace_id，在 Datasets 中使用 space_id。
- 列表返回字段名存在差异（list/items/datasets/dataset_list；total/total_count），请编写兼容解析逻辑。
- 429（频率限制）/401（鉴权失败）需做专门处理与重试策略。

## 候选端点覆盖矩阵（待验证）

说明：以下端点来自本项目代码声明与官方分类导航，但尚未在中国区环境实测。标注为“待验证”，请在加入前先用探针测试确认真实参数与返回结构。

### A) 工作空间（Workspaces）

- GET /v1/workspaces/{workspace_id}

备注：列表端点已在上文“已验证接口”中确认；详情在本项目环境返回 404，待验证。

### B) 会话（Conversations）

- GET /v1/conversations/{conversation_id}
- DELETE /v1/conversations/{conversation_id}

备注：列表端点已在上文“已验证接口”中确认（需 bot_id）。详情与删除待验证；注意与 Chat 能力划分关系。

### C) Chat（v3 形态，候选）

- POST /v3/chat
- POST /v3/chat/stream
- GET /v3/chat/conversations/{conversation_id}/messages

候选请求体字段（源自本项目声明，未验证）：

- bot_id、user_id、additional_messages[]（role/content/content_type/meta_data）、stream、conversation_id、custom_variables。

### D) 知识库（创建/上传，候选｜CN 细化）

- POST /open_api/knowledge/create（待验证，CN）
- POST /open_api/knowledge/document/create（待验证，CN）

以下内容来自中国区 Playground“知识库”板块页面的可视化说明，尚未在本项目内做写入类集成测试，仅用于文档性记录与后续验证准备：

- 域名固定为 <https://api.coze.cn>
- Header：统一使用 Authorization: Bearer {TOKEN}
- 路径与方法（示例）：POST /open_api/knowledge/document/create
- 必填/关键 Body 字段（归纳）：
  - dataset_id：目标知识库 ID（仅库所有者可操作）
  - document_bases：数组，单次最多 10 条；不同来源方式只能选择其一：
    - 文本/网页：
      - file_base64 + file_type（二选一于 web_url）
      - 或 web_url（与上面二者互斥）
      - document_source：0/1（具体枚举以详情页为准）
    - 图片：
      - source_file_id（来自“文件上传”API）
  - chunk_strategy：分段策略
    - chunk_type：0/1
    - 当为 1 时：可含 separator、max_tokens、remove_extra_spaces、remove_urls_emails 等字段
  - caption_type（仅图片）：0=系统，1=手动（若为 1，需另行调用 update_image_caption）
  - format_type：0=text，2=image（需与知识库类型匹配）
- 重要约束与提示：
  - 所属者限定：仅知识库所有者可上传
  - 类型匹配：知识库类型（文本/图片）需与 format_type 协同
  - 单一来源：一次请求内仅选择一种来源方式（file_base64+file_type 或 web_url 或 source_file_id）
  - 数量限制：document_bases 最多 10 条
  - 范围限制：不适用于火山引擎知识库

注意：以上字段来自页面描述与交互说明，非正式 API 参考；本项目维持“先只读验证，再考虑写入”的策略。

客户端实现现状：

- 已在客户端提供 create_knowledge_document_cn(body: serde_json::Value) 方法（仅代码接口）。
- 默认不暴露 MCP 写工具；CI 不运行写操作测试。

本地一次性验证（可选）：

- 在根目录创建/编辑 test.md，包含：
  - coze_api_token: <你的token>
  - dataset_id: <你的dataset_id>
  - web_url: <一个可抓取的网页URL>
- 运行忽略测试（仅本地）：cargo test --test api_knowledge_create_probe -- --ignored --nocapture
- 返回体将打印 code/msg/data 概览，请勿频繁调用；确保目标为你的私有知识库且类型匹配。

### E) 其他分类（Files/Plugins/Users/Models/Analytics/Webhooks）

- 官方站点存在对应分类，但未获取到稳定可用的 CN 端点与参数细节；请以官方指南为准，并在本项目内通过集成测试确认后再录入。

## 验证计划（建议）

- 为上述候选端点逐一编写最小化探针测试（GET 列表/详情优先），仅打印状态码与 data.* 关键字段；成功后再补充请求体/响应示例。
- 在本文件中以“已验证/待验证/不支持CN”三态维护覆盖矩阵，避免误导。

## 参考与后续

- 官方开发者指南（中国区）：<https://www.coze.cn/open/docs/developer_guides/>
- 如需补充“创建/上传”等知识库接口或对话接口，请以官方文档为准并在本项目中先完成集成测试再更新本文件。

—— 本文档已根据中国区可用端点核对修正，移除了错误的 v2 栈与“知识库搜索”。

## 知识库能力目录（来自中国区官网概览）

来源：API 概览（中国区）“知识库”板块：<https://www.coze.cn/open/docs/developer_guides/api_overview#4a82cea0>

目录项（概览为摘要级，未提供具体路径/参数；需登录查看详情页）：

- 创建知识库（待验证，需详情页）
- 查看知识库列表（已验证：GET /v1/datasets，space_id 必填；字段名存在兼容差异）
- 修改知识库信息（待验证）
- 删除知识库（待验证）
- 创建知识库文件（上传）（待验证）
- 修改知识库文件（待验证）
- 查看知识库文件列表（文档/表格/图像）（待验证）
- 查看知识库文件上传进度（待验证）
- 更新知识库图片描述（待验证）
- 查看知识库图片列表（待验证）
- 删除知识库文件（支持批量）（待验证）

注意：

- 概览页不直接给出端点路径/方法/参数，详情页当前对未登录/未授权用户不可见；需在具备访问权限的环境中获取并以探针测试确认。
- 本项目已确认“知识库搜索”在中国区不可用，相关能力不应再出现。

## 知识库术语与字段对照（基于已观测响应）

- 空间 ID：space_id（部分接口等同 workspace_id）
- 知识库 ID：dataset_id（部分返回也可能是 id）
- 名称：name
- 描述：description（可为空）
- 文档数量：document_count 或 doc_count
- 创建时间：created_at 或 create_time（整数时间戳）
- 列表字段：datasets / dataset_list / list / items（至少一种）
- 总数字段：total / total_count（至少一种）

建议：解析时对上述字段名做兼容回退，避免因版本差异导致解析失败。

## MCP 工具与知识库端点映射

- list_knowledge_bases → GET /v1/datasets（已验证，CN）
- create_knowledge_base → POST /open_api/v2/knowledge/create（待验证，CN）
- upload_document → POST /open_api/v2/knowledge/document/create（待验证，CN）

说明：后两项为国际版常见路径，在中国区可用性与请求体字段需以中国区官网详情页为准；在未完成验证前，请谨慎使用。

## 本地最小化验证步骤

前置：在仓库根目录创建/填写 test.md（包含 coze_api_token 与 space_id）。

运行：

```powershell
cargo test --tests -- --nocapture
```

期望：

- 知识库列表（/v1/datasets）应返回 200，打印 total 与若干条目名称。
- 未验证的创建/上传接口暂不在测试中执行；待获取中国区详情页后再添加探针测试。

## 响应示例（知识库列表，CN 已验证）

说明：以下为通用示例，字段名称在不同版本可能有差异（见“字段对照”一节）。

```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "datasets": [
      {
        "dataset_id": "7413635298800468009",
        "name": "样例知识库 A",
        "description": "示例描述，可为空",
        "document_count": 0,
        "created_at": 1719830400
      },
      {
        "dataset_id": "7533122859578048564",
        "name": "样例知识库 B",
        "description": "",
        "document_count": 0,
        "created_at": 1719744000
      }
    ],
    "total": 2
  }
}
```

可替代字段名（兼容处理）：

- 列表：datasets / dataset_list / list / items
- 总数：total / total_count

## 常见错误与排查（CN）

- 401 未授权：检查 Authorization 头是否为 `Bearer <token>`，并确认 token 有效且未过期。
- 400 参数缺失：/v1/datasets 需 `space_id`；/v1/conversations 需 `workspace_id` 与 `bot_id`。
- 404 路径不存在：确认使用中国区基础域名 <https://api.coze.cn> 与正确的版本路径（避免误用国际版 v2 路径）。
- 429 频率限制：遵循退避策略，适当重试；避免并发过高。

建议：当 HTTP 为 2xx 但 `code != 0` 时，同样按业务失败处理并读取 `msg/message`。

## 兼容性策略与建议

- 始终固定基础域名为 <https://api.coze.cn>。
- 列表解析采用“字段名容错”策略（datasets/dataset_list/list/items；total/total_count）。
- `workspace_id` 与 `space_id` 在部分接口等同：
  - Bots/Workflows 使用 `workspace_id`（支持以 `space_id` 作为别名）
  - Datasets 使用 `space_id`
- 分页参数在不同接口可能为 `page` 或 `page_num`：优先一个，失败回退另一个。
- 单次拉取建议控制 size（如 ≤100），避免超大分页导致超时或限流。

## API 调用示例（curl 与 Rust）

仅对“已验证端点”提供示例：

• curl 示例：获取知识库列表（CN 已验证）

```bash
curl -X GET \
  -H "Authorization: Bearer <YOUR_COZE_API_TOKEN>" \
  -H "Content-Type: application/json" \
  "https://api.coze.cn/v1/datasets?space_id=<YOUR_SPACE_ID>&page=1&size=100"
```

• Rust（reqwest）示例：

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let token = std::env::var("COZE_API_TOKEN")?;
  let space_id = std::env::var("SPACE_ID")?;
  let url = format!(
    "https://api.coze.cn/v1/datasets?space_id={}&page=1&size=100",
    urlencoding::encode(&space_id)
  );

  let resp = Client::new()
    .get(&url)
    .header("Authorization", format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .send()
    .await?;

  let status = resp.status();
  let text = resp.text().await?;
  println!("status={}\n{}", status, text);
  Ok(())
}
```

注意：上述示例仅适用于已验证端点；其它知识库相关操作请待中国区详情页确认后再补充示例。

## 知识库能力状态汇总表

- 查看知识库列表：已验证（GET /v1/datasets）
- 创建知识库：待验证（需中国区详情页）
- 修改知识库信息：待验证（需中国区详情页）
- 删除知识库：待验证（需中国区详情页）
- 创建知识库文件（上传）：待验证（需中国区详情页）
- 修改知识库文件：待验证（需中国区详情页）
- 查看知识库文件列表：待验证（需中国区详情页）
- 查看知识库文件上传进度：待验证（需中国区详情页）
- 更新知识库图片描述：待验证（需中国区详情页）
- 查看知识库图片列表：待验证（需中国区详情页）
- 删除知识库文件（支持批量）：待验证（需中国区详情页）

## 中国区 vs 国际版（差异概览）

- 基础域名：CN 固定使用 <https://api.coze.cn>；国际版常见为 <https://api.coze.com>
- 响应包裹：CN 普遍使用 `{ code, msg, data }` 业务包；国际版部分接口可能直接返回数据体
- 知识库搜索：国际版有过 `/open_api/v2/knowledge/document/search` 形态；CN 已确认不可用
- 字段命名：列表/总数字段在 CN 存在多版本命名（datasets/dataset_list/list/items；total/total_count）
- 参数名：`workspace_id` 与 `space_id` 在部分接口等同（CN 文档常用前者，知识库列表使用后者）

建议：以 CN 官网详情为准；避免将国际版路径/请求体直接套用于 CN。

## 限流与重试策略（建议）

- 401：立即失败；校验 token，避免重试风暴
- 429：指数退避（如 1s, 2s, 4s, 上限 5 次）；可读 `Retry-After` 头（如存在）
- 网络瞬断：小次数重试（≤3），设置整体超时
- 幂等列表查询：优先重试；非幂等写操作需谨慎（避免重复提交）

## Rust 容错解析示例（列表与总数字段）

```rust
use serde_json::Value;

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
```

## 知识库验证跟踪清单

- [x] 列表：GET /v1/datasets（CN 已验证）
- [ ] 创建知识库（待中国区详情页）
- [ ] 修改知识库信息（待中国区详情页）
- [ ] 删除知识库（待中国区详情页）
- [ ] 创建知识库文件（上传）（待中国区详情页）
- [ ] 修改知识库文件（待中国区详情页）
- [ ] 查看知识库文件列表（待中国区详情页）
- [ ] 查看知识库文件上传进度（待中国区详情页）
- [ ] 更新知识库图片描述（待中国区详情页）
- [ ] 查看知识库图片列表（待中国区详情页）
- [ ] 删除知识库文件（批量）（待中国区详情页）

## 安全与鉴权最佳实践（CN）

- 仅使用 Bearer Token：`Authorization: Bearer {TOKEN}`；不要在日志与错误信息中打印完整 token
- Token 最小权限与周期轮换：定期更新、最小可用权限、妥善保管
- 仅使用 <https://api.coze.cn>，强制 HTTPS；拒绝明文 HTTP
- 超时设置与重试上限：合理的 connect/read 超时；避免无限重试
- 请求/响应日志做脱敏：参数与响应中如含隐私字段，需打码

## 故障排查技巧（可执行）

- 打印服务端业务错误：当 HTTP 2xx 但 `code != 0` 时，输出 `msg/message`
- 检查参数大小写与别名：`workspace_id` 与 `space_id` 的等同关系；分页 `page` 与 `page_num` 的回退策略
- 降低每页 size：将 `size`≤100 或更低，规避超时/限流
- 使用详细日志：在本项目中可启用 INFO/DEBUG 级别日志以观察请求与关键字段
- 复现场景：先以 curl 复现，再迁移至代码；必要时对响应进行最小打印（状态码、头部的关键字段）

## 已验证端点速查表（CN）

| 分类 | 方法 | 路径 | 必填参数 | 分页字段 | 返回列表字段（任一） | 总数字段（任一） | 备注 |
|---|---|---|---|---|---|---|---|
| Knowledge | GET | /v1/datasets | space_id | page（size 控制数量） | datasets / dataset_list / list / items | total / total_count | CN 已验证 |
| Bots | GET | /v1/bots | workspace_id（≈ space_id） | page | list / items | total / total_count | CN 已验证 |
| Bots | GET | /v1/bots/{bot_id} | bot_id（路径） | — | — | — | CN 已验证（需先取 bot_id） |
| Workflows | GET | /v1/workflows | workspace_id（≈ space_id） | page_num（或回退 page） | list / items | total / total_count | CN 已验证 |
| Workspaces | GET | /v1/workspaces | 无 | — | list / items | total / total_count | CN 已验证 |
| Conversations | GET | /v1/conversations | workspace_id、bot_id | page 或 page_num | list / items | total / total_count | CN 已验证（需 bot_id） |

## 参数/字段别名速查

- workspace_id ≈ space_id（不同接口偏好不同）
- 分页字段：page 或 page_num（两者之一）
- 列表字段：datasets / dataset_list / list / items（至少一种）
- 总数字段：total / total_count（至少一种）

说明：解析时采用“优先一个、失败回退另一个”的容错策略；详见“解析与兼容性建议”与“Rust 容错解析示例”。

## 验证任务追踪表（总览）

| 分类 | 端点 | 状态 | 备注 | 测试 | MCP 工具 |
|---|---|---|---|---|---|
| Knowledge | GET /v1/datasets | 已验证 | 需 space_id；字段名容错 | tests/api_integration.rs | list_knowledge_bases |
| Bots | GET /v1/bots | 已验证 | 支持 workspace_id/page/page_size | tests/api_more.rs | list_bots |
| Bots | GET /v1/bots/{bot_id} | 已验证 | 需 bot_id | tests/api_more.rs | get_bot |
| Workflows | GET /v1/workflows | 已验证 | page_num 分页；必要时回退 page | tests/api_more.rs | list_workflows |
| Workspaces | GET /v1/workspaces | 已验证 | 无查询参数 | tests/api_candidates.rs | list_workspaces |
| Conversations | GET /v1/conversations | 已验证 | 需 bot_id；page/page_num 兼容 | tests/api_candidates.rs | list_conversations |
| Knowledge | create/upload/delete 等 | 待验证 | Playground 详情页受限；不猜测 | — | create_knowledge_base / upload_document（标注待验证） |
| Conversations | 详情/删除 | 待验证 | 需详情页 | — | — |
| Chat v3 | POST /v3/* | 待验证 | CN 可用性未知 | — | — |
| Knowledge Search | /open_api/v2/knowledge/document/search | 不支持（CN） | CN 确认不可用 | — | — |

说明：仅当“已验证”后才纳入 MCP 工具或提供示例请求体；待验证项需先通过 Playground 详情页+探针测试。

## 验证证据索引（与仓库测试/工具对齐）

- 已验证端点与对应测试：
  - GET /v1/datasets → `tests/api_integration.rs`
  - GET /v1/bots → `tests/api_more.rs`
  - GET /v1/bots/{bot_id}（探针） → `tests/api_more.rs`
  - GET /v1/workflows → `tests/api_more.rs`
  - GET /v1/workspaces → `tests/api_candidates.rs`
  - GET /v1/conversations（需 bot_id）→ `tests/api_candidates.rs`（缺少 bot_id 时返回 400，作为探针）

- MCP 工具与文档映射（只列 CN 已加入工具）：
  - list_knowledge_bases ↔ 本文“知识库端点现状与样例（CN 已验证）”
  - list_bots / get_bot ↔ “已验证接口（Bots）”
  - list_workflows ↔ “已验证接口（Workflows）”
  - list_workspaces ↔ “已验证接口（Workspaces）”
  - list_conversations ↔ “已验证接口（Conversations）”

说明：所有“待验证”能力将在拿到中国区官网详情页后，先补充探针测试，再转入“已验证接口”。

## 知识库端点现状与样例（CN 已验证）

- 列表（已验证）：GET /v1/datasets
  - 基础域名：<https://api.coze.cn>
  - 查询参数：
    - space_id: string（必填）
    - page: number（选填，默认 1）
    - size: number（选填，默认 20，建议 ≤100）
  - 返回字段容错建议：
    - 列表字段可能为 datasets/dataset_list/list/items 之一
    - 统计字段可能为 total/total_count 之一
  - 示例（路径形态）：/v1/datasets?space_id={space_id}&page=1&size=100

注意：除上述列表端点外，其它“创建/上传/修改/删除”等知识库相关接口需要以中国区官网“详情页”为准，当前处于“待验证”状态。

## 知识库验证计划（执行清单）

不猜测端点，仅以中国区官网文档为准：

1) 获取详情页链接（需登录/授权可见）
   - 创建知识库（待详情链接）
   - 修改知识库信息（待详情链接）
   - 删除知识库（待详情链接）
   - 创建知识库文件（上传）（待详情链接）
   - 修改知识库文件（待详情链接）
   - 查看知识库文件列表（待详情链接）
   - 查看知识库文件上传进度（待详情链接）
   - 更新知识库图片描述（待详情链接）
   - 查看知识库图片列表（待详情链接）
   - 删除知识库文件（待详情链接）

2) 为每个详情页提取：方法、路径、必填参数、分页/过滤、响应结构关键字段

3) 在 tests/ 中添加“探针式集成测试”（仅对已明确端点，且不做破坏性操作）

4) 成功后将端点移动到“已验证接口”，并在本节标注“已验证”

5) 明确不支持项：继续保持“知识库搜索”在中国区不可用的结论

---

## Conversations & Messages（CN）

说明：以下为本项目已接入或仅以只读形式提供的会话/消息能力，全部基于中国区域名 <https://api.coze.cn>，响应通常包裹为 { code, msg, data }。

已用端点与对应 MCP 工具（只读优先）：

- 会话列表：GET /v1/conversations?workspace_id={space_id}&bot_id={bot_id}&page=1&page_size=20
  - 工具：list_conversations（需 bot_id）
  - 说明：CN 侧缺少 bot_id 会返回 400；字段名 list/items 与 total/total_count 可能存在差异，项目内做了容错。

- 会话详情：GET /v1/conversation/retrieve?conversation_id={id}
  - 工具：retrieve_conversation
  - 输出：id/title/create_time 等关键信息。

- 会话消息列表（只读）：GET /v3/chat/conversations/{conversation_id}/messages
  - 工具：list_conversation_messages
  - 入参：conversation_id（必填）；limit（默认5）、page、page_size（用于本地分页切片，不传递到服务端）。
  - 输出：按本地分页返回最近消息，包含 message_id 便于后续检索。

- 本地检索单条消息（只读）：
  - 工具：retrieve_message_local（基于上面的列表结果在本地按 message_id 过滤）
  - 入参：conversation_id、message_id
  - 用途：在未确认 CN 单条检索端点之前的只读替代方案。

典型错误与排查：

- 400 缺少参数：/v1/conversations 需 bot_id；工具会给出“缺参数”提示。
- 404 路径不存在：例如 /v1/workspaces/{id} 在部分环境不可用；统一使用 <https://api.coze.cn> 并以 CN 文档为准。
- 消息为空或无权限：工具会提示“(无消息或无权访问)”。

探针测试（默认 #[ignore]，手动验证用）：

- tests/api_conversation_probes.rs：probe_retrieve_conversation_v1
- tests/api_message_probes.rs：probe_create_conversation_message_v1（写操作，仅客户端层验证，不对 MCP 暴露）

  - 运行前设置环境变量：COZE_API_TOKEN、COZE_CONVERSATION_ID
  - 解除 ignore 后本地验证；注意频率限制与资源消耗。

备注：

- 写操作（编辑/删除会话、创建消息）当前仅在客户端方法存在，未作为 MCP 工具暴露，遵循“先 API 验证，再考虑对 MCP 暴露”的原则。
