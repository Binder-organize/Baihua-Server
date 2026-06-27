<h1 style="text-align: center;">白桦：属于开发者的沟通工具</h1>

[![Author: Gavin Zheng](https://img.shields.io/badge/Author-Gavin_Zheng-f2f28d)](https://github.com/GavZheng)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)
![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-blue)
![License: Apache v2](https://img.shields.io/badge/License-Apache%20v2-green)
![Github Stars](https://img.shields.io/github/stars/Binder-organize/Baihua-Server?style=flat&color=red)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-3.0-4baaaa.svg)](CODE_OF_CONDUCT_zh-CN.md)

[English](../../README.md) | [简体中文](README_zh-CN.md)

白桦是一个由**Rust**编写的，为开发者定制的团队沟通工具，用于帮助开发者快速完成各种任务。

---

## 目录
- [目录](#目录)
- [为什么我们需要白桦](#为什么我们需要白桦)
- [快速启动](#快速启动)
- [如何参与白桦的开发](#如何参与白桦的开发)
- [特别致谢](#特别致谢)
- [贡献者](#贡献者)
- [FAQ](#faq)
- [许可证](#许可证)

---

## 为什么我们需要白桦
在今天的开发工作中，我们往往需要频繁切换多个工具和平台：例如，在终端操作 Git、在浏览器查看项目的 Issues、在另一个面板管理 CI/CD，以及使用表格或独立工具跟踪进度……这种碎片化的工作流不仅降低效率，更打断了开发者最宝贵的“心流”状态。

白桦 正是为了终结这种碎片化体验而生。它是一个针对开发者定制的团队沟通工具，深度整合了你日常使用的 Git、GitHub/GitLab、依赖管理、构建系统等工具，将它们统一到一个连贯的工作流中。

---

## 快速启动

### 前置条件

- **Docker**（用于 PostgreSQL 数据库）
- **Rust 工具链**（stable，包含 rustfmt 和 clippy）
- **Python 3.10+**，并执行 `pip install -r tests/requirements.txt`（用于测试）

### 开发模式

```bash
# 1. 配置环境变量
cp .env.example .env

# 2. 通过 Docker 启动 PostgreSQL
docker compose up -d database

# 3. 启动服务端（首次启动自动执行数据库迁移）
cargo run
```

服务端运行在 `http://localhost:2424`，终端中会启动交互式控制台——输入 `help` 查看命令。

如需运行完整的测试套件（编译 + 数据库 + 服务端 + pytest）：

```bash
python3 tests/run_tests.py
```

### 生产部署

使用 Docker Compose 部署整个服务栈：

```bash
# 1. 设置生产环境变量
export JWT_SECRET="your-256-bit-secret"
export POSTGRES_USER="baihua"
export POSTGRES_PASSWORD="strong-password"
export POSTGRES_DB="baihua"

# 2. 构建并启动所有服务（首次构建可能需要 10-15 分钟）
docker compose --profile production up -d --build
```

> 首次构建需要在 Docker 里从零下载并编译所有 Rust 依赖。
> 后续构建由于 Docker 层缓存会快得多。
> 如需查看构建进度，去掉 `-d` 参数：`docker compose --profile production up --build`。

启动后包含两个服务：

| 服务           | 容器名               | 端口       |
|--------------|-------------------|----------|
| **database** | `baihua-database` | 2423（映射） |
| **server**   | `baihua-server`   | 2424     |

服务端依赖数据库健康检查才启动，并自带 Docker HEALTHCHECK（`GET /health`）。查看日志：

```bash
docker compose --profile production logs -f
```

停止并清理：

```bash
docker compose --profile production down -v
```

---

## 如何参与白桦的开发
白桦是一个开源项目，我们热忱欢迎并高度期待来自全球各地的开发者能够加入并参与到白桦的开发进程中来。
你可以通过以下方式参与白桦的开发：
1. 提交漏洞报告和功能建议：如果你在使用白桦的过程中发现了漏洞或者有任何功能建议，请参阅[白桦安全指南](SECURITY_zh-CN.md)。
2. 贡献代码：如果你有能力并且愿意为白桦贡献代码，请参阅[白桦贡献者指南](CONTRIBUTING_zh-CN.md)文件。

---

## 特别致谢
对以下为白桦做出突出贡献的人员表示真挚地感谢（以首字母为序）：
- [Bob](https://github.com/ChepleBob30)：为白桦的开发做出了非常多非代码的贡献，是白桦的第一位用户。

---

## 贡献者
<a href="https://github.com/Binder-organize/Baihua-Server/contributors">
  <img src="https://contrib.rocks/image?repo=Binder-organize/Baihua-Server" alt="Contributors"/>
</a>

---

## FAQ
Q1：Gavin 使用什么开发设备？  
A1：MacBook Air M1。

Q2：更多关于 Gavin 的信息？  
A2：你可以访问[Gavin的Github主页](https://github.com/GavZheng)。

Q3：为什么选择Rust？  
A3：Rust凭借其卓越的跨平台能力、内存安全特性和高效执行性能，在综合评估Python/C++/C等候选语言后，被确认为满足项目需求的最佳技术选型。

---

## FAQ
Q1：Gavin 使用什么开发设备？  
A1：MacBook Air M1。

Q2：更多关于 Gavin 的信息？  
A2：你可以访问[Gavin的Github主页](https://github.com/GavZheng)。

Q3：为什么选择Rust？  
A3：Rust凭借其卓越的跨平台能力、内存安全特性和高效执行性能，在综合评估Python/C++/C等候选语言后，被确认为满足项目需求的最佳技术选型。

---

## 许可证
[Apache v2](../../LICENSE), Copyright 2026 Gavin Zheng.
