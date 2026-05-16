# API

StellarTrail 第一期服务端只实现装备库管理。除系统接口、登录接口和公共内容接口外，`/api/me/*` 均需要 Bearer Token。

## System

```http
GET /healthz
GET /api/meta
```

## Auth

```http
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/register
POST /api/auth/login
```

### WeChat login

小程序端传入 `wx.login()` 返回的 `code`。服务端行为：

- 本地开发：`APP_ENV=local` 且 `WECHAT_MOCK_LOGIN=true` 时走 mock openid，便于本地调试。
- 正式环境：设置 `WECHAT_MOCK_LOGIN=false`，并通过环境变量提供 `WECHAT_APP_ID` / `WECHAT_APP_SECRET`，服务端会请求微信 `jscode2session` 换取真实 `openid` 后 upsert 用户。

```json
{
  "code": "wx-js-code",
  "profile": { "nickname": "测试用户", "avatar_url": null }
}
```

### Email / username registration and password login

注册页可在同一表单中填写用户名、邮箱、密码、确认密码，并通过“发送邮箱验证码”按钮调用：

```json
POST /api/auth/email-verification-code
{
  "email": "alice@example.com"
}
```

本地环境响应会带 `debug_code` 方便联调；生产环境不返回明文验证码，后续接入邮件投递服务即可在生成验证码后发送邮件：

```json
{
  "email": "alice@example.com",
  "expires_at": "2026-05-16T10:00:00Z",
  "debug_code": "123456"
}
```

注册接口会校验邮箱验证码和确认密码，密码以 SHA-256 十六进制摘要写入数据库：

```json
POST /api/auth/register
{
  "username": "trail_alice",
  "email": "alice@example.com",
  "password": "OutdoorPass123!",
  "confirm_password": "OutdoorPass123!",
  "email_verification_code": "123456"
}
```

登录接口的 `account` 可填写用户名或邮箱。首次和正常登录不需要验证码；同一账号连续多次输错密码后，接口返回 `captcha_required`，前端应展示图片验证码或滑动验证码后重试。

```json
POST /api/auth/login
{
  "account": "trail_alice",
  "password": "OutdoorPass123!",
  "captcha_ticket": "local-dev-captcha",
  "captcha_answer": "pass"
}
```

验证码门槛响应示例：

```json
{
  "code": "captcha_required",
  "message": "多次登录失败，请先完成验证码验证",
  "captcha": { "type": "image" }
}
```

登录/注册成功响应会返回 `access_token`，后续请求使用：

```http
Authorization: Bearer ***
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
