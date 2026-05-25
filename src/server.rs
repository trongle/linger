use crate::{config::Config, router, state::AppState, telemetry};
use axum::{extract::DefaultBodyLimit, http::StatusCode, Router};
use std::{net::SocketAddr, time::Duration};
use tower_http::timeout::TimeoutLayer;

pub async fn serve(config: Config) -> anyhow::Result<()> {
    telemetry::init(&config);
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let timeout = config.server.request_timeout_secs;
    let limit = config.server.body_limit_bytes;
    let state = AppState::new(config).await?;
    let app: Router = router::router(state)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(timeout),
        ))
        .layer(DefaultBodyLimit::max(limit));
    tracing::info!(%addr, "starting linger");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
