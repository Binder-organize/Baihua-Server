# Changelog

All notable changes to Baihua Server will be documented in this file.

## [0.1.3] - 2026-07-26

### Added

- **End-to-end encrypted private chat** — private rooms can now be promoted to encrypted sessions. A full handshake protocol (`encrypt_request` / `encrypt_accept` / `encrypt_ready`) exchanges X25519 ephemeral public keys over the WebSocket; messages are encrypted with AES-256-GCM and stored as `encrypted_content BYTEA` in the database. The server never touches plaintext — it relays public-key material and ciphertext only.
- **Encrypted session lifecycle** — session states (`pending_states`, `ready_states`, `active_sessions`) are tracked in `ConnectionManager` memory. Both users must send `encrypt_ready` before the session becomes active.
- **30-second grace period** — when either user disconnects, a 30-second timeout starts. If the user reconnects within that window, the session resumes; otherwise messages are purged and the session is terminated.
- **Grace period reconnect flow** — on reconnect, `cancel_grace_periods_for_user` cancels pending timeouts and `check_expired_session_on_connect` detects existing encrypted rooms to offer session resumption.
- **`encrypt_session_ended` / `encrypt_session_expired` server events** — clients are notified when a session ends (partner offline) or expires (both online but no active session).
- **`base64` crate** — added as a dependency for ciphertext encoding.
- **Encrypted room E2E test suite** — 16 tests (E1–E16) covering handshake, message relay, grace period recovery, timeout cleanup, reconnect flow, and coexistence with non-encrypted rooms.

### Changed

- **`CreateRoomRequest`** — added optional `is_encrypted` field to create encrypted private rooms.
- **Room creation query** — `create_private_room` now filters `is_encrypted = false` to avoid returning existing encrypted rooms as regular rooms.
- **Room list response** — last-message preview is omitted for encrypted rooms (`content` is always `null`).
- **Message history response** — encrypted rooms return `ciphertext` (base64-encoded) instead of `content` in `GET /api/v1/chat/rooms/{room_id}/messages`.
- **Broadcast dispatch** — encrypted messages are broadcast as `new_encrypted_message` and are never decrypted or inspected server-side.

## [0.1.2] - 2026-07-22

### Added

- **WebSocket message sending** — `send_message` client message type persisted and broadcast as `new_message`; `message_sent` acknowledgement sent to sender.
- **WebSocket rate limiting** — 30 application messages per 10 seconds per connection with `error` response on exceed.
- **WebSocket token re-validation** — periodic JWT and account-active check every 10 minutes; expired or deactivated sessions are disconnected.
- **Message content validation** — control character stripping (except newlines), empty rejection, 5000-byte maximum.
- **`NOT_FOUND_ERROR` error code** — `404` returned for room-not-found scenarios (replaces `400` in room detail; new in message listing).

### Changed

- **WebSocket authentication** — moved from query parameter (`?token=`) to `Authorization: Bearer <token>` header, consistent with the HTTP API.
- **Room detail endpoint** — `GET /api/v1/chat/rooms/{room_id}` now returns `404 NOT_FOUND_ERROR` instead of `400 BAD_REQUEST_ERROR` when the room does not exist.
- **Message listing endpoint** — `GET /api/v1/chat/rooms/{room_id}/messages` now validates room existence before checking membership, returning `404` for unknown rooms.
- **Broadcast channel capacity** — increased from 256 to 1024 to reduce lag under heavy load.
- **Typing indicator routing** — a user's own typing events are no longer echoed back to them.
- **Connection cleanup** — room subscriptions are explicitly cancelled on disconnect instead of relying on implicit drop.

### Fixed

- **Broadcast lag handling** — `RecvError::Lagged` is now logged and handled gracefully without breaking the forward task.

## [0.1.1] - 2026-07-14

### Added

- **Real-time messaging via WebSocket** — `GET /websocket` with JWT auth, auto-subscribe to rooms, `new_message` push on send.
- **Presence system** — `user_online` / `user_offline` events, multi-device connection counting.
- **Typing indicators** — `typing` event relayed to all room members.
- **Heartbeat** — protocol-level PING every 30s, dead connection detection.

## [0.1.0] - 2026-06-30

### Added

- **User registration and login** — create accounts with username/email/password (bcrypt hashed), authenticate via JWT (HS256, 24h expiry).
- **Private chat** — idempotent two-person room creation, send and retrieve messages with cursor-based pagination.
- **Group chat** — multi-person rooms with admin/member role model, member add/kick/leave with auto-promotion of last admin and auto-deletion of empty rooms.
- **Rate limiting (auth endpoints)** — sliding-window per-IP rate limiting on login (60/min) and register (30/min) with `X-Forwarded-For` / `X-Real-IP` detection; prevents brute-force and account creation abuse.
- **Request validation middleware** — body size limits (1 MB production, 10 MB development), Content-Type enforcement, JSON parsing validation, field-level checks for registration and login.
- **JWT authentication middleware** — Bearer token extraction, validation, expiry check, user-lookup guard for all chat and protected endpoints; tokens never logged in production.
- **Global error handling** — panic recovery, 404 catch-all, structured JSON error responses with UUID v7 `response_id`; internal error details hidden in production.
- **Request tracing middleware** — structured logging for every request.
- **Standard API response envelope** — uniform `{ response_id, error_code, message, data }` format across all endpoints.
- **Health check endpoint** — `GET /health` with PostgreSQL connectivity verification via `SELECT 1`.
- **Interactive console** — rustyline-based CLI (commands: `stop`, `help`) for server lifecycle management.
- **Dual-mode configuration** — development (human-readable logs, seed admin user, verbose errors) vs production (JSON logs, disabled console, generic error messages); config via `.env` with sensible defaults, auto-generation of `~/.baihua/config.toml`.
- **Docker deployment** — production and development Docker Compose profiles, database health check gating, Docker HEALTHCHECK.
- **CI workflows** — GitHub Actions for code quality checks (`cargo fmt`, `clippy`) and Docker-based integration tests.
- **Integration test suite** — Python pytest-based HTTP tests covering user, chat, and health endpoints.
