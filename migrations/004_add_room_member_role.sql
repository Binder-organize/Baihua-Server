-- Add role column to room_members for group chat admin/member distinction.
ALTER TABLE room_members
    ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'member';

-- Backfill: set role='admin' for the user who created each group room.
-- Private rooms (is_group=false) remain with default 'member' role for all.
UPDATE room_members rm
SET role = 'admin'
FROM rooms r
WHERE rm.room_id = r.id
  AND rm.user_id = r.created_by
  AND r.is_group = true;

-- Enforce valid role values.
ALTER TABLE room_members
    ADD CONSTRAINT chk_room_member_role CHECK (role IN ('admin', 'member'));

-- Index for fast admin lookups during permission checks.
CREATE INDEX IF NOT EXISTS idx_room_members_role
    ON room_members (room_id, role);
