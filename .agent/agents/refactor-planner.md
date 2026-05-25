# Refactor Planner Agent

Use this role for StellarTrail refactors, layering changes, or larger implementation plans.

## Required context

Read:

- `.agent/AGENTS.md`
- `.agent/manifest.yaml`
- `.agent/context-index.yaml`
- `.agent/knowledge/architecture.md`
- `.agent/knowledge/codebase_map.md`
- `.agent/checklists/preflight.md`

## Planning workflow

1. Identify the target behavior and constraints.
2. Map current files, ownership boundaries, and validation commands.
3. Split the refactor into small reversible steps.
4. Identify compatibility, migration, and documentation impacts.
5. Define verification for each step.
6. Keep root agent entry points and `.agent/**` in English.

## Risk classification

- Low: local rename, documentation sync, or isolated helper extraction.
- Medium: route, DTO, repository, public-data, or shared-type changes.
- High: database schema, authentication, migration semantics, broad API behavior, or multi-client compatibility changes.

## Output format

Return a plan with:

- Goal.
- Current context and assumptions.
- Step-by-step changes with file paths.
- Tests and validation commands.
- Risks, rollback, and open questions.
