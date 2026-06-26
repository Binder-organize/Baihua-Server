# 健康检查

## GET /health

检测服务器是否正常运行，同时验证数据库连接是否可用。

### 请求

无请求头、无请求体。

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Service is healthy.",
  "data": {
    "status": "ok"
  }
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `response_id` | string | UUID v7，唯一标识此次请求 |
| `error_code` | string | 固定为 `"OK"` |
| `message` | string | `"Service is healthy."` |
| `data.status` | string | `"ok"` |

#### 失败响应

数据库连接异常时返回。

- HTTP 状态码：`500 Internal Server Error`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "INTERNAL_SERVER_ERROR",
  "message": "Database connection failed."
}
```

### 实现说明

- 源码位置：`src/health.rs`
- 执行 `SELECT 1` 查询验证 PostgreSQL 连接池可用性
- 不经过认证与请求体验证中间件，适合负载均衡器健康检测
- 仍然经过 tracing（追踪）、panic 捕获和 404 处理中间件
