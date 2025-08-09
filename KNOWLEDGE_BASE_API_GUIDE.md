# Coze知识库API使用指南

## 概述

本文档详细介绍了如何使用Coze知识库API进行知识库的创建、管理和文件上传操作。所有API端点均基于中国区Coze平台（api.coze.cn）。

## 前置条件

- 已获取Coze API访问令牌（以`pat_`开头的个人访问令牌）
- 已创建Coze工作空间（Space）
- 了解工作空间ID（space_id）

## API端点总览

| 功能 | HTTP方法 | 路径 | 描述 |
|------|----------|------|------|
| 创建知识库 | POST | /open_api/v2/knowledge/create | 创建新的知识库 |
| 上传文档 | POST | /open_api/v2/knowledge/document/create | 上传文档到指定知识库 |
| 获取知识库列表 | GET | /open_api/v3/knowledge/datasets | 获取知识库列表 |
| 获取知识库详情 | GET | /open_api/v3/knowledge/datasets/{dataset_id} | 获取知识库详细信息 |

## 详细API说明

### 1. 创建知识库

**端点**: `POST https://api.coze.cn/open_api/v2/knowledge/create`

**请求头**:
```
Authorization: Bearer {your_api_token}
Content-Type: application/json
```

**请求体**:
```json
{
  "space_id": "your_space_id",
  "name": "知识库名称",
  "description": "知识库描述",
  "permission": "private"
}
```

**参数说明**:
- `space_id`: 工作空间ID（必需）
- `name`: 知识库名称（必需，最大长度100字符）
- `description`: 知识库描述（可选，最大长度500字符）
- `permission`: 权限类型（可选，private|public，默认private）

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "dataset_id": "dataset_123456",
    "name": "技术文档库",
    "description": "存储技术文档和API说明"
  }
}
```

### 2. 上传文档到知识库

**端点**: `POST https://api.coze.cn/open_api/v2/knowledge/document/create`

**请求头**:
```
Authorization: Bearer {your_api_token}
Content-Type: application/json
```

**请求体**:
```json
{
  "dataset_id": "your_dataset_id",
  "document_bases": [
    {
      "name": "文档名称",
      "source_info": {
        "file_type": "pdf",
        "file_url": "https://example.com/document.pdf"
      },
      "splitter_config": {
        "chunk_size": 800,
        "chunk_overlap": 100
      }
    }
  ],
  "update_type": "append"
}
```

**参数说明**:
- `dataset_id`: 知识库ID（必需）
- `document_bases`: 文档基础信息数组（必需）
  - `name`: 文档名称（必需）
  - `source_info`: 源信息（必需）
    - `file_type`: 文件类型（必需，支持：pdf, docx, xlsx, pptx, md, txt）
    - `file_url`: 文件URL或本地路径（必需）
    - `file_base64`: Base64编码文件内容（可选，与file_url二选一）
  - `splitter_config`: 分片配置（可选）
    - `chunk_size`: 分片大小（默认800字符）
    - `chunk_overlap`: 分片重叠（默认100字符）
- `update_type`: 更新类型（可选，append|overwrite，默认append）

**文件大小限制**:
- 单文件最大：100MB
- 支持批量上传：单次最多10个文件

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": [
    {
      "document_id": "doc_789012",
      "status": "processing"
    }
  ]
}
```

### 3. 获取知识库列表

**端点**: `GET https://api.coze.cn/open_api/v3/knowledge/datasets`

**请求头**:
```
Authorization: Bearer {your_api_token}
```

**查询参数**:
- `space_id`: 工作空间ID（可选，默认使用配置的space_id）
- `page_size`: 每页数量（可选，默认10，最大100）
- `page`: 页码（可选，默认1）

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "datasets": [
      {
        "dataset_id": "dataset_123456",
        "name": "技术文档库",
        "description": "存储技术文档和API说明",
        "document_count": 5,
        "file_list": [
          {"file_name": "api-doc.pdf", "file_size": 1024000},
          {"file_name": "README.md", "file_size": 204800}
        ]
      }
    ],
    "total": 1
  }
}
```

### 4. 获取知识库详情

**端点**: `GET https://api.coze.cn/open_api/v3/knowledge/datasets/{dataset_id}`

**请求头**:
```
Authorization: Bearer {your_api_token}
```

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "dataset_id": "dataset_123456",
    "name": "技术文档库",
    "description": "存储技术文档和API说明",
    "permission": "private",
    "create_time": 1700000000,
    "update_time": 1700000000,
    "document_count": 5,
    "file_list": [
      {
        "file_name": "api-doc.pdf",
        "file_size": 1024000,
        "upload_time": 1700000000,
        "status": "completed"
      }
    ]
  }
}
```

## 使用示例

### 完整工作流示例

#### 步骤1: 创建知识库
```bash
curl -X POST "https://api.coze.cn/open_api/v2/knowledge/create" \
  -H "Authorization: Bearer pat_your_token" \
  -H "Content-Type: application/json" \
  -d '{
    "space_id": "your_space_id",
    "name": "技术文档库",
    "description": "存储技术文档和API说明",
    "permission": "private"
  }'
```

#### 步骤2: 上传文档
```bash
curl -X POST "https://api.coze.cn/open_api/v2/knowledge/document/create" \
  -H "Authorization: Bearer pat_your_token" \
  -H "Content-Type: application/json" \
  -d '{
    "dataset_id": "dataset_123456",
    "document_bases": [
      {
        "name": "API文档",
        "source_info": {
          "file_type": "pdf",
          "file_url": "https://your-domain.com/api-guide.pdf"
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

#### 步骤3: 验证上传结果
```bash
curl -X GET "https://api.coze.cn/open_api/v3/knowledge/datasets/dataset_123456" \
  -H "Authorization: Bearer pat_your_token"
```

## 错误处理

### 常见错误码

| 错误码 | 描述 | 解决方案 |
|--------|------|----------|
| 1001 | 无效的token | 检查API令牌是否正确 |
| 1002 | 权限不足 | 确认令牌有访问该空间的权限 |
| 2001 | 参数错误 | 检查请求参数格式和内容 |
| 2002 | 文件过大 | 确保文件小于100MB |
| 2003 | 不支持的文件类型 | 使用支持的文件类型 |
| 3001 | 知识库不存在 | 确认dataset_id是否正确 |
| 3002 | 工作空间不存在 | 确认space_id是否正确 |

### 错误响应示例
```json
{
  "code": 1001,
  "msg": "Invalid token",
  "data": null
}
```

## 最佳实践

1. **文件准备**: 确保文件格式正确，内容清晰
2. **分批上传**: 大文件建议分批上传，避免超时
3. **错误重试**: 网络错误时实现指数退避重试
4. **监控状态**: 上传后定期检查处理状态
5. **权限管理**: 合理设置知识库权限，保护敏感信息

## 注意事项

- 中国区API域名固定为 `api.coze.cn`
- 文件上传后需要一定时间处理，请耐心等待
- 建议在上传前对文档进行预处理，确保内容质量
- 定期检查知识库使用情况，避免超出配额限制