# StellarTrail

> A China-focused outdoor route encyclopedia, gear preparation assistant, and offline field-skills toolbox.

[中文](README.md)

## Product positioning

StellarTrail serves hikers, campers, and lightweight outdoor users in China. The product is designed around one preparation loop: choose a route, understand difficulty, seasonality and risks, generate a packing checklist, compare it with the user's gear library, and learn the related outdoor skills.

The current entry point is a WeChat Mini Program. A Rust API service provides backend capabilities, while routes, mountains, skills, and gear templates are driven by YAML/Markdown content so the project can build a high-quality knowledge base quickly.

## MVP scope

The first release focuses on:

- WeChat login placeholder and account model.
- Mountain and route catalog.
- Route details: difficulty, season, risk, transport, and gear suggestions.
- User gear library.
- Route-based packing checklist generation.
- Skill catalog for knots, camping, packing, navigation, weather, and first aid.
- Content importing from YAML/Markdown.
- Database abstraction: SQLite for local development, PostgreSQL recommended for production, and conservative MySQL compatibility.

Realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for the MVP.

## Current seed content

- Mountain: Wugongshan.
- Route: Wugongshan classic 2-day / 1-night traverse.
- Skill: Taut-line hitch.
- Gear template: beginner backpacking basics.

## Repository layout

```text
StellarTrail/
  apps/
    wechat-miniprogram/     # WeChat Mini Program client
  services/
    api/                    # Rust axum API service
  crates/
    domain/                 # Shared Rust domain models
    db/                     # DB config and repository boundary
    importer/               # Content importer boundary
    migration/              # Migration boundary
  packages/
    api-client-ts/          # TS API client for Mini Program / web / mobile
    shared-types/           # Shared TS DTO types
  content/                  # Mountains, routes, skills, and gear templates
  docs/                     # Product, architecture, API, and content schema docs
  infra/                    # Local/dev deployment files
  scripts/                  # Development helper scripts
```

## Quick start

### Prerequisites

- Rust stable toolchain. The repository includes `rust-toolchain.toml` and expects `rustfmt` and `clippy`.
- Node.js 22+ and npm.
- WeChat DevTools for Mini Program debugging.

### Install dependencies

```bash
npm install
```

### Start the API

```bash
cp .env.example .env
cargo run -p stellartrail-api
```

The API listens on `127.0.0.1:8080` by default and reads configuration from environment variables. The default database URL is `sqlite://stellartrail.db`.

Use these endpoints for local smoke testing:

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/meta
```

Currently implemented endpoints:

```http
GET /healthz
GET /api/meta
```

### Open the WeChat Mini Program

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`.

## Common checks

```bash
# Frontend / TypeScript workspace checks
npm run check

# Markdown / JSON / YAML / TS formatting checks
npm run format:check

# Rust workspace checks
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Documentation

- [MVP scope](docs/mvp.md)
- [Architecture](docs/architecture.md)
- [API draft](docs/api.md)
- [Content schema draft](docs/content-schema.md)

## Naming

- Product: **StellarTrail**
- Chinese placeholder: **星径**
- Repository: `StellarTrail`
