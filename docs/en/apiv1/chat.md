# Chat

Chat room and messaging endpoints.

Private chat: two-person rooms created on demand, with idempotent behavior (re-creation returns the existing room).

Group chat: multi-person rooms with an admin/member role model. The creator is automatically assigned the `admin` role and can add or remove members. Regular members can leave on their own.

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

Create a chat room. The behavior depends on the `is_group` field:

| `is_group` | Behavior |
|---|---|
| `false` (default) | **Private room** — creates a two-person room with the target `username`. If a private room already exists between the two users, the existing room is returned instead (idempotent). |
| `true` | **Group room** — creates a multi-person room with `name` and initial `usernames`. The creator is automatically assigned the `admin` role. |

### Request

#### Private Room

```json
{
  "username": "bob"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `username` | string | Yes | Target user's username |

#### Group Room

```json
{
  "is_group": true,
  "name": "Project Team",
  "usernames": ["bob", "charlie"]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `is_group` | boolean | Yes | Must be `true` to create a group room |
| `name` | string | Yes | Group room name (must not be empty) |
| `usernames` | array[string] | Yes | Initial member usernames (the creator is automatically included and assigned admin) |

### Response

#### Success - Private Room Created

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

#### Success - Private Room Already Exists

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

#### Success - Group Room Created

- HTTP Status: `201 Created`

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

| Field | Type | Description |
|---|---|---|
| `data.id` | string (UUID v7) | Unique room identifier |
| `data.name` | string \| null | Room name (`null` for private chats) |
| `data.created_by` | string (UUID v7) | Creator's user ID |
| `data.created_at` | string (RFC 3339) | Creation timestamp |
| `data.is_group` | boolean | Whether this is a group room |
| `data.members` | array[string] | Member user UUIDs |

#### Errors

**Target user not found (private room)**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user not found.",
  "data": null
}
```

**Cannot create a room with yourself (private room)**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot create a room with yourself.",
  "data": null
}
```

**Group room name is required**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room name is required.",
  "data": null
}
```

**Group room name cannot be empty**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room name cannot be empty.",
  "data": null
}
```

**Group room members (usernames) are required**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room members (usernames) are required.",
  "data": null
}
```

**Group room must have at least one other member**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Group room must have at least one other member.",
  "data": null
}
```

**Duplicate username in usernames list**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Duplicate username: bob",
  "data": null
}
```

**User not found in usernames list**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "User not found: nonexistent_user",
  "data": null
}
```

**Cannot add yourself to the usernames list**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Cannot add yourself to the usernames list. You are automatically included as the creator.",
  "data": null
}
```

### Notes

- Source: `src/chat/room.rs`
- Private rooms are idempotent: creating a room with the same user returns the existing room with `200 OK`
- Group rooms always create a new room (no duplicate check)
- The creator of a group room is automatically assigned the `admin` role; all other initial members are assigned `member`

---

## GET /api/v1/chat/rooms

List all chat rooms the authenticated user is a member of, with optional last-message preview.

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

| Field | Type | Description |
|---|---|---|
| `data.rooms` | array | Array of rooms, ordered by creation date descending |

Each room object:

| Field | Type | Description |
|---|---|---|
| `id` | string (UUID v7) | Unique room identifier |
| `name` | string \| null | Room name (`null` for private chats) |
| `created_by` | string (UUID v7) | Creator's user ID |
| `created_at` | string (RFC 3339) | Creation timestamp |
| `is_group` | boolean | Whether this is a group room |
| `role` | string | Authenticated user's role in this room: `admin` or `member` |
| `member_count` | integer | Total number of members in the room |
| `members` | array[string] | Member user UUIDs |
| `last_message` | object \| null | Most recent message preview, or `null` if no messages exist |

The `last_message` object:

| Field | Type | Description |
|---|---|---|
| `id` | string (UUID v7) | Message ID |
| `content` | string | Message content |
| `sender_username` | string | Username of the sender |
| `created_at` | string (RFC 3339) | Message timestamp |

### Notes

- Source: `src/chat/room.rs`
- Rooms are returned in descending order by `created_at`
- `last_message` is fetched via a lateral join and is `null` for rooms with no messages

---

## GET /api/v1/chat/rooms/{room_id}

Get detailed information about a specific room, including member profiles with roles and join timestamps.

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |

### Request

No request body.

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `data.id` | string (UUID v7) | Unique room identifier |
| `data.name` | string \| null | Room name (`null` for private chats) |
| `data.created_by` | string (UUID v7) | Creator's user ID |
| `data.created_at` | string (RFC 3339) | Creation timestamp |
| `data.is_group` | boolean | Whether this is a group room |
| `data.member_count` | integer | Total number of members |
| `data.members` | array | Array of member profiles with details |

Each member object:

| Field | Type | Description |
|---|---|---|
| `user_id` | string (UUID v7) | User ID |
| `username` | string | Username |
| `nickname` | string \| null | Display name |
| `role` | string | Role in this room: `admin` or `member` |
| `joined_at` | string (RFC 3339) | Timestamp when the user joined |

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

**Room not found**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Room not found.",
  "data": null
}
```

### Notes

- Source: `src/chat/room.rs`
- The room creator is always granted the default role (`member`) in private rooms

---

## POST /api/v1/chat/rooms/{room_id}/members

Add new members to an existing room. Only available for group rooms; the requester must have the `admin` role.

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |

### Request

```json
{
  "usernames": ["dave", "eve"]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `usernames` | array[string] | Yes | Usernames of users to add (must not be empty) |

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `data.added` | array | Array of successfully added members |
| `data.added[].user_id` | string (UUID v7) | User ID of the added member |
| `data.added[].username` | string | Username of the added member |
| `data.added[].joined_at` | string (RFC 3339) | Join timestamp |
| `data.added_count` | integer | Number of members successfully added |

Users who are already room members are silently skipped (not counted in `added_count`).

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

**Only room admins can add members**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Only room admins can add members.",
  "data": null
}
```

**Empty usernames list**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "usernames list cannot be empty.",
  "data": null
}
```

**User not found**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "User not found: nonexistent_user",
  "data": null
}
```

### Notes

- Source: `src/chat/member.rs`
- Only available for group rooms (the endpoint itself does not check `is_group`, but non-group rooms have no admins and will return 403)
- Already-members are silently skipped

---

## GET /api/v1/chat/rooms/{room_id}/members

List all members of a room with their roles and join timestamps.

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |

### Request

No request body.

### Response

#### Success

- HTTP Status: `200 OK`

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

| Field | Type | Description |
|---|---|---|
| `data.members` | array | Array of member profiles, ordered by `joined_at` ascending |
| `data.count` | integer | Total number of members |

Each member object:

| Field | Type | Description |
|---|---|---|
| `user_id` | string (UUID v7) | User ID |
| `username` | string | Username |
| `nickname` | string \| null | Display name |
| `role` | string | Role in this room: `admin` or `member` |
| `joined_at` | string (RFC 3339) | Timestamp when the user joined |

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

- Source: `src/chat/member.rs`
- Members are ordered by `joined_at` ascending (oldest first)

---

## DELETE /api/v1/chat/rooms/{room_id}/members/{user_id}

Remove a member from a room. Two behaviors depending on the relationship between the requester and the target user:

| Relationship | Behavior | Requirements |
|---|---|---|
| Self-removal (`user_id` == requester) | **Leave** — the requester leaves the room | Any member can leave any room |
| Other user (`user_id` != requester) | **Kick** — the requester removes another user | Group room only; requester must have `admin` role |

### Path Parameters

| Parameter | Type | Description |
|---|---|---|
| `room_id` | string (UUID v7) | Room ID |
| `user_id` | string (UUID v7) | User ID to remove |

### Request

No request body.

### Response

#### Success - Leave

- HTTP Status: `200 OK`

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

**Last member leaves:** if the leaving user is the last remaining member, the room is deleted entirely.

- HTTP Status: `200 OK`

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

#### Success - Kick

- HTTP Status: `200 OK`

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

| Field (Leave) | Type | Description |
|---|---|---|
| `data.room_id` | string (UUID v7) | Room ID |
| `data.left_user_id` | string (UUID v7) | User who left |
| `data.room_deleted` | boolean | Whether the room was deleted because this was the last member |

| Field (Kick) | Type | Description |
|---|---|---|
| `data.room_id` | string (UUID v7) | Room ID |
| `data.removed_user_id` | string (UUID v7) | User who was kicked |

#### Errors

**Not a room member (requester not in the room)**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "You are not a member of this room.",
  "data": null
}
```

**Cannot kick members from a private chat**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Cannot kick members from a private chat.",
  "data": null
}
```

**Only room admins can remove members**

- HTTP Status: `403 Forbidden`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "FORBIDDEN_ERROR",
  "message": "Only room admins can remove members.",
  "data": null
}
```

**Target user is not a member**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Target user is not a member of this room.",
  "data": null
}
```

**Room not found**

- HTTP Status: `400 Bad Request`

```json
{
  "response_id": "019ef520-0c59-7902-9959-86975c24af39",
  "error_code": "BAD_REQUEST_ERROR",
  "message": "Room not found.",
  "data": null
}
```

### Notes

- Source: `src/chat/member.rs`
- Self-leave is always allowed for any room member
- Kick is only allowed in group rooms by members with the `admin` role
- If the leaving/kicked user is the last admin, a successor is auto-promoted:
  1. The room creator (if still a member and not the leaving user)
  2. The oldest remaining member by `joined_at`
- If the last member leaves, the room is deleted (cascade handles messages and memberships)

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

### Request

No request body.

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
