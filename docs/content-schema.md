# Public Data Notes

Repo-local `content/` seed folders have been removed. API startup no longer reads YAML/Markdown from `content/assets`, `content/skills`, `content/mountains`, `content/routes`, or `content/gear-templates`.

## Knots3D metadata

绳结数据从 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库。导入后由以下接口返回 locale-resolved 字段：

- `GET /api/skills`
- `GET /api/skills/knots/list`
- `GET /api/skills/knots/detail/:id`

媒体二进制不进入 JSON 文件或 repo-local assets。管理员上传接口把媒体写入 MinIO/S3-compatible object storage，并将公开 URL 写入数据库；公开读接口只返回 DB 中 active media 的公共字段。

## Public locale model

公共内容支持 `zh-CN` 和 `en`。接口通过 `X-StellarTrail-Locale` 或 `Accept-Language` 选择语言，默认 `zh-CN`，不接受 `?locale=` query 参数。数据库主表保存稳定 ID、状态、来源和兼容兜底字段，多语言文案保存在 `*_localizations` 表中；公开 API 只返回当前语言字段，不返回并列的 `zh/en` 字段。

## Gear templates

装备模板由数据库保存，API 启动时幂等 seed 默认系统模板。旧的 `content/gear-templates/*.yaml` 文件源已删除。模板标题、分类名和条目名分别存储在 `gear_template_localizations`、`gear_template_category_localizations`、`gear_template_item_localizations` 中；主表旧 `title/name` 字段保留为兼容兜底。

`GET /api/gear-templates` 返回：

```json
{
  "items": [
    {
      "id": "backpacking-basic",
      "title": "入门徒步基础装备模板",
      "categories": [
        {
          "id": "backpack_system",
          "name": "背负系统",
          "items": ["背包", "防雨罩"]
        }
      ]
    }
  ]
}
```

`GET /api/gear-templates/:id` 返回单个模板，找不到时返回 404。

## Gear atlas

装备图鉴公共浏览读取 `gear_atlas_items` 中已审核通过的公共字段。`name` 和 `description` 可通过 `gear_atlas_item_localizations` 返回不同语言；新用户投稿默认只写入原文和 `zh-CN` 本地化行，不做自动翻译。`category_label` 来自 `gear_category_localizations`，`brand`、`model`、`specs`、价格和重量等事实字段不做翻译。

图鉴投稿会在 `submitted_snapshot_json` 中保存创建时的公共字段快照。管理员审核时可修改公共字段；通过审核时服务端比较原始快照与最终公开字段，把差异写入 `review_changes_json` 供投稿用户查看。拒绝原因保存在 `rejection_reason`，只返回给投稿用户和管理员；公开图鉴接口不返回快照、拒绝原因或审核修改摘要。

外部装备来源导入目前只作为 POC 审核入口：`import-gear-atlas-cn` 支持人工提供的 8264 移动装备详情页 URL，导入标题、类目、品牌/型号启发式拆分、重量、人民币价格、结构化 `specs`、评分汇总和来源链接，并写入 `source_key`、`source_name`、`source_url`、`source_license_note`、`import_batch_id`、`imported_at`、`source_rating_score` 和 `source_rating_count`。公开 API 只返回来源名称、来源链接和评分汇总，不返回内部去重键、批次或授权备注。导入器不保存第三方图片、介绍正文、用户点评正文或评测长文；所有导入条目先进入 `pending` 审核状态。

## Removed route families

山峰和路线模块尚未开始实现，服务端不注册以下旧契约：

- `GET /api/mountains`
- `GET /api/mountains/:id`
- `GET /api/routes`
- `GET /api/routes/:id`
- `GET /assets/*`

后续如果重新启动路线相关工作，应先设计 DB schema、repository、API contract 和客户端类型，再新增路由；不要恢复 repo-local content 文件读取作为兼容层。
