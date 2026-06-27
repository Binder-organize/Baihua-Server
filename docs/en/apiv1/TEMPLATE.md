# API Documentation Template

This file describes the required format for API endpoint documentation. New API docs **must** follow this template to maintain consistency across all endpoints.

---

## 1. File Naming

| Endpoint | File name |
|---|---|
| Single endpoint group (e.g. user, chat) | `<name>.md` — e.g. `user.md`, `chat.md` |
| Standalone endpoint (e.g. greet, health) | `<name>.md` — e.g. `greet.md`, `health.md` |

Place the file in `docs/en/apiv1/` and create a matching file in `docs/zh-CN/apiv1/`.

---

## 2. File Structure

```
# Title (H1) — endpoint group name

Short description.

## Authentication Section (optional, if endpoints require auth)

Only include this section when ALL endpoints in the file share the same auth requirements.
Place it BEFORE the first endpoint, separated by `---`.

### Request Header

```http
Authorization: Bearer <token>
```

### Authentication Error Responses

Brief description.

- HTTP Status: `401 Unauthorized`

| `error_code` | message | Condition |
|---|---|---|

All authentication errors share the same response format:

```json
{
  "response_id": "uuid-v7-string",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "Error description.",
  "data": null
}
```

### Notes

- Source: `src/middleware/authenticate.rs`
- Additional auth-related notes

---

## METHOD /api/v1/path (H2)

Brief description of what this endpoint does.

### Request (H3)

If the endpoint accepts a request body, show it as JSON:

```json
{
  "field": "value"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `field` | string | Yes | Field description |

If no request body is needed, write: "No request body."

### Path Parameters (if applicable)

| Parameter | Type | Description |
|---|---|---|
| `param_id` | string (UUID v7) | Description |

### Query Parameters (if applicable)

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `limit` | integer | No | 50 | Description |

### Response (H3)

#### Success (H4)

- HTTP Status: `200 OK`  (use `201 Created` for resource creation)

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Success message.",
  "data": { ... }
}
```

| Field | Type | Description |
|---|---|---|
| `data.field` | type | Use dot notation for nested fields |

If there are multiple success variants (e.g. new vs existing resource), add multiple `#### Success` sub-sections:
- `#### Success - Variant A`
- `#### Success - Variant B`

#### Errors (H4)

Each error is a separate item with bold name and full JSON response:

**Error name**

- HTTP Status: `4xx` / `5xx`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "ERROR_CODE",
  "message": "Error description.",
  "data": null
}
```

### Notes (H3)

- Source: `src/path/to/file.rs`
- Additional implementation notes (one bullet per note)

---

## Standard Error Envelope (optional, H2)

Include this section only if this is the first documentation file the reader encounters.
Place it at the end with the global error code table.

```json
{
  "response_id": "uuid-v7-string",
  "error_code": "ERROR_CODE",
  "message": "Human-readable message.",
  "data": null
}
```

### Global Error Codes

| HTTP Status | `error_code` | Description | Origin |
|---|---|---|---|

---

## 3. Formatting Rules

### Headings
- Only `##` (H2) and `###` (H3) and `####` (H4) are used.
- `#` (H1) is for the page title only.
- No deeper nesting.

### Language pairs
Every English doc in `docs/en/apiv1/` must have a corresponding Chinese doc in `docs/zh-CN/apiv1/` with the same structure.

### Chinese doc heading mapping

| English | Chinese |
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

### JSON Blocks
- Use the same example values across both language versions (don't translate the JSON content).
- Error JSON must always include `response_id`, `error_code`, `message`, and `data: null`.
- Use realistic UUID v7 values for response_id and entity IDs (e.g. `019ef520-...`).

### Field Tables
- Use dot notation for nested fields: `data.user.id`.
- `|` separator is the only syntax — no extra formatting.
- English: `| Field | Type | Description |`
- Chinese: `| 字段 | 类型 | 描述 |`
- For request tables with required columns: `| Field | Type | Required | Description |`

### Code Blocks
- JSON for JSON examples.
- http for HTTP request/response examples.

### Separator
- Use `---` to separate endpoint sections.

### Source Location
- Every `### Notes` section must include the source location as `- Source: \`src/path/to/file.rs\`` (English) or `- 源码位置：\`src/path/to/file.rs\`` (Chinese).

---

## 4. Example

See existing docs for reference:

| Endpoint | File |
|---|---|
| Simple (no auth, no body) | `greet.md` |
| Simple (with auth) | `health.md` |
| Complex (auth, multiple endpoints, errors) | `user.md` |
| Complex (auth, path/query params, cursor pagination) | `chat.md` |
