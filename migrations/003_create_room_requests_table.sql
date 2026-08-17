-- Room request (pre-negotiation for opening a private room).
-- A pending room request is the only way a private (1-on-1) room can be
-- created between two users who are not already in a room together.

CREATE TABLE IF NOT EXISTS room_requests (
    id           UUID PRIMARY KEY,
    sender_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message      TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'accepted', 'declined', 'expired', 'cancelled')),
    is_encrypted BOOLEAN NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ
);

-- Idempotency: only one pending request per (sender, receiver) pair.
CREATE UNIQUE INDEX IF NOT EXISTS uq_room_requests_pending_pair
    ON room_requests (sender_id, receiver_id) WHERE status = 'pending';

-- Frequent lookup: the pending-requests inbox of a receiver.
CREATE INDEX IF NOT EXISTS idx_room_requests_receiver_status
    ON room_requests (receiver_id, status);

-- Frequent lookup: the outbox of a sender + expiry sweeps.
CREATE INDEX IF NOT EXISTS idx_room_requests_sender_status
    ON room_requests (sender_id, status, created_at);
