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
  <img alt="Android" src="https://img.shields.io/badge/Android-Ready-34D399" />
  <img alt="Other native clients" src="https://img.shields.io/badge/iOS%20%2F%20macOS%20%2F%20HarmonyOS-Unavailable-lightgrey" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-DB%20%2B%20MinIO-8A2BE2" />
</p>

<p align="center">
  <strong>WeChat Mini Program, Web, and Android are ready</strong> · <strong>iOS / macOS / HarmonyOS are unavailable</strong>
</p>

---

## ✨ At a glance

StellarTrail serves hikers, campers, and lightweight outdoor users. The app is organized around **gear organization**, **route planning**, **outdoor skills**, and the related preparation workflows around them.

The directly usable entry points today are the **WeChat Mini Program**, **Web**, and **Android**. They cover account access, personal gear, gear atlas, knot skills, and feedback. Web adds administrator capabilities on top of the other client surfaces. Android is a Kotlin + Jetpack Compose client and CI can produce signed release APKs. iOS, macOS, and HarmonyOS are currently unavailable and are not delivery targets for the active product.

| Product capability | Current notes                                                       |
| ------------------ | ------------------------------------------------------------------- |
| 🎒 Gear org        | Personal gear, gear atlas, and gear templates                       |
| 🗺️ Route planning  | Route-planning capabilities are still being developed               |
| 🧭 Outdoor skills  | Knot skills, media resources, and offline read-only cache           |
| 💬 Feedback/admin  | User feedback; Web adds administrator features on top of other apps |

## 🚀 Quick Try

### WeChat Mini Program

<p align="center">
  <img alt="Xunjing Xingye WeChat Mini Program code" src="assets/brand/wechat-miniprogram-code.png" width="260" />
</p>

<p align="center">
  <strong>Scan with WeChat or search for “寻径星野”.</strong>
</p>

Routes, trips, skills, realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for phase one.

## 🌱 Current public data

| Type             | Source                                                                |
| ---------------- | --------------------------------------------------------------------- |
| 🪢 Knot skills   | DB-backed public knot catalog; media served from MinIO/object storage |
| 🎒 Gear template | Idempotent system defaults seeded into DB at API startup              |

## 📱 Client support

| Client              | Status      | Notes                                                                                                     |
| ------------------- | ----------- | --------------------------------------------------------------------------------------------------------- |
| WeChat Mini Program | Ready       | Home, gear library, gear atlas, knot skills, profile, login/registration, feedback, and offline read-only |
| Web                 | Ready       | Covers the core client capabilities and adds administrator features                                       |
| Android             | Ready       | Native Kotlin + Jetpack Compose client; CI can build signed release APKs                                  |
| iOS                 | Unavailable | Code remains in the repository, but it is not a runnable delivery target today                            |
| macOS               | Unavailable | Code remains in the repository, but it is not a runnable delivery target today                            |
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
    migration/              # Migration boundary
  packages/
    api-client-ts/          # TS API client for Mini Program / web / mobile
    apple/StellarTrailKit/  # Shared Swift package for iOS / macOS
    shared-types/           # Shared TS DTO types
  docs/                     # Product, architecture, API, and deployment docs
  infra/                    # Local integration-test and production deployment config
  scripts/                  # Development helper scripts
```

## ⚡ Quick start

### 1. Prerequisites

- 🦀 Rust 1.95 stable toolchain with Rust 2024 edition. The workspace `rust-version` is `1.95`, the repository includes `rust-toolchain.toml`, and `rustfmt` plus `clippy` are expected.
- 🟢 Node.js 22+ and npm.
- 💬 WeChat DevTools for Mini Program debugging.
- 🤖 Android local builds require JDK 21, Android SDK 36, and Build Tools 36.0.0.
- 🗄️ PostgreSQL 16+ / MySQL-compatible database. Local development defaults to SQLite; production and integration tests prefer PostgreSQL; MySQL URLs are recognized at the configuration boundary.
- 🪣 MinIO or S3-compatible object storage.
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
cargo +1.95.0 run -p stellartrail-api --bin migrate -- up
cargo +1.95.0 run -p stellartrail-api
```

The API listens on `127.0.0.1:8080` by default. Startup loads `.env`, then `config.yaml` / `CONFIG_PATH`, and then environment-variable overrides. Local development defaults to SQLite; production and integration tests prefer PostgreSQL. See the [deployment guide](docs/deployment.md) for database, MinIO, Redis, SMTP, WeChat login, and administrator configuration.

Use these endpoints for local smoke testing:

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/v1/meta
```

### 4. Configure client endpoints

Checked-in clients default to placeholder endpoints:

- API base URL: `https://api.example.invalid`
- Image asset / CORS asset origin: `https://assets.example.invalid`

Copy the matching example when you need to override client endpoints:

| Client              | Example config                                          | Real config (do not commit)                     |
| ------------------- | ------------------------------------------------------- | ----------------------------------------------- |
| Web                 | `apps/web/.env.example`                                 | `apps/web/.env.local`                           |
| WeChat Mini Program | `apps/wechat-miniprogram/miniprogram/config.example.ts` | `apps/wechat-miniprogram/miniprogram/config.ts` |
| Android             | `apps/android/config.example.properties`                | `apps/android/config.properties`                |

Web reads `VITE_STELLARTRAIL_API_BASE_URL` and `VITE_STELLARTRAIL_ASSETS_BASE_URL`; local Vite development uses same-origin `/api/v1` by default and proxies it through `VITE_STELLARTRAIL_API_PROXY_TARGET` to the real or local API to avoid browser CORS failures. The WeChat Mini Program reads `miniprogram/config.ts` and falls back to placeholder endpoints when it is absent.
Android reads the Git-ignored `apps/android/config.properties` file and falls back to checked-in placeholder endpoints when that file is absent. Signed release APKs are built by GitHub Actions with repository-level Secrets for real domains and signing material.

See [API docs](docs/api.md) for the complete API surface.

### 5. Deployment guide

Local Docker Compose integration testing and production Docker / Traefik notes now live in the [deployment guide](docs/deployment.md).

### 6. Open the WeChat Mini Program source

Open `apps/wechat-miniprogram` in WeChat DevTools. The project config points `miniprogramRoot` to `miniprogram/`. The Mini Program currently covers Home, gear library, gear atlas, knot skills, Profile, login/registration, avatar/email binding, feedback, and offline read-only cache.

### 7. Build the Android client

For local Android debugging, copy `apps/android/config.example.properties` to the Git-ignored `apps/android/config.properties` file and replace the API and asset endpoints. Debug builds do not need release signing variables:

```bash
./gradlew :apps:android:testDebugUnitTest :apps:android:lintDebug :apps:android:assembleDebug
```

Local release builds require release keystore path and password environment variables. After Android or Gradle changes land on `main`, CI uses GitHub Actions Secrets to build a signed release APK and uploads `StellarTrail-main-<short_sha>-android-release.apk` plus its `.sha256` file as workflow artifacts. See the [Android README](apps/android/README.md) for signing, Secrets, and download details.

## 🧪 Common checks

```bash
# Frontend / TypeScript workspace checks
npm run check

# Markdown / JSON / YAML / TS formatting checks
npm run format:check

# Rust workspace checks
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 check --workspace
cargo +1.95.0 test --workspace
cargo +1.95.0 clippy --workspace --all-targets -- -D warnings

# Android Debug checks
./gradlew :apps:android:testDebugUnitTest :apps:android:lintDebug :apps:android:assembleDebug
```

## 📚 Documentation

- 🚀 [Deployment guide](docs/deployment.md)
- 🏗️ [Architecture](docs/architecture.md)
- 🔌 [API docs](docs/api.md)
