# StellarTrail Architecture Overview

## Product boundary

StellarTrail supports China-focused outdoor route planning, gear preparation, and offline skill learning. The first phase prioritizes the WeChat Mini Program and the Rust API. The core loop is route selection, route risk and season review, gear checklist generation, gear inventory comparison, and related skill learning.

Out of scope unless explicitly requested: real-time navigation, social networking, e-commerce, complex track editing, and a broad route encyclopedia platform.

## Runtime architecture

- `apps/wechat-miniprogram` provides the Mini Program client with five main tabs: Home, Routes, Gear, Skills, and Profile.
- `apps/web` provides the web client.
- `services/api` exposes the Rust Axum API and composes state, routes, errors, auth, repositories, migrations, and content catalogs.
- `crates/domain` owns business types, enums, and validation helpers.
- `crates/db` owns database connections and repository implementations.
- `crates/migration` owns schema migrations.
- `crates/importer` owns YAML and Markdown content parsing plus import boundaries.
- `packages/shared-types` and `packages/api-client-ts` provide shared DTOs and TypeScript client helpers.
- `content/` stores seed content for mountains, routes, skills, and gear templates.

## Data and content model

- Local development uses SQLite by default.
- PostgreSQL is the recommended production database.
- MySQL-compatible configuration may be kept where needed, but phase one should avoid MySQL-specific assumptions unless explicitly required.
- Content is authored in YAML and Markdown, then parsed through importer/API layers instead of being served directly from local `.hermes` resources.

## Layering rules

- API routes should handle HTTP, DTO conversion, authentication, and orchestration.
- Domain code should define business models, enums, and validation rules.
- DB code should hide SeaORM or SQL details behind repositories.
- Migration code should own schema DDL and reversible migration boundaries.
- Importer code should parse and normalize content without embedding HTTP concerns.

## Documentation and language rules

- Root `AGENTS.md`, `CLAUDE.md`, and every file under `.agent/` must be written in English.
- Code comments should be English. Server-side Rust should include rustdoc for modules and public or important items, plus inline comments for critical logic.
- Keep README, docs, shared types, API clients, and `.agent` knowledge synchronized when behavior changes.
