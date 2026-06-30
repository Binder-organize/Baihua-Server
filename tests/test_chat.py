import time
import uuid
from datetime import datetime

import requests


# ── helpers (module-level, mirroring test_user.py style) ──────────────

def _unique(name: str) -> str:
    """Generate a unique username/email to avoid cross-run conflicts."""
    suffix = uuid.uuid4().hex[:8]
    return f"{name}_{suffix}"


def _register_user(
    session: requests.Session, base_url: str, *, prefix: str = "chatuser"
) -> dict:
    """Register a user with a unique name and return the user data dict."""
    uname = _unique(prefix)
    resp = session.post(
        f"{base_url}/api/v1/user/register",
        json={
            "username": uname,
            "email": f"{uname}@example.com",
            "password": "P@ssw0rd!",
        },
    )
    assert resp.status_code == 201, resp.text
    return resp.json()["data"]["user"]


def _register_and_login(
    session: requests.Session, base_url: str, *, prefix: str = "chatuser"
) -> tuple[str, dict]:
    """Register + login a unique user.  Returns (token, user_data)."""
    user = _register_user(session, base_url, prefix=prefix)
    login_resp = session.post(
        f"{base_url}/api/v1/user/login",
        json={"username": user["username"], "password": "P@ssw0rd!"},
    )
    assert login_resp.status_code == 200, login_resp.text
    body = login_resp.json()
    assert body["error_code"] == "OK"
    return body["data"]["token"], user


# ── shared state (class-level, survives across test methods) ──────────

class _State:
    """Mutable bag so class-level attributes are easily reassignable."""


# ── tests ─────────────────────────────────────────────────────────────

class ChatTest:
    """10 scenarios for the /api/v1/chat feature."""

    @staticmethod
    def _auth(token: str) -> dict:
        return {"Authorization": f"Bearer {token}"}

    # ── S1 ────────────────────────────────────────────────────────────

    def test_s1_create_private_room(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S1 – Create private room (happy path).

        Register user A (login) and user B (register only).  A creates a
        room with B.  Expect 201, a UUID room id, and 2 members."""
        user_b = _register_user(session, base_url, prefix="chatb")
        token_a, user_a = _register_and_login(session, base_url, prefix="chata")

        resp = session.post(
            f"{base_url}/api/v1/chat/rooms",
            json={"username": user_b["username"]},
            headers=self._auth(token_a),
        )
        assert resp.status_code == 201, resp.text

        body = resp.json()
        assert body["error_code"] == "OK"
        data = body["data"]

        # room id must be a valid UUID
        room_id = data["id"]
        uuid.UUID(room_id)

        # exactly 2 members containing both A and B
        members = data["members"]
        assert len(members) == 2
        assert user_a["id"] in members
        assert user_b["id"] in members

        # persist for downstream tests
        ChatTest.room_id = room_id
        ChatTest.token_a = token_a
        ChatTest.user_a = user_a
        ChatTest.user_b = user_b

    # ── S2 ────────────────────────────────────────────────────────────

    def test_s2_idempotent_create(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S2 – Idempotent create.

        Sending the same room-creation request again returns 200 and the
        same room id."""
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms",
            json={"username": ChatTest.user_b["username"]},
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 200, resp.text
        assert resp.json()["data"]["id"] == ChatTest.room_id

    # ── S3 ────────────────────────────────────────────────────────────

    def test_s3_send_message(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S3 – Send a message to the room.

        Expect 201, sender_id matches A, content matches, room_id matches."""
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
            json={"content": "hello"},
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 201, resp.text

        data = resp.json()["data"]
        assert data["sender_id"] == ChatTest.user_a["id"]
        assert data["content"] == "hello"
        assert data["room_id"] == ChatTest.room_id

    # ── S4 ────────────────────────────────────────────────────────────

    def test_s4_get_messages_pagination(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S4 – Get messages with cursor-based pagination.

        1.  Send 2 more messages (total 3 with S3's) for a proper
            pagination run.
        2.  Page 1: limit=2 → 2 messages, has_more=True, next_cursor set.
            Messages must be newest-first.
        3.  Page 2: limit=2 & before=next_cursor → 1 message,
            has_more=False."""
        # send 2 additional messages so we have exactly 3 in the room
        for content in ("world", "again"):
            time.sleep(0.005)  # ensure distinct created_at timestamps
            resp = session.post(
                f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
                json={"content": content},
                headers=self._auth(ChatTest.token_a),
            )
            assert resp.status_code == 201, resp.text

        # page 1 ───────────────────────────────────────────────────────
        resp = session.get(
            f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
            params={"limit": 2},
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 200, resp.text
        body = resp.json()
        data = body["data"]
        messages = data["messages"]

        assert len(messages) == 2
        assert data["has_more"] is True
        assert data["next_cursor"] is not None

        # newest-first order
        t0 = datetime.fromisoformat(messages[0]["created_at"])
        t1 = datetime.fromisoformat(messages[1]["created_at"])
        assert t0 >= t1, f"Expected newest first, but {t0} < {t1}"

        next_cursor = data["next_cursor"]

        # page 2 ──────────────────────────────────────────────────────
        resp = session.get(
            f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
            params={"limit": 2, "before": next_cursor},
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 200, resp.text
        body = resp.json()
        data = body["data"]

        assert len(data["messages"]) == 1
        assert data["has_more"] is False

    # ── S5 ────────────────────────────────────────────────────────────

    def test_s5_unauthorized_access(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S5 – Unauthorized access.

        - No token on GET /rooms → 401 AUTHENTICATION_ERROR
        - Invalid token on POST /rooms → 401
        - No token on POST /rooms/{uuid}/messages → 401"""

        # no auth header → GET rooms
        resp = session.get(f"{base_url}/api/v1/chat/rooms")
        assert resp.status_code == 401, resp.text
        assert resp.json()["error_code"] == "AUTHENTICATION_ERROR"

        # invalid token → POST rooms
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms",
            json={"username": "someone"},
            headers=self._auth("invalid_token"),
        )
        assert resp.status_code == 401, resp.text
        assert resp.json()["error_code"] == "AUTHENTICATION_ERROR"

        # no auth → POST messages
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms/{uuid.uuid4()}/messages",
            json={"content": "x"},
        )
        assert resp.status_code == 401, resp.text
        assert resp.json()["error_code"] == "AUTHENTICATION_ERROR"

    # ── S6 ────────────────────────────────────────────────────────────

    def test_s6_not_room_member(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S6 – Non-member cannot access the room.

        Register user C, then try to send and read messages in the A-B
        room.  Both must return 403 FORBIDDEN_ERROR."""
        token_c, _user_c = _register_and_login(session, base_url, prefix="chatc")

        # send message as C
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
            json={"content": "intruder"},
            headers=self._auth(token_c),
        )
        assert resp.status_code == 403, resp.text
        assert resp.json()["error_code"] == "FORBIDDEN_ERROR"

        # get messages as C
        resp = session.get(
            f"{base_url}/api/v1/chat/rooms/{ChatTest.room_id}/messages",
            headers=self._auth(token_c),
        )
        assert resp.status_code == 403, resp.text
        assert resp.json()["error_code"] == "FORBIDDEN_ERROR"

    # ── S7 ────────────────────────────────────────────────────────────

    def test_s7_target_user_not_found(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S7 – Creating a room with a non-existent username → 400."""
        resp = session.post(
            f"{base_url}/api/v1/chat/rooms",
            json={"username": "nonexistent_user_12345"},
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 400, resp.text
        assert resp.json()["error_code"] == "BAD_REQUEST_ERROR"

    # ── S8 ────────────────────────────────────────────────────────────

    def test_s8_list_rooms(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S8 – List rooms for authenticated user.

        The room created in S1 must appear in the list."""
        resp = session.get(
            f"{base_url}/api/v1/chat/rooms",
            headers=self._auth(ChatTest.token_a),
        )
        assert resp.status_code == 200, resp.text

        body = resp.json()
        rooms = body["data"]["rooms"]
        assert isinstance(rooms, list)
        room_ids = [r["id"] for r in rooms]
        assert ChatTest.room_id in room_ids

    # ── S9 ────────────────────────────────────────────────────────────

    def test_s9_list_users(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S9 – List users (no auth required).

        Returns a list; each entry has id, username, email."""
        resp = session.get(f"{base_url}/api/v1/user/list")
        assert resp.status_code == 200, resp.text

        body = resp.json()
        users = body["data"]["users"]
        assert isinstance(users, list)
        assert len(users) > 0
        for user in users:
            assert "id" in user
            assert "username" in user
            assert "email" in user

    # ── S10 ───────────────────────────────────────────────────────────

    def test_s10_adjacent_surface_regression(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S10 – Adjacent surface regression.

        Sanity-check that unrelated endpoints still work after chat
        changes."""
        # greet
        resp = session.get(f"{base_url}/greet")
        assert resp.status_code == 200

        # health
        resp = session.get(f"{base_url}/health")
        assert resp.status_code == 200

        # register
        uname = _unique("s10reg")
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={
                "username": uname,
                "email": f"{uname}@example.com",
                "password": "P@ssw0rd!",
            },
        )
        assert resp.status_code == 201, resp.text

        # login (full flow via helper)
        token, _user = _register_and_login(session, base_url, prefix="s10log")
        assert token is not None
