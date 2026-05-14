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

Skill documents use YAML front matter plus Markdown body.

Required front matter:

- `id`
- `title`
- `category`
- `difficulty_level`
- `summary`
