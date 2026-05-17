# 🌌 StellarTrail

> A China-focused outdoor route encyclopedia, gear preparation assistant, and offline field-skills toolbox.

[中文](README.md)

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-English-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/Client-WeChat%20Mini%20Program-07C160" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-YAML%20%2B%20Markdown-8A2BE2" />
</p>

<p align="center">
  <strong>Pick routes</strong> · <strong>Understand risks</strong> · <strong>Pack smarter</strong> · <strong>Learn field skills</strong>
</p>

---

## ✨ At a glance

StellarTrail serves hikers, campers, and lightweight outdoor users in China. The product is designed around one preparation loop: choose a route, understand difficulty, seasonality and risks, generate a packing checklist, compare it with the user's gear library, and learn the related outdoor skills.

The current entry point is a **WeChat Mini Program**. A **Rust API service** provides backend capabilities. Phase one focuses on **personal gear library management**, while routes, mountains, skills, and gear templates remain follow-up content directions.

| Module            | Current goal                                             |
| ----------------- | -------------------------------------------------------- |
| 🗺️ Route wiki     | Curate mountains, routes, difficulty, seasons, and risks |
| 🎒 Gear prep      | Generate packing lists and compare personal gear         |
| 🧭 Skills toolbox | Cover knots, camping, navigation, weather, first aid     |
| 📦 Content import | Iterate seed content quickly with YAML/Markdown          |
| 🧱 Rust backend   | Provide stable APIs, domain models, and data boundaries  |

## 🚀 MVP scope

Phase one focuses on:

- 🧑‍💻 WeChat Mini Program login, email/username password login, account model, and refresh-token session renewal.
- 🎒 Personal gear-library CRUD.
- 🔎 Search, category filtering, status filtering, sorting, and pagination.
- 📊 Gear count, total value, total weight, and category counts.
- 🗃️ Available / historical gear via soft archive and restore.
- 🏷️ Tags, status/location fields, sharing toggle, and notes.
- 📤 JSON import and CSV export.
- 🗄️ SeaORM data access: SQLite by default for local development, PostgreSQL recommended for production.
- ⚡ Optional Redis read-through cache for high-traffic gear read APIs.
- 🖼️ User feedback with safe image uploads stored in a private MinIO/S3-compatible bucket.
- 🪢 Unauthenticated public skills/knots read APIs protected by rate limits, cache headers, ETags, and query bounds.

Routes, trips, skills, realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for phase one.

## 🌱 Current seed content

| Type             | Content                                     |
| ---------------- | ------------------------------------------- |
| ⛰️ Mountain      | Wugongshan                                  |
| 🥾 Route         | Wugongshan classic 2-day / 1-night traverse |
| 🪢 Skill         | Taut-line hitch                             |
| 🎒 Gear template | Beginner backpacking basics                 |

## 🧭 Repository layout

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

## ⚡ Quick start

### 1. Prerequisites

- 🦀 Rust stable toolchain with Rust 2024 edition. The repository includes `rust-toolchain.toml` and expects `rustfmt` and `clippy`.
- 🟢 Node.js 22+ and npm.
- 💬 WeChat DevTools for Mini Program debugging.
- ⚡ Redis 7+ (optional; set `REDIS_URL` to enable server-side caching).
- 🪣 MinIO or S3-compatible object storage for feedback image uploads.

### 2. Install dependencies

```bash
npm install
```

### 3. Start the API

```bash
cp .env.example .env
cargo run -p stellartrail-api --bin migrate -- up
cargo run -p stellartrail-api
```

The API listens on `127.0.0.1:8080` by default and reads configuration from environment variables. The default database URL is `sqlite://stellartrail.db`. Local mock login is enabled with `APP_ENV=local` + `WECHAT_MOCK_LOGIN=true`; real WeChat login requires `WECHAT_MOCK_LOGIN=false`, `WECHAT_APP_ID`, and `WECHAT_APP_SECRET`. To enable Redis caching, set `REDIS_URL=redis://127.0.0.1:6379/0`; `REDIS_GEAR_CACHE_TTL_SECONDS` controls the gear API cache TTL. Feedback image uploads use a private MinIO/S3-compatible bucket configured via `OBJECT_STORAGE_ENDPOINT`, `OBJECT_STORAGE_BUCKET`, `OBJECT_STORAGE_ACCESS_KEY_ID`, and `OBJECT_STORAGE_SECRET_ACCESS_KEY`; `/api/me/uploads/:id` proxies authenticated downloads so no public URL is exposed.

Use these endpoints for local smoke testing:

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/meta
```

Currently implemented endpoints:

```http
GET /healthz
GET /api/meta
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/register
POST /api/auth/login
POST /api/auth/captcha
GET /api/mountains
GET /api/mountains/:id
GET /api/routes
GET /api/routes/:id
GET /api/skills
GET /api/skills/knots/list
GET /api/skills/knots/detail/:id
GET /assets/*
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
POST /api/me/uploads
GET /api/me/uploads/:id
POST /api/me/feedback
```

### 4. Start PostgreSQL + Redis + MinIO + API with Docker Compose

You can also run a one-shot local integration test with Docker Compose:

```bash
COMPOSE_PROJECT_NAME=stellartrail_it API_HOST_PORT=18080 POSTGRES_HOST_PORT=15432 REDIS_HOST_PORT=16379 MINIO_API_HOST_PORT=19000 MINIO_CONSOLE_HOST_PORT=19001 \
  bash infra/integration-test.sh
```

The script starts PostgreSQL, Redis caching, a private MinIO bucket, and the API service, then validates username/password registration/login, gear APIs, feedback image upload/download, and feedback submission. It always runs `docker compose down -v --remove-orphans` when the test exits or fails, cleaning containers and volumes. Inject real WeChat, database, and object-storage secrets only through secure production channels.

### 5. Open the WeChat Mini Program

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`.

## 🧪 Common checks

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

## 📚 Documentation

- 📌 [MVP scope](docs/mvp.md)
- 🏗️ [Architecture](docs/architecture.md)
- 🔌 [API draft](docs/api.md)
- 🧩 [Content schema draft](docs/content-schema.md)

## 🏷️ Naming

- Product: **StellarTrail**
- Chinese placeholder: **星径**
- Repository: `StellarTrail`
