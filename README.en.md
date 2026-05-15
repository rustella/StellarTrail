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

The current entry point is a **WeChat Mini Program**. A **Rust API service** provides backend capabilities, while routes, mountains, skills, and gear templates are driven by **YAML/Markdown content** so the project can build a high-quality outdoor knowledge base quickly.

| Module            | Current goal                                             |
| ----------------- | -------------------------------------------------------- |
| 🗺️ Route wiki     | Curate mountains, routes, difficulty, seasons, and risks |
| 🎒 Gear prep      | Generate packing lists and compare personal gear         |
| 🧭 Skills toolbox | Cover knots, camping, navigation, weather, first aid     |
| 📦 Content import | Iterate seed content quickly with YAML/Markdown          |
| 🧱 Rust backend   | Provide stable APIs, domain models, and data boundaries  |

## 🚀 MVP scope

The first release focuses on:

- 🧑‍💻 WeChat login placeholder and account model.
- ⛰️ Mountain and route catalog.
- 🧾 Route details: difficulty, season, risk, transport, and gear suggestions.
- 🎒 User gear library.
- ✅ Route-based packing checklist generation.
- 🪢 Skill catalog for knots, camping, packing, navigation, weather, and first aid.
- 📝 Content importing from YAML/Markdown.
- 🗄️ Database abstraction: SQLite for local development, PostgreSQL recommended for production, and conservative MySQL compatibility.

Realtime navigation, social feeds, guided-trip marketplaces, full GPX editing, and commerce are intentionally out of scope for the MVP.

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

- 🦀 Rust stable toolchain. The repository includes `rust-toolchain.toml` and expects `rustfmt` and `clippy`.
- 🟢 Node.js 22+ and npm.
- 💬 WeChat DevTools for Mini Program debugging.

### 2. Install dependencies

```bash
npm install
```

### 3. Start the API

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

### 4. Open the WeChat Mini Program

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
