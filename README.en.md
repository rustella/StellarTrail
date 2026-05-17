# 🌌 StellarTrail

> A China-focused outdoor route encyclopedia, gear preparation assistant, and offline field-skills toolbox.

[中文](README.md)

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-English-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/Client-WeChat%20Mini%20Program-07C160" />
  <img alt="Android" src="https://img.shields.io/badge/Client-Android%20%2B%20Compose-3DDC84" />
  <img alt="iOS" src="https://img.shields.io/badge/Client-iOS%20%2B%20SwiftUI-000000" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>Pick routes</strong> · <strong>Understand risks</strong> · <strong>Pack smarter</strong> · <strong>Learn field skills</strong>
</p>

---

## ✨ At a glance

StellarTrail serves hikers, campers, and lightweight outdoor users in China. The product is designed around one preparation loop: choose a route, understand difficulty, seasonality and risks, generate a packing checklist, compare it with the user's gear library, and learn the related outdoor skills.

The current entry point is a **WeChat Mini Program**. Native **Android** and **iOS** clients share the same core gear and skills experience. A **Rust API service** provides backend capabilities. Phase one focuses on **personal gear library management**, DB-backed gear templates, and knot skills. Route and mountain modules are not implemented yet.

| Module            | Current goal                                                      |
| ----------------- | ----------------------------------------------------------------- |
| 🎒 Gear prep      | Manage personal gear and serve DB-backed gear templates           |
| 🧭 Skills toolbox | Start with knots; media is delivered through MinIO/object storage |
| 🧱 Rust backend   | Provide stable APIs, domain models, and data boundaries           |
| 🗺️ Route wiki     | Follow-up module for mountains, routes, difficulty, and risks     |

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

Routes, trips, skills, realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for phase one.

## 🌱 Current public data

| Type             | Source                                                                    |
| ---------------- | ------------------------------------------------------------------------- |
| 🪢 Knot skills   | Knots3D metadata imported into DB; media served from MinIO/object storage |
| 🎒 Gear template | Idempotent system defaults seeded into DB at API startup                  |

## 🧭 Repository layout

```text
StellarTrail/
  apps/
    android/                # Android native client (Kotlin + Jetpack Compose)
    ios/                    # iOS native client (Swift + SwiftUI)
    web/                    # Web App
    wechat-miniprogram/     # WeChat Mini Program client
  services/
    api/                    # Rust axum API service
  crates/
    domain/                 # Shared Rust domain models
    db/                     # DB config and repository boundary
    importer/               # Knots3D metadata importer boundary
    migration/              # Migration boundary
  packages/
    api-client-ts/          # TS API client for Mini Program / web / mobile
    shared-types/           # Shared TS DTO types
  docs/                     # Product, architecture, API, and content schema docs
  infra/                    # Local integration-test and production deployment config
  scripts/                  # Development helper scripts
```

## ⚡ Quick start

### 1. Prerequisites

- 🦀 Rust stable toolchain with Rust 2024 edition. The repository includes `rust-toolchain.toml` and expects `rustfmt` and `clippy`.
- 🟢 Node.js 22+ and npm.
- 💬 WeChat DevTools for Mini Program debugging.
- 🤖 Android Studio / Android SDK 36 + JDK 21 for Android debugging.
- ⚡ Redis 7+ (optional; set `REDIS_URL` to enable server-side caching).

### 2. Install dependencies

```bash
npm install
```

### 3. Start the API

```bash
cp .env.example .env
# Optional: copy the YAML template to the Git-ignored config.yaml for local or production-style configuration.
cp config.example.yaml config.yaml
cargo run -p stellartrail-api --bin migrate -- up
cargo run -p stellartrail-api
```

The API listens on `127.0.0.1:8080` by default. Startup loads `.env`, then reads root `config.yaml` when present or the YAML file named by `CONFIG_PATH`, and finally lets environment variables override YAML values. The default database URL is `sqlite://stellartrail.db`. Local mock login is enabled with `APP_ENV=local` + `WECHAT_MOCK_LOGIN=true`; real WeChat login requires `WECHAT_MOCK_LOGIN=false`, `WECHAT_APP_ID`, and `WECHAT_APP_SECRET`. Production email-code delivery uses SMTP: set `MAIL_ENABLED=true`, `MAIL_SMTP_HOST=smtp.example.invalid`, `MAIL_SMTP_USERNAME=[REDACTED]`, and inject `MAIL_SMTP_PASSWORD` plus the sender address through ignored `config.yaml` or a secret manager. Email codes now cover registration, email-code login, and password reset. To enable Redis caching, set `REDIS_URL=redis://127.0.0.1:6379/0`; `REDIS_GEAR_CACHE_TTL_SECONDS` controls the gear read cache TTL. `config.example.yaml` is committed, while real `config.yaml` / `config.*.yaml` files are ignored by Git.

Use these endpoints for local smoke testing:

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/meta
```

Knot media is uploaded through MinIO/S3-compatible object storage. Public read APIs return only DB-backed media URLs and no longer derive knot media paths from `/assets/*`. The API now keeps one shared `minio` connection configuration, while feedback images and knot media use separate business buckets through `object_storage.bucket` and `knots_media_storage.bucket`. After configuring the admin allowlist (`ADMIN_USER_IDS`, `ADMIN_EMAILS`, or `ADMIN_USERNAMES`), use `npm run knots:upload-media -- --dry-run` to inspect the Knots3D plan and the same script to call the admin upload API.

### 4. Configure client endpoints

All clients default to the production endpoints:

- API base URL: `https://api.example.invalid`
- Image asset / CORS asset origin: `https://assets.example.invalid`

Real client config files stay local or in the build environment and are ignored by Git; the repository commits only example files. Copy the matching example when you need to override endpoints:

| Client              | Example config                                                    | Real config (do not commit)                               |
| ------------------- | ----------------------------------------------------------------- | --------------------------------------------------------- |
| Web                 | `apps/web/.env.example`                                           | `apps/web/.env.local`                                     |
| WeChat Mini Program | `apps/wechat-miniprogram/miniprogram/config.example.ts`           | `apps/wechat-miniprogram/miniprogram/config.ts`           |
| Android             | `apps/android/config.example.properties`                          | `apps/android/config.properties`                          |
| iOS                 | `apps/ios/StellarTrail/Resources/ClientConfig.example.plist`      | `apps/ios/StellarTrail/Resources/ClientConfig.plist`      |
| macOS               | `apps/macos/StellarTrailMac/Resources/ClientConfig.example.plist` | `apps/macos/StellarTrailMac/Resources/ClientConfig.plist` |

Web reads `VITE_STELLARTRAIL_API_BASE_URL` and `VITE_STELLARTRAIL_ASSETS_BASE_URL`; Android reads `config.properties` during the Gradle build and writes `BuildConfig`; iOS/macOS first read `ClientConfig.plist` from the app bundle and fall back to the production defaults when it is absent.

Currently implemented endpoints:

```http
GET /healthz
GET /api/meta
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/email-login-code
POST /api/auth/email-login
POST /api/auth/password-reset-code
POST /api/auth/password-reset
POST /api/me/email-binding-code
POST /api/me/email-binding
POST /api/auth/register
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/captcha
GET /api/skills
GET /api/skills/knots/list
GET /api/skills/knots/detail/:id
PUT /api/admin/skills/knots/:knot_id/media/:asset_id
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
```

### 5. Start PostgreSQL + Redis + API with Docker Compose

You can also run a one-shot local integration test with Docker Compose:

```bash
COMPOSE_PROJECT_NAME=stellartrail_it API_HOST_PORT=18080 POSTGRES_HOST_PORT=15432 REDIS_HOST_PORT=16379 \
  bash infra/test/integration-test.sh
```

The script starts PostgreSQL, Redis caching, and the API service, validates them with registration, password login, email-code login, and password reset, and always runs `docker compose down -v --remove-orphans` when the test exits or fails. Inject real WeChat, database, and SMTP secrets only through secure production channels.

### 6. Production Docker / Traefik deployment config

Production deployment config is split under `infra/production/`, with `/www/service/stellartail` as the server deploy root:

- `infra/production/traefik/docker-compose.yml`: the only public edge entrypoint, exposing 80/443 and using Let’s Encrypt for automatic certificate issuance and renewal.
- `infra/production/site/docker-compose.yml`: official site; `site.example.invalid` is canonical and `www.example.invalid` redirects to the apex with HTTP 301.
- `infra/production/web/docker-compose.yml`: Web App on `app.example.invalid`.
- `infra/production/api/docker-compose.yml`: API service plus private PostgreSQL, Redis, and MinIO dependencies; the API reaches them through Docker service names `postgres`, `redis`, and `minio`, root `config.yaml` is mounted to `/app/config.yaml:ro`, `infra/production/api/compose-from-config.sh` derives PostgreSQL, Redis, and MinIO Compose runtime variables from that YAML, and `assets.example.invalid` reaches only the MinIO API through Traefik while the MinIO console stays private.
- `infra/production/domains.example.yaml` and `infra/production/api/config.production.example.yaml`: committed non-sensitive domain and API config examples.

The production API no longer uses `infra/production/api/.env`; real `config.yaml`, ACME storage, and production secret files must stay on the production server or in a secure channel. The repository `.gitignore` ignores those files; commit only API `config.example.yaml` and `*.example.yaml`.

### 7. Open the WeChat Mini Program

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`.

### 8. Android client

The Android app lives in `apps/android` and uses Kotlin, Jetpack Compose, Material 3, Navigation Compose, Ktor Client, and kotlinx.serialization. Gradle reads the default API and image asset endpoints from `apps/android/config.properties`; copy the example and switch it to `http://10.0.2.2:8080` for local emulator testing, or override the API base URL temporarily from the Profile screen.

```bash
./gradlew :apps:android:assembleDebug
./gradlew :apps:android:testDebugUnitTest
./gradlew :apps:android:lintDebug
```

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
- 🧩 [Public data notes](docs/content-schema.md)

## 🏷️ Naming

- Product: **StellarTrail**
- Chinese placeholder: **星径**
- Repository: `StellarTrail`
