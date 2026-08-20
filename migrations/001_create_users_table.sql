CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    nickname TEXT,
    phone_number TEXT,
    bio TEXT,
    avatar TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    token_version BIGINT NOT NULL DEFAULT 0
);

-- User list queries filter on is_active = true and sort by created_at DESC
-- (src/user/list.rs). A partial index matches that exact access pattern.
CREATE INDEX IF NOT EXISTS idx_users_created_at
    ON users (created_at DESC) WHERE is_active = true;
