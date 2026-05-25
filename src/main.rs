#[tokio::main]
async fn main() -> anyhow::Result<()> {
    linger::cli::run().await
}
