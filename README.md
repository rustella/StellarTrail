# StellarTrail

StellarTrail is a China-focused outdoor assistant: routes, mountains, gear planning, and field skills in one product.

## Product scope

MVP focuses on:

- WeChat Mini Program first.
- Rust API service.
- Content-driven mountain/route/skill catalog.
- User gear library and route-based packing checklist.
- Database abstraction prepared for SQLite, PostgreSQL, and MySQL.

## Repository layout

```text
StellarTrail/
  apps/
    wechat-miniprogram/     # 微信小程序端
  services/
    api/                    # Rust API server
  crates/
    domain/                 # Shared Rust domain models
    db/                     # DB config and repository boundary
    importer/               # Content importer boundary
    migration/              # Migration boundary
  packages/
    api-client-ts/          # TS API client shared by mini program / web / mobile
    shared-types/           # TS shared DTOs
  content/                  # Mountains, routes, skills, gear templates
  docs/                     # Product, architecture, API, content schema docs
  infra/                    # Local/dev deployment files
  scripts/                  # Dev helper scripts
```

## Quick start

```bash
# API
cargo run -p stellartrail-api

# Check Rust workspace
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

API defaults to `127.0.0.1:8080` and reads config from environment variables.

```bash
cp .env.example .env
DATABASE_URL=sqlite://stellartrail.db cargo run -p stellartrail-api
```

## Naming

- Product: **StellarTrail**
- Chinese placeholder: **星径**
- Repository: `StellarTrail`

## Outdoor skill knots

Import Knots3D metadata into the local SQLite database before exercising the knots API:

```bash
cargo run -p stellartrail-importer --bin import-knots3d -- \
  --metadata .hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json \
  --database-url sqlite://stellartrail.db
```

The API exposes `GET /api/skills`, `GET /api/skills/knots/list`, `GET /api/skills/knots/detail/:id`, and `GET /assets/*`. Locale is selected with `X-StellarTrail-Locale`; `?locale=` is intentionally unsupported.
