# 🌌 StellarTrail（星径）

> 面向中国户外场景的路线百科、装备准备助手和离线技能工具箱。

[English](README.en.md)

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/Client-WeChat%20Mini%20Program-07C160" />
  <img alt="Android" src="https://img.shields.io/badge/Client-Android%20%2B%20Compose-3DDC84" />
  <img alt="iOS" src="https://img.shields.io/badge/Client-iOS%20%2B%20SwiftUI-000000" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>选路线</strong> · <strong>识风险</strong> · <strong>配装备</strong> · <strong>学技能</strong>
</p>

---

## ✨ 项目一眼看懂

StellarTrail 聚焦中国徒步、露营和轻量户外用户，目标是在一个产品里完成「选路线 → 看难度、季节和风险 → 生成装备清单 → 对比个人装备库 → 学习相关技能」的准备闭环。

当前产品以 **微信小程序** 为优先入口，**Android 原生端** 与 **iOS 原生端** 复用同一套 Rust API 与装备/技能核心体验，**Rust API 服务** 提供后端能力；第一期聚焦 **个人装备库管理**、DB-backed 装备模板和绳结公共技能。路线、山峰模块尚未进入实现阶段。

| 模块          | 当前目标                                |
| ------------- | --------------------------------------- |
| 🎒 装备准备   | 管理个人装备库，提供 DB-backed 装备模板 |
| 🧭 技能工具箱 | 先接入绳结，媒体通过 MinIO/对象存储提供 |
| 🧱 Rust 后端  | 提供稳定 API、领域模型和数据访问边界    |
| 🗺️ 路线百科   | 后续阶段再实现山峰、路线、难度和风险    |

## 🚀 MVP 范围

第一期重点建设：

- 🧑‍💻 微信小程序登录、邮箱/用户名 + 密码登录、账号模型和 refresh token 会话续期。
- 🎒 用户个人装备库 CRUD。
- 🔎 装备搜索、分类筛选、状态筛选、排序和分页。
- 📊 装备数量、装备价值、总重量和分类计数统计。
- 🗃️ 可用装备 / 历史装备（软删除归档与恢复）。
- 🏷️ 标签、状态与位置、共享开关、备注等字段。
- 📤 装备 JSON 导入和 CSV 导出。
- 🗄️ SeaORM 数据访问：本地默认 SQLite，生产推荐 PostgreSQL。
- ⚡ 可选 Redis read-through cache，提升装备库高频读取接口性能。

暂不包含路线、行程、技能、实时导航、社区信息流、路线交易/带队、完整 GPX 编辑和商城能力。

## 🌱 当前公共数据

| 类型        | 来源                                            |
| ----------- | ----------------------------------------------- |
| 🪢 绳结技能 | Knots3D metadata 导入 DB，媒体走 MinIO/对象存储 |
| 🎒 装备模板 | 服务启动时向 DB 幂等 seed 默认系统模板          |

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
- 🤖 Android Studio / Android SDK 36 + JDK 21（调试 Android 端时需要）。
- 🍎 macOS + Xcode 16+ + XcodeGen（调试 iOS 端和生成页面截图时需要）。
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
curl http://127.0.0.1:8080/api/meta
```

绳结媒体通过 MinIO/S3-compatible 对象存储上传，公开读接口只返回数据库中的媒体 URL，不再从 `/assets/*` 拼接绳结媒体路径。服务端只维护一组 `minio` 连接配置，反馈图与绳结媒体分别通过 `object_storage.bucket` 和 `knots_media_storage.bucket` 配置业务 bucket。管理员可在配置 `ADMIN_USER_IDS`/`ADMIN_EMAILS`/`ADMIN_USERNAMES` allowlist 后，通过 `npm run knots:upload-media -- --dry-run` 检查 Knots3D 上传计划，再使用同一脚本调用管理员上传接口写入媒体。

当前已实现接口：

```http
GET /healthz
GET /api/meta
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/email-login-code
POST /api/auth/email-login
POST /api/auth/password-reset-code
POST /api/auth/password-reset
POST /api/auth/register
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/captcha
GET /api/skills
GET /api/skills/knots/list
GET /api/skills/knots/detail/:id
PUT /api/admin/skills/knots/:knot_id/media/:asset_id
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
```

### 4. 使用 Docker Compose 启动 PostgreSQL + Redis + API

也可以用 Docker Compose 启动一次性本地集成测试：

```bash
COMPOSE_PROJECT_NAME=stellartrail_it API_HOST_PORT=18080 POSTGRES_HOST_PORT=15432 REDIS_HOST_PORT=16379 \
  bash infra/test/integration-test.sh
```

该脚本会启动 PostgreSQL、Redis 缓存和 API 服务，使用账号注册、密码登录、邮箱验证码登录和找回密码做 curl 冒烟测试，并在测试结束或失败时自动执行 `docker compose down -v --remove-orphans` 关闭并清理容器；生产环境请通过安全渠道注入真实微信、数据库和 SMTP 密钥。

### 5. 生产 Docker / Traefik 部署配置

生产部署配置拆分在 `infra/production/` 下，目标服务器部署根目录为 `/www/service/stellartail`：

- `infra/production/traefik/docker-compose.yml`：唯一公网入口，暴露 80/443，并通过 Let’s Encrypt 自动申请和续期证书。
- `infra/production/site/docker-compose.yml`：官网，`site.example.invalid` 为 canonical；`www.example.invalid` 通过 Traefik 301 到 apex。
- `infra/production/web/docker-compose.yml`：Web App，域名 `app.example.invalid`。
- `infra/production/api/docker-compose.yml`：后端 API 与私有依赖组件 PostgreSQL、Redis、MinIO；API 可通过 Docker 服务名 `postgres`、`redis`、`minio` 访问组件，根目录 `config.yaml` 挂载到容器 `/app/config.yaml:ro`，并通过 `infra/production/api/compose-from-config.sh` 派生 PostgreSQL、Redis、MinIO 的 Compose 运行变量；`assets.example.invalid` 只通过 Traefik 指向 MinIO API，MinIO console 不直接公网暴露。
- `infra/production/domains.example.yaml` 与 `infra/production/api/config.production.example.yaml`：可提交的非敏感域名 / API 配置示例。

生产 API 不再使用 `infra/production/api/.env`；真实 `config.yaml`、ACME storage 和生产密钥文件必须保留在生产服务器或安全渠道中，仓库 `.gitignore` 会忽略这些文件；API 配置只提交 `config.example.yaml` 和 `*.example.yaml`。

### 6. 打开微信小程序

用微信开发者工具打开 `apps/wechat-miniprogram`。项目配置中的 `miniprogramRoot` 指向 `miniprogram/`。

### 7. Android 原生端

Android 应用位于 `apps/android`，使用 Kotlin、Jetpack Compose、Material 3、Navigation Compose、Ktor Client 与 kotlinx.serialization。Debug 构建默认连接 `http://10.0.2.2:8080`，可在 Profile 页面覆盖 API Base URL 便于本地联调。

```bash
./gradlew :apps:android:assembleDebug
./gradlew :apps:android:testDebugUnitTest
./gradlew :apps:android:lintDebug
```

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
- 🧩 [公共数据说明](docs/content-schema.md)

## 🏷️ 命名约定

- 产品名：**StellarTrail**
- 中文占位名：**星径**
- 仓库名：`StellarTrail`
