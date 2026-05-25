# StellarTrail Codebase Map

## Root files

- `Cargo.toml`: Rust workspace configuration.
- `package.json`: Node workspace configuration.
- `README.md` and `README.en.md`: product and developer documentation.
- `AGENTS.md` and `CLAUDE.md`: root agent entry points.
- `.agent/`: layered agent context, commands, checklists, and knowledge.
- `.github/workflows/`: CI definitions for Rust and frontend checks.

## Applications

- `apps/android/`: Android native client source and tests.
- `apps/ios/`: iOS SwiftUI app, XcodeGen project definition, tests, and screenshot flows.
- `apps/macos/`: macOS SwiftUI app and XcodeGen project definition.
- `apps/wechat-miniprogram/miniprogram/app.ts`: Mini Program app entry point.
- `apps/wechat-miniprogram/miniprogram/app.json`: Mini Program page and tab configuration.
- `apps/wechat-miniprogram/miniprogram/pages/`: page implementations.
- `apps/wechat-miniprogram/miniprogram/utils/`: client utilities and API helpers.
- `apps/web/`: web application source and tests.

## Rust services and crates

- `services/api/src/main.rs`: API binary entry point.
- `services/api/src/lib.rs`: API library and app construction.
- `services/api/src/state.rs`: shared API state construction.
- `services/api/src/routes/`: route registration and handlers.
- `services/api/src/config.rs`: runtime configuration.
- `services/api/src/error.rs`: API error mapping.
- `services/api/tests/`: integration and route tests.
- `crates/domain/src/`: domain models and validation.
- `crates/db/src/`: database configuration and repositories.
- `crates/migration/src/`: schema migrations.

## Packages

- `packages/shared-types/src/index.ts`: shared DTO and API-facing TypeScript types.
- `packages/api-client-ts/src/index.ts`: TypeScript client helpers.
- `packages/apple/StellarTrailKit/`: shared Swift package for iOS and macOS clients.

## Public data and docs

- Gear templates: DB-backed system seed under domain/API/DB/migration code.
- Knot skills: DB-backed public content; media URLs are stored in DB and backed by MinIO/object storage.
- `docs/api.md`: API contract notes.
- `docs/architecture.md`: architecture documentation.
- `docs/mvp.md`: MVP scope notes.

## Generated or local-only paths

Do not edit or commit `target/`, `node_modules/`, `dist/`, `.idea/`, local DB files, `.env`, tool caches, or `.agent/local/`.
