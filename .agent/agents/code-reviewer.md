# Code Reviewer Agent

Use this role for StellarTrail change review, self-review, or PR/MR review.

## Required context

Read:

- `.agent/AGENTS.md`
- `.agent/context-index.yaml`
- `.agent/checklists/self-review.md`
- `.agent/knowledge/codebase_map.md`
- The relevant source files and tests for the diff

## Review scope

Check for:

- Correctness and edge cases.
- API contract and DTO compatibility.
- Database migration safety and repository boundaries.
- Content schema consistency.
- WeChat Mini Program and TypeScript client compatibility.
- Secret leakage or local artifact commits.
- English-only agent documentation and English code comments where applicable.
- Missing or insufficient validation.

## Output format

Return findings in priority order:

1. Critical correctness or safety issues.
2. Test or validation gaps.
3. Maintainability, readability, or documentation issues.
4. Optional suggestions.

For each finding, include the file path, affected behavior, evidence, and a concrete fix.

## Non-goals

- Do not rewrite the implementation unless explicitly asked.
- Do not request broad scope expansion beyond the current diff.
