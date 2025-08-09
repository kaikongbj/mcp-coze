# Coze MCP Server

一个基于 MCP（Model Context Protocol）的 Coze 中国区 API 适配服务器（CN-only，固定 <https://api.coze.cn>）。已精简为最小可用集合，现包含 9 个核心工具，**新增对话功能支持流式和非流式聊天**。

## 功能特性（增强版）

- 中国区专用：固定使用 <https://api.coze.cn>，内置中国区响应包裹 { code, msg, data } 的兼容解析
- **对话功能**：支持非流式和流式聊天，实时与 Coze Bot 交互
- 最小接口面：仅暴露少量稳定、高频、清晰的操作，降低维护成本
- 结构化输出：所有工具返回文本 + structured_content
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

## 可用工具（当前 9 个）

| 工具 | 说明 |
|------|------|
| set_api_key | 设置 / 更新 API Key |
| list_workspaces | 列出工作空间 |
| list_bots | 列出 Bots（支持分页） |
| list_knowledge_bases | 列出知识库（支持名称过滤、分页、文档数量精确刷新） |
| create_knowledge_base_v2 | 创建知识库（支持 permission=private/public） |
| upload_document_to_knowledge_base | 上传本地文件到知识库（<=10MB 示例限制） |
| list_conversations | 按 bot_id 列出会话 |
| **chat** | **发送聊天消息（非流式）** |
| **chat_stream** | **发送流式聊天消息** |

**新增聊天功能**：与 Coze Bot 实时对话，支持流式响应、对话历史管理、自定义变量等高级功能。

已移除的大量导出/统计/检索类工具，若后续需要再按需恢复。

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
│   └── (knowledge.rs 已移除)
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

- [API参考](./API_REFERENCE.md) - 25个已实现API接口和精选MCP工具清单
- [Coze API文档](./COZE_API_DOCUMENTATION.md) - 中国区API接口详细说明
- [使用指南](./USAGE.md) - 安装配置和工具使用说明
- **[聊天功能指南](./CHAT_USAGE_GUIDE.md) - 对话功能详细使用说明和示例**
- (历史) 知识库实现、开发计划等文档与大批已移除工具对应内容已过期，将在需要时重新整理
- [RMCP使用](./RMCP_USAGE_DOCUMENTATION.md) - RMCP服务器实现细节

## 相关链接

- 中国区 Coze 开放平台：<https://www.coze.cn/open/docs>
- MCP 规范：<https://github.com/modelcontextprotocol/spec>
- MCP 官方文档：<https://modelcontextprotocol.io>
