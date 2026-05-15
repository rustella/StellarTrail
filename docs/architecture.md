# Architecture

## Runtime

```text
WeChat Mini Program
        |
        | HTTPS JSON API
        v
Rust API service (axum, Rust 2024)
        |
        | repository boundary (SeaORM)
        v
SQLite / PostgreSQL
```

## Phase-one scope

第一期仅实现装备库管理，不实现路线、行程、技能、路线装备建议等其它 MVP 模块。

服务端分层：

- `services/api`：HTTP 路由、DTO、mock 登录、认证、错误响应、导入导出。
- `crates/domain`：装备分类、状态、共享状态、校验规则。
- `crates/db`：SeaORM 连接、repository、用户会话和装备持久化。
- `crates/migration`：`users`、`sessions`、`user_gear_items` 迁移。
- `packages/shared-types` / `packages/api-client-ts`：小程序侧复用 DTO 和 API client。

## Database strategy

- Local development: SQLite (`sqlite://stellartrail.db`).
- Production recommendation: PostgreSQL.
- MySQL compatibility is no longer part of phase-one implementation; keep future compatibility conservative if needed.

`user_gear_items` 使用软删除字段 `archived_at` 支撑“可用装备 / 历史装备”。金额以分为单位保存为 `purchase_price_cents`，重量以克为单位保存为 `weight_g`。
