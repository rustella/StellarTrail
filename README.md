# 🌌 StellarTrail（寻径星野）

> 面向中国户外场景的装备准备、路线灵感和离线技能工具箱。

[English](README.en.md)

<p align="center">
  <img alt="StellarTrail app icon" src="assets/brand/app-icon-180.png" width="96" height="96" />
</p>

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/WeChat%20Mini%20Program-%E5%8F%AF%E7%94%A8-07C160" />
  <img alt="Web" src="https://img.shields.io/badge/Web-%E5%8F%AF%E7%94%A8-0EA5E9" />
  <img alt="Native mobile" src="https://img.shields.io/badge/iOS%20%2F%20Android%20%2F%20HarmonyOS-%E4%B8%8D%E5%8F%AF%E7%94%A8-lightgrey" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>微信小程序与 Web 已可用</strong> · <strong>当前功能最完整</strong> · <strong>原生移动端暂不可用</strong>
</p>

---

## ✨ 项目一眼看懂

StellarTrail 聚焦中国徒步、露营和轻量户外用户，目标是在一个产品里完成「了解路线灵感 → 准备装备 → 对比个人装备库 → 学习相关技能 → 留存反馈」的准备闭环。

当前最完整、可直接体验的入口是 **微信小程序端** 和 **Web**：两端复用同一套 Rust API，覆盖账号登录、个人装备库、装备图鉴、绳结技能、反馈和管理能力。Android、iOS、HarmonyOS 暂不可用，不作为当前交付入口。

| 方向          | 当前状态                                       |
| ------------- | ---------------------------------------------- |
| 📱 用户入口   | 微信小程序 + Web 可用，也是当前功能最完整入口  |
| 🎒 装备准备   | 个人装备库、装备图鉴、DB-backed 装备模板       |
| 🧭 技能工具箱 | 绳结技能已接入，媒体通过 MinIO/对象存储提供    |
| 🧱 Rust 服务  | 账号、装备、技能、反馈、上传、管理和统计能力   |
| 🧪 原生移动端 | Android / iOS / HarmonyOS 当前不可用，暂不交付 |

## 🟢 微信小程序入口

<p align="center">
  <img alt="寻径星野微信小程序码" src="assets/brand/wechat-miniprogram-code.png" width="260" />
</p>

<p align="center">
  <strong>微信扫码或搜索「寻径星野」进入小程序。</strong>
</p>

## 🌱 当前公共数据

| 类型        | 来源                                            |
| ----------- | ----------------------------------------------- |
| 🪢 绳结技能 | Knots3D metadata 导入 DB，媒体走 MinIO/对象存储 |
| 🎒 装备模板 | 服务启动时向 DB 幂等 seed 默认系统模板          |

## 📱 客户端支持

| 客户端       | 当前状态       | 说明                                                                     |
| ------------ | -------------- | ------------------------------------------------------------------------ |
| 微信小程序端 | 可用，功能完整 | 首页、装备库、装备图鉴、绳结技能、我的、登录/注册、反馈和离线只读缓存    |
| Web          | 可用，功能完整 | Web App 与管理入口，复用同一套公开数据、装备、反馈、上传、管理和统计能力 |
| Android      | 不可用         | 代码仍在仓库中，但当前不作为可运行交付入口                               |
| iOS          | 不可用         | 代码仍在仓库中，但当前不作为可运行交付入口                               |
| HarmonyOS    | 不可用         | 尚未作为当前交付客户端接入                                               |

## 🧭 仓库结构

```text
StellarTrail/
  apps/
    android/                # Android 原生端（Kotlin + Jetpack Compose）
    ios/                    # iOS 原生端（Swift + SwiftUI）
    web/                    # Web App
    wechat-miniprogram/     # 微信小程序端
  services/
    api/                    # Rust axum API 服务
  crates/
    domain/                 # 共享 Rust 领域模型
    db/                     # DB 配置和仓储边界
    importer/               # Knots3D metadata 导入边界
    migration/              # 数据迁移边界
  packages/
    api-client-ts/          # 供小程序 / Web / Mobile 复用的 TS API client
    shared-types/           # TS 共享 DTO 类型
  docs/                     # 产品、架构、API、内容 schema 文档
  infra/                    # 本地集成测试与生产部署配置
  scripts/                  # 开发辅助脚本
```

## ⚡ 快速开始

### 1. 环境要求

- 🦀 Rust stable toolchain（Rust 2024 edition；仓库包含 `rust-toolchain.toml`，需要 `rustfmt` 和 `clippy`）。
- 🟢 Node.js 22+ 与 npm。
- 💬 微信开发者工具（调试小程序时需要）。
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
cargo run -p stellartrail-api --bin migrate -- up
cargo run -p stellartrail-api
```

API 默认监听 `127.0.0.1:8080`。启动时会先加载 `.env`，再读取根目录 `config.yaml`（存在时）或 `CONFIG_PATH` 指定的 YAML 文件，最后由环境变量覆盖 YAML 配置。默认数据库地址为 `sqlite://stellartrail.db`。本地可通过 `APP_ENV=local` + `WECHAT_MOCK_LOGIN=true` 启用 mock 登录；正式微信登录需设置 `WECHAT_MOCK_LOGIN=false`、`WECHAT_APP_ID` 和 `WECHAT_APP_SECRET`。邮箱验证码生产投递通过 SMTP：设置 `MAIL_ENABLED=true`、`MAIL_SMTP_HOST=smtp.example.invalid`、`MAIL_SMTP_USERNAME=[REDACTED]`，并通过被忽略的 `config.yaml` 或 secret manager 注入 `MAIL_SMTP_PASSWORD` 和发件人地址。邮箱验证码现在用于注册、邮箱验证码登录和找回密码。如需启用 Redis 缓存，设置 `REDIS_URL=redis://127.0.0.1:6379/0`；`REDIS_GEAR_CACHE_TTL_SECONDS` 控制装备读取缓存 TTL。`config.example.yaml` 会提交到 Git，实际 `config.yaml` / `config.*.yaml` 会被忽略。

可用以下接口做本地冒烟验证：

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/v1/meta
```

绳结媒体通过 MinIO/S3-compatible 对象存储上传，公开读接口只返回数据库中的媒体 URL，不再从 `/assets/*` 拼接绳结媒体路径。服务端只维护一组 `minio` 连接配置，反馈图与绳结媒体分别通过 `object_storage.bucket` 和 `knots_media_storage.bucket` 配置业务 bucket。管理员权限存储在数据库 `admin_roles` 表：已有且未删除的 `stellarisw` 用户会在迁移时被 seed 为 `super_admin`，`super_admin` 可通过 `/api/v1/admin/admins` 授予或移除普通 `admin`。`admin` 与 `super_admin` 都可调用 Knots 媒体上传、Gear Atlas 审核、反馈查看和 `GET /api/v1/admin/api-usage`。统计使用异步上报，只保存 matched route 模板和聚合计数，不记录 query、请求体、Authorization、token、Cookie、IP 或 User-Agent。

### 4. 配置客户端访问地址

仓库中的客户端默认使用占位地址：

- API 地址：`https://api.example.invalid`
- 图片资源 / 允许跨域资源域名：`https://assets.example.invalid`

真实客户端配置文件只保留在本地或构建环境，已被 `.gitignore` 忽略；仓库只提交示例文件。需要调整地址时，复制对应示例文件后修改：

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
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 📚 文档索引

- 🚀 [部署指南](docs/deployment.md)
- 🏗️ [架构说明](docs/architecture.md)
- 🔌 [API 文档](docs/api.md)
- 🧩 [公共数据说明](docs/content-schema.md)

## 🏷️ 命名约定

- 产品名：**StellarTrail**
- 中文名：**寻径星野**
- 仓库名：`StellarTrail`
