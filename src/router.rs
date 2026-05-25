use crate::state::AppState;
use axum::{middleware, Router};
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};

pub fn router(state: AppState) -> Router {
    let static_mount = state.config().static_files.mount.clone();
    let static_path = state.config().static_files.path.clone();
    Router::new()
        .merge(crate::home::routes())
        .nest("/health", crate::health::routes())
        .nest("/auth", crate::auth::routes())
        .nest_service(&static_mount, ServeDir::new(static_path))
        .with_state(state)
        .layer(middleware::from_fn(crate::csrf::middleware))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}
