# Chat

Chat room and messaging endpoints.

## Authentication

All chat endpoints require JWT authentication.

### Request Header

```http
Authorization: Bearer <token>
```

Obtain a token via `POST /api/v1/user/login` (see [user documentation](./user.md#post-apiv1userlogin)).

### Authentication Error Responses

The following errors may be returned by any chat endpoint:

- HTTP Status: `401 Unauthorized`

| `error_code` | message | Condition |
|---|---|---|
| `AUTHENTICATION_ERROR` | `Missing authorization header.` | No Authorization header provided |
| `AUTHENTICATION_ERROR` | `Authorization header must start with 'Bearer '.` | Malformed header format |
| `AUTHENTICATION_ERROR` | `Token is missing in Authorization header.` | Empty token after Bearer prefix |
| `AUTHENTICATION_ERROR` | `Invalid or expired token.` | Token is invalid or expired |
| `AUTHENTICATION_ERROR` | `Invalid token.` | Token payload format is invalid |
| `AUTHENTICATION_ERROR` | `User not found or inactive.` | The user account does not exist or is disabled |

All authentication errors share the same response format:

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "AUTHENTICATION_ERROR",
  "message": "Invalid or expired token.",
  "data": null
}
```

### Notes

- Source: `src/middleware/authenticate.rs`
- On successful authentication, user information is injected into request extensions for downstream handlers

---

## POST /api/v1/chat/rooms

Create a private chat room with another user. If a private room already exists between the two users, the existing room is returned instead.

### Request

```json
{
  "username": "bob"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `username` | string | Yes | Target user's username |

### Response

#### Success - New Room Created

- HTTP Status: `201 Created`

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

| Field | Type | Description |
|---|---|---|
| `data.id` | string (UUID v7) | Unique room identifier |
| `data.name` | string \| null | Room name (always null for private chats) |
| `data.created_by` | string (UUID v7) | Creator's user ID |
| `data.created_at` | string (RFC 3339) | Creation timestamp |
| `data.is_group` | boolean | Whether this is a group chat (currently always false) |
| `data.members` | array[string] | Member user UUIDs |

#### Success - Room Already Exists

- HTTP Status: `200 OK`

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

#### Errors

**Target user not found**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user not found.",
  "data": null
}
```

**Cannot create room with yourself**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot create a room with yourself.",
  "data": null
}
```

### Notes

- Source: `src/chat/room.rs`

---

## GET /api/v1/chat/rooms

List all chat rooms the authenticated user is a member of.

### Request

No request body.

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `data.rooms` | array | Array of rooms, ordered by creation date descending |

Each room object has the same structure as the room object in the `POST /api/v1/chat/rooms` response.

### Notes

- Source: `src/chat/room.rs`

---

## POST /api/v1/chat/rooms/{room_id}/messages

Send a message to a chat room.

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |

### Request

```json
{
  "content": "Hello, Bob!"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `content` | string | Yes | Message content |

### Response

#### Success

- HTTP Status: `201 Created`

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

| Field | Type | Description |
|---|---|---|
| `data.id` | string (UUID v7) | Unique message identifier |
| `data.room_id` | string (UUID v7) | Room this message belongs to |
| `data.sender_id` | string (UUID v7) | Sender's user ID |
| `data.content` | string | Message content |
| `data.created_at` | string (RFC 3339) | Timestamp |

#### Errors

**Not a room member**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

### Notes

- Source: `src/chat/message.rs`

---

## GET /api/v1/chat/rooms/{room_id}/messages

Retrieve historical messages from a chat room with cursor-based pagination.

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |

### Query Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `limit` | integer | No | 50 | Maximum number of messages to return (max 100) |
| `before` | string (UUID v7) | No | - | Cursor: fetch messages older than this message ID |

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `data.messages` | array | Array of messages, ordered by creation date descending |
| `data.has_more` | boolean | Whether there are more messages before the returned set |
| `data.next_cursor` | string (UUID v7) \| null | Cursor to use for fetching the next page; null if no more messages |

Each message object contains `id`, `room_id`, `sender_id`, `content`, and `created_at`.

#### Errors

**Not a room member**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

### Pagination Usage

Fetch the most recent messages:

```http
GET /api/v1/chat/rooms/{room_id}/messages?limit=50
```

Fetch older messages (using `next_cursor` from the previous response):

```http
GET /api/v1/chat/rooms/{room_id}/messages?limit=50&before={next_cursor}
```

When `has_more` is `false` or `next_cursor` is `null`, there are no more messages.

### Notes

- Source: `src/chat/message.rs`
- Messages are returned in **descending** order (newest first)
