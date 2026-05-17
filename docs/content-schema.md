# Public Data Notes

Repo-local `content/` seed folders have been removed. API startup no longer reads YAML/Markdown from `content/assets`, `content/skills`, `content/mountains`, `content/routes`, or `content/gear-templates`.

## Knots3D metadata

绳结数据从 `.hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json` 导入数据库。导入后由以下接口返回 locale-resolved 字段：

- `GET /api/skills`
- `GET /api/skills/knots/list`
- `GET /api/skills/knots/detail/:id`

媒体二进制不进入 JSON 文件或 repo-local assets。管理员上传接口把媒体写入 MinIO/S3-compatible object storage，并将公开 URL 写入数据库；公开读接口只返回 DB 中 active media 的公共字段。

## Gear templates

装备模板由数据库保存，API 启动时幂等 seed 默认系统模板。旧的 `content/gear-templates/*.yaml` 文件源已删除。

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

## Removed route families

山峰和路线模块尚未开始实现，服务端不注册以下旧契约：

- `GET /api/mountains`
- `GET /api/mountains/:id`
- `GET /api/routes`
- `GET /api/routes/:id`
- `GET /assets/*`

后续如果重新启动路线相关工作，应先设计 DB schema、repository、API contract 和客户端类型，再新增路由；不要恢复 repo-local content 文件读取作为兼容层。
