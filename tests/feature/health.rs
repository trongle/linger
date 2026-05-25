use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn health_ok() {
    let mut cfg = linger::config::Config::load().unwrap();
    let dir = tempfile::tempdir().unwrap();
    cfg.database.url = format!("sqlite://{}", dir.path().join("health.sqlite3").display());
    let state = linger::state::AppState::new(cfg).await.unwrap();
    let app = linger::router::router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
