# StellarTrail Architecture Overview

## Product boundary

StellarTrail supports outdoor gear organization, route planning, and offline skill learning. The active product prioritizes the WeChat Mini Program, Web, and the Rust API for account access, personal gear inventory, gear templates, feedback, administrator workflows, and knot skills.

Out of scope unless explicitly requested: real-time navigation, social networking, e-commerce, complex track editing, and a broad route encyclopedia platform.

## Runtime architecture

- `apps/wechat-miniprogram` provides the Mini Program client with Home, Gear, Skills, Profile, and future route surfaces.
- `apps/web` provides the web client.
- `services/api` exposes the Rust Axum API and composes state, routes, errors, auth, repositories, migrations, startup gear-template seed, and object-storage-backed media flows.
- `crates/domain` owns business types, enums, and validation helpers.
- `crates/db` owns database connections and repository implementations.
- `crates/migration` owns schema migrations.
- `packages/shared-types` and `packages/api-client-ts` provide shared DTOs and TypeScript client helpers.

## Data and content model

- Local development uses SQLite by default.
- PostgreSQL is the recommended production database.
- MySQL-compatible configuration may be kept where needed, but phase one should avoid MySQL-specific assumptions unless explicitly required.
- Gear templates are seeded into the database by the API; knot public content is DB-backed; media URLs come from MinIO/S3-compatible object storage metadata stored in DB.

## Layering rules

- API routes should handle HTTP, DTO conversion, authentication, and orchestration.
- Domain code should define business models, enums, and validation rules.
- DB code should hide SeaORM or SQL details behind repositories.
- Migration code should own schema DDL and reversible migration boundaries.

## Documentation and language rules

- Root `AGENTS.md`, `CLAUDE.md`, and every file under `.agent/` must be written in English.
- Code comments should be English. Server-side Rust should include rustdoc for modules and public or important items, plus inline comments for critical logic.
- Keep README, docs, shared types, API clients, and `.agent` knowledge synchronized when behavior changes.
