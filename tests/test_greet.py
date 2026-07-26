import tomllib
from pathlib import Path

import requests


def _server_version() -> str:
    cargo_toml = Path(__file__).resolve().parent.parent / "Cargo.toml"
    with open(cargo_toml, "rb") as file:
        data = tomllib.load(file)
    return data["package"]["version"]


class TestGreet:
    def test_greet_ok(self, session: requests.Session, base_url: str):
        resp = session.get(f"{base_url}/greet")
        assert resp.status_code == 200

        body = resp.json()
        assert body["server_version"] == _server_version()
        assert body["api_version"] == "v1"
