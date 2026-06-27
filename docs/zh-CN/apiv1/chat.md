# 聊天

聊天室与消息接口。

## 认证

所有聊天接口都需要通过 JWT Token 进行认证。

### 请求头

```http
Authorization: Bearer <token>
```

Token 通过 `POST /api/v1/user/login` 获取，详情见[用户文档](./user.md#post-apiv1userlogin)。

### 认证错误响应

以下错误在所有聊天接口中均可能返回：

- HTTP 状态码：`401 Unauthorized`

| `error_code`           | message                                           | 条件                    |
|------------------------|---------------------------------------------------|-----------------------|
| `AUTHENTICATION_ERROR` | `Missing authorization header.`                   | 未提供 Authorization 请求头 |
| `AUTHENTICATION_ERROR` | `Authorization header must start with 'Bearer '.` | 请求头格式错误               |
| `AUTHENTICATION_ERROR` | `Token is missing in Authorization header.`       | Bearer 后缺少 Token      |
| `AUTHENTICATION_ERROR` | `Invalid or expired token.`                       | Token 无效或已过期          |
| `AUTHENTICATION_ERROR` | `Invalid token.`                                  | Token 负载格式不合法         |
| `AUTHENTICATION_ERROR` | `User not found or inactive.`                     | 对应用户不存在或已停用           |

所有认证错误返回统一格式：

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "Invalid or expired token.",
  "data": null
}
```

### 说明

- 源码位置：`src/middleware/authenticate.rs`
- 认证通过后，用户信息会注入请求扩展中，供后续处理器使用

---

## POST /api/v1/chat/rooms

创建与另一用户之间的私密聊天室。如果两人之间已存在私密聊天室，则直接返回现有聊天室。

### 请求

```json
{
  "username": "bob"
}
```

| 字段         | 类型     | 必填 | 说明       |
|------------|--------|----|----------|
| `username` | string | 是  | 目标用户的用户名 |

### 响应

#### 成功响应 - 新建聊天室

- HTTP 状态码：`201 Created`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Room created successfully.",
  "data": {
    "id": "019ef520-0c59-7902-9959-86975c24af39",
    "name": null,
    "created_by": "019ef520-0c59-7902-9959-86975c24af39",
    "created_at": "2026-06-23T15:35:27.871353+00:00",
    "is_group": false,
    "members": [
      "019ef520-0c59-7902-9959-86975c24af39",
      "019ef530-0c59-7902-9959-86975c24af39"
    ]
  }
}
```

| 字段                | 类型                | 说明                       |
|-------------------|-------------------|--------------------------|
| `data.id`         | string (UUID v7)  | 聊天室唯一标识                  |
| `data.name`       | string \| null    | 聊天室名称（私密聊天为 null）        |
| `data.created_by` | string (UUID v7)  | 创建者用户 ID                 |
| `data.created_at` | string (RFC 3339) | 创建时间                     |
| `data.is_group`   | boolean           | 是否为群组（当前仅支持私聊，固定为 false） |
| `data.members`    | array[string]     | 成员用户 UUID 列表             |

#### 成功响应 - 聊天室已存在

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Room already exists.",
  "data": {
    "id": "019ef520-0c59-7902-9959-86975c24af39",
    "name": null,
    "created_by": "019ef520-0c59-7902-9959-86975c24af39",
    "created_at": "2026-06-23T15:35:27.871353+00:00",
    "is_group": false,
    "members": [
      "019ef520-0c59-7902-9959-86975c24af39",
      "019ef530-0c59-7902-9959-86975c24af39"
    ]
  }
}
```

#### 错误响应

**目标用户不存在**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user not found.",
  "data": null
}
```

**不能与自己创建聊天室**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot create a room with yourself.",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/room.rs`

---

## GET /api/v1/chat/rooms

获取当前用户参与的所有聊天室列表。

### 请求

无请求体。

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Rooms listed successfully.",
  "data": {
    "rooms": [
      {
        "id": "019ef520-0c59-7902-9959-86975c24af39",
        "name": null,
        "created_by": "019ef520-0c59-7902-9959-86975c24af39",
        "created_at": "2026-06-23T15:35:27.871353+00:00",
        "is_group": false,
        "members": [
          "019ef520-0c59-7902-9959-86975c24af39",
          "019ef530-0c59-7902-9959-86975c24af39"
        ]
      }
    ]
  }
}
```

| 字段           | 类型    | 说明              |
|--------------|-------|-----------------|
| `data.rooms` | array | 聊天室数组，按创建时间倒序排列 |

每个聊天室对象的结构与 `POST /api/v1/chat/rooms` 返回一致。

### 说明

- 源码位置：`src/chat/room.rs`

---

## POST /api/v1/chat/rooms/{room_id}/messages

在指定聊天室中发送消息。

### 路径参数

| 参数        | 类型               | 说明     |
|-----------|------------------|--------|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 请求

```json
{
  "content": "Hello, Bob!"
}
```

| 字段        | 类型     | 必填 | 说明   |
|-----------|--------|----|------|
| `content` | string | 是  | 消息内容 |

### 响应

#### 成功响应

- HTTP 状态码：`201 Created`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Message sent successfully.",
  "data": {
    "id": "019ef550-0c59-7902-9959-86975c24af39",
    "room_id": "019ef520-0c59-7902-9959-86975c24af39",
    "sender_id": "019ef520-0c59-7902-9959-86975c24af39",
    "content": "Hello, Bob!",
    "created_at": "2026-06-23T15:35:27.871353+00:00"
  }
}
```

| 字段                | 类型                | 说明       |
|-------------------|-------------------|----------|
| `data.id`         | string (UUID v7)  | 消息唯一标识   |
| `data.room_id`    | string (UUID v7)  | 所属聊天室 ID |
| `data.sender_id`  | string (UUID v7)  | 发送者用户 ID |
| `data.content`    | string            | 消息内容     |
| `data.created_at` | string (RFC 3339) | 发送时间     |

#### 错误响应

**不是聊天室成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/message.rs`

---

## GET /api/v1/chat/rooms/{room_id}/messages

获取指定聊天室的历史消息，支持游标分页。

### 路径参数

| 参数        | 类型               | 说明     |
|-----------|------------------|--------|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 查询参数

| 参数       | 类型               | 必填 | 默认值 | 说明                  |
|----------|------------------|----|-----|---------------------|
| `limit`  | integer          | 否  | 50  | 返回消息数量上限（最大 100）    |
| `before` | string (UUID v7) | 否  | -   | 游标：获取此消息 ID 之前的更早消息 |

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Messages retrieved successfully.",
  "data": {
    "messages": [
      {
        "id": "019ef550-0c59-7902-9959-86975c24af39",
        "room_id": "019ef520-0c59-7902-9959-86975c24af39",
        "sender_id": "019ef520-0c59-7902-9959-86975c24af39",
        "content": "Hello, Bob!",
        "created_at": "2026-06-23T15:35:27.871353+00:00"
      }
    ],
    "has_more": false,
    "next_cursor": null
  }
}
```

| 字段                 | 类型                       | 说明                                |
|--------------------|--------------------------|-----------------------------------|
| `data.messages`    | array                    | 消息数组，按创建时间倒序排列                    |
| `data.has_more`    | boolean                  | 是否还有更早的消息                         |
| `data.next_cursor` | string (UUID v7) \| null | 用于获取下一页消息的游标（消息 ID），没有更多消息时为 null |

每个消息对象包含 `id`、`room_id`、`sender_id`、`content`、`created_at` 字段。

#### 错误响应

**不是聊天室成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

### 分页用法

首次请求获取最新消息：

```http
GET /api/v1/chat/rooms/{room_id}/messages?limit=50
```

获取更早的消息（使用上一次响应中的 `next_cursor`）：

```http
GET /api/v1/chat/rooms/{room_id}/messages?limit=50&before={next_cursor}
```

当 `has_more` 为 `false` 或 `next_cursor` 为 `null` 时，表示已无更早消息。

### 说明

- 源码位置：`src/chat/message.rs`
- 消息按创建时间**倒序**返回（最新的在前）
