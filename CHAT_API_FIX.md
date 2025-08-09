# Coze Chat API 格式修复说明

## 问题描述

在使用 Coze Chat API 时遇到了 4000 错误代码，具体错误信息为：
```
Bad request: API returned error code 4000: The field http body provided is not a valid json or chat request. Please check your input.
```

## 问题根因

根据 Coze API 官方文档，聊天消息的格式有特定要求：

1. **纯文本消息**应该直接使用 `content` 字段，而不是 `object_string` 数组
2. **`content_type` 字段只在使用 `object_string` 数组时才需要**
3. 对于纯文本消息，不应该包含 `content_type: "text"` 字段

## 修复内容

### 1. 更新 ChatMessage 结构体

**修复前**:
```rust
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub content_type: Option<ContentType>,
    pub meta_data: Option<MessageMetaData>,
}
```

**修复后**:
```rust
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: Option<String>,
    pub content_type: Option<ContentType>,
    pub object_string: Option<String>,
    pub meta_data: Option<MessageMetaData>,
}
```

### 2. 添加便利构造方法

```rust
impl ChatMessage {
    /// 创建纯文本消息
    pub fn text(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content: Some(content),
            content_type: None,
            object_string: None,
            meta_data: None,
        }
    }
    
    /// 创建包含文件/图片的消息  
    pub fn object_string(role: MessageRole, object_string: String) -> Self {
        Self {
            role,
            content: None,
            content_type: None,
            object_string: Some(object_string),
            meta_data: None,
        }
    }
}
```

### 3. JSON 格式对比

**修复前的错误格式**:
```json
{
  "additional_messages": [
    {
      "content": "请详细介绍kVPAC IDE的功能和特点",
      "content_type": "text",  // ← 这个字段导致4000错误
      "role": "user"
    }
  ],
  "auto_save_history": true,
  "bot_id": "7409830408747073570",
  "stream": false
}
```

**修复后的正确格式**:
```json
{
  "additional_messages": [
    {
      "content": "请详细介绍kVPAC IDE的功能和特点",
      "role": "user"
    }
  ],
  "auto_save_history": true,
  "bot_id": "7409830408747073570",
  "stream": false
}
```

## API 格式规则总结

根据 Coze API 文档：

1. **纯文本消息**：直接使用 `content` 字段，不使用 `content_type` 或 `object_string`
2. **多媒体消息**：使用 `object_string` 数组格式，包含 `type` 和 `file_id` 等字段
3. **混合消息**：一个 `object_string` 数组中最多包含一条 text 类型消息，但可以包含多个 file、image 类型的消息

## 验证结果

- ✅ 所有现有测试通过
- ✅ JSON 格式符合 Coze API 要求
- ✅ 去除了导致 4000 错误的 `content_type` 字段
- ✅ 支持未来扩展多媒体消息格式

## 向后兼容性

此修复保持了向后兼容性：
- 现有的 `ChatRequest::new()` 方法继续正常工作
- 测试用例已更新以验证新格式
- API 调用接口保持不变

## 使用示例

```rust
// 创建简单的文本聊天请求
let request = ChatRequest::new(
    "bot_id".to_string(),
    "你好，请介绍一下自己".to_string()
);

// 生成的 JSON 现在符合 Coze API 要求
let json = serde_json::to_value(&request)?;
```

这个修复解决了 Coze Chat API 调用失败的问题，现在可以正常进行聊天交互了。
