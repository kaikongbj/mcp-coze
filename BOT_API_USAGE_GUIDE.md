## Bot API 使用示例

基于您提供的 API 文档，我已经为项目实现了完整的智能体列表 API 支持。以下是主要改进和使用示例：

### 🔧 主要改进

1. **新增类型化模型** (`src/api/bot_models.rs`)
   - `BotInfo`: 智能体信息结构
   - `ListBotsRequest`: 查询请求参数
   - `ListBotsResponse`: API 响应结构
   - `BotPublishStatus`: 发布状态枚举

2. **改进的 API 客户端** (`src/api/client.rs`)
   - 新增 `list_bots_typed()` 方法，使用类型化模型
   - 完整的错误处理和响应解析

3. **增强的工具功能** (`src/tools/coze_tools.rs`)
   - 更新 `list_bots` 工具，支持所有 API 参数
   - 更好的结构化输出和错误处理

### 📋 支持的参数

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `workspace_id` | String | 工作空间 ID (必填) | - |
| `publish_status` | String | 发布状态筛选 | `published_online` |
| `connector_id` | String | 渠道 ID | `1024` |
| `page` | Number | 页码 | `1` |
| `page_size` | Number | 每页数量 | `20` |

#### 发布状态选项：
- `all`: 全部状态
- `published_online`: 已发布正式版
- `published_draft`: 已发布草稿  
- `unpublished_draft`: 未发布

### 🚀 使用示例

#### 1. 基本查询（获取已发布的智能体）
```json
{
  "workspace_id": "5123945629***"
}
```

#### 2. 获取所有状态的智能体
```json
{
  "workspace_id": "5123945629***",
  "publish_status": "all"
}
```

#### 3. 分页查询
```json
{
  "workspace_id": "5123945629***",
  "page": 2,
  "page_size": 10
}
```

#### 4. 完整参数查询
```json
{
  "workspace_id": "5123945629***",
  "publish_status": "published_draft",
  "connector_id": "1024",
  "page": 1,
  "page_size": 50
}
```

### 📤 输出格式

工具会返回：
- **文本格式**: 可读的智能体列表摘要
- **结构化数据**: 包含完整智能体信息的 JSON

#### 输出示例：
```
找到 1 个 Bot:

1. 语音伴侣 (id: 7493066380997****, status: draft)
```

#### 结构化数据示例：
```json
{
  "total": 1,
  "items": [
    {
      "bot_id": "7493066380997****",
      "name": "语音伴侣",
      "status": "draft",
      "description": "语音伴侣",
      "icon_url": "https://example.com/agent1***.png",
      "updated_at": 1718289297,
      "owner_user_id": "23423423****"
    }
  ],
  "page_num": 1,
  "page_size": 20
}
```

### 🔒 权限要求

确保您的访问令牌具有 `listBot` 权限，否则 API 调用会失败。

### ✅ 测试验证

已通过 10 个综合测试验证：
- 序列化/反序列化测试
- 参数构建和验证
- 查询字符串生成
- 响应解析测试

所有功能均按照官方 API 文档规范实现，确保与 Coze API 完全兼容。
