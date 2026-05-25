use crate::{config::Config, state::AppState};

pub async fn build_state(config: Config) -> anyhow::Result<AppState> {
    AppState::new(config).await
}
