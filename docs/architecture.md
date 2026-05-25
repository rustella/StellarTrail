# Architecture

## Runtime

```text
WeChat Mini Program / Android / iOS / Web
        |
        | HTTPS JSON API
        v
Rust API service (axum, Rust 2024)
        |
        | repository boundary (SeaORM)
        v
SQLite / PostgreSQL
        ^
        |
optional Redis cache (read-through gear API cache)

Knots3D metadata import CLI -> DB
MinIO / S3-compatible object storage -> public media URLs stored in DB
```

## Phase-one scope

第一期核心是装备库管理、DB-backed 装备模板、账号登录和绳结公共技能。路线、山峰、行程和导航模块尚未开始实现；服务端不注册 `/api/v1/mountains*` 或 `/api/v1/routes*`，也不再通过 repo-local `content/` 文件树启动加载公共内容。

服务端分层：

- `services/api/v1`：HTTP 路由、DTO、mock 登录、认证、错误响应、Redis read-through cache、装备导入导出、绳结公开读接口、装备模板公开读接口和管理员绳结媒体上传接口。
- `crates/domain`：装备、装备模板、技能、用户和反馈等领域模型、枚举和校验规则。
- `crates/db`：SeaORM 连接、repository、用户会话、装备、绳结、媒体资源和装备模板持久化。
- `crates/migration`：数据库 schema 迁移。
- `crates/importer`：Knots3D metadata 解析边界；导入结果写 DB，不被 API 启动直接读取为内存 catalog。
- `packages/shared-types` / `packages/api-client-ts`：客户端复用 DTO 和 API client。
- `apps/ios`：SwiftUI 原生端，使用 MVVM、repository、URLSession/Codable 和 Keychain 会话存储复用同一套装备与技能体验。

## Database strategy

- Local development: SQLite (`sqlite://stellartrail.db`).
- Production recommendation: PostgreSQL.
- MySQL compatibility is no longer part of phase-one implementation; keep future compatibility conservative if needed.

`users` 支持微信 openid 与邮箱/用户名登录并存；密码按当前需求以 SHA-256 十六进制摘要保存，连续密码错误会累计失败次数并触发验证码门槛。`user_gear_items` 使用 `archived_at` 支撑“可用装备 / 历史装备”，并使用 `is_deleted` 作为真正软删除标记；`gear_atlas_items`、`user_feedback` 和 `upload_images` 也使用顶层 `is_deleted` 标记隐藏业务记录。金额以分为单位保存为 `purchase_price_cents`，重量以克为单位保存为 `weight_g`。

## Public data

- 绳结内容通过 `import-knots3d` 将 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库。
- 绳结媒体通过管理员上传接口写入 MinIO/S3-compatible object storage，并把 public URL 与 metadata 写入 `media_resources` / `knot_media_resources`。
- 装备模板由 API 启动 seed 逻辑向 `gear_templates`、`gear_template_categories` 和 `gear_template_items` 幂等写入默认系统模板。
- repo-local `content/assets`、`content/skills`、`content/mountains`、`content/routes` 和 `content/gear-templates` 已删除；公开 API 不从这些路径读取，也不通过 `/assets/*` 直接服务媒体。

## Cache strategy

`REDIS_URL` 为空时服务端不启用缓存；配置 Redis 后，`AppState` 会初始化 Redis-backed cache。当前缓存优先覆盖装备库的高频只读接口：

- `GET /api/v1/me/gears/categories`
- `GET /api/v1/me/gears/stats`
- `GET /api/v1/me/gears`
- `GET /api/v1/me/gears/:id`

缓存采用 read-through 模式，默认 TTL 为 `REDIS_GEAR_CACHE_TTL_SECONDS=30` 秒。每个用户有独立 gear cache version；创建、更新、归档、软删除、恢复和导入装备后会递增版本号，让后续读请求自动绕过旧 key，避免跨用户或写后读到旧数据。Redis 读写异常只会降级为直连数据库，不影响接口可用性。

## Production deployment topology

Production Docker assets keep Traefik, the official site, the Web App, and the API in separate compose entrypoints under `infra/production/`. The API compose file also owns PostgreSQL, Redis, MinIO, and the MinIO bucket initializer so the API can resolve its private dependencies by Docker service names (`postgres`, `redis`, `minio`) on the backend network. Only Traefik exposes public 80/443; the API and MinIO API are routed by Traefik labels, while PostgreSQL, Redis, and the MinIO console stay private.
