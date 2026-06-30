# Server Greeting

## GET /greet

Returns server version information and a greeting message.

### Request

No headers, no request body.

### Response

#### Success

- HTTP Status: `200 OK`

```json
{
  "server_version": "0.1.0",
  "api_version": "v1",
  "message": "Hello Baihua."
}
```

| Field | Type | Description |
|---|---|---|
| `server_version` | string | Server version, hardcoded to `"0.1.0"` |
| `api_version` | string | API version, hardcoded to `"v1"` |
| `message` | string | Greeting, hardcoded to `"Hello Baihua."` |

### Notes

- Source: `src/greet.rs`
- Response is plain JSON, **not** wrapped in the standard response envelope
- The `server_version` field must be updated when the server version is bumped
