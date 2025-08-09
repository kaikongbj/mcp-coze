# 扣子发起对话 API 调用文档（简化版）

## 基础信息
- **请求方式**：POST
- **请求地址**：https://api.coze.cn/v3/chat
- **权限要求**：需开通 `chat` 权限，使用个人令牌鉴权

## 请求头（Header）
| 参数 | 取值 | 说明 |
|------|------|------|
| Authorization | Bearer $Access_Token | 访问令牌，用于身份验证 |
| Content-Type | application/json | 请求正文格式 |

## 查询参数（Query）
| 参数 | 类型 | 是否必选 | 说明 |
|------|------|----------|------|
| conversation_id | String | 可选 | 会话 ID，标识对话所属会话（不填则自动创建） |

## 请求体（Body）
| 参数 | 类型 | 是否必选 | 说明 |
|------|------|----------|------|
| bot_id | String | 必选 | 智能体 ID（从开发页面 URL 中获取） |
| user_id | String | 必选 | 用户唯一标识（自定义，用于隔离不同用户数据） |
| additional_messages | Array of object | 可选 | 上下文消息（最后一条为本次用户输入） |
| stream | Boolean | 可选 | 是否流式响应（默认 false） |
| auto_save_history | Boolean | 可选 | 是否保存对话记录（默认 true，非流式响应必须为 true） |

## 响应说明
### 流式响应（stream=true）
- 以数据流形式返回增量消息，包含中间过程（如 function_call、tool_response）
- 最终通过 `conversation.message.completed` 事件返回完整回复
- 结束标志为 `event: done`

### 非流式响应（stream=false）
- 立即返回对话元数据（chat_id、状态等）
- 需通过「查看对话详情」接口轮询状态，完成后调用「查看对话消息详情」获取完整回复

## 简单示例（流式响应）
```shell
curl --location --request POST 'https://api.coze.cn/v3/chat' \
--header 'Authorization: Bearer pat_your_token' \
--header 'Content-Type: application/json' \
--data-raw '{
  "bot_id": "your_bot_id",
  "user_id": "user123",
  "stream": true,
  "additional_messages": [
    {
      "role": "user",
      "content": "今天天气如何？",
      "content_type": "text"
    }
  ]
}'
```