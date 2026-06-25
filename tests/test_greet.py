import requests


class TestGreet:
    def test_greet_ok(self, session: requests.Session, base_url: str):
        resp = session.get(f"{base_url}/greet")
        assert resp.status_code == 200

        body = resp.json()
        assert body["server_version"] == "0.1.0"
        assert body["api_version"] == "v1"
