# API

StellarTrail 第一期服务端只实现装备库管理。除系统接口、登录接口和公共内容接口外，`/api/v1/me/*` 均需要 Bearer Token。

## System

```http
GET /healthz
GET /api/v1/meta
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
POST /api/v1/auth/wechat-login
POST /api/v1/auth/email-verification-code
POST /api/v1/auth/email-login-code
POST /api/v1/auth/email-login
POST /api/v1/auth/password-reset-code
POST /api/v1/auth/password-reset
POST /api/v1/me/email-binding-code
POST /api/v1/me/email-binding
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh
POST /api/v1/auth/captcha
GET /api/v1/me/profile
PUT /api/v1/me/profile/avatar
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
POST /api/v1/auth/email-verification-code
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
POST /api/v1/auth/register
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
POST /api/v1/auth/captcha
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
POST /api/v1/auth/login
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
  "captcha": { "type": "image", "endpoint": "/api/v1/auth/captcha" }
}
```

邮箱验证码登录先对已存在账号发送一次性验证码。为避免账号枚举，不存在的邮箱也返回同样结构，但不会发送邮件，也不会返回 `debug_code`：

```json
POST /api/v1/auth/email-login-code
{
  "email": "alice@example.com"
}
```

```json
POST /api/v1/auth/email-login
{
  "email": "alice@example.com",
  "email_verification_code": "123456"
}
```

找回密码同样先发送一次性验证码。验证码只可用于找回密码，不能复用注册或登录验证码；重置成功后旧 session 会失效，并签发新的登录态：

```json
POST /api/v1/auth/password-reset-code
{
  "email": "alice@example.com"
}
```

```json
POST /api/v1/auth/password-reset
{
  "email": "alice@example.com",
  "password": "***",
  "confirm_password": "***",
  "email_verification_code": "123456"
}
```

微信一键登录创建的账号初始可以没有邮箱。登录后可以先发送绑定邮箱验证码，再用同一用途的验证码绑定邮箱；注册、登录或找回密码验证码不能混用：

```json
POST /api/v1/me/email-binding-code
Authorization: Bearer ***
{
  "email": "alice@example.com"
}
```

```json
POST /api/v1/me/email-binding
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
GET /api/v1/me/profile
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
PUT /api/v1/me/profile/avatar
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
POST /api/v1/auth/refresh
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

公共技能、装备模板和装备图鉴浏览接口不需要 Bearer Token。API 启动不再读取 repo-local `content/` 文件树，也不再挂载 `/assets/*` 静态目录；公开媒体 URL 均来自 DB 中保存的 MinIO/S3-compatible 对象存储地址。山峰和路线模块尚未开始实现，因此不注册 `/api/v1/mountains*` 或 `/api/v1/routes*`。

```http
GET /api/v1/skills
GET /api/v1/skills/knots/list?offset=0&limit=20&category=camping-knots&q=wind
GET /api/v1/skills/knots/filters
GET /api/v1/skills/knots/offline-manifest
GET /api/v1/skills/knots/detail/:id
GET /api/v1/me/skills/knots/disclaimer
POST /api/v1/me/skills/knots/disclaimer/acceptance
GET /api/v1/gear-templates
GET /api/v1/gear-templates/:id
GET /api/v1/gear-atlas?category=lighting_system&q=headlamp&sort=name_asc&limit=20&cursor=0
GET /api/v1/gear-atlas/:id
```

`/api/v1/skills` 返回技能分类（第一期仅 `knots`）；绳结列表和详情走 DB-backed Knots3D metadata，不暴露 Markdown mock。`/api/v1/gear-templates` 和 `/api/v1/gear-templates/:id` 从数据库读取装备模板分类和条目；服务启动时会幂等写入默认系统模板，替代旧的 `content/gear-templates/*.yaml` 文件源。`/api/v1/gear-atlas` 和 `/api/v1/gear-atlas/:id` 返回已审核通过且 `is_deleted=false` 的公共装备图鉴，不包含用户个人购买、位置、标签、备注、拒绝原因、原始投稿快照、审核改动摘要、来源名称、来源链接或来源评分字段；响应保留 `created_at`、`updated_at` 和 `is_deleted` 供客户端统一显示记录时间与可见性状态。图鉴公共尺寸使用 `variants` 数组表示，每项包含 `key`、`label`，以及可选 `official_price_cents`、`official_price_currency`、`weight_g`；分类参数 `specs` 不再接受或返回 `size`、`backpack_size`、`size_or_length`。外部导入来源只在管理员审核接口暴露 `source_name`、`source_url`、`source_rating_score` 和 `source_rating_count` 等审计摘要，不暴露内部去重键、导入批次或授权备注。

用户自己的 `GET /api/v1/me/gear-atlas-submissions` 和管理员审核接口会返回投稿状态字段 `status`、可选 `rejection_reason`、以及审核通过时的 `review_changes`。`review_changes` 是数组，每项包含 `field`、中文 `label`、`before` 和 `after`，表示管理员按原始投稿快照和最终通过值生成的公共字段差异。管理员列表默认只返回 `is_deleted=false`，可用 `deleted=active|deleted|all` 切换可见性。管理员 `PATCH /api/v1/admin/gear-atlas-submissions/:id` 只能替换图鉴公共字段；`DELETE /api/v1/admin/gear-atlas-submissions/:id` 会设置 `is_deleted=true`，`POST /api/v1/admin/gear-atlas-submissions/:id/restore` 会恢复；`POST /api/v1/admin/gear-atlas-submissions/:id/reject` 必须提交非空 `reason`，空白原因返回 `422`。

公共内容语言不使用 query 参数，统一通过请求头：

```http
X-StellarTrail-Locale: zh-CN
# 或
X-StellarTrail-Locale: en
```

未显式传 `X-StellarTrail-Locale` 时会尝试 `Accept-Language`，再 fallback 到 `zh-CN`。`?locale=...` 会返回 `400 unsupported_query_parameter`。公开响应只返回当前语言字段，不返回并列的 `zh/en` 字段；缺少目标语言行时 fallback 到另一种受支持语言，再 fallback 到主表兼容字段。

### Outdoor skills / knots

一期户外技能只有绳结。服务端通过 `import-knots3d` 将 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库；绳结媒体不再从 `/assets/*` 或本地静态目录拼 URL。管理员使用 `PUT /api/v1/admin/skills/knots/:knot_id/media/:asset_id` 上传 GIF/MP4/WebP/PNG 等二进制到 MinIO/S3-compatible object storage，服务端把 `media_resources` 与 `knot_media_resources` 元数据写入数据库。公开读接口只返回 DB 中 active media 的 `url`/`mime_type`/`size_bytes` 等公共字段，允许这些 URL 指向与 API 不同域名的 MinIO/CDN。运行配置只保留一组 `minio` 连接信息，私有反馈图和公开绳结媒体分别配置业务 bucket。

核心媒体 `asset_id` / `media_type`：`thumbnail`、`preview`、`draw_gif`、`turntable_gif`、`draw_mp4`、`turntable_mp4`。Knots3D 全量一期验收目标为 `225 knots × 6 core media = 1350`。

绳结分页参数为 `offset`/`limit`，筛选参数为 `category`，关键词为 `q`；响应字段为 `next_offset`，不返回 `cursor`/`next_cursor`，也不再接受 `difficulty`。`/api/v1/skills/knots/filters` 返回当前语言下可选用途及数量。`/api/v1/skills/knots/offline-manifest` 不接受 query 参数，返回完整离线清单、`item_count`、去重后的 `media_count` 和 `estimated_bytes`，并复用 public response cache 与 `ETag`。public response 不暴露 `zh_slug`、`english_slug`、`source_slug_zh`、`source_slug_en`。

微信端进入绳结列表和首页展示绳结精选前，必须用登录态确认当前账号已同意绳结教程免责声明。`GET /api/v1/me/skills/knots/disclaimer` 返回当前声明 `key`、`version`、`title`、`content`、`accepted` 和 `accepted_at`；`POST /api/v1/me/skills/knots/disclaimer/acceptance` 接受可选的 `client_platform`、`client_version`、`device_model`，幂等写入 `user_disclaimer_acceptances` 留档。同一账号同一声明版本只保存一条记录；声明版本升级后需要重新同意。当前 `v1` 声明按“个人兴趣免费整理、仅供学习和非承重练习、不得直接用于承载人体/攀登/救援/吊装/高空/航海安全等场景、法定责任除外”的保守口径提供；微信端绳结详情页也必须在来源说明前展示同一安全边界提示。

公开绳结分类的 `id` 和 `slug` 保持 Knots3D 导入值不变，但高风险分类标题在 API 返回层使用保守展示名，例如 `climbing-knots` 显示为“攀岩知识（仅供学习）”、`fire-search-rescue-sar-knots` 显示为“消防与救援知识（仅供学习）”、`boating-knots` 显示为“船艇知识（仅供学习）”、`caving-knots` 显示为“探洞知识（仅供学习）”，`essential-knots` 显示为“基础绳结”。上线前可用 `npm run knots:audit-risk-copy -- --url https://api.example.invalid/api/v1/skills/knots/offline-manifest` 只读扫描公开绳结文案；加 `--fail-on-critical` 可在命中“救命”“救援安全带”“承载人体”“吊装”“系在攀岩安全带”等强适用文案时返回非零状态。

登录用户可以通过 `/api/v1/me/skills/favorites` 管理收藏技能。第一期只支持绳结：`GET /api/v1/me/skills/favorites?skill_category=all|knots&offset=0&limit=20` 返回收藏清单和筛选计数，`GET /api/v1/me/skills/favorites/knots/:id` 返回单个绳结收藏状态，`PUT /api/v1/me/skills/favorites/knots/:id` 幂等收藏，`DELETE /api/v1/me/skills/favorites/knots/:id` 幂等软删除收藏记录并保留数据库行。公开绳结列表、详情和离线清单不包含 `is_favorited`，避免 public cache 混入用户态。

### Gear templates and gear atlas i18n

装备模板的模板标题、分类名、条目名存储在 `gear_template_*_localizations` 表中；默认系统模板同时 seed `zh-CN` 和 `en`。装备图鉴的公共 `name` 和 `description` 存储在 `gear_atlas_item_localizations` 表中，新用户投稿默认写入 `zh-CN` 原文行，不做自动机翻；公共 `category_label` 从 `gear_category_localizations` 表解析。`brand`、`model`、`specs`、价格和重量等事实字段不做翻译。

### Gear atlas external import POC

`stellartrail-importer` 提供 `import-gear-atlas-cn` POC CLI，用于把人工挑选的 8264 移动装备详情页导入为待审核图鉴条目。该工具只接受显式 URL 或 URL 文件，不自动扫描全站；默认 dry-run 输出解析预览，真实写库必须传 `--write --submitter-user-id <existing-user-id>`。导入器只保存标题、类目、品牌/型号启发式拆分、重量、人民币价格、公共尺寸 `variants`、结构化 `specs`、评分汇总和来源链接；不复制第三方图片、介绍正文、用户点评正文或评测长文。`source_key` 用于幂等刷新同一来源条目，未删除的已审核条目不会被后续导入覆盖；如果同一 `source_key` 命中已删除条目，会复用原记录、清除 `is_deleted` 并刷新为待审核状态。

### Admin knot media upload

管理员上传接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。该接口是批量导入 Knots3D 媒体的唯一写入入口：脚本不得绕过 API 直接写 MinIO 或 DB。

```http
PUT /api/v1/admin/skills/knots/:knot_id/media/:asset_id
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

## Roadmap

Roadmap 是 DB-backed 产品计划展示，不代表对应功能已经实现。第一版用于微信小程序“我的”页展示后续规划，并支持登录用户投票和站内订阅；订阅只记录数据库状态，不发送微信订阅消息、邮件或推送。`route-encyclopedia` 只作为路线百科规划项出现，本次不会注册 `/api/v1/routes*` 或路线内容表。

```http
GET /api/v1/roadmap?client_key=wechat_miniprogram&status=planned&limit=50&cursor=0
GET /api/v1/me/roadmap?client_key=wechat_miniprogram&status=planned&limit=50&cursor=0
PUT /api/v1/me/roadmap/:id/vote
DELETE /api/v1/me/roadmap/:id/vote
PUT /api/v1/me/roadmap/:id/subscription
DELETE /api/v1/me/roadmap/:id/subscription
GET /api/v1/admin/roadmap?client_key=wechat_miniprogram&status=planned&limit=50&cursor=0
POST /api/v1/admin/roadmap
PATCH /api/v1/admin/roadmap/:id
DELETE /api/v1/admin/roadmap/:id
```

公开接口只返回 `is_published=true` 且 `is_deleted=false` 的条目；`/me/roadmap` 需要 Bearer Token，并额外返回当前用户的 `is_voted` 和 `is_subscribed`。投票和订阅都是幂等操作，取消时软删除用户态记录并保留历史行。管理员接口需要 `admin` 或 `super_admin`，`DELETE` 对 Roadmap 条目执行软删除。

支持的 `status` 为 `planned`、`designing`、`building`、`preview`、`shipped`；支持的 `category` 为 `gear`、`skills`、`routes`、`offline`、`safety`、`community`。初始 seed 包含：

- `smart-packing-template`：按路线/目的地、天数和季节，根据个人装备和历史打包习惯生成打包清单模板。
- `knot-scenario-videos`：绳结增加实际使用场景的视频演示。
- `route-encyclopedia`：路线百科，展示路线难度、季节、风险、交通和准备要点。
- `skill-scenario-index`：按扎营固定、收纳、连接、应急等场景查找绳结和技能。
- `gear-maintenance-reminders`：装备保养、充电、耗材补充和有效期提醒。
- `offline-trip-pack`：一键缓存出行前需要的技能、清单、装备和安全资料。
- `safety-weather-precheck`：出发前天气、风险和急救检查清单。
- `learning-progress`：技能学习进度、已掌握标记和复习提醒。

管理员创建或更新请求：

```json
{
  "client_key": "wechat_miniprogram",
  "title": "智能打包清单模板",
  "summary": "按路线或目的地、天数和季节，结合个人装备和历史打包习惯生成建议清单。",
  "details": "本次 Roadmap 只记录规划，不实现推荐算法。",
  "category": "gear",
  "status": "planned",
  "priority": 100,
  "sort_order": 10,
  "is_published": true
}
```

## Admin API usage statistics

```http
GET /api/v1/admin/api-usage?from=2026-05-01&to=2026-05-18&method=GET&route=/api/v1/me/gears&limit=50&offset=0
```

该接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。它只返回按日期、用户、HTTP 方法、路由模板和状态码聚合后的计数，不返回单次请求日志。

## Admin feedback

用户反馈图片通过 `POST /api/v1/me/uploads` 上传后，再把返回的 `id` 放进 `POST /api/v1/me/feedback` 的 `image_ids`。服务端对反馈图片同时执行四层保护：单张图片大小 `upload.max_image_bytes`、固定窗口上传张数 `upload.max_images_per_window`、用户累计反馈图片张数 `upload.max_total_images_per_user`、用户累计反馈图片大小 `upload.max_total_bytes_per_user`。累计配额默认是每个用户 `100` 张、`200000000` bytes；超过累计配额时返回 `422 validation_failed`，字段为 `image_quota`。

```http
GET /api/v1/admin/feedback?status=open&deleted=active&limit=50&cursor=0
DELETE /api/v1/admin/feedback/:id
POST /api/v1/admin/feedback/:id/restore
Authorization: Bearer <admin-token>
```

该接口需要 Bearer Token，且当前用户必须拥有数据库 `admin_roles` 中的 `admin` 或 `super_admin` 角色。管理员反馈列表默认只返回 `is_deleted=false`，可用 `deleted=active|deleted|all` 切换可见性。返回用户提交的反馈内容、联系方式、页面、客户端环境、提交用户概要和已关联图片元数据。反馈图片仍是私有资源；管理员下载图片时使用返回的 `download_url` 并携带管理员 Bearer Token。已删除反馈或已删除图片都不能下载。

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
          "download_url": "/api/v1/admin/feedback-images/upload-uuid",
          "is_deleted": false,
          "created_at": "2026-05-19T00:00:00Z"
        }
      ],
      "is_deleted": false,
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
POST /api/v1/admin/admins
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
DELETE /api/v1/admin/admins?username=trail_admin
Authorization: Bearer <super-admin-token>
```

删除成功返回 `204`。目标不存在、未拥有管理员角色时返回 `404`；目标是 `super_admin` 时返回 `422 validation_failed`，该接口不负责移除超级管理员。

统计写入由服务端 middleware 异步上报：请求返回路径不会等待数据库写入；统计队列满、后台 worker 失败或写库失败时只丢弃统计，不影响业务请求。

隐私边界：

- 记录 `user_id`（认证成功时）、`method`、matched route 模板、`status_code`、日期桶和调用次数。
- 动态路径只保存模板，例如 `/api/v1/me/gears/:id`，不保存真实装备 ID。
- 不记录 query string、请求体、响应体、Authorization header、access token、refresh token、token hash、Cookie、IP、User-Agent。

查询参数：

| 字段      | 必填 | 说明                                                                   |
| --------- | ---- | ---------------------------------------------------------------------- |
| `from`    | 否   | 起始日期，`YYYY-MM-DD`；默认最近 30 天。                               |
| `to`      | 否   | 结束日期，`YYYY-MM-DD`；默认当天。                                     |
| `user_id` | 否   | 仅查看某个用户的聚合统计。                                             |
| `method`  | 否   | `GET` / `POST` / `PUT` / `PATCH` / `DELETE`。                          |
| `route`   | 否   | matched route 模板，例如 `/api/v1/me/gears/:id`；不能带 query string。 |
| `limit`   | 否   | 默认 50，最大 100。                                                    |
| `offset`  | 否   | 分页偏移。                                                             |

响应示例：

```json
{
  "items": [
    {
      "bucket_date": "2026-05-18",
      "user_id": "user-id",
      "method": "GET",
      "route_pattern": "/api/v1/me/gears",
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

服务端支持可选 Redis 缓存。设置 `REDIS_URL` 后，装备库高频只读接口会走 Redis read-through cache；`POST /api/v1/me/gears`、`PATCH /api/v1/me/gears/:id`、`DELETE /api/v1/me/gears/:id`、`POST /api/v1/me/gears/:id/delete`、`POST /api/v1/me/gears/:id/undelete`、`POST /api/v1/me/gears/:id/restore` 和非 dry-run 导入会递增用户级缓存版本，确保写入后后续读取不会命中旧 key。公共读接口（装备图鉴、技能、绳结列表、筛选、详情、离线 manifest）也会缓存最终 JSON 响应和 ETag，Redis 不可用时退回 API 进程内存缓存；管理员删除/恢复图鉴会同步失效图鉴公共缓存。默认 TTL 为 `REDIS_GEAR_CACHE_TTL_SECONDS=30` 秒，可通过 `REDIS_KEY_PREFIX` 区分环境；公共接口 HTTP 缓存 TTL 由 `public_api.cache_ttl_seconds` 控制。装备 specs 字段频次只保存在 Redis sorted set 中，key 为 `<prefix>:gear:<user_id>:spec-keys:<category>`，member 只保存 spec key，不保存用户填写的具体值。用户标签建议也只保存在 Redis：标签频次 key 为 `<prefix>:gear:<user_id>:tags`，标签颜色偏好 hash key 为 `<prefix>:gear:<user_id>:tag-colors`。

## Gear inventory

个人装备使用顶层 `quantity` 表示同款同规格库存数量，默认 `1`，最小值 `1`。`weight_g`、`official_price_cents`、`purchase_price_cents` 仍表示单件数据；分类计数、状态计数、`current_count`、总重量和总价值都会按 `quantity` 乘算。`specs.quantity` 不再表示个人拥有数量；旧请求里可解析的 `specs.quantity` 会被折入顶层 `quantity`，响应和导出以顶层字段为准。创建装备时若命中同款同规格的现有可用记录，会返回已有记录并递增 `quantity`，不会创建重复行。

```http
GET /api/v1/me/gears/categories?tab=available
GET /api/v1/me/gears/stats?tab=available
GET /api/v1/me/gears/overview?tab=available&limit=20&sort=created_at_desc
GET /api/v1/me/gears/spec-key-rankings?category=electronics_system
GET /api/v1/me/gears/tag-suggestions?limit=20
GET /api/v1/me/gears?tab=available&category=electronics_system&status=available&q=nitecore&sort=created_at_desc&limit=20&cursor=0
POST /api/v1/me/gears
GET /api/v1/me/gears/:id
PATCH /api/v1/me/gears/:id
DELETE /api/v1/me/gears/:id
POST /api/v1/me/gears/:id/delete
POST /api/v1/me/gears/:id/undelete
POST /api/v1/me/gears/:id/restore
GET /api/v1/me/gears/export?tab=available&format=csv
POST /api/v1/me/gears/import
```

## Gear packing lists

打包清单是用户私有的出发前准备清单，用于按路线/目的地和徒步时长挑选本人已有装备，并在出发前逐项勾选已打包。第一版不依赖路线模块，不做自动推荐；`duration_label` 是自由文本，例如 `一日`、`两天一夜`。所有接口都需要 Bearer Token。

```http
GET /api/v1/me/packing-lists?limit=20&cursor=0
POST /api/v1/me/packing-lists
GET /api/v1/me/packing-lists/:id
PATCH /api/v1/me/packing-lists/:id
DELETE /api/v1/me/packing-lists/:id
POST /api/v1/me/packing-lists/:id/items
PATCH /api/v1/me/packing-lists/:id/items/:item_id
DELETE /api/v1/me/packing-lists/:id/items/:item_id
```

创建/更新清单：

```json
{
  "name": "一日武功山",
  "route_name": "武功山",
  "duration_label": "一日"
}
```

清单列表返回每份清单的计划携带数量、已打包数量和按计划数量计算的总重量：

```json
{
  "items": [
    {
      "id": "packing-list-id",
      "name": "一日武功山",
      "route_name": "武功山",
      "duration_label": "一日",
      "item_count": 2,
      "packed_count": 1,
      "total_weight_g": 890,
      "created_at": "2026-05-24T00:00:00Z",
      "updated_at": "2026-05-24T00:00:00Z"
    }
  ],
  "next_cursor": null
}
```

批量加入装备：

```json
{
  "gear_ids": ["gear-id-1", "gear-id-2"]
}
```

同一装备重复加入同一清单保持幂等。只能加入当前用户自己的、未归档、未软删除装备；若提交其它用户、已归档、已软删除或不存在的装备 ID，返回 `422 validation_failed`。已加入清单的装备之后若被归档或软删除，清单详情仍保留该条目，并在条目上返回 `unavailable=true` 与 `unavailable_reason=archived|deleted`。

清单详情：

```json
{
  "id": "packing-list-id",
  "name": "一日武功山",
  "route_name": "武功山",
  "duration_label": "一日",
  "stats": {
    "item_count": 2,
    "packed_count": 1,
    "total_weight_g": 890
  },
  "items": [
    {
      "id": "packing-item-id",
      "gear_id": "gear-id-1",
      "planned_quantity": 1,
      "packed_quantity": 1,
      "packed": true,
      "unavailable": false,
      "unavailable_reason": null,
      "gear": {
        "id": "gear-id-1",
        "category": "backpack_system",
        "category_label": "背负系统",
        "name": "轻量小包",
        "brand": null,
        "model": null,
        "status": "available",
        "status_label": "可用",
        "quantity": 2,
        "weight_g": 800,
        "tags": [],
        "tag_colors": {},
        "is_deleted": false,
        "created_at": "2026-05-24T00:00:00Z",
        "updated_at": "2026-05-24T00:00:00Z"
      },
      "created_at": "2026-05-24T00:00:00Z",
      "updated_at": "2026-05-24T00:00:00Z"
    }
  ],
  "created_at": "2026-05-24T00:00:00Z",
  "updated_at": "2026-05-24T00:00:00Z"
}
```

更新打包状态可继续只传兼容字段 `packed`，也可以传 `planned_quantity` 或 `packed_quantity`。服务端会限制 `planned_quantity` 不超过当前库存数量，并用 `packed_quantity >= planned_quantity` 推导完成状态。

```json
{
  "planned_quantity": 2,
  "packed_quantity": 1
}
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
  "atlas_item_id": null,
  "selected_variant_key": "standard",
  "selected_variant_label": "标准版",
  "quantity": 2,
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

个人装备可以独立存在，不要求先关联装备图鉴。用户实际选择或手填的尺寸保存在 `selected_variant_label`，可选的稳定 key 保存在 `selected_variant_key`；当装备关联图鉴时，`atlas_item_id` 指向公开图鉴条目，客户端可读取该图鉴条目的 `variants` 作为“可选尺寸”。个人装备 `specs` 继续保存容量、背长、收纳尺寸等参数，但不再接受 `size`、`backpack_size`、`size_or_length`。

装备响应包含 `created_at`、`updated_at` 和 `is_deleted`。`DELETE /api/v1/me/gears/:id` 只表示归档，会设置 `archived_at` 并进入 `tab=history`；`POST /api/v1/me/gears/:id/restore` 可从 history 恢复。真正软删除使用 `POST /api/v1/me/gears/:id/delete` 设置 `is_deleted=true`；默认列表、详情、统计、导出和导入去重都忽略已删除记录。`GET /api/v1/me/gears` 支持 `deleted=active|deleted|all`，默认 `active`；`POST /api/v1/me/gears/:id/undelete` 只恢复 `is_deleted=false`，不改变原来的 `archived_at`。

### Gear overview

`GET /api/v1/me/gears/overview` 是小程序首屏聚合接口，不是新的装备业务模型。它一次返回装备分类筛选、统计卡片和首屏列表，避免小程序进入首页或装备页时连续请求 `categories`、`stats`、`list` 三个接口。数据仍来自现有装备分类统计、装备统计和装备列表逻辑。

支持参数：

- `tab`: `available` 或 `history`，默认 `available`。
- `limit`: 首屏列表数量，默认 `20`，仓储层会按列表接口同样规则限制在 `1..100`。
- `sort`: 首屏列表排序，默认 `created_at_desc`。

不支持 `cursor`、`q`、`category`、`status`。筛选、搜索、状态切换和后续分页仍调用 `GET /api/v1/me/gears`，避免每次筛选都重新计算统计。

```json
{
  "categories": {
    "items": [{ "id": "all", "label": "全部装备", "count": 1 }]
  },
  "stats": {
    "current_count": 1,
    "archived_count": 0,
    "total_value_cents": 63900,
    "total_weight_g": 315,
    "by_category": [],
    "by_status": []
  },
  "list": {
    "items": [],
    "next_cursor": null
  }
}
```

### Spec key rankings

`GET /api/v1/me/gears/spec-key-rankings?category=electronics_system` 返回当前登录用户在该装备分类下常填写的 specs 字段 key：

```json
{
  "keys": ["battery_capacity", "rated_energy"]
}
```

该接口只返回当前分类允许的 spec key。Redis 未启用或不可用时返回空数组，装备保存不受影响。

### Tag suggestions

`GET /api/v1/me/gears/tag-suggestions?limit=20` 返回当前登录用户常用装备标签，以及该标签当前颜色偏好：

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
