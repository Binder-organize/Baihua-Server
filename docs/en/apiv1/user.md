# User

User registration and login endpoints.

## Request Validation

The validation middleware is applied to `POST /api/v1/user/register` and `POST /api/v1/user/login` (not to `GET /api/v1/user/list`).

### Common Validation

The following rules apply to both endpoints:

| Constraint | Description |
|---|---|
| Content-Type | Must be `application/json` |
| Max Body Size | **1 MB** in production, **10 MB** in development |
| JSON Validity | Request body must be valid JSON |

### Required Fields

| Endpoint | Required Non-Empty Fields |
|---|---|
| `POST /api/v1/user/register` | `username`, `email`, `password` |
| `POST /api/v1/user/login` | `username`, `password` |

Source: `src/middleware/validate.rs`

---

## POST /api/v1/user/register

Create a new user account.

### Request

```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "secureP@ss1"
}
```

| Field | Type | Required | Validation |
|---|---|---|---|
| `username` | string | Yes | 4–40 characters |
| `email` | string | Yes | Valid email format, max 254 characters, must not be `gav.zheng@outlook.com` |
| `password` | string | Yes | Must not be empty |

### Response

#### Success

- HTTP Status: `201 Created`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "User created successfully",
  "data": {
    "user": {
      "id": "019ef520-0c59-7902-9959-86975c24af39",
      "username": "alice",
      "email": "alice@example.com",
      "nickname": null,
      "phone_number": null,
      "created_at": "2026-06-23T15:35:27.871353Z",
      "is_active": true
    }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `data.user.id` | string (UUID v7) | Unique user identifier |
| `data.user.username` | string | Username |
| `data.user.email` | string | Email address |
| `data.user.nickname` | string \| null | Display name, defaults to null |
| `data.user.phone_number` | string \| null | Phone number, defaults to null |
| `data.user.created_at` | string (RFC 3339) | Account creation timestamp |
| `data.user.is_active` | boolean | Whether the account is active, defaults to true |

**Note:** The response never includes the password.

#### Errors

**Username or email already exists**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "VALIDATION_ERROR",
  "message": "Username or email already exists.",
  "data": null
}
```

**Malformed request body**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "INVALID_JSON_ERROR",
  "message": "Invalid JSON format: ...",
  "data": null
}
```

**Validation failure**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "VALIDATION_ERROR",
  "message": "Username must be between 4 and 40 characters long.",
  "data": null
}
```

Other possible validation error messages:

| Message | Condition |
|---|---|
| `Username, email, and password cannot be empty.` | Any required field is empty |
| `Invalid email format. Please provide a valid email address.` | Email format is invalid |
| `Email address is too long (maximum 254 characters).` | Email exceeds 254 characters |
| `Username must be between 4 and 40 characters long.` | Username length is out of range |
| `Email cannot be 'gav.zheng@outlook.com'.` | Reserved email used |

### Notes

- Source: `src/user/register.rs`

---

## POST /api/v1/user/login

Authenticate a user and receive a JWT token.

### Request

```json
{
  "username": "alice",
  "password": "secureP@ss1"
}
```

| Field | Type | Required |
|---|---|---|
| `username` | string | Yes |
| `password` | string | Yes |

### Response

#### Success

- HTTP Status: `200 OK`

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
      "created_at": "2026-06-23T15:35:27.871353Z",
      "is_active": true
    }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `data.token` | string (JWT) | Bearer token for subsequent authenticated requests |
| `data.user` | object | User profile (same structure as the register response) |

**Token Details:**

- Algorithm: HS256
- Lifetime: Controlled by `user.jsonwebtoken_expiration_hours` in `config.toml`; defaults to **24 hours**
- Claims:

```json
{
  "sub": "019ef520-0c59-7902-9959-86975c24af39",
  "iat": 1758640000,
  "exp": 1758726400
}
```

| Claim | Description |
|---|---|
| `sub` | User UUID (v7) |
| `iat` | Issued-at timestamp (Unix epoch) |
| `exp` | Expiration timestamp (Unix epoch) |

**Development behavior:** The token is logged for debugging convenience.

**Production behavior:** The token is **never** written to logs.

#### Errors

**Invalid username or password**

- HTTP Status: `401 Unauthorized`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "Invalid username or password.",
  "data": null
}
```

### Using the Token

Include the token in the `Authorization` header for authenticated requests:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSIsImlhdCI6MTc1ODY0MDAwMCwiZXhwIjoxNzU4NzI2NDAwfQ...
```

### Notes

- Source: `src/user/login.rs`

---

## GET /api/v1/user/list

Retrieve all active users (no authentication required).

### Request

No headers, no request body.

### Response

#### Success

- HTTP Status: `200 OK`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "OK",
  "message": "Users retrieved successfully.",
  "data": {
    "users": [
      {
        "id": "019ef520-0c59-7902-9959-86975c24af39",
        "username": "alice",
        "email": "alice@example.com",
        "nickname": null,
        "phone_number": null,
        "created_at": "2026-06-23T15:35:27.871353Z",
        "is_active": true
      }
    ]
  }
}
```

| Field | Type | Description |
|---|---|---|
| `data.users` | array | Array of active users, ordered by creation date descending |

Each user object has the same structure as the `user` field in the register response.

### Notes

- Source: `src/user/list.rs`
- Only returns users with `is_active = true`
- Bypasses the validation middleware (no Content-Type check or JSON parsing)

---

## Standard Error Envelope

All error responses share a uniform structure:

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
| 400 | `VALIDATION_ERROR` | Request validation failed | Business logic |
| 400 | `INVALID_JSON_ERROR` | JSON parsing failed | Validation middleware |
| 400 | `BAD_REQUEST_ERROR` | Request body too large, target user not found, etc. | Validation middleware, chat endpoints |
| 401 | `AUTHENTICATION_ERROR` | Authentication failed (missing token, invalid/expired token, user not found) | Login endpoint, auth middleware |
| 403 | `FORBIDDEN_ERROR` | Insufficient permissions (not a chat room member, etc.) | Chat endpoints |
| 404 | `NOT_FOUND_ERROR` | Route not found | Global middleware |
| 500 | `INTERNAL_SERVER_ERROR` | Internal server error | Global |
| 500 | `DATABASE_ERROR` | Database error | Business logic |
| 500 | `SERVER_CRASHES` | Service panic | Global middleware |

**Production security policy:** Detailed error messages for `INTERNAL_SERVER_ERROR` and `DATABASE_ERROR` are only returned in development. In production, the generic message `"An internal error occurred. Please try again later."` is used instead.
