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
