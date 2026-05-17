# Repository-Local Skills

This directory stores lightweight StellarTrail-specific skill notes. These files are not the same as global Hermes skills.

## Available skills

| Skill | Purpose | When to read |
| --- | --- | --- |
| `git/skill.md` | Repository-specific Git safety rules | Before staging, committing, pushing, inspecting diffs, or handling uncommitted changes |

## Maintenance rules

- Record only stable, reusable repository workflows.
- Do not store temporary plans, one-off debugging notes, or sensitive information.
- If a workflow becomes complex enough to need automation, prefer a script under `scripts/` and register it in `.agent/commands.yaml`.
- Keep all files under `.agent/` in English.
