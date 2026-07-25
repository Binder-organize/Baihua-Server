-- Add encrypted chat support:
--   - is_encrypted flag on rooms to identify encrypted one-on-one sessions
--   - encrypted_content BYTEA column on messages (NULL for regular messages)
--   - content becomes nullable (NULL for encrypted messages)

ALTER TABLE rooms ADD COLUMN IF NOT EXISTS is_encrypted BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE messages ALTER COLUMN content DROP NOT NULL;
ALTER TABLE messages ADD COLUMN IF NOT EXISTS encrypted_content BYTEA;

CREATE INDEX IF NOT EXISTS idx_messages_encrypted_room
    ON messages (room_id) WHERE encrypted_content IS NOT NULL;
