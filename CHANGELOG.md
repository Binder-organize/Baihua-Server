# Changelog

All notable changes to Baihua Server will be documented in this file.

## [0.1.1] - 2026-07-14

### Added

- **Real-time messaging via WebSocket** — `GET /websocket` with JWT auth, auto-subscribe to rooms, `new_message` push on send
- **Presence system** — `user_online` / `user_offline` events, multi-device connection counting
- **Typing indicators** — `typing` event relayed to all room members
- **Heartbeat** — protocol-level PING every 30s, dead connection detection

## [0.1.0] - 2026-06-30

### Added

- **User registration and login** — create accounts with username/email/password (bcrypt hashed), authenticate via JWT (HS256, 24h expiry)
- **Private chat** — idempotent two-person room creation, send and retrieve messages with cursor-based pagination
- **Group chat** — multi-person rooms with admin/member role model, member add/kick/leave with auto-promotion of last admin and auto-deletion of empty rooms
- **Rate limiting (auth endpoints)** — sliding-window per-IP rate limiting on login (60/min) and register (30/min) with `X-Forwarded-For` / `X-Real-IP` detection; prevents brute-force and account creation abuse
- **Request validation middleware** — body size limits (1 MB production, 10 MB development), Content-Type enforcement, JSON parsing validation, field-level checks for registration and login
- **JWT authentication middleware** — Bearer token extraction, validation, expiry check, user-lookup guard for all chat and protected endpoints; tokens never logged in production
- **Global error handling** — panic recovery, 404 catch-all, structured JSON error responses with UUID v7 `response_id`; internal error details hidden in production
- **Request tracing middleware** — structured logging for every request
- **Standard API response envelope** — uniform `{ response_id, error_code, message, data }` format across all endpoints
- **Health check endpoint** — `GET /health` with PostgreSQL connectivity verification via `SELECT 1`
- **Interactive console** — rustyline-based CLI (commands: `stop`, `help`) for server lifecycle management
- **Dual-mode configuration** — development (human-readable logs, seed admin user, verbose errors) vs production (JSON logs, disabled console, generic error messages); config via `.env` with sensible defaults, auto-generation of `~/.baihua/config.toml`
- **Docker deployment** — production and development Docker Compose profiles, database health check gating, Docker HEALTHCHECK
- **CI workflows** — GitHub Actions for code quality checks (`cargo fmt`, `clippy`) and Docker-based integration tests
- **Integration test suite** — Python pytest-based HTTP tests covering user, chat, and health endpoints
