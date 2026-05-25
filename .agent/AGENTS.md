# StellarTrail Agent Guide

This is the routing entry point for agents in the StellarTrail repository. Keep it compact and load deeper context only when needed.

## Bootstrap

At the start of each task:

1. Confirm the repository is `rustella/StellarTrail`.
2. Read `.agent/manifest.yaml` for project metadata, source boundaries, entry points, and hard policies.
3. Read `.agent/local/AGENTS.md` if it exists; it may define temporary branch or worktree rules.
4. Read `.agent/context-index.yaml` to choose the smallest relevant context set.
5. Read `.agent/commands.yaml` to identify validation commands.
6. Check `git status --short --branch` before modifying files.

## New chat worktree default

For every new chat or fresh task that asks for development work, default to a
new dedicated Git worktree before editing files. Treat the current checkout as
inspection-only until one of these conditions is true:

- The current directory is already a task-specific worktree for this exact
  request.
- The user explicitly says to use the current checkout, avoid creating a
  worktree, or only inspect without editing.
- The task is limited to repository agent-context maintenance under `.agent`
  and the user is already working on the agent-context branch for that purpose.

When a new worktree is needed, create it before making code or documentation
changes, use the branch naming policy from `.agent/local/AGENTS.md` when present,
and keep unrelated dirty changes in the original checkout untouched.

## Product scope

StellarTrail is an outdoor product for China-focused hiking scenarios. The first phase prioritizes the WeChat Mini Program and the Rust API, centered on account access, personal gear inventory, DB-backed gear templates, and knot skills. Route and mountain modules are future work until explicitly restarted.

Do not expand the product into real-time navigation, social networking, e-commerce, complex track editing, or an all-purpose route encyclopedia unless the task explicitly asks for it.

## Technology and directory boundaries

- Android app: `apps/android`, with Kotlin, Jetpack Compose, Material 3, Home, Gear, Skills, and Profile surfaces.
- WeChat Mini Program: `apps/wechat-miniprogram`, with Home, Gear, Skills, Profile, and placeholder/future route surfaces.
- Web app: `apps/web`.
- Rust API: `services/api`, using Axum and Tokio.
- Rust crates: `crates/domain`, `crates/db`, `crates/importer`, and `crates/migration`.
- TypeScript packages: `packages/shared-types` and `packages/api-client-ts`.
- Public data: gear templates are seeded into DB at API startup; knots are imported from Knots3D metadata into DB; media is served through MinIO/object storage URLs stored in DB.
- Documentation: `docs/`, including API, architecture, content schema, and MVP notes.

## Hard rules

1. Inspect the real repository before editing; do not copy assumptions from other projects.
2. Protect existing uncommitted changes. This repository can contain large in-progress MVP diffs.
3. API code orchestrates HTTP, DTOs, authentication, startup seed, and object-storage writes; domain code owns business models and validation; db code owns repositories and connections; migration code owns schema; importer code owns Knots3D metadata parsing and import boundaries.
4. All content in `AGENTS.md`, `CLAUDE.md`, and `.agent/**` must be written in English.
5. Code comments across the repository should be written in English. Server-side Rust code under `services/api` and `crates/{domain,db,importer,migration}` should use detailed rustdoc and inline comments for module responsibilities, public items, important private functions, authentication, database access, caching, state transitions, external calls, and migration DDL.
6. Do not commit real tokens, secrets, personal credentials, or connection strings. `.env.example` may only contain example values.
7. Do not edit or commit generated or local artifacts such as `target/`, `node_modules/`, `dist/`, `.idea/`, local database files, and tool caches.
8. Sync relevant documentation when APIs, schemas, directory boundaries, or validation commands change.
9. Run the most relevant `npm`, `cargo`, or git validation after code changes. If validation cannot run, explain why and provide an alternative.

## Final response after code or PR work

After updating code, committing, pushing, or creating a pull request or merge
request, the final response must include these exact delivery details:

- Worktree directory path.
- Branch name.
- Commit hash and commit message for each relevant commit.
- Pull request or merge request link.

If a requested action was not performed, explicitly say so in the corresponding
item instead of omitting it.

## Task routing

| Task type | Read first | Then read as needed |
| --- | --- | --- |
| Understand architecture or module boundaries | `.agent/knowledge/architecture.md`, `.agent/knowledge/codebase_map.md` | `docs/architecture.md`, `README.md` |
| Backend API, DB, or migration changes | `.agent/knowledge/architecture.md`, `.agent/commands.yaml` | `docs/api.md`, `services/api`, `crates/{domain,db,migration}` |
| WeChat Mini Program or TypeScript client changes | `.agent/knowledge/codebase_map.md`, `.agent/commands.yaml` | `apps/wechat-miniprogram`, `packages/*` |
| Public data or importer changes | `.agent/knowledge/architecture.md`, `.agent/commands.yaml` | `docs/content-schema.md`, `crates/importer`, `crates/db`, `crates/migration` |
| Review or self-review | `.agent/agents/code-reviewer.md`, `.agent/checklists/self-review.md` | `git diff`, related docs |
| Refactor planning | `.agent/agents/refactor-planner.md`, `.agent/checklists/preflight.md` | `.agent/knowledge/*`, related code |
| Agent context changes | `.agent/checklists/doc-sync.md` | `.agent/context-index.yaml`, `.agent/commands.yaml` |

## Required checks after editing `.agent`

- Parse `.agent/manifest.yaml`, `.agent/context-index.yaml`, and `.agent/commands.yaml` as YAML.
- Confirm Markdown relative links exist, except links explicitly marked optional or external.
- Confirm paths referenced by `context-index.yaml` and `commands.yaml` exist, or are marked as `status: stub`, `status: needs_confirmation`, optional, or external.
- Confirm no Chinese characters remain in `AGENTS.md`, `CLAUDE.md`, or `.agent/**`.
- Report changed files and `git diff --stat`.
