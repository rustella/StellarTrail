# StellarTrail Agent Rules

This is the agent entrypoint for the StellarTrail repository. Keep it compact and route deeper architecture, command, and checklist details through `.agent/knowledge/`, `.agent/commands.yaml`, and `.agent/checklists/`.

## Code Comment Rules

- All code comments across the repository must be written in English.
- Server-side Rust code under `services/api/src` and `crates/{domain,db,importer,migration}/src` must use detailed English comments.
- Crate and module files must start with rustdoc-compatible `//!` comments that describe module responsibilities, boundaries, and key constraints.
- `pub` types, `pub` functions, and important private functions must use rustdoc-compatible `///` comments that describe purpose, inputs/outputs, errors, or safety boundaries.
- Critical logic inside functions, including authentication, database access, cache behavior, state transitions, external calls, import/export, and migration DDL, must use regular `//` comments that explain intent and invariants.
- After adding or modifying server code, run at least `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`; when rustdoc changes are involved, also run `cargo doc --workspace --no-deps`.
