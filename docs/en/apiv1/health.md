# Health Check

## GET /health

Checks whether the server is running and verifies database connectivity.

### Request

No headers, no request body.

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `response_id` | string | UUID v7 uniquely identifying this request |
| `error_code` | string | Always `"OK"` |
| `message` | string | `"Service is healthy."` |
| `data.status` | string | `"ok"` |

#### Errors

**Database connection failure**

- HTTP Status: `500 Internal Server Error`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "INTERNAL_SERVER_ERROR",
  "message": "Database connection failed.",
  "data": null
}
```

### Notes

- Source: `src/health.rs`
- Runs `SELECT 1` against PostgreSQL to verify pool availability
- Bypasses authentication and request-body validation middleware; suitable for load-balancer health probes
- Still passes through tracing, panic-catch, and 404-handler middleware
