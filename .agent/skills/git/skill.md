# Git Operation Safety Rules

Use this repository-local skill for staging, committing, pushing, reverting, and diff inspection in StellarTrail.

## Basic rules

1. Check `git status --short --branch` before and after changes.
2. This repository may contain unrelated in-progress changes. Do not overwrite, format, or stage unrelated files.
3. Before committing, list untracked files and confirm each one belongs to the current delivery scope.
4. Do not use `git add -A` by default. Stage explicit paths only.
5. Do not commit `target/`, `node_modules/`, `.env`, local databases, IDE config, tool caches, or `.agent/local/`.
6. Follow `.agent/local/AGENTS.md` when it exists for machine-local branch and worktree policy.

## Recommended checks

```bash
git status --short --branch
git diff --stat
git diff -- AGENTS.md CLAUDE.md .agent .gitignore
git diff --check
```

If preparing to stage agent-context changes, stage only explicit files, for example:

```bash
git add AGENTS.md CLAUDE.md .gitignore .agent/AGENTS.md .agent/manifest.yaml .agent/context-index.yaml .agent/commands.yaml
```

## Commit message suggestion

For agent-context initialization or maintenance, use a message such as:

```text
chore(agent): update repository agent context
```

Adjust the type and scope to the actual change.
