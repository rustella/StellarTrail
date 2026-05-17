# Architecture

## Runtime

```text
WeChat Mini Program
        |
        | HTTPS JSON API
        v
Rust API service (axum, Rust 2024)
        |
        | repository boundary (SeaORM) + content catalog loader
        v
SQLite / PostgreSQL
        ^
        |
optional Redis cache (read-through gear API cache)

content/ YAML + Markdown
        |
        v
crates/importer -> AppState in-memory public catalog
```

## Phase-one scope

第一期核心仍是装备库管理；本轮补齐只读内容目录 API，用于把已有山峰、路线、技能和装备模板种子内容暴露给前端，不进入路线导航、社区或交易等复杂能力。

服务端分层：

- `services/api`：HTTP 路由、DTO、mock 登录、认证、错误响应、Redis read-through cache、装备导入导出和只读内容 API。
- `crates/domain`：装备分类、状态、共享状态、山峰/路线/技能枚举和校验规则。
- `crates/db`：SeaORM 连接、repository、用户会话和装备持久化。
- `crates/migration`：`users`、`sessions`、邮箱验证码、密码登录字段和 `user_gear_items` 迁移。
- `crates/importer`：解析 `content/` 下 YAML/Markdown，在 API 启动时加载为内存只读 catalog。
- `packages/shared-types` / `packages/api-client-ts`：小程序侧复用 DTO 和 API client。

## Database strategy

- Local development: SQLite (`sqlite://stellartrail.db`).
- Production recommendation: PostgreSQL.
- MySQL compatibility is no longer part of phase-one implementation; keep future compatibility conservative if needed.

`users` 支持微信 openid 与邮箱/用户名登录并存；密码按当前需求以 SHA-256 十六进制摘要保存，连续密码错误会累计失败次数并触发验证码门槛。`user_gear_items` 使用软删除字段 `archived_at` 支撑“可用装备 / 历史装备”。金额以分为单位保存为 `purchase_price_cents`，重量以克为单位保存为 `weight_g`。

## Public content catalog

`ApiConfig::content_dir` 默认读取 `CONTENT_DIR=content`。`build_state` 会通过 `crates/importer` 解析山峰、路线、非绳结技能 Markdown front matter 和装备模板，并将 `ContentCatalog` 放入 `AppState`。公共接口包括 `/api/mountains*`、`/api/routes*`、`/api/skills*` 和 `/api/gear-templates*`。当前 catalog 是启动时加载的只读内存数据，不写入 DB。

## Cache strategy

`REDIS_URL` 为空时服务端不启用缓存；配置 Redis 后，`AppState` 会初始化 Redis-backed cache。当前缓存优先覆盖装备库的高频只读接口：

- `GET /api/me/gears/categories`
- `GET /api/me/gears/stats`
- `GET /api/me/gears`
- `GET /api/me/gears/:id`

缓存采用 read-through 模式，默认 TTL 为 `REDIS_GEAR_CACHE_TTL_SECONDS=30` 秒。每个用户有独立 gear cache version；创建、更新、归档、恢复和导入装备后会递增版本号，让后续读请求自动绕过旧 key，避免跨用户或写后读到旧数据。Redis 读写异常只会降级为直连数据库，不影响接口可用性。

## Production deployment topology

Production Docker assets keep Traefik, the official site, the Web App, and the API in separate compose entrypoints under `infra/production/`. The API compose file also owns PostgreSQL, Redis, MinIO, and the MinIO bucket initializer so the API can resolve its private dependencies by Docker service names (`postgres`, `redis`, `minio`) on the backend network. Only Traefik exposes public 80/443; the API and MinIO API are routed by Traefik labels, while PostgreSQL, Redis, and the MinIO console stay private.
