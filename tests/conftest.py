import pytest
import requests

BASE_URL = "http://localhost:2424"


@pytest.fixture(scope="session")
def base_url():
    return BASE_URL


@pytest.fixture(scope="function")
def session():
    s = requests.Session()
    yield s
    s.close()
