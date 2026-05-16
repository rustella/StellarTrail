# 🌌 StellarTrail（星径）

> 面向中国户外场景的路线百科、装备准备助手和离线技能工具箱。

[English](README.en.md)

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/Client-WeChat%20Mini%20Program-07C160" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-YAML%20%2B%20Markdown-8A2BE2" />
</p>

<p align="center">
  <strong>选路线</strong> · <strong>识风险</strong> · <strong>配装备</strong> · <strong>学技能</strong>
</p>

---

## ✨ 项目一眼看懂

StellarTrail 聚焦中国徒步、露营和轻量户外用户，目标是在一个产品里完成「选路线 → 看难度、季节和风险 → 生成装备清单 → 对比个人装备库 → 学习相关技能」的准备闭环。

当前产品以 **微信小程序** 为优先入口，**Rust API 服务** 提供后端能力；第一期聚焦 **个人装备库管理**，路线、山峰、技能和装备模板继续作为后续内容方向保留。

| 模块          | 当前目标                             |
| ------------- | ------------------------------------ |
| 🗺️ 路线百科   | 沉淀山峰、路线、难度、季节、风险     |
| 🎒 装备准备   | 生成打包清单，对照个人装备库         |
| 🧭 技能工具箱 | 覆盖绳结、露营、导航、天气、急救等   |
| 📦 内容导入   | 用 YAML/Markdown 快速迭代种子内容    |
| 🧱 Rust 后端  | 提供稳定 API、领域模型和数据访问边界 |

## 🚀 MVP 范围

第一期重点建设：

- 🧑‍💻 微信小程序登录、邮箱/用户名 + 密码登录和账号模型。
- 🎒 用户个人装备库 CRUD。
- 🔎 装备搜索、分类筛选、状态筛选、排序和分页。
- 📊 装备数量、装备价值、总重量和分类计数统计。
- 🗃️ 可用装备 / 历史装备（软删除归档与恢复）。
- 🏷️ 标签、状态与位置、共享开关、备注等字段。
- 📤 装备 JSON 导入和 CSV 导出。
- 🗄️ SeaORM 数据访问：本地默认 SQLite，生产推荐 PostgreSQL。
- ⚡ 可选 Redis read-through cache，提升装备库高频读取接口性能。
- 🖼️ 用户反馈与安全图片上传，图片存储在私有 MinIO/S3-compatible bucket。

暂不包含路线、行程、技能、实时导航、社区信息流、路线交易/带队、完整 GPX 编辑和商城能力。

## 🌱 当前种子内容

| 类型        | 内容                     |
| ----------- | ------------------------ |
| ⛰️ 山峰     | 武功山                   |
| 🥾 路线     | 武功山经典 2 天 1 夜穿越 |
| 🪢 技能     | 可调节帐绳结             |
| 🎒 装备模板 | 入门徒步基础装备模板     |

## 🧭 仓库结构

```text
StellarTrail/
  apps/
    wechat-miniprogram/     # 微信小程序端
  services/
    api/                    # Rust axum API 服务
  crates/
    domain/                 # 共享 Rust 领域模型
    db/                     # DB 配置和仓储边界
    importer/               # 内容导入边界
    migration/              # 数据迁移边界
  packages/
    api-client-ts/          # 供小程序 / Web / Mobile 复用的 TS API client
    shared-types/           # TS 共享 DTO 类型
  content/                  # 山峰、路线、技能、装备模板内容
  docs/                     # 产品、架构、API、内容 schema 文档
  infra/                    # 本地 / 开发部署文件
  scripts/                  # 开发辅助脚本
```

## ⚡ 快速开始

### 1. 环境要求

- 🦀 Rust stable toolchain（Rust 2024 edition；仓库包含 `rust-toolchain.toml`，需要 `rustfmt` 和 `clippy`）。
- 🟢 Node.js 22+ 与 npm。
- 💬 微信开发者工具（调试小程序时需要）。
- ⚡ Redis 7+（可选；配置 `REDIS_URL` 后启用服务端缓存）。
- 🪣 MinIO 或 S3-compatible 对象存储（反馈图片上传需要）。

### 2. 安装依赖

```bash
npm install
```

### 3. 启动 API

```bash
cp .env.example .env
cargo run -p stellartrail-api --bin migrate -- up
cargo run -p stellartrail-api
```

API 默认监听 `127.0.0.1:8080`，并从环境变量读取配置。默认数据库地址为 `sqlite://stellartrail.db`。本地可通过 `APP_ENV=local` + `WECHAT_MOCK_LOGIN=true` 启用 mock 登录；正式微信登录需设置 `WECHAT_MOCK_LOGIN=false`、`WECHAT_APP_ID` 和 `WECHAT_APP_SECRET`。如需启用 Redis 缓存，设置 `REDIS_URL=redis://127.0.0.1:6379/0`，可通过 `REDIS_GEAR_CACHE_TTL_SECONDS` 调整装备接口缓存 TTL。反馈图片上传使用私有 MinIO/S3-compatible bucket，配置 `OBJECT_STORAGE_ENDPOINT`、`OBJECT_STORAGE_BUCKET`、`OBJECT_STORAGE_ACCESS_KEY_ID`、`OBJECT_STORAGE_SECRET_ACCESS_KEY`；API 通过 `/api/me/uploads/:id` 做鉴权代理下载，不暴露 public URL。

可用以下接口做本地冒烟验证：

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/meta
```

当前已实现接口：

```http
GET /healthz
GET /api/meta
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/register
POST /api/auth/login
POST /api/auth/captcha
GET /api/mountains
GET /api/mountains/:id
GET /api/routes
GET /api/routes/:id
GET /api/skills
GET /api/skills/knots/list
GET /api/skills/knots/detail/:id
GET /assets/*
GET /api/gear-templates
GET /api/gear-templates/:id
GET /api/me/gears/categories
GET /api/me/gears/stats
GET /api/me/gears
POST /api/me/gears
GET /api/me/gears/:id
PATCH /api/me/gears/:id
DELETE /api/me/gears/:id
POST /api/me/gears/:id/restore
GET /api/me/gears/export
POST /api/me/gears/import
POST /api/me/uploads
GET /api/me/uploads/:id
POST /api/me/feedback
```

### 4. 使用 Docker Compose 启动 PostgreSQL + Redis + MinIO + API

也可以用 Docker Compose 启动一次性本地集成测试：

```bash
COMPOSE_PROJECT_NAME=stellartrail_it API_HOST_PORT=18080 POSTGRES_HOST_PORT=15432 REDIS_HOST_PORT=16379 MINIO_API_HOST_PORT=19000 MINIO_CONSOLE_HOST_PORT=19001 \
  bash infra/integration-test.sh
```

该脚本会启动 PostgreSQL、Redis 缓存、MinIO 私有 bucket 和 API 服务，使用用户名/密码账号注册与登录、装备接口、反馈图片上传/下载与反馈提交做 curl 冒烟测试，并在测试结束或失败时自动执行 `docker compose down -v --remove-orphans` 关闭并清理容器和 volume；生产环境请通过安全渠道注入真实微信、数据库与对象存储密钥。

### 5. 打开微信小程序

用微信开发者工具打开 `apps/wechat-miniprogram`。项目配置中的 `miniprogramRoot` 指向 `miniprogram/`。

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

- 📌 [MVP 范围](docs/mvp.md)
- 🏗️ [架构说明](docs/architecture.md)
- 🔌 [API 草案](docs/api.md)
- 🧩 [内容 Schema 草案](docs/content-schema.md)

## 🏷️ 命名约定

- 产品名：**StellarTrail**
- 中文占位名：**星径**
- 仓库名：`StellarTrail`
