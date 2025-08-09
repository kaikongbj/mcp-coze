# Coze MCP Server - 精选工具指南

## 概述

为了提高性能和简化使用，Coze MCP Server 现在只提供 **10个最常用的核心工具**。这些工具覆盖了日常使用中最重要的功能。

## 🛠️ 精选工具列表

### 1. 配置管理

#### `set_api_key`
- **功能**: 设置 Coze API 密钥
- **参数**: 
  - `api_key` (必需): Coze 个人访问令牌 (以 pat_ 开头)
- **用途**: 初始化和更新 API 认证

### 2. 工作空间管理

#### `list_workspaces`
- **功能**: 列出所有工作空间
- **参数**: 无
- **用途**: 查看可用的工作空间，获取 workspace_id

### 3. Bot 管理

#### `list_bots`
- **功能**: 列出 Bots
- **参数**:
  - `workspace_id` (可选): 工作区ID，未提供时使用默认值
  - `page` (可选): 页码，默认1
  - `page_size` (可选): 每页数量，默认20
- **用途**: 查看可用的 Bot，获取 bot_id

### 4. 知识库管理

#### `list_knowledge_bases`
- **功能**: 列出所有知识库
- **参数**:
  - `space_id` (可选): 空间ID，未提供时使用默认值
  - `name` (可选): 按名称模糊搜索
  - `page_num` (可选): 页码，默认1
  - `page_size` (可选): 每页数量
- **用途**: 查看现有知识库，获取 dataset_id

#### `create_knowledge_base_v2`
- **功能**: 创建新的知识库
- **参数**:
  - `name` (必需): 知识库名称
  - `description` (可选): 知识库描述
  - `space_id` (可选): 空间ID，未提供时使用默认值
  - `permission` (可选): 权限类型 (private/public)，默认 private
- **用途**: 创建新的知识库用于存储文档

#### `upload_document_to_knowledge_base`
- **功能**: 上传本地文档到知识库
- **参数**:
  - `dataset_id` (必需): 知识库ID
  - `file_path` (必需): 本地文件路径
  - `document_name` (可选): 文档名称
  - `chunk_size` (可选): 分片大小，默认800
  - `chunk_overlap` (可选): 分片重叠，默认100
- **用途**: 将本地文档上传到指定知识库
- **支持格式**: PDF, DOCX, XLSX, PPTX, MD, TXT
- **文件大小限制**: 最大 10MB

### 5. 对话管理

#### `list_conversations`
- **功能**: 列出对话
- **参数**:
  - `bot_id` (必需): Bot ID
  - `workspace_id` (可选): 工作区ID，未提供时使用默认值
  - `page` (可选): 页码，默认1
  - `page_size` (可选): 每页数量，默认20
- **用途**: 查看指定 Bot 的对话列表

#### `list_conversation_messages`
- **功能**: 查看会话消息列表
- **参数**:
  - `conversation_id` (必需): 对话ID
  - `limit` (可选): 返回条数，默认10
- **用途**: 查看指定对话的消息内容

### 6. 数据导出

#### `export_conversation_markdown`
- **功能**: 导出会话为 Markdown 格式
- **参数**:
  - `conversation_id` (必需): 对话ID
  - `limit` (可选): 限制导出条数
- **用途**: 将对话内容导出为易读的 Markdown 格式

### 7. 系统工具

#### `test_connection`
- **功能**: 测试 Coze API 连接
- **参数**: 无
- **用途**: 验证 API 密钥和网络连接是否正常

## 🚀 使用流程

### 基本设置流程
1. **设置 API 密钥**: 使用 `set_api_key` 配置认证
2. **测试连接**: 使用 `test_connection` 验证配置
3. **查看工作空间**: 使用 `list_workspaces` 获取可用空间

### 知识库操作流程
1. **查看现有知识库**: 使用 `list_knowledge_bases`
2. **创建新知识库**: 使用 `create_knowledge_base_v2`
3. **上传文档**: 使用 `upload_document_to_knowledge_base`

### 对话分析流程
1. **查看 Bots**: 使用 `list_bots` 获取 bot_id
2. **查看对话**: 使用 `list_conversations` 获取对话列表
3. **查看消息**: 使用 `list_conversation_messages` 查看具体内容
4. **导出数据**: 使用 `export_conversation_markdown` 导出分析结果

## 📝 使用示例

### 设置 API 密钥
```json
{
  "tool": "set_api_key",
  "arguments": {
    "api_key": "pat_your_api_token_here"
  }
}
```

### 创建知识库
```json
{
  "tool": "create_knowledge_base_v2",
  "arguments": {
    "name": "技术文档库",
    "description": "存储技术相关文档",
    "permission": "private"
  }
}
```

### 上传文档
```json
{
  "tool": "upload_document_to_knowledge_base",
  "arguments": {
    "dataset_id": "dataset_123456",
    "file_path": "./docs/api-guide.pdf",
    "document_name": "API使用指南"
  }
}
```

### 导出对话
```json
{
  "tool": "export_conversation_markdown",
  "arguments": {
    "conversation_id": "conv_123456",
    "limit": 50
  }
}
```

## ⚡ 性能优化

通过精简到10个核心工具，我们实现了：

- **更快的启动速度**: 减少了工具注册和初始化时间
- **更低的内存占用**: 只加载必要的功能模块
- **更简单的使用**: 专注于最常用的功能
- **更好的维护性**: 减少了代码复杂度

## 🔧 配置要求

### 环境变量
- `COZE_API_TOKEN`: Coze API 令牌 (必需)
- `COZE_SPACE_ID`: 默认空间ID (可选)

### 命令行参数
- `-t, --coze-api-token <TOKEN>`: API 令牌
- `-s, --space-id <SPACE_ID>`: 默认空间ID
- `-h, --help`: 显示帮助信息

## 📋 支持的文件格式

| 格式 | 扩展名 | 最大大小 | 说明 |
|------|--------|----------|------|
| PDF | .pdf | 10MB | 便携式文档格式 |
| Word | .docx, .doc | 10MB | Microsoft Word 文档 |
| Excel | .xlsx, .xls | 10MB | Microsoft Excel 表格 |
| PowerPoint | .pptx, .ppt | 10MB | Microsoft PowerPoint 演示文稿 |
| Markdown | .md | 10MB | Markdown 格式文档 |
| 纯文本 | .txt | 10MB | 纯文本文件 |

## 🛡️ 安全注意事项

- API 密钥会被安全存储，不会记录到日志中
- 文件上传前会进行格式和大小验证
- 所有 API 通信使用 HTTPS 加密
- 支持的文件类型经过白名单验证

## 📞 故障排除

### 常见问题

1. **API 密钥无效**
   - 确保使用正确的 `pat_` 开头的令牌
   - 使用 `test_connection` 验证连接

2. **文件上传失败**
   - 检查文件大小是否超过 10MB 限制
   - 确认文件格式是否支持
   - 验证文件路径是否正确

3. **找不到工作空间或 Bot**
   - 使用 `list_workspaces` 和 `list_bots` 确认可用资源
   - 检查 API 令牌是否有相应权限

### 获取帮助

如果遇到问题，请：
1. 使用 `test_connection` 检查基本连接
2. 查看错误消息中的具体提示
3. 确认所有必需参数都已提供

---

**版本**: 0.1.0 (精简版)  
**更新日期**: 2025-08-09  
**工具数量**: 10个核心工具