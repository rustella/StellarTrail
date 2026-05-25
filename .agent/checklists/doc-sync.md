# Documentation Sync Checklist

When changing documentation, context, APIs, schema, or commands, confirm:

- [ ] New or removed directories are reflected in `.agent/knowledge/codebase_map.md`.
- [ ] Architecture, dependency direction, database strategy, or importer capabilities are reflected in `.agent/knowledge/architecture.md` and `docs/architecture.md` where applicable.
- [ ] API endpoints, request or response fields, and error semantics are reflected in `docs/api.md`, `packages/shared-types`, and `packages/api-client-ts` where applicable.
- [ ] Public content, seed data, or importer changes are reflected in `docs/api.md` or `docs/architecture.md` where applicable.
- [ ] New or changed validation commands are reflected in `.agent/commands.yaml` and relevant README or CI docs.
- [ ] New task types or knowledge documents are reflected in `.agent/context-index.yaml`.
- [ ] Markdown relative links exist, except optional or external links that are clearly marked.
- [ ] `.agent/manifest.yaml`, `.agent/context-index.yaml`, and `.agent/commands.yaml` parse as YAML.
- [ ] `AGENTS.md`, `CLAUDE.md`, and `.agent/**` contain no Chinese characters.
