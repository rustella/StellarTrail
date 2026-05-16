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

## Outdoor skill knots

Phase 1 no longer reads public knot data from Markdown files under `content/skills/knots`.
The old `taut-line-hitch.md` mock is intentionally removed and is not a compatibility fixture.

Knot metadata is imported from the local Knots3D metadata JSON into the application database:

```text
.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json
```

Import command:

```bash
cargo run -p stellartrail-importer --bin import-knots3d -- \
  --metadata .hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json \
  --database-url sqlite://stellartrail.db
```

Public API responses expose locale-resolved generic fields (`title`, `slug`, `summary`) and do
not expose source/language-specific fields such as `zh_slug`, `english_slug`, `source_slug_zh`,
or `source_slug_en`.

## Knot media assets

API JSON only contains media metadata and URLs. The actual binary files are served from
`CONTENT_ASSETS_DIR` via `/assets/*` in the MVP deployment.

```text
content/assets/skills/knots/<knot_id>/thumbnail.webp
content/assets/skills/knots/<knot_id>/preview.webp
content/assets/skills/knots/<knot_id>/draw.mp4
content/assets/skills/knots/<knot_id>/draw.gif
```

`.hermes/local/knots3d` must not be mounted as the production static asset directory. Promote
only authorized media into `content/assets` or a future CDN/object-storage bucket.
