<h1 style="text-align: center;">Baihua: The Communication Tool for Developers</h1>

[![Author: Gavin Zheng](https://img.shields.io/badge/Author-Gavin_Zheng-f2f28d)](https://github.com/GavZheng)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)
![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-blue)
![License: Apache v2](https://img.shields.io/badge/License-Apache%20v2-green)
![Github Stars](https://img.shields.io/github/stars/Binder-organize/Baihua-Server?style=flat&color=red)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-3.0-4baaaa.svg)](CODE_OF_CONDUCT.md)

[English](README.md) | [简体中文](./docs/zh-CN/README_zh-CN.md)

Baihua is a team communication tool customized for developers, built with **Rust**, designed to help developers complete various tasks efficiently.

---

## Table of Contents
- [Table of Contents](#table-of-contents)
- [Why We Need Baihua](#why-we-need-baihua)
- [Quick Start](#quick-start)
- [How to Participate in Baihua's Development](#how-to-participate-in-baihuas-development)
- [Special Thanks](#special-thanks)
- [Contributors](#contributors)
- [FAQ](#faq)
- [License](#license)

---

## Why We Need Baihua
In today's development work, we often need to switch frequently between multiple tools and platforms: for example, operating Git in the terminal, checking project Issues in the browser, managing CI/CD in another panel, and tracking progress using spreadsheets or separate tools... This fragmented workflow not only reduces efficiency but also interrupts the developer's most precious state of 'flow'.

Baihua is born to end this fragmented experience. It is a team communication tool customized for developers, deeply integrating the tools you use daily—such as Git, GitHub/GitLab, dependency management, and build systems—unifying them into a coherent workflow.

---

## Quick Start

### Prerequisites

- **Docker** (for PostgreSQL)
- **Rust toolchain** (stable, with rustfmt + clippy)
- **Python 3.10+** and `pip install -r tests/requirements.txt` (for tests)

### Development

```bash
# 1. Set up environment variables
cp .env.example .env

# 2. Start PostgreSQL via Docker
docker compose up -d database

# 3. Start the server (auto-runs migrations on first start)
cargo run
```

The server starts on `http://localhost:2424`. An interactive console is available in the terminal — type `help` for commands.

To run the full test suite (build + DB + server + pytest):

```bash
python3 tests/run_tests.py
```

### Production

Deploy the entire stack with Docker Compose:

```bash
# 1. Set production environment variables
export JWT_SECRET="your-256-bit-secret"
export POSTGRES_USER="baihua"
export POSTGRES_PASSWORD="strong-password"
export POSTGRES_DB="baihua"

# 2. Build and start everything (first build may take 10-15 min)
docker compose --profile production up -d --build
```

> The first build downloads and compiles all Rust dependencies from scratch inside Docker.
> Subsequent builds are much faster thanks to Docker's layer caching.
> To watch build progress, use `docker compose --profile production up --build` (without `-d`).

This starts two services:

| Service | Container | Port |
|---------|-----------|------|
| **database** | `baihua-database` | 2423 (mapped) |
| **server** | `baihua-server` | 2424 |

The server is gated by the database health check and includes a Docker HEALTHCHECK (`GET /health`). Logs:

```bash
docker compose --profile production logs -f
```

To stop and clean up:

```bash
docker compose --profile production down -v
```

---

## How to Participate in Baihua's Development
Baihua is an open-source project. We warmly welcome and highly anticipate developers from all over the world to join and participate in Baihua's development process.
You can contribute to Baihua's development in the following ways:
1.  Submit bug reports and feature suggestions: If you discover bugs or have any feature suggestions while using Baihua, please refer to the [Baihua Security Policy](SECURITY.md).
2.  Contribute code: If you have the ability and willingness to contribute code to Baihua, please refer to the [Baihua Contributor Guide](CONTRIBUTING.md).

---

## Special Thanks
We sincerely thank the following individuals for their outstanding contributions to Baihua (listed in alphabetical order by first name):
-   [Bob](https://github.com/ChepleBob30): Made many non-code contributions to Baihua's development and is Baihua's first user.

---

## Contributors
<a href="https://github.com/Binder-organize/Baihua-Server/contributors">
  <img src="https://contrib.rocks/image?repo=Binder-organize/Baihua-Server" alt="Contributors"/>
</a>

---

## FAQ
**Q1:** What development equipment does Gavin use?  
**A1:** MacBook Air M1.

**Q2:** More information about Gavin?  
**A2:** You can visit [Gavin's GitHub Profile](https://github.com/GavZheng).

**Q3:** Why choose Rust?  
**A3:** After a comprehensive evaluation of candidate languages such as Python/C++/C, Rust was confirmed as the optimal technical choice to meet the project requirements, thanks to its excellent cross-platform capabilities, memory safety features, and high execution efficiency.

---

## License
[Apache v2](LICENSE), Copyright 2026 Gavin Zheng.