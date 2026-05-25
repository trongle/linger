use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn csrf_rejects_missing_token() {
    let mut cfg = linger::config::Config::load().unwrap();
    let dir = tempfile::tempdir().unwrap();
    cfg.database.url = format!("sqlite://{}", dir.path().join("csrf.sqlite3").display());
    let state = linger::state::AppState::new(cfg).await.unwrap();
    let app = linger::router::router(state);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=a@b.test&password=password"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
