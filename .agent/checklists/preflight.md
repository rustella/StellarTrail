# Preflight Checklist

Before modifying files:

- [ ] Confirm the target repository and worktree.
- [ ] Read `AGENTS.md`, `.agent/AGENTS.md`, and `.agent/local/AGENTS.md` if present.
- [ ] Read `.agent/manifest.yaml`, `.agent/context-index.yaml`, and `.agent/commands.yaml`.
- [ ] Run or inspect `git status --short --branch` and identify existing user changes.
- [ ] Confirm the current branch. During initial MVP work, default to `wx/chore/initial-mvp` until it is merged or deleted.
- [ ] Identify generated or local-only paths that must not be edited or committed.
- [ ] Identify the smallest relevant docs, source files, and tests for the task.
- [ ] Decide which validation commands should run after the change.
- [ ] Keep root agent entry points and `.agent/**` in English.
