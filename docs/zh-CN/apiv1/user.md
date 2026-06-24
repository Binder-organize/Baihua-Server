# 用户

用户注册与登录接口。

## 公共约束

以下中间件应用于所有 `/api/v1/user/*` 请求：

| 约束 | 说明 |
|---|---|
| Content-Type | 必须为 `application/json` |
| 请求体大小上限 | 生产环境 **1 MB**，开发环境 **10 MB** |
| JSON 合法性 | 请求体必须是合法 JSON |
| 必填字段 | JSON 中必须包含非空的 `username` 和 `email` 字段 |

源码位置：`src/middleware/validate.rs`

---

## POST /api/v1/user/register

创建新用户。

### 请求

```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "secureP@ss1"
}
```

| 字段 | 类型 | 必填 | 校验规则 |
|---|---|---|---|
| `username` | string | 是 | 4-40 个字符 |
| `email` | string | 是 | 合法邮箱格式，最长 254 字符，不能为 `gav.zheng@outlook.com` |
| `password` | string | 是 | 不能为空 |

### 响应

#### 成功响应

- HTTP 状态码：`201 Created`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "User created successfully",
  "data": {
    "message": "User created successfully",
    "user": {
      "id": "019ef520-0c59-7902-9959-86975c24af39",
      "username": "alice",
      "email": "alice@example.com",
      "nickname": null,
      "phone_number": null,
      "created_at": "2026-06-23 15:35:27.871353 UTC",
      "is_active": true
    }
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.user.id` | string (UUID v7) | 用户唯一标识 |
| `data.user.username` | string | 用户名 |
| `data.user.email` | string | 邮箱 |
| `data.user.nickname` | string \| null | 昵称，默认为 null |
| `data.user.phone_number` | string \| null | 手机号，默认为 null |
| `data.user.created_at` | string (UTC) | 创建时间 |
| `data.user.is_active` | boolean | 是否激活，默认为 true |

**注意：** 响应中不返回密码字段。

#### 错误响应

**用户名或邮箱已存在**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "VALIDATION_ERROR",
  "message": "Username or email already exists.",
  "data": null
}
```

**请求体格式错误**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "INVALID_JSON_ERROR",
  "message": "Invalid JSON format: ...",
  "data": null
}
```

**字段校验不通过**

- HTTP 状态码：`400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "VALIDATION_ERROR",
  "message": "Username must be between 4 and 40 characters long.",
  "data": null
}
```

其他可能的校验错误消息：

| 错误消息 | 触发条件 |
|---|---|
| `Username, email, and password cannot be empty.` | 任意必填字段为空 |
| `Invalid email format. Please provide a valid email address.` | 邮箱格式不合法 |
| `Email address is too long (maximum 254 characters).` | 邮箱超长 |
| `Username must be between 4 and 40 characters long.` | 用户名长度超出范围 |
| `Email cannot be 'gav.zheng@outlook.com'.` | 使用了保留邮箱 |

---

## POST /api/v1/user/login

用户登录，返回 JWT Token。

### 请求

```json
{
  "username": "alice",
  "password": "secureP@ss1"
}
```

| 字段 | 类型 | 必填 |
|---|---|---|
| `username` | string | 是 |
| `password` | string | 是 |

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "User logged in successfully.",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSIsImlhdCI6MTc1ODY0MDAwMCwiZXhwIjoxNzU4NzI2NDAwfQ...",
    "user": {
      "id": "019ef520-0c59-7902-9959-86975c24af39",
      "username": "alice",
      "email": "alice@example.com",
      "nickname": null,
      "phone_number": null,
      "created_at": "2026-06-23 15:35:27.871353 UTC",
      "is_active": true
    }
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `data.token` | string (JWT) | Bearer Token，用于后续认证 |
| `data.user` | object | 用户信息（同注册接口） |

**Token 说明：**

- 签发算法：HS256
- 有效时长：由 `config.toml` 中 `user.jsonwebtoken_expiration_hours` 控制，默认 **24 小时**
- 负载（Claims）：

```json
{
  "sub": "alice",
  "iat": 1758640000,
  "exp": 1758726400
}
```

| 声明 | 描述 |
|---|---|
| `sub` | 用户名 |
| `iat` | 签发时间（Unix 时间戳） |
| `exp` | 过期时间（Unix 时间戳） |

**开发环境行为：** Token 会打印在日志中，方便调试。

**生产环境行为：** Token **不会**出现在日志中。

#### 错误响应

**用户名或密码错误**

- HTTP 状态码：`401 Unauthorized`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "Invalid username or password.",
  "data": null
}
```

### Token 使用示例

后续请求在 `Authorization` 头中携带 Token：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSIsImlhdCI6MTc1ODY0MDAwMCwiZXhwIjoxNzU4NzI2NDAwfQ...
```

---

## 统一错误响应格式

所有错误响应遵循统一结构：

```json
{
  "response_id": "uuid-v7-string",
  "error_code": "ERROR_CODE",
  "message": "Human-readable message.",
  "data": null
}
```

### 全局错误码

| HTTP 状态码 | `error_code` | 说明 | 来源 |
|---|---|---|---|
| 400 | `VALIDATION_ERROR` | 请求参数校验不通过 | 各业务接口 |
| 400 | `INVALID_JSON_ERROR` | JSON 解析失败 | 中间件 |
| 400 | `BAD_REQUEST_ERROR` | 请求体过大等 | 中间件 |
| 401 | `AUTHENTICATION_ERROR` | 认证失败 | 登录接口 |
| 403 | `FORBIDDEN_ERROR` | 权限不足（预留） | 全局 |
| 404 | `NOT_FOUND_ERROR` | 路由不存在 | 全局中间件 |
| 500 | `INTERNAL_SERVER_ERROR` | 服务器内部错误 | 全局 |
| 500 | `DATABASE_ERROR` | 数据库错误 | 各业务接口 |
| 500 | `SERVER_CRASHES` | 服务 panic | 全局中间件 |

**生产环境安全策略：** `INTERNAL_SERVER_ERROR` 和 `DATABASE_ERROR` 的详细错误信息仅在开发环境返回，生产环境统一返回 `"An internal error occurred. Please try again later."`。
