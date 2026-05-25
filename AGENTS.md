# AGENTS Entry Point

This repository uses a layered agent context architecture. Any AI agent or collaborator working in this repository should read this file first, then load the smallest task-specific context from `.agent/`.

## Read order

1. This file: confirm the repository-level entry point and read order.
2. `.agent/AGENTS.md`: hard rules, task routing, and context-loading policy.
3. `.agent/local/AGENTS.md`: machine-local or temporary rules when present. These rules may add branch and worktree constraints, but must not override repository hard rules.
4. `.agent/context-index.yaml`: choose task-specific documents, checklists, and subagent contracts.
5. `.agent/commands.yaml`: canonical validation, test, startup, and migration commands.
6. Load `.agent/knowledge/`, `.agent/checklists/`, `.agent/agents/`, and `.agent/skills/` only when they are relevant to the current task.

## Current local development constraint

If `.agent/local/AGENTS.md` exists, follow its temporary branch policy first. During the initial MVP phase, code development should default to the `wx/chore/initial-mvp` worktree. Resume the normal branch and MR workflow only after that branch is merged or no longer exists locally and remotely.

## Conventions

- Keep this root entry point thin; durable project knowledge belongs under `.agent/`.
- All content in `AGENTS.md`, `CLAUDE.md`, and `.agent/**` must be written in English.
- `.agent/local/AGENTS.md` is for local temporary rules and should remain ignored through `.gitignore`.
- Check `git status --short --branch` before modifying files, and do not overwrite unrelated user changes.
- After changing `.agent`, validate YAML files, relative links, and paths referenced by `context-index.yaml` and `commands.yaml`.
- Do not commit real tokens, secrets, local databases, `target/`, `node_modules/`, `dist/`, IDE state, or other generated artifacts.
