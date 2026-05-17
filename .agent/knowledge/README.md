# Agent Knowledge Index

This directory contains durable project knowledge for agents. Keep the root entry points short and load these files only when the task needs them.

## Files

- `architecture.md`: Product boundaries, backend layering, data flow, and cross-module invariants.
- `codebase_map.md`: Repository directory map, important entry points, and common change locations.

## Maintenance rules

- Keep this knowledge aligned with the real repository structure.
- Prefer concise facts and stable invariants over temporary task notes.
- Update the relevant knowledge file when APIs, schema, module boundaries, commands, or product scope change.
- Keep all files under `.agent/` in English.
