# Coze MCP Server

一个基于 MCP（Model Context Protocol）的 Coze 中国区 API 适配服务器（CN-only，固定 <https://api.coze.cn>）。当前仅暴露只读类工具，写入类接口先通过集成探针验证后再考虑开放。

## 功能特性（与当前实现一致）

- 中国区专用：固定使用 <https://api.coze.cn>，内置中国区响应包裹 { code, msg, data } 的兼容解析
- 只读优先：对外仅开放“列表/详情/检索”等只读工具，避免误用未验证写入接口
- 结构化输出：所有工具同时返回文本与 structured_content，便于上层消费
- 传输方式：当前实现 stdio（适配 Claude Desktop 等 MCP 客户端）

## 安装

### 从源码编译

```bash
git clone <你的仓库地址>
cd coze-mcp
cargo build --release
```

注：不再提供 `cargo install` 指南；请直接在本仓库构建二进制。

## 使用方法

### 环境变量（建议）

仅需提供 Token 与可选的默认空间 ID：

- Windows PowerShell
  - `$env:COZE_API_TOKEN="pat_xxx"`
  - `$env:COZE_SPACE_ID="your_space_id"`（可选）
- Linux/macOS
  - `export COZE_API_TOKEN=pat_xxx`
  - `export COZE_SPACE_ID=your_space_id`（可选）

基础 URL 不可配置，强制使用中国区 <https://api.coze.cn>。

### 启动（stdio，推荐）

```bash
./target/release/coze-mcp-server -t pat_xxx -s your_space_id
```

或仅设环境变量后直接启动可执行文件。

## 可用工具（只读 + 配置类）

当前已实现 **46个MCP工具**，覆盖以下功能模块：

### 配置管理 (3个工具)
- `set_api_key`: 设置API密钥
- `get_config_status`: 获取配置状态
- `test_connection`: 测试API连接

### 知识库管理 (7个工具)
- `list_knowledge_bases`: 按 space_id 列表
- `get_knowledge_base`: 按 dataset_id 详情
- `list_knowledge_base_ids`: 仅返回 {dataset_id, name}
- `upload_document_to_knowledge_base`: 上传本地文档（支持PDF、DOCX等，最大10MB）
- `upload_document_from_url`: 从URL上传文档（支持公开URL，最大100MB）
- `create_knowledge_base_v2`: 创建知识库（v2 API，支持权限设置）
- `find_dataset_id_by_name`: 通过名称查找知识库ID
- `create_knowledge_base`: 创建知识库
- `upload_document`: 上传文档到知识库

**知识库文件上传功能**：支持PDF、DOCX、XLSX、PPTX、MD、TXT格式，单文件最大100MB。详见[知识库API指南](./KNOWLEDGE_BASE_API_GUIDE.md)。

### 工作空间管理 (3个工具)
- `list_workspaces`: 列出工作空间
- `list_workspace_ids`: 仅返回 {workspace_id, name}

### Bot管理 (4个工具)
- `list_bots`: 列出Bots（workspace_id≈space_id）
- `get_bot`: 获取Bot详情
- `list_bot_ids`: 仅返回 {bot_id, name}

### 工作流管理 (4个工具)
- `list_workflows`: 列出工作流（workspace_id≈space_id）
- `get_workflow`: 工作流详情
- `list_workflow_ids`: 仅返回 {workflow_id, name}

### 会话管理 (16个工具)
- `list_conversations`: 需 bot_id；workspace_id≈space_id
- `list_conversation_ids`: 需 bot_id
- `count_conversations`: 需 bot_id
- `retrieve_conversation`: 获取会话详情
- `list_conversation_messages`: 本地分页
- `retrieve_message_local`: 按 message_id 本地检索
- `get_conversation_first_message`: 获取最早一条消息
- `get_conversation_last_message`: 获取最新一条消息
- `get_conversation_message_range`: 按区间索引获取消息，支持负索引
- `export_conversation_markdown`: 导出会话为 Markdown
- `get_conversation_overview`: 详情+最新消息
- `search_conversations_by_title`: 本地筛选
- `search_conversation_messages`: 本地检索 content
- `count_conversation_messages`
- `get_message_by_index`: 0=最旧，-1=最新

### 数据导出 (7个工具)
- `export_conversation_markdown`: 导出为Markdown
- `export_conversation_json`: 导出为JSON
- `export_conversation_csv`: 导出为CSV
- `export_conversation_ndjson`: 导出为NDJSON
- `export_conversation_pairs`: 导出为问答对
- `export_conversation_html`: 导出为HTML
- `export_conversation_text`: 导出为纯文本

### 统计分析 (8个工具)
- `get_conversation_duration`: 获取会话时长
- `get_conversation_participants`: 获取参与者
- `get_conversation_timeline`: 获取时间线
- `get_conversation_stats`: 获取会话统计
- `get_message_length_stats`: 获取消息长度统计

提示：
- list_conversations 在中国区必须提供 bot_id（schema 已强制）。
- 工具返回同时包含 human_text 与 structured_content，字段更稳定，建议优先消费 structured_content。

## 开发

### 项目结构

```text
coze-mcp/
├── src/
│   ├── main.rs              # 主程序入口和MCP服务器注册
│   ├── lib.rs               # 库的根模块声明
│   ├── api/
│   │   ├── mod.rs          # API模块定义
│   │   ├── client.rs       # Coze API客户端实现
│   │   └── endpoints.rs    # API端点定义
│   ├── tools/
│   │   ├── mod.rs          # 工具模块声明
│   │   ├── coze_tools.rs   # Coze平台工具实现
│   │   ├── config_tool.rs  # 配置相关工具
│   │   └── context.rs      # 上下文管理
│   ├── models/
│   │   ├── mod.rs          # 数据模型模块
│   │   └── responses.rs    # API响应模型
│   ├── handlers/
│   │   └── mod.rs          # 请求处理器
│   ├── utils/              # 工具函数
│   └── knowledge.rs        # 知识库管理器
├── Cargo.toml
└── README.md
```

### 测试与常见问题（Windows）

- 类型检查：`cargo check`
- 运行集成测试（不执行二进制）：VS Code 任务“cargo-test-no-run”或 `cargo test --tests --no-run`
- Windows 提示 “failed to remove … coze-mcp-server.exe (os error 5) 拒绝访问”：
  - 原因：测试构建会尝试覆盖 target\debug 下的二进制，若同名进程正在运行（例如你已在 Claude/终端里启动了服务器），Windows 会锁定该文件，导致删除失败。
  - 处理：先停止已运行的 `coze-mcp-server.exe` 后再执行测试构建；或改用 `cargo check` 做快速验证。

### 代码格式检查

```bash
cargo fmt --check
cargo clippy
```

## 贡献

欢迎提交 Issue 和 Pull Request。新增写入类能力前，请先：

- 从中国区 Playground 详情页获取真实端点/参数
- 在 tests/ 目录增加探针测试验证
- 通过后再注册 MCP 工具（默认只读优先）

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 相关文档

- [API参考](./API_REFERENCE.md) - 25个已实现API接口和10个精选MCP工具清单
- [Coze API文档](./COZE_API_DOCUMENTATION.md) - 中国区API接口详细说明
- [使用指南](./USAGE.md) - 安装配置和工具使用说明
- [知识库实现](./KNOWLEDGE_BASE_IMPLEMENTATION.md) - 知识库功能架构设计
- [开发计划](./DEVELOPMENT_PLAN.md) - 详细的开发路线图
- [RMCP使用](./RMCP_USAGE_DOCUMENTATION.md) - RMCP服务器实现细节

## 相关链接

- 中国区 Coze 开放平台：<https://www.coze.cn/open/docs>
- MCP 规范：<https://github.com/modelcontextprotocol/spec>
- MCP 官方文档：<https://modelcontextprotocol.io>
