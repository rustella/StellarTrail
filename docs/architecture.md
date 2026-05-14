# Architecture

## Runtime

```text
WeChat Mini Program
        |
        | HTTPS JSON API
        v
Rust API service (axum)
        |
        | repository boundary
        v
SQLite / PostgreSQL / MySQL
```

## Monorepo principles

- Keep API, mini program, shared types, and content schema in one repository while product shape is changing quickly.
- Keep domain types in `crates/domain` and mirror public DTOs in `packages/shared-types` until OpenAPI code generation is added.
- Store route/mountain/skill seed content in `content/` and import it into the database.

## Database strategy

- Local development: SQLite.
- Production recommendation: PostgreSQL.
- MySQL compatibility: keep SQL and types conservative until a real need appears.
- Avoid DB-specific features in MVP. Add PostGIS/search engine later when route discovery needs it.
