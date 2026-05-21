# API

StellarTrail 第一期服务端只实现装备库管理。除系统接口、登录接口和公共内容接口外，`/api/me/*` 均需要 Bearer Token。

## System

```http
GET /healthz
GET /api/meta
```

## 全局限流

所有非 `OPTIONS` 请求都会经过全局限流。未登录请求按客户端 IP 计数；带有效 Bearer Token 的请求会同时按客户端 IP 和用户 ID 计数，任一维度超限都会返回 `429 Too Many Requests`。

默认配置为每 60 秒同一 IP 最多 120 次、同一用户最多 240 次，可通过 `rate_limit` 配置块或 `RATE_LIMIT_*` 环境变量调整：

```yaml
rate_limit:
  enabled: true
  window_seconds: 60
  max_requests_per_ip: 120
  max_requests_per_user: 240
  trust_proxy_headers: false
  trusted_proxy_cidrs: []
```

服务默认只使用直连 IP。只有在 `RATE_LIMIT_TRUST_PROXY_HEADERS=true` 且直连 IP 命中 `RATE_LIMIT_TRUSTED_PROXY_CIDRS` 时，才会读取 `X-Forwarded-For` 第一段作为客户端 IP；不要无条件信任客户端传入的转发头。生产反向代理必须覆盖或清洗外部传入的 `X-Forwarded-For`，避免客户端伪造第一段 IP。

Redis 可用时限流计数写入 Redis；Redis 未配置或不可用时会退回单进程内存计数，仅作为降级保护。生产多实例部署应配置 Redis 并监控限流降级日志。

超限响应示例：

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 42
X-RateLimit-Limit: 120
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1770000000
```

```json
{
  "code": "rate_limited",
  "message": "Too many requests. Please retry after 42 seconds."
}
```

## Auth

```http
POST /api/auth/wechat-login
POST /api/auth/email-verification-code
POST /api/auth/email-login-code
POST /api/auth/email-login
POST /api/auth/password-reset-code
POST /api/auth/password-reset
POST /api/me/email-binding-code
POST /api/me/email-binding
POST /api/auth/register
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/captcha
GET /api/me/profile
PUT /api/me/profile/avatar
```

### WeChat login

小程序端传入 `wx.login()` 返回的 `code`。服务端行为：

- 本地开发：`APP_ENV=local` 且 `WECHAT_MOCK_LOGIN=true` 时走 mock openid，便于本地调试。
- 正式环境：设置 `WECHAT_MOCK_LOGIN=false`，并通过环境变量提供 `WECHAT_APP_ID` / `WECHAT_APP_SECRET`，服务端会请求微信 `jscode2session` 换取真实 `openid` 后 upsert 用户。

```json
{
  "code": "wx-js-code",
  "profile": { "nickname": "测试用户" }
}
```

`profile` 可以省略。省略时服务端不会清空用户已有昵称或头像；传入非空 `nickname` / `avatar_url` 时才更新对应字段。小程序端头像导入使用微信 `chooseAvatar` 选择本地临时文件，登录成功拿到 Bearer Token 后再调用头像上传接口保存公开头像 URL。

### Email / username registration, email-code login, and password reset

注册页可在同一表单中填写用户名、邮箱、密码、确认密码，并通过“发送邮箱验证码”按钮调用：

```json
POST /api/auth/email-verification-code
{
  "email": "alice@example.com"
}
```

本地环境响应会带 `debug_code` 方便联调；生产环境不返回明文验证码，服务端会通过配置的 SMTP 邮箱发送验证码。SMTP 主机为 `smtp.example.invalid`，真实账号、发件人地址和密码只放在 `.env`、被忽略的 `config.yaml` 或 secret manager：

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

登录接口的 `account` 可填写用户名或邮箱。首次和正常登录不需要验证码；同一账号连续多次输错密码后，接口返回 `captcha_required`，前端应先调用图片验证码接口获取 `captcha_ticket` 与 `image_svg`，用户填写图形内容后带回登录接口。

```json
POST /api/auth/captcha
{
  "account": "trail_alice"
}
```

```json
{
  "captcha_ticket": "captcha-ticket",
  "captcha_type": "image",
  "image_svg": "<svg ...>...</svg>",
  "expires_at": "2026-05-16T10:05:00Z"
}
```

```json
POST /api/auth/login
{
  "account": "trail_alice",
  "password": "OutdoorPass123!",
  "captcha_ticket": "captcha-ticket",
  "captcha_answer": "A7K2"
}
```

验证码门槛响应示例：

```json
{
  "code": "captcha_required",
  "message": "多次登录失败，请先完成验证码验证",
  "captcha": { "type": "image", "endpoint": "/api/auth/captcha" }
}
```

邮箱验证码登录先对已存在账号发送一次性验证码。为避免账号枚举，不存在的邮箱也返回同样结构，但不会发送邮件，也不会返回 `debug_code`：

```json
POST /api/auth/email-login-code
{
  "email": "alice@example.com"
}
```

```json
POST /api/auth/email-login
{
  "email": "alice@example.com",
  "email_verification_code": "123456"
}
```

找回密码同样先发送一次性验证码。验证码只可用于找回密码，不能复用注册或登录验证码；重置成功后旧 session 会失效，并签发新的登录态：

```json
POST /api/auth/password-reset-code
{
  "email": "alice@example.com"
}
```

```json
POST /api/auth/password-reset
{
  "email": "alice@example.com",
  "password": "***",
  "confirm_password": "***",
  "email_verification_code": "123456"
}
```

微信一键登录创建的账号初始可以没有邮箱。登录后可以先发送绑定邮箱验证码，再用同一用途的验证码绑定邮箱；注册、登录或找回密码验证码不能混用：

```json
POST /api/me/email-binding-code
Authorization: Bearer ***
{
  "email": "alice@example.com"
}
```

```json
POST /api/me/email-binding
Authorization: Bearer ***
{
  "email": "alice@example.com",
  "email_verification_code": "123456"
}
```

成功响应：

```json
{
  "user": {
    "id": "user-id",
    "username": null,
    "email": "alice@example.com",
    "nickname": "微信用户",
    "avatar_url": null
  }
}
```

若当前账号已经绑定邮箱，或目标邮箱已被其他账号使用，会返回 `validation_failed`。绑定成功后，可继续使用找回密码流程为该账号设置密码。

### Current profile

登录后可读取当前用户资料，用于客户端从后端刷新头像和昵称。

```http
GET /api/me/profile
Authorization: Bearer ***
```

成功响应：

```json
{
  "user": {
    "id": "user-id",
    "username": null,
    "email": null,
    "nickname": "微信用户",
    "avatar_url": "https://assets.example.invalid/stellartrail-avatars/users/user-id/avatar/hash.png"
  }
}
```

### Profile avatar upload

登录后可上传当前用户头像。服务端会校验图片文件签名，上传到公开头像 bucket，并更新当前用户的 `avatar_url`。通用客户端传入标准文件名和 MIME 时仍会校验扩展名、声明 MIME 与文件签名一致；微信小程序 `chooseAvatar` 产生的临时文件名或 `application/octet-stream` 会按文件签名推断图片类型。微信小程序 `wx.uploadFile` 只发起 `POST` 上传，因此服务端同时接受 `PUT` 和 `POST`，推荐通用客户端使用 `PUT`。

```http
PUT /api/me/profile/avatar
Authorization: Bearer ***
Content-Type: multipart/form-data
```

Multipart 字段：

- `file`：必填，支持 `jpg` / `jpeg` / `png` / `webp`。

成功响应：

```json
{
  "user": {
    "id": "user-id",
    "username": null,
    "email": null,
    "nickname": "微信用户",
    "avatar_url": "https://assets.example.invalid/stellartrail-avatars/users/user-id/avatar/hash.png"
  }
}
```

登录、注册、邮箱验证码登录、找回密码成功响应会返回短期 `access_token` 和长期 `refresh_token`。服务端只保存 token hash，不保存明文 token；客户端后续私有请求使用：

```http
Authorization: Bearer ***
```

当 `access_token` 过期或私有接口返回 401 时，客户端可使用 `refresh_token` 换取新的 token pair。refresh 成功会轮换 refresh token，旧 refresh token 立即失效，客户端必须持久化新的 `refresh_token`。

```json
POST /api/auth/refresh
{
  "refresh_token": "opaque-refresh-token"
}
```

登录、注册、邮箱验证码登录、找回密码和刷新成功响应结构一致：

```json
{
  "access_token": "opaque-access-token",
  "expires_at": "2026-05-16T12:00:00Z",
  "refresh_token": "opaque-refresh-token",
  "refresh_expires_at": "2026-06-15T10:00:00Z",
  "user": {
    "id": "user-id",
    "username": "trail_alice",
    "email": "alice@example.com",
    "nickname": null,
    "avatar_url": null
  }
}
```

## Public skills, gear templates, and gear atlas

公共技能、装备模板和装备图鉴浏览接口不需要 Bearer Token。API 启动不再读取 repo-local `content/` 文件树，也不再挂载 `/assets/*` 静态目录；公开媒体 URL 均来自 DB 中保存的 MinIO/S3-compatible 对象存储地址。山峰和路线模块尚未开始实现，因此不注册 `/api/mountains*` 或 `/api/routes*`。

```http
GET /api/skills
GET /api/skills/knots/list?offset=0&limit=20&category=camping-knots&difficulty=beginner&q=wind
GET /api/skills/knots/filters
GET /api/skills/knots/offline-manifest
GET /api/skills/knots/detail/:id
GET /api/gear-templates
GET /api/gear-templates/:id
GET /api/gear-atlas?category=lighting_system&q=headlamp&sort=name_asc&limit=20&cursor=0
GET /api/gear-atlas/:id
```

`/api/skills` 返回技能分类（第一期仅 `knots`）；绳结列表和详情走 DB-backed Knots3D metadata，不暴露 Markdown mock。`/api/gear-templates` 和 `/api/gear-templates/:id` 从数据库读取装备模板分类和条目；服务启动时会幂等写入默认系统模板，替代旧的 `content/gear-templates/*.yaml` 文件源。`/api/gear-atlas` 和 `/api/gear-atlas/:id` 返回已审核通过的公共装备图鉴，不包含用户个人购买、位置、标签或备注字段。

公共内容语言不使用 query 参数，统一通过请求头：

```http
X-StellarTrail-Locale: zh-CN
# 或
X-StellarTrail-Locale: en
```

未显式传 `X-StellarTrail-Locale` 时会尝试 `Accept-Language`，再 fallback 到 `zh-CN`。`?locale=...` 会返回 `400 unsupported_query_parameter`。公开响应只返回当前语言字段，不返回并列的 `zh/en` 字段；缺少目标语言行时 fallback 到另一种受支持语言，再 fallback 到主表兼容字段。

### Outdoor skills / knots

一期户外技能只有绳结。服务端通过 `import-knots3d` 将 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库；绳结媒体不再从 `/assets/*` 或本地静态目录拼 URL。管理员使用 `PUT /api/admin/skills/knots/:knot_id/media/:asset_id` 上传 GIF/MP4/WebP/PNG 等二进制到 MinIO/S3-compatible object storage，服务端把 `media_resources` 与 `knot_media_resources` 元数据写入数据库。公开读接口只返回 DB 中 active media 的 `url`/`mime_type`/`size_bytes` 等公共字段，允许这些 URL 指向与 API 不同域名的 MinIO/CDN。运行配置只保留一组 `minio` 连接信息，私有反馈图和公开绳结媒体分别配置业务 bucket。

核心媒体 `asset_id` / `media_type`：`thumbnail`、`preview`、`draw_gif`、`turntable_gif`、`draw_mp4`、`turntable_mp4`。Knots3D 全量一期验收目标为 `225 knots × 6 core media = 1350`。

绳结分页参数为 `offset`/`limit`，筛选参数为 `category`、`difficulty`，关键词为 `q`；响应字段为 `next_offset`，不返回 `cursor`/`next_cursor`。`/api/skills/knots/filters` 返回当前语言下可选用途、难度及数量。`/api/skills/knots/offline-manifest` 不接受 query 参数，返回完整离线清单、`item_count`、去重后的 `media_count` 和 `estimated_bytes`，并复用 public response cache 与 `ETag`。public response 不暴露 `zh_slug`、`english_slug`、`source_slug_zh`、`source_slug_en`。

### Gear templates and gear atlas i18n

装备模板的模板标题、分类名、条目名存储在 `gear_template_*_localizations` 表中；默认系统模板同时 seed `zh-CN` 和 `en`。装备图鉴的公共 `name` 和 `description` 存储在 `gear_atlas_item_localizations` 表中，新用户投稿默认写入 `zh-CN` 原文行，不做自动机翻；公共 `category_label` 从 `gear_category_localizations` 表解析。`brand`、`model`、`specs`、价格和重量等事实字段不做翻译。

### Admin knot media upload

管理员上传接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。该接口是批量导入 Knots3D 媒体的唯一写入入口：脚本不得绕过 API 直接写 MinIO 或 DB。

```http
PUT /api/admin/skills/knots/:knot_id/media/:asset_id
Authorization: Bearer <admin-token>
Content-Type: multipart/form-data
```

Multipart 字段：

| 字段           | 必填 | 说明                                                    |
| -------------- | ---- | ------------------------------------------------------- |
| `file`         | 是   | 媒体二进制；MIME 与文件 magic 会按 `asset_id` 校验。    |
| `media_type`   | 是   | 必须与 `asset_id` 一致。                                |
| `attribution`  | 否   | 默认建议 `Knots 3D`。                                   |
| `license_note` | 否   | 授权说明；版权未确认时不要上传到生产 public bucket。    |
| `source_name`  | 否   | 来源名称。                                              |
| `source_path`  | 否   | 本地素材相对路径，仅存内部 metadata，不在公开响应暴露。 |

成功响应：

```json
{
  "status": "uploaded",
  "knot_id": "adjustable-grip-hitch-knot",
  "media": {
    "id": "draw_gif",
    "media_type": "draw_gif",
    "url": "https://cdn.example.com/skills/knots/adjustable-grip-hitch-knot/draw_gif/<sha256>.gif",
    "mime_type": "image/gif",
    "width": null,
    "height": null,
    "size_bytes": 123456,
    "attribution": "Knots 3D",
    "license_note": "Use only after authorization is confirmed."
  }
}
```

本地批量上传/验证：

```bash
STELLARTRAIL_API_BASE_URL=http://127.0.0.1:8080 STELLARTRAIL_ADMIN_TOKEN=<admin-token> KNOTS3D_METADATA_PATH=.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json npm run knots:upload-media -- --concurrency 4

# 只打印计划，不写入：
npm run knots:upload-media -- --dry-run

# 上传后只通过公开读接口校验 225×6 媒体完整性：
npm run knots:upload-media -- --verify-only
```

## Admin API usage statistics

```http
GET /api/admin/api-usage?from=2026-05-01&to=2026-05-18&method=GET&route=/api/me/gears&limit=50&offset=0
```

该接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。它只返回按日期、用户、HTTP 方法、路由模板和状态码聚合后的计数，不返回单次请求日志。

## Admin feedback

```http
GET /api/admin/feedback?status=open&limit=50&cursor=0
Authorization: Bearer <admin-token>
```

该接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。返回用户提交的反馈内容、联系方式、页面、客户端环境、提交用户概要和已关联图片元数据。反馈图片仍是私有资源；管理员下载图片时使用返回的 `download_url` 并携带管理员 Bearer Token。

```json
{
  "items": [
    {
      "id": "feedback-uuid",
      "user": {
        "id": "user-uuid",
        "username": "trail_user",
        "email": "trail@example.com",
        "nickname": "寻径用户",
        "avatar_url": null
      },
      "category": "bug",
      "content": "装备详情页图片没有显示",
      "contact": "feedback@example.test",
      "page": "/pages/gears/detail/index?id=gear-1",
      "client_platform": "wechat_miniprogram",
      "client_version": "0.1.0",
      "device_model": "iPhone 15",
      "status": "open",
      "images": [
        {
          "id": "upload-uuid",
          "purpose": "feedback",
          "original_filename": "screen.png",
          "image_type": "png",
          "content_type": "image/png",
          "size_bytes": 1024,
          "sha256": "hex",
          "download_url": "/api/admin/feedback-images/upload-uuid",
          "created_at": "2026-05-19T00:00:00Z"
        }
      ],
      "created_at": "2026-05-19T00:00:00Z",
      "updated_at": "2026-05-19T00:00:00Z"
    }
  ],
  "next_cursor": null
}
```

## Admin role management

管理员角色存储在 `admin_roles` 表，角色值为 `admin` 或 `super_admin`。迁移只会把数据库中已存在且未删除的 `username = 'stellarisw'` 用户写入或升级为 `super_admin`；如果该用户不存在，迁移不会创建用户或预留用户名。`admin` 与 `super_admin` 都能访问管理员能力，只有 `super_admin` 可以授予或移除普通 `admin`。

```http
POST /api/admin/admins
Authorization: Bearer <super-admin-token>
Content-Type: application/json

{"username": "trail_admin"}
```

请求体必须且只能包含一个目标标识：`username` 或 `user_id`。目标用户必须已存在且未删除。新增普通管理员返回 `201`，目标已是 `admin` 或 `super_admin` 时返回 `200` 和当前角色；重复授予不会把 `super_admin` 降级为 `admin`。

```json
{
  "user_id": "user-uuid",
  "role": "admin"
}
```

移除普通管理员：

```http
DELETE /api/admin/admins?username=trail_admin
Authorization: Bearer <super-admin-token>
```

删除成功返回 `204`。目标不存在、未拥有管理员角色时返回 `404`；目标是 `super_admin` 时返回 `422 validation_failed`，该接口不负责移除超级管理员。

统计写入由服务端 middleware 异步上报：请求返回路径不会等待数据库写入；统计队列满、后台 worker 失败或写库失败时只丢弃统计，不影响业务请求。

隐私边界：

- 记录 `user_id`（认证成功时）、`method`、matched route 模板、`status_code`、日期桶和调用次数。
- 动态路径只保存模板，例如 `/api/me/gears/:id`，不保存真实装备 ID。
- 不记录 query string、请求体、响应体、Authorization header、access token、refresh token、token hash、Cookie、IP、User-Agent。

查询参数：

| 字段      | 必填 | 说明                                                                |
| --------- | ---- | ------------------------------------------------------------------- |
| `from`    | 否   | 起始日期，`YYYY-MM-DD`；默认最近 30 天。                            |
| `to`      | 否   | 结束日期，`YYYY-MM-DD`；默认当天。                                  |
| `user_id` | 否   | 仅查看某个用户的聚合统计。                                          |
| `method`  | 否   | `GET` / `POST` / `PUT` / `PATCH` / `DELETE`。                       |
| `route`   | 否   | matched route 模板，例如 `/api/me/gears/:id`；不能带 query string。 |
| `limit`   | 否   | 默认 50，最大 100。                                                 |
| `offset`  | 否   | 分页偏移。                                                          |

响应示例：

```json
{
  "items": [
    {
      "bucket_date": "2026-05-18",
      "user_id": "user-id",
      "method": "GET",
      "route_pattern": "/api/me/gears",
      "status_code": 200,
      "call_count": 12,
      "first_called_at": "2026-05-18T08:00:00Z",
      "last_called_at": "2026-05-18T10:00:00Z"
    }
  ],
  "page": { "limit": 50, "offset": 0, "next_offset": null }
}
```

## Cache / performance

服务端支持可选 Redis 缓存。设置 `REDIS_URL` 后，装备库高频只读接口会走 Redis read-through cache；`POST /api/me/gears`、`PATCH /api/me/gears/:id`、`DELETE /api/me/gears/:id`、`POST /api/me/gears/:id/restore` 和非 dry-run 导入会递增用户级缓存版本，确保写入后后续读取不会命中旧 key。默认 TTL 为 `REDIS_GEAR_CACHE_TTL_SECONDS=30` 秒，可通过 `REDIS_KEY_PREFIX` 区分环境。装备 specs 字段频次也只保存在 Redis sorted set 中，key 为 `<prefix>:gear:<user_id>:spec-keys:<category>`，member 只保存 spec key，不保存用户填写的具体值。用户标签建议也只保存在 Redis：标签频次 key 为 `<prefix>:gear:<user_id>:tags`，标签颜色偏好 hash key 为 `<prefix>:gear:<user_id>:tag-colors`。

## Gear inventory

```http
GET /api/me/gears/categories?tab=available
GET /api/me/gears/stats?tab=available
GET /api/me/gears/spec-key-rankings?category=electronics_system
GET /api/me/gears/tag-suggestions?limit=20
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
  "description": "冬季徒步备用电源",
  "weight_g": 315,
  "official_price_cents": 69900,
  "official_price_currency": "CNY",
  "purchase_date": "2026-01-22",
  "purchase_price_cents": 63900,
  "purchase_price_currency": "CNY",
  "purchase_location": "京东",
  "status": "available",
  "storage_location": "装备柜 A1",
  "specs": {
    "battery_capacity": "20000 mAh",
    "rated_energy": "74 Wh"
  },
  "tags": ["冬季", "电子"],
  "tag_colors": {
    "冬季": "blue",
    "电子": "teal"
  },
  "share_enabled": false,
  "notes": "充满电后入库"
}
```

装备表只保存 `tags` 文本数组；`tag_colors` 是用户标签库颜色偏好，只写入 Redis，不写入装备表。装备列表和详情响应会返回当前装备标签对应的 `tag_colors` 映射，客户端刷新后即可按后端最新颜色重新渲染。支持颜色 token：`teal`、`blue`、`violet`、`rose`、`orange`、`amber`、`green`、`slate`。

删除接口是软删除：`DELETE /api/me/gears/:id` 会进入 `tab=history`；`POST /api/me/gears/:id/restore` 可恢复。

### Spec key rankings

`GET /api/me/gears/spec-key-rankings?category=electronics_system` 返回当前登录用户在该装备分类下常填写的 specs 字段 key：

```json
{
  "keys": ["battery_capacity", "rated_energy"]
}
```

该接口只返回当前分类允许的 spec key。Redis 未启用或不可用时返回空数组，装备保存不受影响。

### Tag suggestions

`GET /api/me/gears/tag-suggestions?limit=20` 返回当前登录用户常用装备标签，以及该标签当前颜色偏好：

```json
{
  "items": [
    { "tag": "冬季", "color": "blue" },
    { "tag": "电子", "color": "teal" }
  ]
}
```

标签建议按 Redis 频次倒序返回，`limit` 默认 20、最大 50。Redis 未启用或不可用时返回空数组；装备保存不受影响。

## Enums

装备分类：`backpack_system`、`sleep_system`、`kitchen_system`、`walking_system`、`clothing_system`、`lighting_system`、`first_aid_system`、`electronics_system`、`technical_gear`、`other_gear`、`consumable`。

装备状态：`available`、`in_use`、`maintenance`、`damaged`、`lost`、`retired`、`sold`、`idle`。

共享状态：`not_shared`、`pending`、`approved`、`rejected`、`withdrawn`。
