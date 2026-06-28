# 聊天

聊天室与消息接口。

私密聊天：按需创建的双人聊天室，具有幂等行为（重复创建返回已有聊天室）。

群组聊天：多人聊天室，采用管理员/成员角色模型。创建者自动获得 `admin` 角色，可以添加或移除成员。普通成员可以自行离开。

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

| `error_code` | message | 条件 |
|---|---|---|
| `AUTHENTICATION_ERROR` | `Missing authorization header.` | 未提供 Authorization 请求头 |
| `AUTHENTICATION_ERROR` | `Authorization header must start with 'Bearer '.` | 请求头格式错误 |
| `AUTHENTICATION_ERROR` | `Token is missing in Authorization header.` | Bearer 后缺少 Token |
| `AUTHENTICATION_ERROR` | `Invalid or expired token.` | Token 无效或已过期 |
| `AUTHENTICATION_ERROR` | `Invalid token.` | Token 负载格式不合法 |
| `AUTHENTICATION_ERROR` | `User not found or inactive.` | 对应用户不存在或已停用 |

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

创建聊天室。根据 `is_group` 字段决定行为：

| `is_group` | 行为 |
|---|---|
| `false`（默认） | **私密聊天室** — 与目标用户 `username` 创建双人聊天室。如果两人之间已存在私密聊天室，则直接返回现有聊天室（幂等）。 |
| `true` | **群组聊天室** — 使用 `name`（群组名称）和 `usernames`（初始成员）创建多人聊天室。创建者自动获得 `admin` 角色。 |

### 请求

#### 私密聊天室

```json
{
  "username": "bob"
}
```

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `username` | string | 是 | 目标用户的用户名 |

#### 群组聊天室

```json
{
  "is_group": true,
  "name": "Project Team",
  "usernames": ["bob", "charlie"]
}
```

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `is_group` | boolean | 是 | 必须为 `true` 以创建群组聊天室 |
| `name` | string | 是 | 群组聊天室名称（不能为空） |
| `usernames` | array[string] | 是 | 初始成员用户名列表（创建者自动被包含并设置为管理员） |

### 响应

#### 成功响应 - 新建私密聊天室

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

#### 成功响应 - 私密聊天室已存在

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

#### 成功响应 - 新建群组聊天室

- HTTP 状态码：`201 Created`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Group room created successfully.",
  "data": {
    "id": "019ef520-0c59-7902-9959-86975c24af39",
    "name": "Project Team",
    "created_by": "019ef520-0c59-7902-9959-86975c24af39",
    "created_at": "2026-06-23T15:35:27.871353+00:00",
    "is_group": true,
    "members": [
      "019ef520-0c59-7902-9959-86975c24af39",
      "019ef530-0c59-7902-9959-86975c24af39",
      "019ef540-0c59-7902-9959-86975c24af39"
    ]
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.id` | string (UUID v7) | 聊天室唯一标识 |
| `data.name` | string \| null | 聊天室名称（私密聊天为 `null`） |
| `data.created_by` | string (UUID v7) | 创建者用户 ID |
| `data.created_at` | string (RFC 3339) | 创建时间 |
| `data.is_group` | boolean | 是否为群组聊天室 |
| `data.members` | array[string] | 成员用户 UUID 列表 |

#### 错误响应

**目标用户不存在（私密聊天室）**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user not found.",
  "data": null
}
```

**不能与自己创建聊天室（私密聊天室）**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot create a room with yourself.",
  "data": null
}
```

**群组聊天室名称不能为空**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room name is required.",
  "data": null
}
```

**群组聊天室名称为空字符串**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room name cannot be empty.",
  "data": null
}
```

**缺少成员列表**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room members (usernames) are required.",
  "data": null
}
```

**成员列表为空**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room must have at least one other member.",
  "data": null
}
```

**成员列表中存在重复用户名**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Duplicate username: bob",
  "data": null
}
```

**成员列表中包含不存在的用户**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "User not found: nonexistent_user",
  "data": null
}
```

**不能将自己添加到成员列表**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot add yourself to the usernames list. You are automatically included as the creator.",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/room.rs`
- 私密聊天室具有幂等性：与同一用户重复创建聊天室会以 `200 OK` 返回已有聊天室
- 群组聊天室始终创建新聊天室（不做重复检查）
- 群组聊天室的创建者自动获得 `admin` 角色，其他初始成员获得 `member` 角色

---

## GET /api/v1/chat/rooms

获取当前用户参与的所有聊天室列表，包含最近消息预览。

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
        "role": "admin",
        "member_count": 2,
        "members": [
          "019ef520-0c59-7902-9959-86975c24af39",
          "019ef530-0c59-7902-9959-86975c24af39"
        ],
        "last_message": {
          "id": "019ef550-0c59-7902-9959-86975c24af39",
          "content": "Hello, Bob!",
          "sender_username": "alice",
          "created_at": "2026-06-23T15:35:27.871353+00:00"
        }
      },
      {
        "id": "019ef560-0c59-7902-9959-86975c24af39",
        "name": "Project Team",
        "created_by": "019ef520-0c59-7902-9959-86975c24af39",
        "created_at": "2026-06-23T16:00:00.000000+00:00",
        "is_group": true,
        "role": "admin",
        "member_count": 3,
        "members": [
          "019ef520-0c59-7902-9959-86975c24af39",
          "019ef530-0c59-7902-9959-86975c24af39",
          "019ef540-0c59-7902-9959-86975c24af39"
        ],
        "last_message": null
      }
    ]
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.rooms` | array | 聊天室数组，按创建时间倒序排列 |

每个聊天室对象：

| 字段 | 类型 | 描述 |
|---|---|---|
| `id` | string (UUID v7) | 聊天室唯一标识 |
| `name` | string \| null | 聊天室名称（私密聊天为 `null`） |
| `created_by` | string (UUID v7) | 创建者用户 ID |
| `created_at` | string (RFC 3339) | 创建时间 |
| `is_group` | boolean | 是否为群组聊天室 |
| `role` | string | 当前用户在此聊天室的角色：`admin` 或 `member` |
| `member_count` | integer | 聊天室成员总数 |
| `members` | array[string] | 成员用户 UUID 列表 |
| `last_message` | object \| null | 最近消息预览，无消息时为 `null` |

`last_message` 对象：

| 字段 | 类型 | 描述 |
|---|---|---|
| `id` | string (UUID v7) | 消息 ID |
| `content` | string | 消息内容 |
| `sender_username` | string | 发送者用户名 |
| `created_at` | string (RFC 3339) | 消息时间戳 |

### 说明

- 源码位置：`src/chat/room.rs`
- 聊天室按 `created_at` 倒序返回
- `last_message` 通过 lateral join 查询获取，无消息时为 `null`

---

## GET /api/v1/chat/rooms/{room_id}

获取指定聊天室的详细信息，包含成员资料（用户名、昵称、角色、加入时间）。

### 路径参数

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 请求

无请求体。

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Room detail retrieved successfully.",
  "data": {
    "id": "019ef520-0c59-7902-9959-86975c24af39",
    "name": "Project Team",
    "created_by": "019ef520-0c59-7902-9959-86975c24af39",
    "created_at": "2026-06-23T15:35:27.871353+00:00",
    "is_group": true,
    "member_count": 3,
    "members": [
      {
        "user_id": "019ef520-0c59-7902-9959-86975c24af39",
        "username": "alice",
        "nickname": null,
        "role": "admin",
        "joined_at": "2026-06-23T15:35:27.871353+00:00"
      },
      {
        "user_id": "019ef530-0c59-7902-9959-86975c24af39",
        "username": "bob",
        "nickname": "Bobby",
        "role": "member",
        "joined_at": "2026-06-23T15:35:27.871353+00:00"
      },
      {
        "user_id": "019ef540-0c59-7902-9959-86975c24af39",
        "username": "charlie",
        "nickname": null,
        "role": "member",
        "joined_at": "2026-06-23T15:35:27.871353+00:00"
      }
    ]
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.id` | string (UUID v7) | 聊天室唯一标识 |
| `data.name` | string \| null | 聊天室名称（私密聊天为 `null`） |
| `data.created_by` | string (UUID v7) | 创建者用户 ID |
| `data.created_at` | string (RFC 3339) | 创建时间 |
| `data.is_group` | boolean | 是否为群组聊天室 |
| `data.member_count` | integer | 成员总数 |
| `data.members` | array | 成员详细资料数组 |

每个成员对象：

| 字段 | 类型 | 描述 |
|---|---|---|
| `user_id` | string (UUID v7) | 用户 ID |
| `username` | string | 用户名 |
| `nickname` | string \| null | 昵称 |
| `role` | string | 成员角色：`admin` 或 `member` |
| `joined_at` | string (RFC 3339) | 加入时间 |

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

**聊天室不存在**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Room not found.",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/room.rs`
- 私密聊天室中，创建者默认获得 `member` 角色

---

## POST /api/v1/chat/rooms/{room_id}/members

向现有聊天室添加新成员。仅群组聊天室可用，且请求者必须拥有 `admin` 角色。

### 路径参数

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 请求

```json
{
  "usernames": ["dave", "eve"]
}
```

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `usernames` | array[string] | 是 | 要添加的用户名列表（不能为空） |

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Members added successfully.",
  "data": {
    "added": [
      {
        "user_id": "019ef570-0c59-7902-9959-86975c24af39",
        "username": "dave",
        "joined_at": "2026-06-23T16:00:00.000000+00:00"
      }
    ],
    "added_count": 1
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.added` | array | 成功添加的成员数组 |
| `data.added[].user_id` | string (UUID v7) | 添加成员的用户 ID |
| `data.added[].username` | string | 添加成员的用户名 |
| `data.added[].joined_at` | string (RFC 3339) | 加入时间 |
| `data.added_count` | integer | 成功添加的成员数量 |

已是聊天室成员的用户会被静默跳过（不计入 `added_count`）。

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

**仅管理员可以添加成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Only room admins can add members.",
  "data": null
}
```

**成员列表不能为空**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "usernames list cannot be empty.",
  "data": null
}
```

**用户不存在**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "User not found: nonexistent_user",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/member.rs`
- 仅群组聊天室可用（端点本身不检查 `is_group`，但非群组聊天室没有管理员，将返回 403）
- 已是成员的用户会被静默跳过

---

## GET /api/v1/chat/rooms/{room_id}/members

获取聊天室所有成员列表，包含角色和加入时间。

### 路径参数

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 请求

无请求体。

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Members listed successfully.",
  "data": {
    "members": [
      {
        "user_id": "019ef520-0c59-7902-9959-86975c24af39",
        "username": "alice",
        "nickname": null,
        "role": "admin",
        "joined_at": "2026-06-23T15:35:27.871353+00:00"
      },
      {
        "user_id": "019ef530-0c59-7902-9959-86975c24af39",
        "username": "bob",
        "nickname": "Bobby",
        "role": "member",
        "joined_at": "2026-06-23T15:35:27.871353+00:00"
      }
    ],
    "count": 2
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.members` | array | 成员资料数组，按 `joined_at` 升序排列 |
| `data.count` | integer | 成员总数 |

每个成员对象：

| 字段 | 类型 | 描述 |
|---|---|---|
| `user_id` | string (UUID v7) | 用户 ID |
| `username` | string | 用户名 |
| `nickname` | string \| null | 昵称 |
| `role` | string | 成员角色：`admin` 或 `member` |
| `joined_at` | string (RFC 3339) | 加入时间 |

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

- 源码位置：`src/chat/member.rs`
- 成员按 `joined_at` 升序排列（最早加入的在前）

---

## DELETE /api/v1/chat/rooms/{room_id}/members/{user_id}

从聊天室中移除成员。根据请求者与目标用户的关系有两种行为：

| 关系 | 行为 | 要求 |
|---|---|---|
| 移除自己（`user_id` == 请求者） | **离开** — 请求者离开聊天室 | 任何成员均可离开任意聊天室 |
| 移除其他用户（`user_id` != 请求者） | **踢出** — 请求者将其他用户移出聊天室 | 仅群组聊天室，且请求者必须拥有 `admin` 角色 |

### 路径参数

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |
| `user_id` | string (UUID v7) | 要移除的用户 ID |

### 请求

无请求体。

### 响应

#### 成功响应 - 离开

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "You have left the room.",
  "data": {
    "room_id": "019ef520-0c59-7902-9959-86975c24af39",
    "left_user_id": "019ef530-0c59-7902-9959-86975c24af39",
    "room_deleted": false
  }
}
```

**最后一个成员离开：** 如果离开用户是聊天室中最后一名成员，聊天室将被删除。

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "You left the room. The room has been deleted as you were the last member.",
  "data": {
    "room_id": "019ef520-0c59-7902-9959-86975c24af39",
    "left_user_id": "019ef530-0c59-7902-9959-86975c24af39",
    "room_deleted": true
  }
}
```

#### 成功响应 - 踢出

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Member removed successfully.",
  "data": {
    "room_id": "019ef520-0c59-7902-9959-86975c24af39",
    "removed_user_id": "019ef530-0c59-7902-9959-86975c24af39"
  }
}
```

| 字段（离开） | 类型 | 描述 |
|---|---|---|
| `data.room_id` | string (UUID v7) | 聊天室 ID |
| `data.left_user_id` | string (UUID v7) | 离开的用户 ID |
| `data.room_deleted` | boolean | 是否因最后一名成员离开导致聊天室被删除 |

| 字段（踢出） | 类型 | 描述 |
|---|---|---|
| `data.room_id` | string (UUID v7) | 聊天室 ID |
| `data.removed_user_id` | string (UUID v7) | 被移出的用户 ID |

#### 错误响应

**请求者不是聊天室成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

**不能从私密聊天中踢出成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Cannot kick members from a private chat.",
  "data": null
}
```

**仅管理员可以移除成员**

- HTTP 状态码：`403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Only room admins can remove members.",
  "data": null
}
```

**目标用户不是聊天室成员**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user is not a member of this room.",
  "data": null
}
```

**聊天室不存在**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Room not found.",
  "data": null
}
```

### 说明

- 源码位置：`src/chat/member.rs`
- 自行离开对任何聊天室成员均允许
- 踢出操作仅群组聊天室中具有 `admin` 角色的成员可以执行
- 如果离开/被踢出的用户是最后一名管理员，系统会自动推举继任者：
  1. 聊天室创建者（如果仍是成员且不是离开用户）
  2. 按 `joined_at` 最早的剩余成员
- 如果最后一名成员离开，聊天室将被删除（级联删除消息和成员关系）

---

## POST /api/v1/chat/rooms/{room_id}/messages

在指定聊天室中发送消息。

### 路径参数

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 请求

```json
{
  "content": "Hello, Bob!"
}
```

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `content` | string | 是 | 消息内容 |

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

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.id` | string (UUID v7) | 消息唯一标识 |
| `data.room_id` | string (UUID v7) | 所属聊天室 ID |
| `data.sender_id` | string (UUID v7) | 发送者用户 ID |
| `data.content` | string | 消息内容 |
| `data.created_at` | string (RFC 3339) | 发送时间 |

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

| 参数 | 类型 | 描述 |
|---|---|---|
| `room_id` | string (UUID v7) | 聊天室 ID |

### 查询参数

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|---|---|---|---|---|
| `limit` | integer | 否 | 50 | 返回消息数量上限（最大 100） |
| `before` | string (UUID v7) | 否 | - | 游标：获取此消息 ID 之前的更早消息 |

### 请求

无请求体。

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

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.messages` | array | 消息数组，按创建时间倒序排列 |
| `data.has_more` | boolean | 是否还有更早的消息 |
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
