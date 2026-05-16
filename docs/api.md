# API Draft

## System

```http
GET /healthz
GET /api/meta
```

## Locale

Public catalog endpoints resolve language globally. Do not pass `locale` in the URL query string.

Priority:

1. `X-StellarTrail-Locale`
2. future authenticated user `preferred_locale`
3. `Accept-Language`
4. default `zh-CN`

Supported header values for phase 1:

```http
X-StellarTrail-Locale: zh-CN
X-StellarTrail-Locale: en
```

`?locale=...` returns `400 unsupported_query_parameter`.

## Outdoor skills

```http
GET /api/skills
GET /api/skills/knots/list?offset=0&limit=20
GET /api/skills/knots/detail/:id
GET /assets/*
```

Not supported as compatibility aliases:

```http
GET /api/knots
GET /api/skills?category=knot
GET /api/skills/:id
GET /api/skills/knots
GET /api/skills/knots/:id
```

### `GET /api/skills`

Returns skill categories. Phase 1 only exposes `knots`.

```json
{
  "items": [
    {
      "id": "knots",
      "slug": "knots",
      "title": "绳结",
      "summary": "户外、露营、钓鱼、航海等场景常用绳结技能。",
      "item_count": 225,
      "href": "/api/skills/knots/list"
    }
  ]
}
```

### `GET /api/skills/knots/list`

Query parameters:

- `offset`: non-negative integer. Defaults to `0`.
- `limit`: positive integer, clamped to `1..=100`. Defaults to `20`.
- `category`: optional taxonomy filter.
- `q`: optional search text.

Response pagination uses `offset` / `next_offset`; it does not expose `cursor` / `next_cursor`.

```json
{
  "locale": "en",
  "items": [
    {
      "id": "adjustable-grip-hitch-knot",
      "slug": "adjustable-grip-hitch-knot",
      "title": "Adjustable Grip Hitch",
      "summary": "Adjust tension on a line.",
      "difficulty": null,
      "categories": [
        { "id": "camping-knots", "slug": "camping-knots", "title": "Camping" }
      ],
      "types": [
        { "id": "hitch-knots", "slug": "hitch-knots", "title": "Hitches" }
      ],
      "media": [
        {
          "id": "thumbnail",
          "media_type": "thumbnail",
          "url": "/assets/skills/knots/adjustable-grip-hitch-knot/thumbnail.webp",
          "mime_type": "image/webp"
        }
      ],
      "href": "/api/skills/knots/detail/adjustable-grip-hitch-knot"
    }
  ],
  "page": { "limit": 20, "offset": 0, "next_offset": null }
}
```

### `GET /api/skills/knots/detail/:id`

Returns the selected locale only. Public JSON intentionally does not include `zh_slug`,
`english_slug`, `source_slug_zh`, or `source_slug_en`; those are import/audit-only DB fields.

### `GET /assets/*`

Static media is served from `CONTENT_ASSETS_DIR` and addressed by `MEDIA_BASE_URL`.

Recommended MVP config:

```env
MEDIA_BASE_URL=/assets
CONTENT_ASSETS_DIR=content/assets
```

Authorized knot media should be promoted into:

```text
content/assets/skills/knots/<knot_id>/
```

`.hermes/local/knots3d` is a local read-only import source and is not served directly.

## Future endpoints

```http
POST /api/auth/wechat-login

GET /api/mountains
GET /api/mountains/:id

GET /api/routes
GET /api/routes/:id
GET /api/routes/:id/gear-suggestions
GET /api/routes/:id/skills

GET /api/me/gears
POST /api/me/gears
PATCH /api/me/gears/:id
DELETE /api/me/gears/:id

GET /api/me/trips
POST /api/me/trips
GET /api/me/trips/:id
POST /api/me/trips/:id/generate-checklist
```
