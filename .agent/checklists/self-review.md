# Self-Review Checklist

Before handoff:

- [ ] The change stays within the requested scope.
- [ ] No unrelated formatting or generated artifacts were introduced.
- [ ] Existing user changes were not overwritten.
- [ ] Secrets, tokens, local database files, and personal credentials were not added.
- [ ] Root `AGENTS.md`, `CLAUDE.md`, and `.agent/**` remain English-only.
- [ ] Code comments are English unless the task explicitly required otherwise.
- [ ] Server-side Rust changes include appropriate rustdoc and inline comments for important logic.
- [ ] API, schema, DTO, command, or directory-boundary changes are reflected in docs and `.agent` where needed.
- [ ] The most relevant tests, lint, format, or build commands were run.
- [ ] `git diff --check` was run or an explicit reason was provided.
- [ ] The final response lists changed files, validation results, and remaining risks.
