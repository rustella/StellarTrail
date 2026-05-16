//! API 服务二进制入口，负责初始化配置、日志、路由和优雅停机。

use anyhow::Context;
use stellartrail_api::{build_state, config::ApiConfig, routes::build_router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// 执行 `main` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = ApiConfig::from_env()?;
    let state = build_state(config.clone()).await?;
    let app = build_router(state).layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(config.bind_addr())
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr()))?;

    info!(addr = %config.bind_addr(), "StellarTrail API listening");

    axum::serve(listener, app)
        // 使用 graceful shutdown，让容器停止或 Ctrl-C 时尽量完成已接收请求。
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("api server failed")
}

/// 执行 `init tracing` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("stellartrail_api=debug,tower_http=debug,info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 执行 `shutdown signal` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
