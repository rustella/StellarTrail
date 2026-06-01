# 部署指南

这份指南只保留本地 Docker Compose 验证和生产 Docker / Traefik 部署的入口信息。真实生产密钥、`config.yaml`、ACME storage 和服务器本地覆盖文件必须留在安全渠道或服务器本地，不提交到仓库。

## 本地 Docker Compose 验证

一次性启动 PostgreSQL、Redis 和 API，并执行注册、登录、邮箱验证码登录、找回密码等 curl 冒烟测试：

```bash
COMPOSE_PROJECT_NAME=stellartrail_it API_HOST_PORT=18080 POSTGRES_HOST_PORT=15432 REDIS_HOST_PORT=16379 \
  bash infra/test/integration-test.sh
```

脚本结束或失败时会自动执行 `docker compose down -v --remove-orphans`，清理本次验证容器和卷。

需要单独检查 Compose 配置时：

```bash
docker compose -f infra/test/docker-compose.yml config
```

## 测试机持久环境

测试机的持久联调环境不要使用 `integration-test.sh`，该脚本会在结束时执行 `down -v` 清理临时容器和卷。测试机应固定使用同一套 Compose project 和同一份宿主机数据目录，让不同 worktree 更新的是同一个 Postgres、Redis 和 MinIO 数据集：

```bash
docker-compose \
  --env-file .env \
  -p stellartrail-test \
  -f infra/test/docker-compose.yml \
  -f infra/test/docker-compose.testhost.yml \
  -f infra/test/docker-compose.shared-data.yml \
  up -d --build
```

`infra/test/docker-compose.testhost.yml` 和真实 `.env` 仍只保留在测试机本地；仓库只提交共享数据目录覆盖文件。`STELLARTRAIL_TEST_DATA_ROOT` 是必填项，缺失时 Compose 会报错，避免误把数据写进某个 worktree 自己的目录或自动创建新的默认卷。测试机当前使用 docker-compose v1，建议显式传 `--env-file`，不要依赖当前目录自动发现 `.env`。

## 客户端配置文件

真实客户端配置文件只保留在本地或构建环境，已被 `.gitignore` 忽略；仓库只提交示例文件。需要调整地址时，复制对应示例文件后修改：

| 客户端     | 示例配置                                                | 真实配置（不提交）                              |
| ---------- | ------------------------------------------------------- | ----------------------------------------------- |
| Web        | `apps/web/.env.example`                                 | `apps/web/.env.local`                           |
| 微信小程序 | `apps/wechat-miniprogram/miniprogram/config.example.ts` | `apps/wechat-miniprogram/miniprogram/config.ts` |

Web 可通过 `VITE_STELLARTRAIL_API_BASE_URL` 和 `VITE_STELLARTRAIL_ASSETS_BASE_URL` 覆盖；本地 Vite 开发默认使用同源 `/api/v1`，并通过 `VITE_STELLARTRAIL_API_PROXY_TARGET` 代理到真实或本地 API，避免浏览器 CORS 拦截。微信小程序端会读取 `miniprogram/config.ts`，缺失时回退到占位地址。

## API 配置细节

API 默认监听 `127.0.0.1:8080`。启动时会先加载 `.env`，再读取根目录 `config.yaml`（存在时）或 `CONFIG_PATH` 指定的 YAML 文件，最后由环境变量覆盖大部分 YAML 配置。短信认证的 `sms:` 配置例外：阿里云 access key、签名名、可选方案名和模板号只从 YAML 读取，不通过 `SMS_*` 环境变量传输。`config.example.yaml` 会提交到 Git，实际 `config.yaml` / `config.*.yaml` 会被忽略。

本地默认数据库地址为 `sqlite://stellartrail.db`；生产和集成测试推荐 PostgreSQL 16+。配置层会识别 `sqlite://`、`postgres://`、`postgresql://` 和 `mysql://` URL，其中 MySQL-compatible 数据库边界已保留，当前仓库默认运行路径仍以 SQLite / PostgreSQL 为主。

本地可通过 `APP_ENV=local` + `WECHAT_MOCK_LOGIN=true` 启用 mock 登录；正式微信登录需设置 `WECHAT_MOCK_LOGIN=false`、`WECHAT_APP_ID` 和 `WECHAT_APP_SECRET`。邮箱验证码生产投递通过 SMTP：设置 `MAIL_ENABLED=true`、`MAIL_SMTP_HOST=smtp.example.invalid`、`MAIL_SMTP_USERNAME=[REDACTED]`，并通过被忽略的 `config.yaml` 或 secret manager 注入 `MAIL_SMTP_PASSWORD` 和发件人地址。邮箱验证码用于注册、邮箱验证码登录和找回密码。

短信验证码生产投递通过阿里云号码认证服务。启用时把 `sms.enabled`、`sms.access_key_id`、`sms.access_key_secret` 等写入被忽略的 `config.yaml`，如需指定方案再填写 `sms.scheme_name`；也可以由 secret manager 渲染为同等 YAML 文件后挂载到 `/app/config.yaml`。

如需启用 Redis 缓存，设置 `REDIS_URL=redis://127.0.0.1:6379/0`；`REDIS_GEAR_CACHE_TTL_SECONDS` 控制装备读取缓存 TTL。绳结媒体通过 MinIO/S3-compatible 对象存储上传，公开读接口只返回数据库中的媒体 URL，不再从 `/assets/*` 拼接绳结媒体路径。服务端只维护一组 `minio` 连接配置，反馈图与绳结媒体分别通过 `object_storage.bucket` 和 `knots_media_storage.bucket` 配置业务 bucket。

管理员权限存储在数据库 `admin_roles` 表：已有且未删除的 `stellarisw` 用户会在迁移时被 seed 为 `super_admin`，`super_admin` 可通过 `/api/v1/admin/admins` 授予或移除普通 `admin`。`admin` 与 `super_admin` 都可调用 Knots 媒体上传、Gear Atlas 审核、反馈查看和 `GET /api/v1/admin/api-usage`。统计使用异步上报，只保存 matched route 模板和聚合计数，不记录 query、请求体、Authorization、token、Cookie、IP 或 User-Agent。

## 生产 Docker / Traefik

生产部署配置拆分在 `infra/production/` 下，目标服务器部署根目录为 `/www/service/stellartail`：

| 入口                                                  | 作用                                                           |
| ----------------------------------------------------- | -------------------------------------------------------------- |
| `infra/production/traefik/docker-compose.yml`         | 公网入口，暴露 80/443，并通过 Let’s Encrypt 自动签发和续期证书 |
| `infra/production/site/docker-compose.yml`            | 官网站点                                                       |
| `infra/production/web/docker-compose.yml`             | Web App                                                        |
| `infra/production/api/docker-compose.yml`             | API、PostgreSQL、Redis、MinIO 和 bucket initializer            |
| `infra/production/domains.example.yaml`               | 可提交的非敏感域名示例                                         |
| `infra/production/api/config.production.example.yaml` | 可提交的 API 生产配置示例                                      |

API 容器通过 Docker 服务名访问 `postgres`、`redis`、`minio`，并把服务器根目录的真实 `config.yaml` 只读挂载到 `/app/config.yaml`。`infra/production/api/compose-from-config.sh` 会从 YAML 派生 PostgreSQL、Redis、MinIO 的 Compose 运行变量。

生产 API 不使用 `infra/production/api/.env`。仓库只提交 `config.example.yaml` 和 `*.example.yaml`，真实配置、证书存储和密钥文件必须由部署环境注入。

## 交付前检查

```bash
docker compose --env-file infra/production/traefik/.env.example -f infra/production/traefik/docker-compose.yml config >/dev/null
docker compose --env-file infra/production/site/.env.example -f infra/production/site/docker-compose.yml config >/dev/null
docker compose --env-file infra/production/web/.env.example -f infra/production/web/docker-compose.yml config >/dev/null
docker compose --env-file infra/production/api/.env.example -f infra/production/api/docker-compose.yml config >/dev/null
```

上线前还应确认生产服务器上的 Compose project name、已有数据卷、Traefik entrypoint / resolver 名称和 API 健康检查结果，避免误建新卷或误起第二套容器。
