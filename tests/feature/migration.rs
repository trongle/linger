#[tokio::test]
async fn migration_runs() {
    let dir = tempfile::tempdir().unwrap();
    let url = format!("sqlite://{}", dir.path().join("test.sqlite3").display());
    let cfg = linger::config::DatabaseConfig {
        url,
        auto_migrate: false,
    };
    let pool = linger::db::connect(&cfg).await.unwrap();
    linger::db::migrate(&pool).await.unwrap();
}
