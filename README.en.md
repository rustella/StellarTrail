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
  <strong>Mini Program ready</strong> · <strong>Android ready</strong> · <strong>Gear library</strong> · <strong>Knot skills</strong>
</p>

---

## ✨ At a glance

StellarTrail serves hikers, campers, and lightweight outdoor users in China. The product is designed around one preparation loop: choose a route, understand difficulty, seasonality and risks, generate a packing checklist, compare it with the user's gear library, and learn the related outdoor skills.

The current code supports two user-facing entry points: the **WeChat Mini Program** and the native **Android** app. Both clients cover account access, the personal gear library, knot skills, and profile surfaces while sharing the same Rust service capabilities. Web, iOS, and macOS code also live in the repository for showcase/admin and Apple-ecosystem expansion. Phase one focuses on **personal gear library management**, DB-backed gear templates, and knot skills. Route and mountain modules are not implemented yet.

| Module            | Current goal                                                           |
| ----------------- | ---------------------------------------------------------------------- |
| 📱 Clients        | WeChat Mini Program + Android cover the MVP user flow                  |
| 🎒 Gear prep      | Manage personal gear and serve DB-backed gear templates                |
| 🧭 Skills toolbox | Start with knots; media is delivered through MinIO/object storage      |
| 🧱 Rust service   | Provide account, gear, skills, atlas, feedback, and admin capabilities |
| 🗺️ Route wiki     | Follow-up module for mountains, routes, difficulty, and risks          |

## 🚀 MVP scope

Phase one focuses on:

- 🧑‍💻 WeChat Mini Program: WeChat login, email/username password login, registration, avatar upload, email binding, and refresh-token session renewal.
- 🤖 Android: email/username password login, registration, encrypted local token storage, automatic 401 refresh, and Profile configuration.
- 🎒 Personal gear-library CRUD.
- 🔎 Search, category filtering, status filtering, sorting, and pagination.
- 📊 Gear count, total value, total weight, and category counts.
- 🗃️ Available / historical gear via soft archive and restore.
- 🏷️ Tags, status/location fields, sharing toggle, and notes.
- 📤 JSON import and CSV export.
- 🧭 Public gear atlas browsing, personal gear submissions, and admin review.
- 💬 Feedback image upload, feedback submission, and admin review.
- 📡 WeChat Mini Program offline read-only support for previously loaded skills, gear atlas, personal gear data, and viewed knot media.
- 🗄️ SeaORM data access: SQLite by default for local development, PostgreSQL recommended for production.
- ⚡ Optional Redis read-through cache for high-traffic gear read APIs.

Routes, trips, skills, realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for phase one.

## 🌱 Current public data

| Type             | Source                                                                    |
| ---------------- | ------------------------------------------------------------------------- |
| 🪢 Knot skills   | Knots3D metadata imported into DB; media served from MinIO/object storage |
| 🎒 Gear template | Idempotent system defaults seeded into DB at API startup                  |

## 📱 Client support

| Client              | Status    | Coverage                                                                                                                          |
| ------------------- | --------- | --------------------------------------------------------------------------------------------------------------------------------- |
| WeChat Mini Program | Supported | Home, gear library/atlas, knot skills, profile, login/registration, avatar/email binding, offline read-only cache, and feedback   |
| Android             | Supported | Native Kotlin + Compose app, login/registration, gear list/detail/form, skills list/detail, home stats, and Profile configuration |
| Web                 | Available | Web App and admin-facing surfaces that reuse the same public data, gear, and admin capabilities                                   |
| iOS / macOS         | In repo   | SwiftUI app shells plus StellarTrailKit for shared account, gear, and skill models; still being refined                           |

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
curl http://127.0.0.1:8080/api/v1/meta
```

Knot media is uploaded through MinIO/S3-compatible object storage. Public read APIs return only DB-backed media URLs and no longer derive knot media paths from `/assets/*`. The API now keeps one shared `minio` connection configuration, while feedback images and knot media use separate business buckets through `object_storage.bucket` and `knots_media_storage.bucket`. Administrator permission is stored in the database `admin_roles` table: an existing, non-deleted `stellarisw` user is seeded as `super_admin` by migration, and `super_admin` users can grant or revoke regular `admin` users through `/api/v1/admin/admins`. Both `admin` and `super_admin` can call Knot media upload, Gear Atlas review, feedback review, and `GET /api/v1/admin/api-usage`. Usage reporting is asynchronous and stores only matched route templates plus aggregate counts; it does not store query strings, request bodies, Authorization headers, tokens, cookies, IP addresses, or User-Agent values.

### 4. Configure client endpoints

Checked-in clients default to placeholder endpoints:

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

Web reads `VITE_STELLARTRAIL_API_BASE_URL` and `VITE_STELLARTRAIL_ASSETS_BASE_URL`; local Vite development uses same-origin `/api/v1` by default and proxies it through `VITE_STELLARTRAIL_API_PROXY_TARGET` to the real or local API to avoid browser CORS failures. Android reads `config.properties` during the Gradle build and writes `BuildConfig`; iOS/macOS first read `ClientConfig.plist` from the app bundle and fall back to placeholder endpoints without production domain probing when it is absent.

Currently implemented service capabilities:

```http
GET /healthz
GET /api/v1/meta

POST /api/v1/auth/wechat-login
POST /api/v1/auth/email-verification-code
POST /api/v1/auth/email-login-code
POST /api/v1/auth/email-login
POST /api/v1/auth/password-reset-code
POST /api/v1/auth/password-reset
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh
POST /api/v1/auth/captcha
POST /api/v1/me/email-binding-code
POST /api/v1/me/email-binding
GET /api/v1/me/profile
PUT|POST /api/v1/me/profile/avatar

GET /api/v1/skills
GET /api/v1/skills/knots/list
GET /api/v1/skills/knots/filters
GET /api/v1/skills/knots/offline-manifest
GET /api/v1/skills/knots/detail/:id
GET /api/v1/gear-templates
GET /api/v1/gear-templates/:id

GET /api/v1/gear-atlas
GET /api/v1/gear-atlas/:id
GET|POST /api/v1/me/gear-atlas-submissions
POST /api/v1/me/gears/:id/atlas-submission

GET /api/v1/me/gears/categories
GET /api/v1/me/gears/stats
GET /api/v1/me/gears/spec-key-rankings
GET /api/v1/me/gears/tag-suggestions
GET|POST /api/v1/me/gears
GET|PATCH|DELETE /api/v1/me/gears/:id
POST /api/v1/me/gears/:id/delete
POST /api/v1/me/gears/:id/undelete
POST /api/v1/me/gears/:id/restore
GET /api/v1/me/gears/export
POST /api/v1/me/gears/import

POST /api/v1/me/uploads
GET /api/v1/me/uploads/:id
POST /api/v1/me/feedback

GET /api/v1/admin/api-usage
POST|DELETE /api/v1/admin/admins
PUT /api/v1/admin/skills/knots/:knot_id/media/:asset_id
GET /api/v1/admin/gear-atlas-submissions
GET|PATCH|DELETE /api/v1/admin/gear-atlas-submissions/:id
POST /api/v1/admin/gear-atlas-submissions/:id/restore
POST /api/v1/admin/gear-atlas-submissions/:id/approve
POST /api/v1/admin/gear-atlas-submissions/:id/reject
GET /api/v1/admin/feedback
DELETE /api/v1/admin/feedback/:id
POST /api/v1/admin/feedback/:id/restore
GET /api/v1/admin/feedback-images/:id
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

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`. The Mini Program currently covers Home, gear library, gear atlas, knot skills, Profile, login/registration, avatar/email binding, feedback, and offline read-only cache.

### 8. Android client

The Android app lives in `apps/android` and uses Kotlin, Jetpack Compose, Material 3, Navigation Compose, Ktor Client, and kotlinx.serialization. It currently covers Home, login/registration, gear list/detail/create/edit, skills list/detail, and Profile settings. Gradle reads the default API and image asset endpoints from `apps/android/config.properties`; copy the example and switch it to `http://10.0.2.2:8080` for local emulator testing, or override the API base URL temporarily from the Profile screen.

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
