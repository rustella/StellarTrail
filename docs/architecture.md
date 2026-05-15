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

content/ YAML + Markdown
        |
        v
crates/importer -> AppState in-memory public catalog
```

## Phase-one scope

第一期核心仍是装备库管理；本轮补齐只读内容目录 API，用于把已有山峰、路线、技能和装备模板种子内容暴露给前端，不进入路线导航、社区或交易等复杂能力。

服务端分层：

- `services/api`：HTTP 路由、DTO、mock 登录、认证、错误响应、装备导入导出和只读内容 API。
- `crates/domain`：装备分类、状态、共享状态、山峰/路线/技能枚举和校验规则。
- `crates/db`：SeaORM 连接、repository、用户会话和装备持久化。
- `crates/migration`：`users`、`sessions`、`user_gear_items` 迁移。
- `crates/importer`：解析 `content/` 下 YAML/Markdown，在 API 启动时加载为内存只读 catalog。
- `packages/shared-types` / `packages/api-client-ts`：小程序侧复用 DTO 和 API client。

## Database strategy

- Local development: SQLite (`sqlite://stellartrail.db`).
- Production recommendation: PostgreSQL.
- MySQL compatibility is no longer part of phase-one implementation; keep future compatibility conservative if needed.

`user_gear_items` 使用软删除字段 `archived_at` 支撑“可用装备 / 历史装备”。金额以分为单位保存为 `purchase_price_cents`，重量以克为单位保存为 `weight_g`。

## Public content catalog

`ApiConfig::content_dir` 默认读取 `CONTENT_DIR=content`。`build_state` 会通过 `crates/importer` 解析山峰、路线、技能 Markdown front matter 和装备模板，并将 `ContentCatalog` 放入 `AppState`。公共接口包括 `/api/mountains*`、`/api/routes*`、`/api/skills*` 和 `/api/gear-templates*`。当前 catalog 是启动时加载的只读内存数据，不写入 DB。
