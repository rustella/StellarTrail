# StellarTrail Agent Rules

这是 StellarTrail 仓库的 agent 入口；更详细的架构、命令和检查清单可继续下沉到 `.agent/knowledge/`、`.agent/commands.yaml` 和 `.agent/checklists/`。

## 服务端 Rust 注释规则

- `services/api/src` 与 `crates/{domain,db,importer,migration}/src` 下的服务端 Rust 代码必须写详细中文注释。
- 包、crate、module 文件顶部使用符合 rustdoc 规范的 `//!` 说明模块职责、边界和关键约束。
- `pub` 类型、`pub` 函数，以及重要私有函数使用符合 rustdoc 规范的 `///` 注释说明用途、输入输出、错误或安全边界。
- 函数内部的认证、数据库、缓存、状态切换、外部调用、导入导出、迁移 DDL 等关键逻辑使用普通 `//` 注释解释原因与不变量。
- 新增或修改服务端代码后至少运行 `cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`；涉及 rustdoc 时额外运行 `cargo doc --workspace --no-deps`。
