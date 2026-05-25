# CLAUDE.md - StellarTrail Agent Entry Point

This repository uses a layered agent context architecture.

Read in this order:

1. [AGENTS.md](AGENTS.md)
2. [.agent/AGENTS.md](.agent/AGENTS.md)
3. [.agent/local/AGENTS.md](.agent/local/AGENTS.md) if it exists. It contains local or temporary branch/worktree rules and must be honored without overriding repository hard rules.
4. [.agent/context-index.yaml](.agent/context-index.yaml)
5. [.agent/commands.yaml](.agent/commands.yaml)
6. Only the task-specific files listed in [.agent/knowledge/README.md](.agent/knowledge/README.md), [.agent/checklists/](.agent/checklists/), [.agent/agents/](.agent/agents/), and [.agent/skills/](.agent/skills/).

Current local development note: follow `.agent/local/AGENTS.md` when it exists for machine-local branch and worktree policy.

Language rule: root agent entry points and every file under `.agent/` must be written in English.

Keep root entry points thin. Project facts and workflow details live under `.agent/`.
