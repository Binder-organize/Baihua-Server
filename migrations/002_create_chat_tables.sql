-- Chat tables (rooms, room_members, messages).
-- Includes encrypted-chat columns and member roles from day one:
--   - rooms.is_encrypted marks one-on-one encrypted sessions
--   - messages.encrypted_content BYTEA is NULL for regular messages
--   - messages.content is nullable (NULL for encrypted messages)
--   - room_members.role distinguishes group admins from members

CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY,
    name TEXT,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_group BOOLEAN NOT NULL DEFAULT false,
    is_encrypted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS room_members (
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    role TEXT NOT NULL DEFAULT 'member',
    PRIMARY KEY (room_id, user_id),
    CONSTRAINT chk_room_member_role CHECK (role IN ('admin', 'member'))
);

CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL,
    content TEXT,
    encrypted_content BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Messages use keyset pagination: WHERE room_id = $1
-- ORDER BY created_at DESC, id DESC. Index matches the (room_id, created_at, id)
-- access pattern used by src/chat/message.rs.
CREATE INDEX IF NOT EXISTS idx_messages_room_created
    ON messages (room_id, created_at DESC, id DESC);

-- Rooms of a user: list_rooms joins room_members on user_id (src/chat/room.rs).
CREATE INDEX IF NOT EXISTS idx_room_members_user ON room_members (user_id);

-- Room lists are sorted by created_at DESC (src/chat/room.rs).
CREATE INDEX IF NOT EXISTS idx_rooms_created_at ON rooms (created_at DESC);