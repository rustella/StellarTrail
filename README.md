# 🌌 StellarTrail（寻径星野）

> 面向户外场景的装备整理、路线计划和户外技能工具箱。

[English](README.en.md)

<p align="center">
  <img alt="StellarTrail app icon" src="assets/brand/app-icon-180.png" width="96" height="96" />
</p>

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/WeChat%20Mini%20Program-%E5%8F%AF%E7%94%A8-07C160" />
  <img alt="Web" src="https://img.shields.io/badge/Web-%E5%8F%AF%E7%94%A8-0EA5E9" />
  <img alt="Native clients" src="https://img.shields.io/badge/iOS%20%2F%20macOS%20%2F%20Android%20%2F%20HarmonyOS-%E4%B8%8D%E5%8F%AF%E7%94%A8-lightgrey" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>微信小程序与 Web 已可用</strong> · <strong>原生客户端暂不可用</strong>
</p>

---

## ✨ 项目一眼看懂

StellarTrail 聚焦徒步、露营和轻量户外用户，围绕 **装备整理**、**路线计划**、**户外技能工具箱** 等准备和出行前协作场景展开。

当前可直接体验的入口是 **微信小程序端** 和 **Web**：两端覆盖账号登录、个人装备库、装备图鉴、绳结技能和反馈能力。Web 在其他端能力基础上增加了管理员功能。Android、iOS、macOS、HarmonyOS 暂不可用，不作为当前交付入口。

| 产品功能      | 当前说明                                   |
| ------------- | ------------------------------------------ |
| 🎒 装备整理   | 个人装备库、装备图鉴、装备模板             |
| 🗺️ 路线计划   | 路线计划相关能力持续建设中                 |
| 🧭 户外技能   | 绳结技能、媒体资源和离线只读缓存           |
| 💬 反馈与管理 | 用户反馈；Web 在其他端基础上增加管理员功能 |

## 🚀 快速体验

### 微信小程序入口

<p align="center">
  <img alt="寻径星野微信小程序码" src="assets/brand/wechat-miniprogram-code.png" width="260" />
</p>

<p align="center">
  <strong>微信扫码或搜索「寻径星野」进入小程序。</strong>
</p>

暂不包含路线、行程、技能、实时导航、社区信息流、路线交易/带队、完整 GPX 编辑和商城能力。

## 🌱 当前公共数据

| 类型        | 来源                                          |
| ----------- | --------------------------------------------- |
| 🪢 绳结技能 | DB-backed 公共绳结内容，媒体走 MinIO/对象存储 |
| 🎒 装备模板 | 服务启动时向 DB 幂等 seed 默认系统模板        |

## 📱 客户端支持

| 客户端       | 当前状态 | 说明                                                                  |
| ------------ | -------- | --------------------------------------------------------------------- |
| 微信小程序端 | 可用     | 首页、装备库、装备图鉴、绳结技能、我的、登录/注册、反馈和离线只读缓存 |
| Web          | 可用     | 覆盖其他端的核心能力，并增加管理员功能                                |
| Android      | 不可用   | 代码仍在仓库中，但当前不作为可运行交付入口                            |
| iOS          | 不可用   | 代码仍在仓库中，但当前不作为可运行交付入口                            |
| macOS        | 不可用   | 代码仍在仓库中，但当前不作为可运行交付入口                            |
| HarmonyOS    | 不可用   | 尚未作为当前交付客户端接入                                            |

## 🧭 仓库结构

```text
StellarTrail/
  apps/
    android/                # Android 原生端（Kotlin + Jetpack Compose）
    ios/                    # iOS 原生端（Swift + SwiftUI）
    macos/                  # macOS 原生端（Swift + SwiftUI）
    web/                    # Web App（TypeScript + Vite）
    wechat-miniprogram/     # 微信小程序端（TypeScript + WXML + WXSS）
  services/
    api/                    # Rust axum API 服务
  crates/
    domain/                 # 共享 Rust 领域模型
    db/                     # DB 配置和仓储边界
    migration/              # 数据迁移边界
  packages/
    api-client-ts/          # 供小程序 / Web / Mobile 复用的 TS API client
    apple/StellarTrailKit/  # iOS / macOS 共享 Swift Package
    shared-types/           # TS 共享 DTO 类型
  docs/                     # 产品、架构、API 和部署文档
  infra/                    # 本地集成测试与生产部署配置
  scripts/                  # 开发辅助脚本
```

## ⚡ 快速开始

### 1. 环境要求

- 🦀 Rust 1.95 stable toolchain（Rust 2024 edition；仓库 `rust-version` 为 `1.95`，并包含 `rust-toolchain.toml`；需要 `rustfmt` 和 `clippy`）。
- 🟢 Node.js 22+ 与 npm。
- 💬 微信开发者工具（调试小程序时需要）。
- 🗄️ PostgreSQL 16+ / MySQL-compatible 数据库（本地默认 SQLite；生产和集成测试推荐 PostgreSQL；MySQL URL 已保留识别边界）。
- 🪣 MinIO 或 S3-compatible 对象存储。
- ⚡ Redis 7+（可选；配置 `REDIS_URL` 后启用服务端缓存）。

### 2. 安装依赖

```bash
npm install
```

### 3. 启动 API

```bash
cp .env.example .env
# 可选：如果希望用 YAML 管理本地/生产配置，可复制模板到被 Git 忽略的 config.yaml
cp config.example.yaml config.yaml
cargo +1.95.0 run -p stellartrail-api --bin migrate -- up
cargo +1.95.0 run -p stellartrail-api
```

API 默认监听 `127.0.0.1:8080`，启动配置按 `.env`、`config.yaml` / `CONFIG_PATH`、环境变量覆盖的顺序加载；本地默认 SQLite，生产和集成测试推荐 PostgreSQL。数据库、MinIO、Redis、SMTP、微信登录和管理员配置细节见 [部署指南](docs/deployment.md)。

可用以下接口做本地冒烟验证：

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/v1/meta
```

### 4. 配置客户端访问地址

仓库中的客户端默认使用占位地址：

- API 地址：`https://api.example.invalid`
- 图片资源 / 允许跨域资源域名：`https://assets.example.invalid`

需要调整客户端地址时，复制对应示例文件后修改：

| 客户端     | 示例配置                                                | 真实配置（不提交）                              |
| ---------- | ------------------------------------------------------- | ----------------------------------------------- |
| Web        | `apps/web/.env.example`                                 | `apps/web/.env.local`                           |
| 微信小程序 | `apps/wechat-miniprogram/miniprogram/config.example.ts` | `apps/wechat-miniprogram/miniprogram/config.ts` |

Web 可通过 `VITE_STELLARTRAIL_API_BASE_URL` 和 `VITE_STELLARTRAIL_ASSETS_BASE_URL` 覆盖；本地 Vite 开发默认使用同源 `/api/v1`，并通过 `VITE_STELLARTRAIL_API_PROXY_TARGET` 代理到真实或本地 API，避免浏览器 CORS 拦截。微信小程序端会读取 `miniprogram/config.ts`，缺失时回退到占位地址。

完整接口说明见 [API 文档](docs/api.md)。

### 5. 部署指南

本地 Docker Compose 集成测试和生产 Docker / Traefik 部署说明已经整理到 [部署指南](docs/deployment.md)。

### 6. 打开微信小程序源码

用微信开发者工具打开 `apps/wechat-miniprogram`。项目配置中的 `miniprogramRoot` 指向 `miniprogram/`。当前小程序端已覆盖首页、装备库、装备图鉴、绳结技能、我的、登录/注册、头像/邮箱绑定、反馈和离线只读缓存。

## 🧪 常用检查命令

```bash
# 前端 / TypeScript workspace 检查
npm run check

# Markdown / JSON / YAML / TS 等格式检查
npm run format:check

# Rust workspace 检查
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 check --workspace
cargo +1.95.0 test --workspace
cargo +1.95.0 clippy --workspace --all-targets -- -D warnings
```

## 📚 文档索引

- 🚀 [部署指南](docs/deployment.md)
- 🏗️ [架构说明](docs/architecture.md)
- 🔌 [API 文档](docs/api.md)

## 📄 许可证

Licensed under the Apache License, Version 2.0.

Copyright 2026 Rustella.
