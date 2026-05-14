# API Draft

## System

```http
GET /healthz
GET /api/meta
```

## Future endpoints

```http
POST /api/auth/wechat-login

GET /api/mountains
GET /api/mountains/:id

GET /api/routes
GET /api/routes/:id
GET /api/routes/:id/gear-suggestions
GET /api/routes/:id/skills

GET /api/skills
GET /api/skills/:id

GET /api/me/gears
POST /api/me/gears
PATCH /api/me/gears/:id
DELETE /api/me/gears/:id

GET /api/me/trips
POST /api/me/trips
GET /api/me/trips/:id
POST /api/me/trips/:id/generate-checklist
```
