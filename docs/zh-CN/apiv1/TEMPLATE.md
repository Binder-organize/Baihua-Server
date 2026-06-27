# API 文档模板

本文定义了 API 端点文档的统一格式。新增 API 文档**必须**遵循此模板，以保证所有端点文档风格一致。

---

## 1. 文件命名

| 端点 | 文件名 |
|---|---|
| 单一功能组（如 user、chat） | `<名称>.md` — 例如 `user.md`、`chat.md` |
| 独立端点（如 greet、health） | `<名称>.md` — 例如 `greet.md`、`health.md` |

将文件放入 `docs/zh-CN/apiv1/` 目录，并在 `docs/en/apiv1/` 中创建对应的英文版本。

---

## 2. 文件结构

```
# 标题（H1）—— 端点组名称

简短描述。

## 认证（可选，仅当端点需要鉴权时）

仅当文件中**所有**端点共享相同的认证要求时，才包含此章节。
将其放在第一个端点**之前**，用 `---` 分隔。

### 请求头

```http
Authorization: Bearer <token>
```

### 认证错误响应

简要说明。

- HTTP 状态码：`401 Unauthorized`

| `error_code` | message | 条件 |
|---|---|---|

所有认证错误共享相同的响应格式：

```json
{
  "response_id": "uuid-v7-string",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "错误描述。",
  "data": null
}
```

### 说明

- 源码位置：`src/middleware/authenticate.rs`
- 关于认证的额外说明

---

## METHOD /api/v1/path（H2）

此端点的功能描述。

### 请求（H3）

如果端点的请求体接受 JSON，以 JSON 代码块展示：

```json
{
  "field": "value"
}
```

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `field` | string | 是 | 字段说明 |

如果没有请求体，写："无请求体。"

### 路径参数（如适用）

| 参数 | 类型 | 描述 |
|---|---|---|
| `param_id` | string (UUID v7) | 描述 |

### 查询参数（如适用）

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|---|---|---|---|---|
| `limit` | integer | 否 | 50 | 描述 |

### 响应（H3）

#### 成功响应（H4）

- HTTP 状态码：`200 OK`（资源创建使用 `201 Created`）

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "成功消息。",
  "data": { ... }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.field` | type | 嵌套字段使用点号表示法 |

如果有多个成功变体（如新建 vs 已有资源），添加多个 `#### 成功响应` 子章节：
- `#### 成功响应 - 变体 A`
- `#### 成功响应 - 变体 B`

#### 错误响应（H4）

每个错误是一个独立项目，包含加粗的错误名称和完整的 JSON 响应：

**错误名称**

- HTTP 状态码：`4xx` / `5xx`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "ERROR_CODE",
  "message": "错误描述。",
  "data": null
}
```

### 说明（H3）

- 源码位置：`src/path/to/file.rs`
- 其他实现说明（每个要点一行）

---

## 统一错误响应格式（可选，H2）

仅当这个文件是读者最先接触的文档时才包含此章节。
放在文件末尾，附上全局错误码表格。

```json
{
  "response_id": "uuid-v7-string",
  "error_code": "ERROR_CODE",
  "message": "人类可读的错误信息。",
  "data": null
}
```

### 全局错误码

| HTTP 状态码 | `error_code` | 描述 | 来源 |
|---|---|---|---|

---

## 3. 格式规则

### 标题层级
- 仅使用 `##`（H2）、`###`（H3）和 `####`（H4）。
- `#`（H1）仅用于页面标题。
- 禁止更深层嵌套。

### 语言对照
`docs/zh-CN/apiv1/` 中的每个中文文档，都必须在 `docs/en/apiv1/` 中有结构相同的英文对应版本。

### 中文文档标题对照表

| 英文 | 中文 |
|---|---|
| `### Request` | `### 请求` |
| `### Response` | `### 响应` |
| `#### Success` | `#### 成功响应` |
| `#### Errors` | `#### 错误响应` |
| `### Notes` | `### 说明` |
| `### Path Parameters` | `### 路径参数` |
| `### Query Parameters` | `### 查询参数` |
| `- HTTP Status:` | `- HTTP 状态码：` |
| `### Request Header` | `### 请求头` |
| `### Authentication Error Responses` | `### 认证错误响应` |

### JSON 代码块
- 中英文版本使用相同的 JSON 示例值（不要翻译 JSON 内容）。
- 错误 JSON 必须包含 `response_id`、`error_code`、`message` 和 `data: null`。
- 使用真实的 UUID v7 值作为 response_id 和实体 ID（例如 `019ef520-...`）。

### 字段表格
- 嵌套字段使用点号表示法：`data.user.id`。
- 仅使用 `|` 分隔符，不加额外格式。
- 英文表头：`| Field | Type | Description |`
- 中文表头：`| 字段 | 类型 | 描述 |`
- 对于请求表格包含必填列：`| 字段 | 类型 | 必填 | 描述 |`

### 代码块
- JSON 用于 JSON 示例。
- http 用于 HTTP 请求/响应示例。

### 分隔符
- 使用 `---` 分隔端点章节。

### 源码位置
- 每个 `### 说明` / `### Notes` 章节**必须**包含源码位置：
  - 英文：`- Source: \`src/path/to/file.rs\``
  - 中文：`- 源码位置：\`src/path/to/file.rs\``
