# API

StellarTrail 第一期服务端只实现装备库管理。除系统接口和本地 mock 登录外，`/api/me/*` 均需要 Bearer Token。

## System

```http
GET /healthz
GET /api/meta
```

## Auth

```http
POST /api/auth/wechat-login
```

本地开发设置 `APP_ENV=local` 且 `WECHAT_MOCK_LOGIN=true` 时可用。

```json
{
  "code": "local-dev-user",
  "profile": { "nickname": "测试用户", "avatar_url": null }
}
```

响应会返回 `access_token`，后续请求使用：

```http
Authorization: Bearer <access_token>
```

## Public content catalog

服务启动时会从 `CONTENT_DIR`（默认 `content`）读取 YAML/Markdown 种子内容，提供只读公共内容 API；这些接口不需要 Bearer Token。

```http
GET /api/mountains
GET /api/mountains/:id
GET /api/routes
GET /api/routes/:id
GET /api/skills
GET /api/skills/:id
GET /api/gear-templates
GET /api/gear-templates/:id
```

`/api/routes/:id` 会返回路线点位、路线装备建议和关联技能；`/api/skills/:id` 会返回 Markdown 正文 `body_markdown`；`/api/gear-templates/:id` 会返回装备模板分类和条目。

## Gear inventory

```http
GET /api/me/gears/categories?tab=available
GET /api/me/gears/stats?tab=available
GET /api/me/gears?tab=available&category=electronics_system&status=available&q=nitecore&sort=created_at_desc&limit=20&cursor=0
POST /api/me/gears
GET /api/me/gears/:id
PATCH /api/me/gears/:id
DELETE /api/me/gears/:id
POST /api/me/gears/:id/restore
GET /api/me/gears/export?tab=available&format=csv
POST /api/me/gears/import
```

### Create gear

最低必填字段：

```json
{
  "category": "electronics_system",
  "name": "NITECORE奈特科尔SUMMIT 20000超薄充电宝"
}
```

完整字段覆盖页面中的基本信息、性能指标、购买信息、状态与位置、标签、共享设置和备注：

```json
{
  "category": "electronics_system",
  "name": "NITECORE奈特科尔SUMMIT 20000超薄充电宝",
  "brand": "NITECORE奈特科尔",
  "model": "SUMMIT 20000",
  "color": null,
  "material": null,
  "capacity": "20000mAh",
  "size": null,
  "description": "冬季徒步备用电源",
  "weight_g": 315,
  "warmth_index": null,
  "waterproof_index": null,
  "purchase_date": "2026-01-22",
  "purchase_price_cents": 63900,
  "expiry_or_warranty_date": null,
  "purchase_location": "京东",
  "status": "available",
  "storage_location": "装备柜 A1",
  "tags": ["冬季", "电子"],
  "share_enabled": false,
  "notes": "充满电后入库"
}
```

删除接口是软删除：`DELETE /api/me/gears/:id` 会进入 `tab=history`；`POST /api/me/gears/:id/restore` 可恢复。

## Enums

装备分类：`backpack_system`、`sleep_system`、`kitchen_system`、`walking_system`、`clothing_system`、`lighting_system`、`first_aid_system`、`electronics_system`、`technical_gear`、`other_gear`、`consumable`。

装备状态：`available`、`in_use`、`maintenance`、`damaged`、`lost`、`retired`、`sold`、`idle`。

共享状态：`not_shared`、`pending`、`approved`、`rejected`、`withdrawn`。
