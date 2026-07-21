import json
import time
import uuid
from datetime import datetime

import pytest
import requests
import websocket


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


# ── WebSocket helpers ──────────────────────────────────────────────

def _ws_connect(ws_base: str, token: str, timeout: int = 10) -> websocket.WebSocket:
    """Create a WebSocket connection with JWT token in Authorization header."""
    return websocket.create_connection(
        f"{ws_base}/websocket",
        header={"authorization": f"Bearer {token}"},
        timeout=timeout,
    )


def _recv(ws: websocket.WebSocket, timeout: int = 10) -> dict:
    """Receive one WebSocket message, parse JSON, with timeout."""
    ws.settimeout(timeout)
    raw = ws.recv()
    return json.loads(raw)


def _recv_until(
    ws: websocket.WebSocket, expected_type: str, timeout: int = 10
) -> dict:
    """Consume messages until one of the expected type arrives. Skips others."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        remaining = deadline - time.time()
        if remaining <= 0:
            break
        ws.settimeout(remaining)
        try:
            raw = ws.recv()
            msg = json.loads(raw)
            if msg["type"] == expected_type:
                return msg
        except websocket.WebSocketTimeoutException:
            break
    pytest.fail(f"Did not receive '{expected_type}' within {timeout}s")


# ── shared state (class-level, survives across test methods) ──────────

class _State:
    """Mutable bag so class-level attributes are easily reassignable."""


# ── tests ─────────────────────────────────────────────────────────────

class ChatTest:
    """21 scenarios for the /api/v1/chat feature (HTTP + WebSocket)."""

    @staticmethod
    def _auth(token: str) -> dict:
        return {"Authorization": f"Bearer {token}"}

    # ── S1 ────────────────────────────────────────────────────────────

    def test_s1_create_private_room(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S1 – Create private room (happy path).

        Register user A (login) and user B (login).  A creates a
        room with B.  Expect 201, a UUID room id, and 2 members."""
        token_a, user_a = _register_and_login(session, base_url, prefix="chata")
        token_b, user_b = _register_and_login(session, base_url, prefix="chatb")

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
        ChatTest.token_b = token_b
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
        """S3 – Send a message to the room via WebSocket.

        Expect message_sent ack, sender_id matches A, content matches, room_id matches."""
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, ChatTest.token_a)
        try:
            _recv(ws)  # consume "connected"
            ws.send(
                json.dumps({
                    "type": "send_message",
                    "data": {
                        "room_id": ChatTest.room_id,
                        "content": "hello",
                    },
                })
            )
            msg = _recv_until(ws, "message_sent")
            data = msg["data"]
            assert data["sender_id"] == ChatTest.user_a["id"]
            assert data["content"] == "hello"
            assert data["room_id"] == ChatTest.room_id
        finally:
            ws.close()

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
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, ChatTest.token_a)
        try:
            _recv(ws)  # consume "connected"
            for content in ("world", "again"):
                time.sleep(0.005)  # ensure distinct created_at timestamps
                ws.send(
                    json.dumps({
                        "type": "send_message",
                        "data": {
                            "room_id": ChatTest.room_id,
                            "content": content,
                        },
                    })
                )
                _recv_until(ws, "message_sent")  # consume ack
        finally:
            ws.close()

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
        - Invalid token on POST /rooms → 401"""

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

    # ── S6 ────────────────────────────────────────────────────────────

    def test_s6_not_room_member(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S6 – Non-member cannot access the room.

        Register user C, then try to read messages in the A-B
        room.  Must return 403 FORBIDDEN_ERROR."""
        token_c, _user_c = _register_and_login(session, base_url, prefix="chatc")

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
        """S9 – List users (auth required).

        Returns a list; each entry has id, username, email."""
        resp = session.get(
            f"{base_url}/api/v1/user/list",
            headers=self._auth(ChatTest.token_a),
        )
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

    # ── S11 ────────────────────────────────────────────────────────────

    def test_s11_websocket_fresh_user_empty_rooms(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S11 – Fresh user connects via WS.  `connected.rooms` must be []."""
        token, _user = _register_and_login(session, base_url, prefix="s11")
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token)
        try:
            msg = _recv(ws)
            assert msg["type"] == "connected"
            assert msg["data"]["rooms"] == []
        finally:
            ws.close()

    # ── S12 ────────────────────────────────────────────────────────────

    def test_s12_websocket_invalid_token(
        self, base_url: str
    ) -> None:
        """S12 – Invalid JWT on WS upgrade → HTTP 401 before upgrade."""
        ws_url = base_url.replace("http", "ws") + "/websocket?token=garbage"
        with pytest.raises(websocket.WebSocketBadStatusException) as exc_info:
            websocket.create_connection(ws_url, timeout=5)
        assert exc_info.value.status_code == 401

    # ── S13 ────────────────────────────────────────────────────────────

    def test_s13_websocket_typing_missing_room_id(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S13 – Typing without room_id → server replies with `error`."""
        token, _user = _register_and_login(session, base_url, prefix="s13")
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token)
        try:
            _recv(ws)
            ws.send(json.dumps({"type": "typing"}))
            msg = _recv(ws)
            assert msg["type"] == "error"
            assert "room_id" in msg["data"]["message"]
        finally:
            ws.close()

    # ── S14 ────────────────────────────────────────────────────────────

    def test_s14_websocket_missing_type_field(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S14 – WS message without `type` field → server replies with `error`."""
        token, _user = _register_and_login(session, base_url, prefix="s14")
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token)
        try:
            _recv(ws)
            ws.send(json.dumps({"room_id": "anything"}))
            msg = _recv(ws)
            assert msg["type"] == "error"
            assert "type" in msg["data"]["message"]
        finally:
            ws.close()

    # ── S15 ────────────────────────────────────────────────────────────

    def test_s15_websocket_unknown_message_type(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S15 – Unknown WS message type → server replies with `error`."""
        token, _user = _register_and_login(session, base_url, prefix="s15")
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token)
        try:
            _recv(ws)
            ws.send(json.dumps({"type": "foobar"}))
            msg = _recv(ws)
            assert msg["type"] == "error"
            assert "foobar" in msg["data"]["message"]
        finally:
            ws.close()

    # ── S16 ────────────────────────────────────────────────────────────

    def test_s16_websocket_pong_backward_compat(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S16 – `pong` is silently ignored (legacy); connection stays alive."""
        token, _user = _register_and_login(session, base_url, prefix="s16")
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token)
        try:
            _recv(ws)
            ws.send(json.dumps({"type": "pong"}))
            # Connection should still be usable after pong
            ws.send(json.dumps({"type": "foobar"}))
            msg = _recv(ws)
            assert msg["type"] == "error"  # foobar rejected, but not pong
        finally:
            ws.close()

    # ── S17 ────────────────────────────────────────────────────────────

    def test_s17_websocket_rooms_listed_on_connect(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S17 – User with rooms connects → `connected.rooms` lists them."""
        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, ChatTest.token_a)
        try:
            msg = _recv(ws)
            assert msg["type"] == "connected"
            assert ChatTest.room_id in msg["data"]["rooms"]
        finally:
            ws.close()

    # ── S18 ────────────────────────────────────────────────────────────

    def test_s18_websocket_new_message_broadcast(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S18 – WS send_message → subscriber receives `new_message`."""
        ws_base = base_url.replace("http", "ws")
        ws_b = _ws_connect(ws_base, ChatTest.token_b)
        try:
            _recv(ws_b)

            ws_a = _ws_connect(ws_base, ChatTest.token_a)
            try:
                _recv(ws_a)

                ws_a.send(
                    json.dumps({
                        "type": "send_message",
                        "data": {
                            "room_id": ChatTest.room_id,
                            "content": "hello from S18",
                        },
                    })
                )

                # A receives ack
                _recv_until(ws_a, "message_sent")

                # B receives broadcast
                msg = _recv_until(ws_b, "new_message")
                assert msg["data"]["room_id"] == ChatTest.room_id
                assert msg["data"]["sender_id"] == ChatTest.user_a["id"]
                assert msg["data"]["content"] == "hello from S18"
            finally:
                ws_a.close()
        finally:
            ws_b.close()

    # ── S19 ────────────────────────────────────────────────────────────

    def test_s19_websocket_typing_indicator(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S19 – User sends `typing` → other room member receives indicator."""
        ws_base = base_url.replace("http", "ws")

        ws_b = _ws_connect(ws_base, ChatTest.token_b)
        try:
            _recv(ws_b)

            ws_a = _ws_connect(ws_base, ChatTest.token_a)
            try:
                _recv(ws_a)

                ws_a.send(
                    json.dumps({"type": "typing", "room_id": ChatTest.room_id})
                )

                msg = _recv_until(ws_b, "typing")
                assert msg["data"]["room_id"] == ChatTest.room_id
                assert msg["data"]["user_id"] == ChatTest.user_a["id"]
                assert msg["data"]["username"] == ChatTest.user_a["username"]
                assert msg["data"]["typing"] is True
            finally:
                ws_a.close()
        finally:
            ws_b.close()

    # ── S20 ────────────────────────────────────────────────────────────

    def test_s20_websocket_user_online_offline(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S20 – A connects → B sees `user_online`; A disconnects → B sees `user_offline`."""
        ws_base = base_url.replace("http", "ws")

        ws_b = _ws_connect(ws_base, ChatTest.token_b)
        try:
            _recv(ws_b)

            ws_a = _ws_connect(ws_base, ChatTest.token_a)
            try:
                _recv(ws_a)

                msg = _recv_until(ws_b, "user_online")
                assert msg["data"]["user_id"] == ChatTest.user_a["id"]
                assert msg["data"]["username"] == ChatTest.user_a["username"]
            finally:
                ws_a.close()
                time.sleep(0.3)

            msg = _recv_until(ws_b, "user_offline")
            assert msg["data"]["user_id"] == ChatTest.user_a["id"]
            assert msg["data"]["username"] == ChatTest.user_a["username"]
        finally:
            ws_b.close()

    # ── S21 ────────────────────────────────────────────────────────────

    def test_s21_websocket_leave_stops_messages(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S21 – User leaves room → WS stops receiving new_message for that room.

        Flow:
          1. B connects WS (auto-subscribed to A-B room).
          2. A connects WS.
          3. A sends msg → B receives `new_message` (subscription active).
          4. B leaves the room via HTTP DELETE.
          5. A sends another msg → B must NOT receive it (cancel done).
        """
        ws_base = base_url.replace("http", "ws")
        ws_b = _ws_connect(ws_base, ChatTest.token_b)
        try:
            _recv(ws_b)

            ws_a = _ws_connect(ws_base, ChatTest.token_a)
            try:
                _recv(ws_a)

                # Confirm subscription works before leave
                ws_a.send(
                    json.dumps({
                        "type": "send_message",
                        "data": {
                            "room_id": ChatTest.room_id,
                            "content": "before leave",
                        },
                    })
                )
                _recv_until(ws_a, "message_sent")  # consume ack
                _recv_until(ws_b, "new_message")  # consume broadcast

                # B leaves the room
                resp = session.delete(
                    f"{base_url}/api/v1/chat/rooms/"
                    f"{ChatTest.room_id}/members/{ChatTest.user_b['id']}",
                    headers=self._auth(ChatTest.token_b),
                )
                assert resp.status_code == 200, resp.text
                time.sleep(0.5)  # allow tokio to cancel the forward task

                # A sends another message
                ws_a.send(
                    json.dumps({
                        "type": "send_message",
                        "data": {
                            "room_id": ChatTest.room_id,
                            "content": "after leave",
                        },
                    })
                )
                _recv_until(ws_a, "message_sent")  # consume ack

                # B should NOT receive this message
                ws_b.settimeout(3)
                try:
                    while True:
                        raw = ws_b.recv()
                        m = json.loads(raw)
                        if m["type"] == "new_message":
                            pytest.fail(
                                f"Received new_message after leave: {m}"
                            )
                except websocket.WebSocketTimeoutException:
                    pass  # expected — no message arrived
            finally:
                ws_a.close()
        finally:
            ws_b.close()

    # ── S22 ────────────────────────────────────────────────────────────

    def test_s22_typing_not_sent_to_sender(
        self, session: requests.Session, base_url: str
    ) -> None:
        """S22 – Typing indicator is not echoed back to the sender.

        Flow:
          1. A and B both connect WS to the shared room.
          2. A sends a typing event.
          3. B receives the typing indicator.
          4. A must NOT receive their own typing indicator.
        """
        ws_base = base_url.replace("http", "ws")

        ws_a = _ws_connect(ws_base, ChatTest.token_a)
        try:
            _recv(ws_a)  # consume "connected"

            ws_b = _ws_connect(ws_base, ChatTest.token_b)
            try:
                _recv(ws_b)  # consume "connected"

                # B may have received A's user_online — consume it
                _recv_until(ws_b, "connected", timeout=3)

                # Send typing from A
                ws_a.send(
                    json.dumps({"type": "typing", "room_id": ChatTest.room_id})
                )

                # B must receive the typing indicator
                msg = _recv_until(ws_b, "typing", timeout=5)
                assert msg["data"]["user_id"] == ChatTest.user_a["id"]
                assert msg["data"]["username"] == ChatTest.user_a["username"]
                assert msg["data"]["typing"] is True

                # A must NOT receive the typing indicator
                ws_a.settimeout(3)
                try:
                    while True:
                        raw = ws_a.recv()
                        m = json.loads(raw)
                        if m["type"] == "typing":
                            pytest.fail(
                                f"A received their own typing indicator: {m}"
                            )
                except websocket.WebSocketTimeoutException:
                    pass  # expected — no typing broadcast to sender
            finally:
                ws_b.close()
        finally:
            ws_a.close()
