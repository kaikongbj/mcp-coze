# Coze MCP 服务器使用说明（CN-only）

本服务器固定对接中国区 Coze API（<https://api.coze.cn>），当前仅通过 stdio 传输作为 MCP Provider 使用。

## 启动方式

### 1) 命令行参数（推荐）

```powershell
# Windows PowerShell
./coze-mcp-server.exe -t YOUR_COZE_API_TOKEN -s YOUR_SPACE_ID

# 或使用完整参数名
./coze-mcp-server.exe --coze-api-token YOUR_COZE_API_TOKEN --space-id YOUR_SPACE_ID
```

```bash
# Linux / macOS
./coze-mcp-server -t YOUR_COZE_API_TOKEN -s YOUR_SPACE_ID
```

### 2) 环境变量

```powershell
# Windows PowerShell（当前会话）
$env:COZE_API_TOKEN="your_api_token_here"
$env:COZE_SPACE_ID="your_space_id_here"
./coze-mcp-server.exe
```

```bash
# Linux / macOS
export COZE_API_TOKEN=your_api_token_here
export COZE_SPACE_ID=your_space_id_here
./coze-mcp-server
```

### 3) 使用 Cargo 运行（开发模式）

```bash
# 设置环境变量后运行
cargo run -- --coze-api-token YOUR_COZE_API_TOKEN --space-id YOUR_SPACE_ID

# 或使用环境变量
export COZE_API_TOKEN=your_api_token_here
export COZE_SPACE_ID=your_space_id_here
cargo run
```

### 4) 使用 Cargo 运行（发布模式）

```bash
# 构建发布版本
cargo build --release

# 运行发布版本
./target/release/coze-mcp-server --coze-api-token YOUR_COZE_API_TOKEN --space-id YOUR_SPACE_ID
```

提示：若同时提供环境变量与命令行参数，命令行参数优先。

## 参数说明

| 参数 | 简写 | 环境变量 | 说明 | 必需 |
|------|------|----------|------|------|
| --coze-api-token | -t | COZE_API_TOKEN | Coze API访问令牌 (以 pat_ 开头) | 是 |
| --space-id | -s | COZE_SPACE_ID | 默认空间ID | 否 |
| --base-url | -b | COZE_BASE_URL | API基础URL（默认：https://api.coze.cn） | 否 |
| --log-level | -l | RUST_LOG | 日志级别（debug, info, warn, error） | 否 |

## 获取 API Token 与 Space ID（中国区）

- API Token：登录 <https://www.coze.cn> → 个人设置 → 开发者设置 → 生成个人令牌（pat_xxx）
- Space ID：登录 Coze 平台 → 空间管理页面 → 复制 Space ID

## Claude Desktop 配置示例

```json
{
  "mcpServers": {
    "coze": {
      "command": "D:\\mcp-coze\\target\\release\\coze-mcp-server.exe",
      "args": [
        "--coze-api-token",
        "pat_your_actual_token_here",
        "--space-id",
        "your_actual_space_id"
      ]
    }
  }
}
```

## 命令行快速测试

```powershell
# Windows PowerShell
./coze-mcp-server.exe --help
```

```bash
# Linux / macOS
./coze-mcp-server --help
```

## 支持的工具（43个完整列表）

### 配置管理 (3个)
- `set_api_key`: 设置API密钥
- `get_config_status`: 获取配置状态
- `test_connection`: 测试API连接

### 知识库管理 (4个)
- `list_knowledge_bases`: 列出知识库
- `create_knowledge_base`: 创建知识库
- `upload_document`: 上传文档到知识库
- `get_knowledge_base`: 获取知识库详情

#### 知识库文件上传示例

```bash
# 创建知识库
coze-mcp-server create_knowledge_base --name "技术文档库" --description "存储技术文档和API说明" --permission private

# 上传PDF文档
coze-mcp-server upload_document --dataset-id YOUR_DATASET_ID --file-path ./api-doc.pdf --file-name "API文档" --file-type pdf

# 上传Markdown文档
coze-mcp-server upload_document --dataset-id YOUR_DATASET_ID --file-path ./README.md --file-name "项目说明" --file-type md
```

支持文件类型：pdf, docx, xlsx, pptx, md, txt
单文件大小限制：100MB

### 工作空间管理 (4个)
- `list_workspaces`: 列出工作空间
- `list_workspace_ids`: 列出工作空间ID列表
- `get_workspace`: 获取工作空间详情
- `find_workspace_id_by_name`: 按名称查找工作空间ID

### Bot管理 (4个)
- `list_bots`: 列出Bots
- `get_bot`: 获取Bot详情
- `list_bot_ids`: 列出Bot ID列表
- `find_bot_id_by_name`: 按名称查找Bot ID

### 工作流管理 (4个)
- `list_workflows`: 列出工作流
- `get_workflow`: 获取工作流详情
- `list_workflow_ids`: 列出工作流ID列表
- `find_workflow_id_by_name`: 按名称查找工作流ID

### 会话管理 (11个)
- `list_conversations`: 列出会话
- `get_conversation`: 获取会话详情
- `list_conversation_ids`: 列出会话ID列表
- `count_conversations`: 统计会话数量
- `get_conversation_overview`: 获取会话概览
- `search_conversations_by_title`: 按标题搜索会话
- `get_conversation_duration`: 获取会话时长
- `get_conversation_participants`: 获取参与者
- `get_conversation_timeline`: 获取时间线
- `get_conversation_stats`: 获取会话统计

### 消息管理 (8个)
- `list_conversation_messages`: 列出会话消息
- `get_conversation_first_message`: 获取会话首条消息
- `get_conversation_last_message`: 获取会话最新消息
- `get_conversation_message_range`: 按范围获取消息
- `search_conversation_messages`: 搜索会话消息
- `get_message_by_index`: 按索引获取消息
- `count_conversation_messages`: 统计消息数量
- `get_message_index_by_id`: 按ID获取消息索引

### 数据导出 (7个)
- `export_conversation_markdown`: 导出为Markdown
- `export_conversation_json`: 导出为JSON
- `export_conversation_csv`: 导出为CSV
- `export_conversation_ndjson`: 导出为NDJSON
- `export_conversation_pairs`: 导出为问答对
- `export_conversation_html`: 导出为HTML
- `export_conversation_text`: 导出为纯文本

### 统计分析 (2个)
- `get_message_length_stats`: 获取消息长度统计
- `retrieve_message_local`: 按message_id本地检索

### 新增工具 structured_content 形状示例

- list_bot_ids → [{ bot_id, name }]
- list_workflow_ids → [{ workflow_id, name }]
- list_knowledge_base_ids → [{ dataset_id, name }]
- list_workspace_ids → [{ workspace_id, name }]
- get_workflow → { workflow_id, name, status, create_time }
- get_conversation_first_message → { message_id, index?, role, content, created_at? }
- get_conversation_message_range → { start, end, items: [ { index?, message_id, role, content, created_at? } ] }
- export_conversation_markdown → { conversation_id, title, total, markdown, direction }

## Space ID 的默认行为

当通过命令行或环境变量设置了默认 space_id 后：

- 需要 space_id/workspace_id 的工具可不再显式传参
- 工具调用显式传入的 space_id 优先级高于默认值

## 故障排除（Windows 常见）

1) 构建测试失败：failed to remove ... coze-mcp-server.exe (os error 5)

- 原因：Windows 正在运行的同名进程会锁定 target\debug 二进制，`cargo test --tests --no-run` 在覆盖时删除失败。
- 解决：先结束已运行的 `coze-mcp-server.exe` 再执行；或使用 `cargo check` 做快速验证。

1) 认证或权限错误

- 确认 `Authorization: Bearer pat_xxx` 令牌有效且有目标空间权限。

1) 连接问题

- 网络可达性检查；可用 `test_connection` 工具快速验证。
