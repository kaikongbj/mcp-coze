# Coze MCP Server - API 接口与工具参考

## 项目概述

Coze MCP Server 是一个专门为中国区Coze平台设计的MCP服务器，完整实现了25个Coze API接口和10个精选MCP工具，专注于知识库管理、会话导出、Bot管理等核心功能。

## 已实现API接口 (25个)

### 知识库管理 (8个)
- `GET /v1/knowledge_bases` - 列出知识库
- `POST /v1/knowledge_bases` - 创建知识库
- `GET /v1/knowledge_bases/{id}` - 获取知识库详情
- `PATCH /v1/knowledge_bases/{id}` - 更新知识库
- `DELETE /v1/knowledge_bases/{id}` - 删除知识库
- `POST /v1/knowledge_bases/{id}/documents` - 上传文档
- `GET /v1/knowledge_bases/{id}/documents` - 列出文档
- `DELETE /v1/knowledge_bases/{id}/documents/{doc_id}` - 删除文档

### 会话管理 (9个)
- `GET /v1/conversations` - 列出会话
- `POST /v1/conversations` - 创建会话
- `GET /v1/conversations/{id}` - 获取会话详情
- `PATCH /v1/conversations/{id}` - 更新会话
- `DELETE /v1/conversations/{id}` - 删除会话
- `GET /v1/conversations/{id}/messages` - 获取消息
- `POST /v3/chat` - 发送消息
- `GET /v1/conversations/search` - 搜索会话
- `GET /v1/conversations/{id}/stats` - 获取统计

### Bot管理 (3个)
- `GET /v1/bots` - 列出Bots
- `GET /v1/bots/{id}` - 获取Bot详情
- `PATCH /v1/bots/{id}` - 更新Bot

### 工作流管理 (3个)
- `GET /v1/workflows` - 列出工作流
- `GET /v1/workflows/{id}` - 获取工作流详情
- `POST /v1/workflows/{id}/run` - 运行工作流

### 工作空间管理 (2个)
- `GET /v1/workspaces` - 列出工作空间
- `GET /v1/workspaces/{id}` - 获取工作空间详情

## MCP工具清单 (10个精选工具)

### 1. 配置管理
- `set_api_key` - 设置 Coze API 密钥

### 2. 工作空间管理  
- `list_workspaces` - 列出所有工作空间

### 3. Bot 管理
- `list_bots` - 列出 Bots

### 4. 知识库管理
- `list_knowledge_bases` - 列出所有知识库
- `create_knowledge_base_v2` - 创建知识库
- `upload_document_to_knowledge_base` - 上传本地文档到知识库

### 5. 对话管理
- `list_conversations` - 列出对话
- `list_conversation_messages` - 查看会话消息列表

### 6. 数据导出
- `export_conversation_markdown` - 导出会话为 Markdown 格式

### 7. 系统工具
- `test_connection` - 测试 Coze API 连接

## 快速开始

### 1. 获取API Token
登录 [Coze平台](https://www.coze.cn) → 个人设置 → 开发者选项 → 创建Personal Access Token

### 2. 配置Claude Desktop
```json
{
  "mcpServers": {
    "coze": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/to/coze-mcp-server/Cargo.toml"],
      "env": {
        "COZE_API_TOKEN": "pat_xxx"
      }
    }
  }
}
```

### 3. 常用操作示例

#### 列出工作空间
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_workspaces",
    "arguments": {}
  }
}
```

#### 列出知识库
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_knowledge_bases",
    "arguments": {
      "space_id": "your_space_id"
    }
  }
}
```

#### 导出会话为Markdown
```json
{
  "method": "tools/call",
  "params": {
    "name": "export_conversation_markdown",
    "arguments": {
      "conversation_id": "conv_xxx"
    }
  }
}
```

## 技术特性

- **中国区专精**: 针对coze.cn平台优化
- **只读优先**: 以数据导出和分析为主
- **完整实现**: 25个API接口全部可用
- **精选工具**: 10个核心MCP工具覆盖主要场景
- **多格式导出**: 支持7种数据导出格式
- **实时同步**: 与Coze平台数据实时同步

## 注意事项

1. **API限制**: 默认100次/分钟频率限制
2. **权限要求**: 确保Token具有对应接口权限
3. **中国区**: 部分功能与国际版不同
4. **分页支持**: 列表接口支持分页查询
5. **错误处理**: 完善的错误码和提示信息