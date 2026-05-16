# Content Schema Draft

## Mountain YAML

Required fields:

- `id`
- `name`
- `province`
- `summary`
- `difficulty_level`
- `status`

Optional fields:

- `aliases`
- `city`
- `area`
- `elevation_m`
- `lat`
- `lng`
- `best_seasons`

## Route YAML

Required fields:

- `id`
- `title`
- `province`
- `route_type`
- `difficulty_level`
- `summary`
- `status`

Recommended fields:

- `mountain_id`
- `distance_m`
- `ascent_m`
- `descent_m`
- `duration_min`
- `best_seasons`
- `transport_info`
- `permit_info`
- `risk_summary`
- `points`
- `gear_suggestions`
- `skill_links`

## Skill Markdown

历史 `content/skills/**/*.md` mock 不再作为绳结 API 数据源；`content/skills/knots/taut-line-hitch.md` 已删除且不保留兼容路由。

## Knots3D metadata

绳结数据从 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库。导入后由 `GET /api/skills/knots/list` 和 `GET /api/skills/knots/detail/:id` 返回 locale-resolved 字段。媒体二进制不进 JSON，由 `/assets/*` 返回。

## Gear Template YAML

Required fields:

- `id`
- `title`
- `categories[]`

Each category contains:

- `id`
- `name`
- `items[]`

## Public content API mapping

- `content/mountains/*.yaml` -> `GET /api/mountains*`
- `content/routes/*.yaml` -> `GET /api/routes*`
- DB-backed `knots` metadata -> `GET /api/skills`, `GET /api/skills/knots/list`, `GET /api/skills/knots/detail/:id`
- `content/gear-templates/*.yaml` -> `GET /api/gear-templates*`
