import uuid

import requests


def _unique(name: str) -> str:
    """Generate a unique username/email to avoid cross-run conflicts."""
    suffix = uuid.uuid4().hex[:8]
    return f"{name}_{suffix}"


class TestRegister:
    VALIDATION_FIELDS = [
        ("username", {}, "Username"),
        ("email", {"username": "testuser"}, "Email"),
        ("password", {"username": "testuser", "email": "tests@example.com"}, "Password"),
    ]

    def test_missing_username(self, session: requests.Session, base_url: str):
        """Should reject register with empty body."""
        resp = session.post(f"{base_url}/api/v1/user/register", json={})
        assert resp.status_code == 400

        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "username is required" in body["message"].lower()

    def test_missing_email(self, session: requests.Session, base_url: str):
        """Should reject register with only username."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": "testuser"},
        )
        assert resp.status_code == 400

        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "email is required" in body["message"].lower()

    def test_missing_password(self, session: requests.Session, base_url: str):
        """Should reject register without password."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": "testuser", "email": "tests@example.com"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "password is required" in body["message"].lower()

    def test_register_success(self, session: requests.Session, base_url: str):
        """Should create a new user and return 201."""
        uname = _unique("regtest")
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={
                "username": uname,
                "email": f"{uname}@example.com",
                "password": "P@ssw0rd!",
            },
        )
        assert resp.status_code == 201, resp.text

        body = resp.json()
        assert body["code"] == "SUCCESS"
        assert body["data"]["user"]["username"] == uname
        assert body["data"]["user"]["email"] == f"{uname}@example.com"

    def test_duplicate_username(self, session: requests.Session, base_url: str):
        """Should reject registration with an existing username."""
        uname = _unique("dupuser")
        # Register the first time
        session.post(
            f"{base_url}/api/v1/user/register",
            json={
                "username": uname,
                "email": f"{uname}@example.com",
                "password": "P@ssw0rd!",
            },
        )
        # Register with the same username but different email
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={
                "username": uname,
                "email": f"{uname}_other@example.com",
                "password": "P@ssw0rd!",
            },
        )
        assert resp.status_code == 400
        assert resp.json()["code"] == "VALIDATION_ERROR"

    def test_empty_username_string(self, session: requests.Session, base_url: str):
        """Should reject register with empty username string."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": "", "email": "a@b.com", "password": "P@ssw0rd!"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "username is required" in body["message"].lower()

    def test_empty_email_string(self, session: requests.Session, base_url: str):
        """Should reject register with empty email string."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": "testuser", "email": "", "password": "P@ssw0rd!"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "email is required" in body["message"].lower()

    def test_empty_password_string(self, session: requests.Session, base_url: str):
        """Should reject register with empty password string."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": "testuser", "email": "a@b.com", "password": ""},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "password is required" in body["message"].lower()


class TestLogin:
    def test_missing_username(self, session: requests.Session, base_url: str):
        """Login without username should fail validation."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"password": "somepass"},
        )
        assert resp.status_code == 400

        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "username is required" in body["message"].lower()

    def test_missing_password(self, session: requests.Session, base_url: str):
        """Login without password should fail validation."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": "someuser"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "password is required" in body["message"].lower()

    def test_login_without_email_succeeds_validation(self, session: requests.Session, base_url: str):
        """
        Login should NOT require email.

        This was the bug that motivated the middleware refactor:
        validate_user() required BOTH username AND email on all /api/v1/user/*
        routes, which broke login. After the split, validate_login should
        only check username + password and should *not* reject requests
        that omit email.
        """
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": "nonexistent", "password": "somepass"},
        )
        # If validation passes but credentials are wrong → 401, not 400
        assert resp.status_code == 401, (
            f"Expected 401 (wrong credentials, not 400 for missing email). "
            f"Got {resp.status_code}: {resp.text}"
        )

    def test_empty_body(self, session: requests.Session, base_url: str):
        """Login with empty body should fail validation."""
        resp = session.post(f"{base_url}/api/v1/user/login", json={})
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "username is required" in body["message"].lower()

    def test_empty_username_string(self, session: requests.Session, base_url: str):
        """Should reject login with empty username string."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": "", "password": "somepass"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "username is required" in body["message"].lower()

    def test_empty_password_string(self, session: requests.Session, base_url: str):
        """Should reject login with empty password string."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": "someuser", "password": ""},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "password is required" in body["message"].lower()

    def test_login_wrong_credentials(self, session: requests.Session, base_url: str):
        """Should return 401 for wrong password."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": "nobody_will_register_this", "password": "wrongpass"},
        )
        assert resp.status_code == 401

        body = resp.json()
        assert body["code"] == "AUTHENTICATION_ERROR"

class TestValidationMiddleware:
    """Tests for shared middleware behavior (validate_json_body)."""

    def test_register_wrong_content_type(self, session: requests.Session, base_url: str):
        """Register with wrong Content-Type should be rejected before handler."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            data="not json",
            headers={"Content-Type": "text/plain"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "content-type" in body["message"].lower()

    def test_register_invalid_json(self, session: requests.Session, base_url: str):
        """Register with malformed JSON should be rejected."""
        resp = session.post(
            f"{base_url}/api/v1/user/register",
            data="not valid json{{{",
            headers={"Content-Type": "application/json"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "INVALID_JSON_ERROR"

    def test_login_wrong_content_type(self, session: requests.Session, base_url: str):
        """Login with wrong Content-Type should be rejected before handler."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            data="not json",
            headers={"Content-Type": "text/plain"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "VALIDATION_ERROR"
        assert "content-type" in body["message"].lower()

    def test_login_invalid_json(self, session: requests.Session, base_url: str):
        """Login with malformed JSON should be rejected."""
        resp = session.post(
            f"{base_url}/api/v1/user/login",
            data="not valid json{{{",
            headers={"Content-Type": "application/json"},
        )
        assert resp.status_code == 400
        body = resp.json()
        assert body["code"] == "INVALID_JSON_ERROR"


class TestLoginFullFlow:
    def test_register_then_login_success(self, session: requests.Session, base_url: str):
        """Full flow: register a user, then login, expect token back."""
        uname = _unique("flowtest")
        email = f"{uname}@example.com"
        password = "C0rrect!pass"

        # Register
        reg = session.post(
            f"{base_url}/api/v1/user/register",
            json={"username": uname, "email": email, "password": password},
        )
        assert reg.status_code == 201, reg.text

        # Login (without email — verifying the fix)
        log = session.post(
            f"{base_url}/api/v1/user/login",
            json={"username": uname, "password": password},
        )
        assert log.status_code == 200, log.text

        body = log.json()
        assert body["code"] == "SUCCESS"
        assert "token" in body["data"]
        assert body["data"]["user"]["username"] == uname
