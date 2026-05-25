# 🌌 StellarTrail (Xunjing Xingye)

> An outdoor gear organization, route planning, and field-skills toolbox.

[中文](README.md)

<p align="center">
  <img alt="StellarTrail app icon" src="assets/brand/app-icon-180.png" width="96" height="96" />
</p>

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-English-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/WeChat%20Mini%20Program-Ready-07C160" />
  <img alt="Web" src="https://img.shields.io/badge/Web-Ready-0EA5E9" />
  <img alt="Native mobile" src="https://img.shields.io/badge/iOS%20%2F%20Android%20%2F%20HarmonyOS-Unavailable-lightgrey" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>WeChat Mini Program and Web are ready</strong> · <strong>Native mobile is paused</strong>
</p>

---

## ✨ At a glance

StellarTrail serves hikers, campers, and lightweight outdoor users. The app is organized around **gear organization**, **route planning**, **outdoor skills**, and the related preparation workflows around them.

The directly usable entry points today are the **WeChat Mini Program** and **Web**. Both cover account access, personal gear, gear atlas, knot skills, and feedback. Web adds administrator capabilities on top of the other client surfaces. Android, iOS, and HarmonyOS are currently unavailable and are not delivery targets for the active product.

| Product capability | Current notes                                                       |
| ------------------ | ------------------------------------------------------------------- |
| 🎒 Gear org        | Personal gear, gear atlas, and DB-backed gear templates             |
| 🗺️ Route planning  | Route-planning capabilities are still being developed               |
| 🧭 Outdoor skills  | Knot skills, media resources, and offline read-only cache           |
| 💬 Feedback/admin  | User feedback; Web adds administrator features on top of other apps |

## 🟢 WeChat Mini Program

<p align="center">
  <img alt="Xunjing Xingye WeChat Mini Program code" src="assets/brand/wechat-miniprogram-code.png" width="260" />
</p>

<p align="center">
  <strong>Scan with WeChat or search for “寻径星野”.</strong>
</p>

## 🌱 Current public data

| Type             | Source                                                                    |
| ---------------- | ------------------------------------------------------------------------- |
| 🪢 Knot skills   | Knots3D metadata imported into DB; media served from MinIO/object storage |
| 🎒 Gear template | Idempotent system defaults seeded into DB at API startup                  |

## 📱 Client support

| Client              | Status      | Notes                                                                                                     |
| ------------------- | ----------- | --------------------------------------------------------------------------------------------------------- |
| WeChat Mini Program | Ready       | Home, gear library, gear atlas, knot skills, profile, login/registration, feedback, and offline read-only |
| Web                 | Ready       | Covers the core client capabilities and adds administrator features                                       |
| Android             | Unavailable | Code remains in the repository, but it is not a runnable delivery target today                            |
| iOS                 | Unavailable | Code remains in the repository, but it is not a runnable delivery target today                            |
| HarmonyOS           | Unavailable | Not connected as an active delivery client                                                                |

## 🧭 Repository layout

```text
StellarTrail/
  apps/
    android/                # Android native client (Kotlin + Jetpack Compose)
    ios/                    # iOS native client (Swift + SwiftUI)
    macos/                  # macOS native client (Swift + SwiftUI)
    web/                    # Web App (TypeScript + Vite)
    wechat-miniprogram/     # WeChat Mini Program client (TypeScript + WXML + WXSS)
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

Copy the matching example when you need to override client endpoints:

| Client              | Example config                                          | Real config (do not commit)                     |
| ------------------- | ------------------------------------------------------- | ----------------------------------------------- |
| Web                 | `apps/web/.env.example`                                 | `apps/web/.env.local`                           |
| WeChat Mini Program | `apps/wechat-miniprogram/miniprogram/config.example.ts` | `apps/wechat-miniprogram/miniprogram/config.ts` |

Web reads `VITE_STELLARTRAIL_API_BASE_URL` and `VITE_STELLARTRAIL_ASSETS_BASE_URL`; local Vite development uses same-origin `/api/v1` by default and proxies it through `VITE_STELLARTRAIL_API_PROXY_TARGET` to the real or local API to avoid browser CORS failures. The WeChat Mini Program reads `miniprogram/config.ts` and falls back to placeholder endpoints when it is absent.

See [API docs](docs/api.md) for the complete API surface.

### 5. Deployment guide

Local Docker Compose integration testing and production Docker / Traefik notes now live in the [deployment guide](docs/deployment.md).

### 6. Open the WeChat Mini Program source

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`. The Mini Program currently covers Home, gear library, gear atlas, knot skills, Profile, login/registration, avatar/email binding, feedback, and offline read-only cache.

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

- 🚀 [Deployment guide](docs/deployment.md)
- 🏗️ [Architecture](docs/architecture.md)
- 🔌 [API docs](docs/api.md)
- 🧩 [Public data notes](docs/content-schema.md)

## 🏷️ Naming

- Product: **StellarTrail**
- Chinese name: **寻径星野**
- Repository: `StellarTrail`
