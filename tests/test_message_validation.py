"""Integration tests for WS message content validation.

Replaces the Rust unit tests that were removed from
src/websocket/handler.rs (validate_message_content).

Tests empty, whitespace-only, oversized, and valid content
at the boundary via the live WebSocket endpoint.
"""
import json
import time
import uuid

import pytest
import requests
import websocket


def _unique(name: str) -> str:
    suffix = uuid.uuid4().hex[:8]
    return f"{name}_{suffix}"


def _register_user(
    session: requests.Session, base_url: str, *, prefix: str = "valuser"
) -> dict:
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
    session: requests.Session, base_url: str, *, prefix: str = "valuser"
) -> tuple[str, dict]:
    user = _register_user(session, base_url, prefix=prefix)
    login_resp = session.post(
        f"{base_url}/api/v1/user/login",
        json={"username": user["username"], "password": "P@ssw0rd!"},
    )
    assert login_resp.status_code == 200, login_resp.text
    body = login_resp.json()
    assert body["error_code"] == "OK"
    return body["data"]["token"], user


def _auth(token: str) -> dict:
    return {"Authorization": f"Bearer {token}"}


def _ws_connect(ws_base: str, token: str, timeout: int = 10) -> websocket.WebSocket:
    return websocket.create_connection(
        f"{ws_base}/websocket",
        header={"authorization": f"Bearer {token}"},
        timeout=timeout,
    )


def _recv(ws: websocket.WebSocket, timeout: int = 10) -> dict:
    ws.settimeout(timeout)
    raw = ws.recv()
    return json.loads(raw)


def _recv_until(
    ws: websocket.WebSocket, expected_type: str, timeout: int = 10
) -> dict:
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


class TestMessageValidation:
    """WS message content validation — replaces Rust #[cfg(test)] unit tests."""

    @staticmethod
    def _send_and_expect(
        ws: websocket.WebSocket,
        room_id: str,
        content: str,
        expect_success: bool,
    ) -> dict | None:
        """Send a send_message WS frame and return the response."""
        ws.send(
            json.dumps({
                "type": "send_message",
                "data": {"room_id": room_id, "content": content},
            })
        )
        if expect_success:
            return _recv_until(ws, "message_sent", timeout=10)
        msg = _recv(ws, timeout=10)
        assert msg["type"] == "error", (
            f"Expected 'error' for invalid content, got '{msg['type']}': {msg}"
        )
        return msg

    @pytest.fixture(autouse=True)
    def _setup_room(self, session: requests.Session, base_url: str) -> None:
        """Set up a private room between two users for the duration of each test."""
        token_a, user_a = _register_and_login(session, base_url, prefix="valfa")
        token_b, user_b = _register_and_login(session, base_url, prefix="valfb")

        resp = session.post(
            f"{base_url}/api/v1/chat/rooms",
            json={"username": user_b["username"]},
            headers=_auth(token_a),
        )
        assert resp.status_code == 201, resp.text
        room_id = resp.json()["data"]["id"]

        ws_base = base_url.replace("http", "ws")
        ws = _ws_connect(ws_base, token_a)
        try:
            _recv(ws)  # consume "connected"
            yield (ws, token_a, user_a, room_id)
        finally:
            ws.close()

    # ── cases ────────────────────────────────────────────────────────

    def test_valid_content_short(self, _setup_room) -> None:
        """Short valid message → message_sent ack."""
        ws, _token, _user, room_id = _setup_room
        ack = self._send_and_expect(ws, room_id, "hello", expect_success=True)
        assert ack["data"]["content"] == "hello"
        assert ack["data"]["room_id"] == room_id

    def test_valid_content_100(self, _setup_room) -> None:
        """100-character message → message_sent ack."""
        ws, _token, _user, room_id = _setup_room
        content = "a" * 100
        ack = self._send_and_expect(ws, room_id, content, expect_success=True)
        assert ack["data"]["content"] == content

    def test_valid_content_boundary_65536(self, _setup_room) -> None:
        """65536-byte message (boundary) → message_sent ack."""
        ws, _token, _user, room_id = _setup_room
        content = "a" * 65536
        ack = self._send_and_expect(ws, room_id, content, expect_success=True)
        assert len(ack["data"]["content"]) == 65536

    def test_empty_content(self, _setup_room) -> None:
        """Empty string → server replies with error."""
        ws, _token, _user, room_id = _setup_room
        msg = self._send_and_expect(ws, room_id, "", expect_success=False)
        assert "empty" in msg["data"]["message"].lower()

    def test_whitespace_only_content(self, _setup_room) -> None:
        """Whitespace-only content → server replies with error."""
        ws, _token, _user, room_id = _setup_room
        msg = self._send_and_expect(ws, room_id, "   ", expect_success=False)
        assert "empty" in msg["data"]["message"].lower()

    def test_content_too_long(self, _setup_room) -> None:
        """65537-byte content (1 over boundary) → server replies with error."""
        ws, _token, _user, room_id = _setup_room
        content = "a" * 65537
        msg = self._send_and_expect(ws, room_id, content, expect_success=False)
        assert "65536" in msg["data"]["message"]
