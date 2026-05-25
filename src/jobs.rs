use tokio::time::{sleep, Duration};

/// Disabled example background job. Spawn manually from server startup if desired.
pub async fn heartbeat_example() {
    loop {
        tracing::info!("linger heartbeat");
        sleep(Duration::from_secs(60)).await;
    }
}
