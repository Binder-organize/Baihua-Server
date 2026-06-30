# 服务器问候

## GET /greet

返回服务器版本信息和问候语。

### 请求

无请求头、无请求体。

### 响应

#### 成功响应

- HTTP 状态码：`200 OK`

```json
{
  "server_version": "0.1.0",
  "api_version": "v1",
  "message": "Hello Baihua."
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `server_version` | string | 服务器版本号，固定为 `"0.1.0"` |
| `api_version` | string | API 版本号，固定为 `"v1"` |
| `message` | string | 问候语，固定为 `"Hello Baihua."` |

### 说明

- 源码位置：`src/greet.rs`
- 返回值为直接 JSON，不包装在标准响应格式中
- 服务器版本升级时需要同步修改 `server_version` 字段
