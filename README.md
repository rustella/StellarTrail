# 🌌 StellarTrail（星径）

> 面向中国户外场景的路线百科、装备准备助手和离线技能工具箱。

[English](README.en.md)

<p align="center">
  <img alt="Language" src="https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue" />
  <img alt="Rust" src="https://img.shields.io/badge/API-Rust%20%2B%20Axum-orange" />
  <img alt="WeChat Mini Program" src="https://img.shields.io/badge/Client-WeChat%20Mini%20Program-07C160" />
  <img alt="Content Driven" src="https://img.shields.io/badge/Content-YAML%20%2B%20Markdown-8A2BE2" />
</p>

<p align="center">
  <strong>选路线</strong> · <strong>识风险</strong> · <strong>配装备</strong> · <strong>学技能</strong>
</p>

---

## ✨ 项目一眼看懂

StellarTrail 聚焦中国徒步、露营和轻量户外用户，目标是在一个产品里完成「选路线 → 看难度、季节和风险 → 生成装备清单 → 对比个人装备库 → 学习相关技能」的准备闭环。

当前产品以 **微信小程序** 为优先入口，**Rust API 服务** 提供后端能力；路线、山峰、技能和装备模板由 **YAML/Markdown 内容驱动**，方便先沉淀高质量户外知识库。

| 模块          | 当前目标                             |
| ------------- | ------------------------------------ |
| 🗺️ 路线百科   | 沉淀山峰、路线、难度、季节、风险     |
| 🎒 装备准备   | 生成打包清单，对照个人装备库         |
| 🧭 技能工具箱 | 覆盖绳结、露营、导航、天气、急救等   |
| 📦 内容导入   | 用 YAML/Markdown 快速迭代种子内容    |
| 🧱 Rust 后端  | 提供稳定 API、领域模型和数据访问边界 |

## 🚀 MVP 范围

首个版本重点建设：

- 🧑‍💻 微信登录占位和账号模型。
- ⛰️ 山峰与路线目录。
- 🧾 路线详情：难度、季节、风险、交通和装备建议。
- 🎒 用户个人装备库。
- ✅ 基于路线生成打包清单。
- 🪢 绳结、露营、打包、导航、天气、急救等户外技能目录。
- 📝 从 YAML/Markdown 导入内容的能力。
- 🗄️ 数据库抽象：本地开发默认 SQLite，生产推荐 PostgreSQL，并保留 MySQL 兼容空间。

暂不包含实时导航、社区信息流、路线交易/带队、完整 GPX 编辑和商城能力。

## 🌱 当前种子内容

| 类型        | 内容                     |
| ----------- | ------------------------ |
| ⛰️ 山峰     | 武功山                   |
| 🥾 路线     | 武功山经典 2 天 1 夜穿越 |
| 🪢 技能     | 可调节帐绳结             |
| 🎒 装备模板 | 入门徒步基础装备模板     |

## 🧭 仓库结构

```text
StellarTrail/
  apps/
    wechat-miniprogram/     # 微信小程序端
  services/
    api/                    # Rust axum API 服务
  crates/
    domain/                 # 共享 Rust 领域模型
    db/                     # DB 配置和仓储边界
    importer/               # 内容导入边界
    migration/              # 数据迁移边界
  packages/
    api-client-ts/          # 供小程序 / Web / Mobile 复用的 TS API client
    shared-types/           # TS 共享 DTO 类型
  content/                  # 山峰、路线、技能、装备模板内容
  docs/                     # 产品、架构、API、内容 schema 文档
  infra/                    # 本地 / 开发部署文件
  scripts/                  # 开发辅助脚本
```

## ⚡ 快速开始

### 1. 环境要求

- 🦀 Rust stable toolchain（仓库包含 `rust-toolchain.toml`，需要 `rustfmt` 和 `clippy`）。
- 🟢 Node.js 22+ 与 npm。
- 💬 微信开发者工具（调试小程序时需要）。

### 2. 安装依赖

```bash
npm install
```

### 3. 启动 API

```bash
cp .env.example .env
cargo run -p stellartrail-api
```

API 默认监听 `127.0.0.1:8080`，并从环境变量读取配置。默认数据库地址为 `sqlite://stellartrail.db`。

可用以下接口做本地冒烟验证：

```bash
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/meta
```

当前已实现接口：

```http
GET /healthz
GET /api/meta
```

### 4. 打开微信小程序

用微信开发者工具打开 `apps/wechat-miniprogram`。项目配置中的 `miniprogramRoot` 指向 `miniprogram/`。

## 🧪 常用检查命令

```bash
# 前端 / TypeScript workspace 检查
npm run check

# Markdown / JSON / YAML / TS 等格式检查
npm run format:check

# Rust workspace 检查
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 📚 文档索引

- 📌 [MVP 范围](docs/mvp.md)
- 🏗️ [架构说明](docs/architecture.md)
- 🔌 [API 草案](docs/api.md)
- 🧩 [内容 Schema 草案](docs/content-schema.md)

## 🏷️ 命名约定

- 产品名：**StellarTrail**
- 中文占位名：**星径**
- 仓库名：`StellarTrail`
