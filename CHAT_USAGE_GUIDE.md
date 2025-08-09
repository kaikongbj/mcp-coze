# Coze MCP 聊天功能使用指南

## 概述

Coze MCP 服务器现已支持对话功能，包括非流式和流式聊天。这使得您可以通过 MCP 协议与 Coze Bot 进行交互对话。

**重要更新**: 我们已修复了 Chat API 格式问题，解决了之前遇到的 4000 错误代码。现在的实现完全符合 Coze API 规范。

## 修复说明

**问题**: 之前的实现在纯文本消息中错误地包含了 `content_type: "text"` 字段，导致 API 返回 4000 错误。

**解决方案**: 
- 纯文本消息现在直接使用 `content` 字段
- 去除了不必要的 `content_type` 字段
- 支持未来的多媒体消息格式（通过 `object_string` 字段）

## 新增功能

### 1. 非流式聊天 (`chat`)

发送消息给 Bot 并获得完整响应。

**参数:**
- `bot_id` (必填): Bot ID
- `message` (必填): 要发送的消息内容
- `user_id` (可选): 用户ID
- `conversation_id` (可选): 对话ID，不提供则创建新对话
- `custom_variables` (可选): 自定义变量对象

**示例:**
```json
{
  "bot_id": "your_bot_id",
  "message": "你好，请介绍一下你自己",
  "user_id": "user_123",
  "custom_variables": {
    "context": "formal",
    "language": "chinese"
  }
}
```

### 2. 流式聊天 (`chat_stream`)

发送消息给 Bot 并接收流式响应，适合长内容生成。

**参数:** 与 `chat` 相同

**特点:**
- 实时接收响应内容
- 支持增量内容更新
- 提供完整的事件流信息

## 工作流程

### 1. 基本对话流程

1. **设置 API Key**
   ```json
   {
     "tool": "set_api_key",
     "arguments": {
       "api_key": "pat_your_token_here"
     }
   }
   ```

2. **列出可用的 Bot**
   ```json
   {
     "tool": "list_bots",
     "arguments": {
       "workspace_id": "your_workspace_id"
     }
   }
   ```

3. **开始对话**
   ```json
   {
     "tool": "chat",
     "arguments": {
       "bot_id": "selected_bot_id",
       "message": "你好！"
     }
   }
   ```

4. **继续对话**
   ```json
   {
     "tool": "chat",
     "arguments": {
       "bot_id": "selected_bot_id",
       "message": "请详细解释一下",
       "conversation_id": "returned_conversation_id"
     }
   }
   ```

### 2. 流式对话流程

对于需要实时响应的场景：

```json
{
  "tool": "chat_stream",
  "arguments": {
    "bot_id": "selected_bot_id",
    "message": "请写一篇关于人工智能的长文章",
    "conversation_id": "existing_conversation_id"
  }
}
```

## 响应格式

### 非流式响应

```json
{
  "content": [
    {
      "type": "text",
      "text": "对话ID: conv_xxx\n消息ID: msg_xxx\n状态: completed\n"
    }
  ],
  "isError": false,
  "structured_content": {
    "conversation_id": "conv_xxx",
    "id": "msg_xxx",
    "created_at": 1234567890,
    "completed_at": 1234567891,
    "status": "completed",
    "usage": {
      "input_tokens": 10,
      "output_tokens": 50,
      "total_tokens": 60
    }
  }
}
```

### 流式响应

```json
{
  "content": [
    {
      "type": "text", 
      "text": "对话ID: conv_xxx\n消息ID: msg_xxx\n完整回复:\n这是 Bot 的完整回复内容...\n\n使用情况: {...}"
    }
  ],
  "isError": false,
  "structured_content": {
    "conversation_id": "conv_xxx",
    "message_id": "msg_xxx", 
    "content": "这是 Bot 的完整回复内容...",
    "usage": {
      "input_tokens": 10,
      "output_tokens": 100,
      "total_tokens": 110
    },
    "events": [
      {
        "event": "conversation_message_delta",
        "delta": {
          "content": "这是",
          "role": "assistant"
        }
      },
      {
        "event": "conversation_message_delta", 
        "delta": {
          "content": " Bot 的",
          "role": "assistant"
        }
      }
    ]
  }
}
```

## 高级用法

### 1. 使用自定义变量

自定义变量允许您向 Bot 传递上下文信息：

```json
{
  "tool": "chat",
  "arguments": {
    "bot_id": "your_bot_id",
    "message": "根据当前情况给出建议",
    "custom_variables": {
      "user_level": "expert",
      "domain": "technology",
      "urgency": "high"
    }
  }
}
```

### 2. 管理对话历史

- 不提供 `conversation_id`：创建新对话
- 提供 `conversation_id`：在现有对话中继续

### 3. 错误处理

当发生错误时，响应将包含错误信息：

```json
{
  "content": [
    {
      "type": "text",
      "text": "[Chat] 聊天失败: API returned error code 4001: Unauthorized"
    }
  ],
  "isError": true,
  "structured_content": {
    "error": "API returned error code 4001: Unauthorized"
  }
}
```

## 注意事项

1. **API 配额**: 聊天功能会消耗 Coze 的 API 配额，请合理使用
2. **速率限制**: 遵守 Coze API 的速率限制政策
3. **网络超时**: 长时间的流式对话可能会遇到网络超时
4. **权限控制**: 确保您的 API Token 有足够的权限访问指定的 Bot

## 与现有功能的集成

新的聊天功能与现有的 MCP 工具完美集成：

1. 使用 `list_bots` 发现可用的 Bot
2. 使用 `list_conversations` 查看历史对话
3. 使用 `chat` 或 `chat_stream` 进行交互
4. 使用 `list_knowledge_bases` 管理知识库

## 技术实现

### 支持的事件类型

流式聊天支持以下事件类型：
- `conversation_message_delta`: 增量内容更新
- `conversation_chat_completed`: 对话完成
- `conversation_chat_in_progress`: 对话进行中
- `conversation_chat_failed`: 对话失败
- `conversation_chat_requires_action`: 需要用户操作
- `done`: 流结束
- `error`: 发生错误

### API 端点

- 非流式: `POST /v3/chat`
- 流式: `POST /v3/chat` (带 `stream: true`)

这些功能扩展了 Coze MCP 服务器的能力，使其成为一个完整的 AI 对话和知识管理平台。
