#!/usr/bin/env python3
"""
Baihua Server tests runner.

Starts the server with PostgreSQL, runs pytest, cleans up.

Modes:
    (default)      DB in Docker, server binary local — fast, production-like
    --docker       Full Docker mode (--profile production), like CI
    --local        Local PostgreSQL, local binary

Usage:
    python3 tests/run_tests.py              # default
    python3 tests/run_tests.py --docker     # pure Docker (long build)
    python3 tests/run_tests.py --local      # local PostgreSQL
    python3 tests/run_tests.py --keep       # keep server running after tests
    python3 tests/run_tests.py --skip-checks  # skip cargo fmt + clippy
"""

import argparse
import datetime
import os
import signal
import subprocess
import sys
import time
import uuid
from io import StringIO

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SERVER_PORT = 2424
DB_PORT_ENV = 5432  # local PostgreSQL default
DB_PORT_DOCKER = 2423  # docker compose mapped port
LOG_DIR = os.path.join(PROJECT_ROOT, "tests", "logs")

_log_file = None  # set by setup_logging()
_log_stream = StringIO()  # in-memory capture of everything


def setup_logging():
    """Create log file and directory. Returns the log file path."""
    global _log_file
    os.makedirs(LOG_DIR, exist_ok=True)
    ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
    path = os.path.join(LOG_DIR, f"run_{ts}.log")
    _log_file = open(path, "w")  # noqa: SIM115
    return path


def log(msg):
    """Print to console + write to in-memory buffer and log file."""
    text = f"[tests-runner] {msg}"
    print(text, flush=True)
    print(text, file=_log_stream, flush=True)
    if _log_file:
        _log_file.write(text + "\n")
        _log_file.flush()


def log_file_path():
    """Return the current log file path (None if not set up yet)."""
    return _log_file.name if _log_file else None


def find_server_binary():
    """Locate the server binary (debug first, then release)."""
    candidates = [
        os.path.join(PROJECT_ROOT, "target", "debug", "baihua-server"),
        os.path.join(PROJECT_ROOT, "target", "release", "baihua-server"),
    ]
    for path in candidates:
        if os.path.isfile(path):
            return path
    return None


def check_postgres_local():
    """Check if local PostgreSQL is accepting connections."""
    import psycopg2

    try:
        conn = psycopg2.connect(
            host="localhost",
            port=DB_PORT_ENV,
            user=os.environ.get("USER", "postgres"),
            dbname="postgres",
        )
        conn.close()
        return {"host": "localhost", "port": DB_PORT_ENV, "type": "local"}
    except Exception:
        return None


def check_postgres_docker():
    """Check if Docker PostgreSQL is running on mapped port."""
    import psycopg2

    try:
        conn = psycopg2.connect(
            host="localhost",
            port=DB_PORT_DOCKER,
            user="baihua_user",
            password="password",
            dbname="baihua-database",
        )
        conn.close()
        return {"host": "localhost", "port": DB_PORT_DOCKER, "type": "docker"}
    except Exception:
        return None


def setup_database_local(db_info):
    """Create baihua_user and baihua-database if they don't exist."""
    import psycopg2

    conn = psycopg2.connect(
        host=db_info["host"],
        port=db_info["port"],
        user=os.environ.get("USER", "postgres"),
        dbname="postgres",
    )
    conn.autocommit = True
    cur = conn.cursor()

    # Create role if not exists
    cur.execute("SELECT 1 FROM pg_roles WHERE rolname = 'baihua_user'")
    if cur.fetchone() is None:
        cur.execute("CREATE ROLE baihua_user WITH LOGIN PASSWORD 'password'")
        log("Created role baihua_user")

    # Create database if not exists
    cur.execute("SELECT 1 FROM pg_database WHERE datname = 'baihua-database'")
    if cur.fetchone() is None:
        cur.execute(
            "CREATE DATABASE \"baihua-database\" OWNER baihua_user"
        )
        log("Created database baihua-database")

    cur.close()
    conn.close()


def wait_for_server(url, timeout=60):
    """Poll endpoint until it returns 200 or timeout."""
    import urllib.request
    import urllib.error

    for i in range(timeout):
        try:
            resp = urllib.request.urlopen(url, timeout=2)
            if resp.status == 200:
                return True
        except (urllib.error.URLError, ConnectionRefusedError, OSError):
            pass
        time.sleep(1)
    return False


def docker_is_available():
    """Quick check if Docker daemon is running (exit code 0 = yes)."""
    try:
        result = subprocess.run(
            ["docker", "info"],
            capture_output=True, timeout=10,
        )
        return result.returncode == 0
    except (subprocess.TimeoutExpired, OSError):
        return False


def run_checks():
    """Run cargo fmt --check and clippy. Return False if any check fails."""
    checks = [
        ("cargo fmt --check", ["cargo", "fmt", "--check"]),
        ("cargo clippy", ["cargo", "clippy", "--locked", "--", "-D", "warnings"]),
    ]
    all_passed = True
    for name, cmd in checks:
        log(f"Running {name}...")
        result = subprocess.run(cmd, cwd=PROJECT_ROOT, capture_output=True, text=True)
        if result.returncode != 0:
            log(f"{name} FAILED:")
            print(result.stdout)
            print(result.stderr)
            all_passed = False
        else:
            log(f"{name} passed")
    return all_passed


def _run_docker(args):
    """Pure Docker mode: build + run everything in containers (like CI)."""
    log_path = log_file_path()
    timeout = 1200  # 20 min for full Docker build + compile

    log("Starting docker compose --profile production (timeout: 1200s)...")
    env_ci_path = os.path.join(PROJECT_ROOT, ".env.ci")
    with open(env_ci_path, "w") as f:
        f.write("\n".join([
            "JWT_SECRET=docker-tests-jwt-not-for-production",
            "POSTGRES_USER=baihua_test",
            "POSTGRES_PASSWORD=test_pass",
            "POSTGRES_DB=baihua_test",
        ]))
    try:
        result = subprocess.run(
            ["docker", "compose", "--profile", "production", "--env-file", env_ci_path, "up", "-d", "--build"],
            cwd=PROJECT_ROOT, capture_output=True, text=True, timeout=timeout,
        )
        if result.returncode != 0:
            log("docker compose --profile production failed!")
            log("--- stderr ---")
            log(result.stderr)
            log("--- stdout ---")
            log(result.stdout)
            log(f"Full log saved to: {log_path}")
            sys.exit(1)
    except subprocess.TimeoutExpired:
        log(f"docker compose timed out after {timeout}s")
        log("If this is a large build, increase the timeout in tests/run_tests.py")
        sys.exit(1)
    except KeyboardInterrupt:
        log("\nInterrupted, cleaning up...")
        subprocess.run(
            ["docker", "compose", "--profile", "production", "down", "-v"],
            cwd=PROJECT_ROOT, capture_output=True,
        )
        sys.exit(130)
    finally:
        if os.path.exists(env_ci_path):
            os.unlink(env_ci_path)

    log("Waiting for server to become healthy...")
    healthy = wait_for_server(f"http://localhost:{SERVER_PORT}/health")
    if not healthy:
        log("Server did not become healthy!")
        logs = subprocess.run(
            ["docker", "compose", "logs"],
            cwd=PROJECT_ROOT, capture_output=True, text=True,
        )
        log("--- docker compose logs ---")
        log(logs.stdout)
        sys.exit(1)
    log("Server is healthy!")

    log("Running pytest...")
    test_exit = subprocess.run(
        [sys.executable, "-m", "pytest", "tests/", "-v", "--tb=short"],
        cwd=PROJECT_ROOT,
    ).returncode

    if not args.keep:
        log("Stopping docker compose...")
        subprocess.run(
            ["docker", "compose", "--profile", "production", "down", "-v"],
            cwd=PROJECT_ROOT, capture_output=True,
        )

    if test_exit == 0:
        log("All tests passed! ✅")
    else:
        log(f"Tests failed (exit code {test_exit}) ❌")
    sys.exit(test_exit)


def _run_default(args):
    """Default path: DB in Docker, server binary local (fast, production-like)."""
    log_path = log_file_path()
    env = os.environ.copy()

    # ── Phase 1: Build local binary ────────────────────────────────
    log("Building server (cargo build)...")
    result = subprocess.run(
        ["cargo", "build"],
        cwd=PROJECT_ROOT, capture_output=True, text=True,
    )
    if result.returncode != 0:
        log(f"Build failed:\n{result.stderr}")
        sys.exit(1)

    binary = find_server_binary()
    if not binary:
        log("Server binary not found after build!")
        sys.exit(1)

    # ── Phase 2: Start Docker database ─────────────────────────────
    log("Starting Docker Compose (database only)...")
    # Clean up any previous container + volume for a fresh start
    subprocess.run(
        ["docker", "compose", "down", "-v"],
        cwd=PROJECT_ROOT, capture_output=True,
    )
    result = subprocess.run(
        ["docker", "compose", "up", "-d", "database"],
        cwd=PROJECT_ROOT, capture_output=True, text=True,
    )
    if result.returncode != 0:
        log("Failed to start Docker database!")
        log("--- stderr ---")
        log(result.stderr)
        log("--- stdout ---")
        log(result.stdout)
        sys.exit(1)

    log("Waiting for Docker database...")
    for i in range(30):
        result = subprocess.run(
            ["docker", "compose", "exec", "database", "pg_isready", "-U", "baihua_user", "-d", "baihua-database"],
            cwd=PROJECT_ROOT, capture_output=True, text=True,
        )
        if result.returncode == 0:
            break
        time.sleep(2)
    else:
        log("Database did not become ready!")
        log("--- docker compose logs database ---")
        logs = subprocess.run(
            ["docker", "compose", "logs", "database"],
            cwd=PROJECT_ROOT, capture_output=True, text=True,
        )
        log(logs.stdout)
        sys.exit(1)

    # Set production-like environment variables
    env["POSTGRES_HOST"] = "localhost"
    env["POSTGRES_PORT"] = str(DB_PORT_DOCKER)
    env["POSTGRES_USER"] = "baihua_user"
    env["POSTGRES_PASSWORD"] = "password"
    env["POSTGRES_DB"] = "baihua-database"
    env["BAIHUA_ENV"] = "production"
    env["JWT_SECRET"] = f"tests-jwt-{uuid.uuid4().hex}"

    # In production mode the server looks for ./migrations/ next to the
    # binary (target/debug/). Symlink so it finds them.
    binary_dir = os.path.dirname(binary)
    link = os.path.join(binary_dir, "migrations")
    if not os.path.islink(link) and not os.path.isdir(link):
        os.symlink(
            os.path.join(PROJECT_ROOT, "migrations"),
            link,
            target_is_directory=True,
        )

    # ── Phase 3: Start server ─────────────────────────────────────
    log(f"Starting server ({binary})...")
    pipe_r, pipe_w = os.pipe()
    server_proc = subprocess.Popen(
        [binary],
        stdin=pipe_r,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=env,
        cwd=PROJECT_ROOT,
        pass_fds=(pipe_r,),
    )
    os.close(pipe_r)

    # ── Phase 4: Wait for health ──────────────────────────────────
    log("Waiting for server to become healthy...")
    healthy = wait_for_server(f"http://localhost:{SERVER_PORT}/health")
    if not healthy:
        log("Server did not become healthy!")
        server_proc.terminate()
        try:
            stdout, _ = server_proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            server_proc.kill()
            stdout, _ = server_proc.communicate()
        if stdout:
            log("--- server output (last 30 lines) ---")
            lines = stdout.decode() if isinstance(stdout, bytes) else stdout
            for line in lines.strip().splitlines()[-30:]:
                log(f"  {line}")
        os.close(pipe_w)
        log(f"Full log saved to: {log_path}")
        sys.exit(1)
    log("Server is healthy!")

    # ── Phase 5: Run tests ────────────────────────────────────────
    log("Running pytest...")
    test_exit = subprocess.run(
        [sys.executable, "-m", "pytest", "tests/", "-v", "--tb=short"],
        cwd=PROJECT_ROOT,
    ).returncode

    # ── Phase 6: Cleanup ──────────────────────────────────────────
    if not args.keep:
        log("Stopping server...")
        os.close(pipe_w)
        server_proc.send_signal(signal.SIGINT)
        try:
            server_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server_proc.kill()
            server_proc.wait()

        log("Stopping Docker Compose...")
        subprocess.run(
            ["docker", "compose", "down"],
            cwd=PROJECT_ROOT, capture_output=True,
        )

    if test_exit == 0:
        log("All tests passed! ✅")
    else:
        log(f"Tests failed (exit code {test_exit}) ❌")
    sys.exit(test_exit)


def _run_local(args):
    """Local path: build binary, connect to local/auto-detected PostgreSQL."""
    # ── Phase 1: Build ────────────────────────────────────────────
    log("Building server (cargo build)...")
    result = subprocess.run(
        ["cargo", "build"],
        cwd=PROJECT_ROOT, capture_output=True, text=True,
    )
    if result.returncode != 0:
        log(f"Build failed:\n{result.stderr}")
        sys.exit(1)

    binary = find_server_binary()
    if not binary:
        log("Server binary not found after build!")
        sys.exit(1)

    # ── Phase 2: Database ─────────────────────────────────────────
    env = os.environ.copy()
    db_info = None

    try:
        import psycopg2  # noqa: F401
    except ImportError:
        log("psycopg2 not installed. Install with: pip install psycopg2-binary")
        sys.exit(1)

    local = check_postgres_local()
    if local:
        db_info = local
        setup_database_local(local)
        env["POSTGRES_PORT"] = str(DB_PORT_ENV)
    else:
        docker_db = check_postgres_docker()
        if docker_db:
            db_info = docker_db
        else:
            log(
                "No PostgreSQL found. Run 'docker compose up -d database' or "
                "install PostgreSQL locally."
            )
            sys.exit(1)
    env["BAIHUA_ENV"] = "development"

    # ── Phase 3: Start server ─────────────────────────────────────
    log(f"Starting server ({binary})...")
    pipe_r, pipe_w = os.pipe()
    server_proc = subprocess.Popen(
        [binary],
        stdin=pipe_r,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=env,
        cwd=PROJECT_ROOT,
        pass_fds=(pipe_r,),
    )
    os.close(pipe_r)

    # ── Phase 4: Wait for health ──────────────────────────────────
    log("Waiting for server to become healthy...")
    healthy = wait_for_server(f"http://localhost:{SERVER_PORT}/health")
    if not healthy:
        log("Server did not become healthy!")
        server_proc.terminate()
        try:
            stdout, _ = server_proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            server_proc.kill()
            stdout, _ = server_proc.communicate()
        if stdout:
            log("--- server output (last 30 lines) ---")
            lines = stdout.decode() if isinstance(stdout, bytes) else stdout
            for line in lines.strip().splitlines()[-30:]:
                log(f"  {line}")
        os.close(pipe_w)
        log(f"Full log saved to: {log_file_path()}")
        sys.exit(1)
    log("Server is healthy!")

    # ── Phase 5: Run tests ────────────────────────────────────────
    log("Running pytest...")
    test_exit = subprocess.run(
        [sys.executable, "-m", "pytest", "tests/", "-v", "--tb=short"],
        cwd=PROJECT_ROOT,
    ).returncode

    # ── Phase 6: Cleanup ──────────────────────────────────────────
    if not args.keep:
        log("Stopping server...")
        os.close(pipe_w)
        server_proc.send_signal(signal.SIGINT)
        try:
            server_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server_proc.kill()
            server_proc.wait()

    if test_exit == 0:
        log("All tests passed! ✅")
    else:
        log(f"Tests failed (exit code {test_exit}) ❌")
    sys.exit(test_exit)


def run():
    parser = argparse.ArgumentParser(description="Run baihua-server integration tests")
    parser.add_argument("--docker", action="store_true", help="Full Docker mode (build + run everything in containers, like CI)")
    parser.add_argument("--local", action="store_true", help="Use local PostgreSQL instead of Docker")
    parser.add_argument("--keep", action="store_true", help="Keep server running after tests")
    parser.add_argument("--skip-checks", action="store_true", help="Skip cargo fmt + clippy pre-flight checks")
    args = parser.parse_args()

    log_path = setup_logging()
    log(f"Log file: {log_path}")

    # ── Phase 0: Pre-flight checks ─────────────────────────────────
    if not args.skip_checks:
        if not run_checks():
            sys.exit(1)
    else:
        log("Skipping pre-flight checks")

    if args.docker:
        _run_docker(args)
    elif args.local:
        _run_local(args)
    else:
        _run_default(args)


if __name__ == "__main__":
    run()
