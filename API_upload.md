# 扣子知识库文件上传API使用文档

## 一、接口概述
本API用于向指定的扣子知识库上传文件，支持文本知识库和图片知识库的文件上传操作。需注意，本接口仅适用于扣子知识库，火山知识库的文件上传请参考专门的接口文档。

### 核心功能
- 向文本知识库上传本地文件（Base64格式）或在线网页
- 向图片知识库上传图片文件（通过文件ID关联）
- 支持文件分段策略配置及图片标注方式设置

## 二、基础信息
| 项目 | 详情 |
|------|------|
| 请求方式 | POST |
| 请求地址 | https://api.coze.cn/open_api/knowledge/document/create |
| 权限要求 | 需开通`createDocument`权限，调用令牌需包含该权限 |
| 适用范围 | 仅扣子知识库（非火山知识库） |

## 三、权限准备
1. 登录扣子平台，生成个人访问令牌
2. 确保该令牌已开通`createDocument`权限
3. 权限配置详情参考[鉴权方式]文档

## 四、请求参数

### 4.1 Header参数
| 参数名 | 取值 | 说明 |
|--------|------|------|
| Authorization | Bearer $Access_Token | 客户端身份验证令牌，格式为"Bearer " + 访问令牌 |
| Content-Type | application/json | 固定值，指定请求正文格式 |
| Agw-Js-Conv | str | 防止数字类型参数精度丢失 |

### 4.2 Body参数
| 参数名 | 类型 | 是否必选 | 示例 | 说明 |
|--------|------|----------|------|------|
| dataset_id | String | 是 | 736356924530694**** | 扣子知识库ID，可从知识库页面URL的`knowledge`参数获取 |
| document_bases | Array of DocumentBase | 是 | 见示例 | 待上传文件元数据数组，最多包含10个元素 |
| chunk_strategy | Object of ChunkStrategy | 是 | 见示例 | 文件分段规则，每次请求必须传入 |
| format_type | Integer | 是 | 0或2 | 知识库类型（0：文本类型；2：图片类型） |

## 五、关键对象定义

### 5.1 DocumentBase对象
| 参数名 | 类型 | 是否必选 | 说明 |
|--------|------|----------|------|
| name | String | 是 | 文件名称（如"Coze.pdf"） |
| source_info | Object of SourceInfo | 是 | 文件元数据信息，根据上传方式不同填写不同参数 |
| update_rule | Object of UpdateRule | 否 | 仅上传在线网页时需设置，指定更新策略 |
| caption | String | 否 | 图片知识库专用，设置图片描述 |

### 5.2 SourceInfo对象（文件源信息）
根据上传方式选择以下参数组合：
- **本地文件（Base64）**：
  - file_base64（String，必选）：本地文件的Base64编码
  - file_type（String，必选）：文件格式（支持pdf、txt、doc、docx）
- **在线网页**：
  - web_url（String，必选）：网页URL地址
  - document_source（Integer，必选）：固定值1（表示网页来源）
- **图片文件（通过文件ID）**：
  - source_file_id（String，必选）：通过[上传文件]API获取的file_id
  - document_source（Integer，必选）：固定值5（表示图片来源）

### 5.3 ChunkStrategy对象（分段策略）
| 参数名 | 类型 | 是否必选 | 说明 |
|--------|------|----------|------|
| chunk_type | Integer | 否 | 分段方式（0：自动分段；1：自定义分段） |
| separator | String | 条件必选 | 分段标识符，chunk_type=1时必选 |
| max_tokens | Long | 条件必选 | 最大分段长度（100~2000），chunk_type=1时必选 |
| remove_extra_spaces | Boolean | 否 | 是否过滤连续空格/换行（默认false） |
| remove_urls_emails | Boolean | 否 | 是否过滤URL和邮箱（默认false） |
| caption_type | Integer | 否 | 图片标注方式（0：系统自动标注；1：手动标注） |

### 5.4 UpdateRule对象（网页更新策略）
| 参数名 | 类型 | 说明 |
|--------|------|------|
| update_type | Integer | 0（不自动更新，默认）；1（自动更新） |
| update_interval | Integer | 自动更新频率（小时），最小值24 |

## 六、上传方式说明
| 上传方式 | 适用知识库 | 关键参数 |
|----------|------------|----------|
| Base64本地文件 | 文本知识库 | file_base64 + file_type |
| 在线网页 | 文本知识库 | web_url + document_source=1 |
| 文件ID关联 | 图片知识库 | source_file_id + document_source=5 |

## 七、注意事项
1. 每次请求最多上传10个文件
2. 文件类型必须与知识库类型匹配（如文本文件不能上传到图片知识库）
3. 每个请求只能使用一种上传方式
4. 仅知识库所有者可执行上传操作
5. 图片知识库首次上传图片时，必须指定caption_type参数
6. 手动标注图片需后续调用[更新知识库图片描述]API

## 八、返回参数说明
| 参数名 | 类型 | 说明 |
|--------|------|------|
| code | Long | 状态码（0：成功；其他：失败） |
| msg | String | 状态信息，失败时显示错误详情 |
| document_infos | Array of DocumentInfo | 上传文件的详细信息数组 |
| detail | Object of ResponseDetail | 响应详情（包含logid） |

### DocumentInfo对象（文件信息）
包含文件ID、大小、字符数、上传时间、处理状态等信息，其中`status`字段表示处理状态：
- 0：处理中
- 1：处理完毕
- 9：处理失败（需重新上传）

## 九、调用示例

### 9.1 上传本地文件（文本知识库）
```shell
curl --location --request POST https://api.coze.cn/open_api/knowledge/document/create \
--header 'Authorization: Bearer your_access_token' \
--header 'Content-Type: application/json' \
--header 'Agw-Js-Conv: str' \
--data-raw '{
  "dataset_id": "736356924530694****",
  "document_bases": [
    {
      "name": "Coze.pdf",
      "source_info": {
        "file_base64": "5rWL6K+V5LiA5LiL5ZOm",
        "file_type": "pdf"
      }
    }
  ],
  "chunk_strategy": {
    "separator": "\n\n",
    "max_tokens": 800,
    "remove_extra_spaces": false,
    "remove_urls_emails": false,
    "chunk_type": 1
  },
  "format_type": 0
}'
```

### 9.2 返回示例
```json
{
  "code": 0,
  "document_infos": [
    {
      "char_count": 4,
      "chunk_strategy": {
        "chunk_type": 1,
        "max_tokens": 800,
        "remove_extra_spaces": false,
        "remove_urls_emails": false,
        "separator": "\n\n"
      },
      "create_time": 1719907964,
      "document_id": "738694205603010****",
      "format_type": 0,
      "hit_count": 0,
      "name": "Coze.pdf",
      "size": 14164,
      "slice_count": 1,
      "source_type": 0,
      "status": 0,
      "tos_uri": "FileBizType.BIZ_BOT_DATASET/xxx.docx",
      "type": "pdf",
      "update_interval": 0,
      "update_time": 1719907969,
      "update_type": 0
    }
  ],
  "msg": "",
  "detail": {
    "logid": "20250106172024B5F607030EFFAD653960"
  }
}
```

## 十、错误处理
- 调用失败时，`code`字段不为0，`msg`字段显示错误原因
- 常见错误场景：权限不足、文件类型不匹配、参数缺失等
- 如需技术支持，需提供`detail.logid`和错误码联系扣子团队

## 十一、相关接口
- [火山知识库上传文件]：火山知识库专用上传接口
- [更新知识库图片描述]：图片手动标注时使用
- [上传文件]：获取图片文件ID的前置接口
- [错误码]：完整错误码说明文档