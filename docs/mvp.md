# StellarTrail MVP

## Goal

Build a WeChat Mini Program-first outdoor assistant for gear planning and field skills, with route and mountain features reserved for later phases.

## First release scope

1. WeChat Mini Program login and account model.
2. User gear library.
3. Gear search, category/status filters, sorting, import, export, archive, and restore.
4. Gear templates seeded by the API.
5. Knot skill catalog backed by imported Knots3D metadata.
6. Knot media upload and public media delivery through MinIO/S3-compatible object storage.
7. WeChat Mini Program offline read-only support for previously loaded skills, gear atlas, personal gear data, and viewed knot media.
8. Optional Redis read-through cache for high-traffic gear read APIs.

## Explicitly out of scope for MVP

- Mountain and route catalog implementation.
- Route detail with difficulty, season, risk, transport, and gear suggestions.
- Route-based packing checklist generation.
- Realtime navigation.
- Social feed/community.
- Route marketplace or guided trips.
- Full GPX editing.
- Storefront/commerce.
- Repo-local YAML/Markdown content startup loading.
