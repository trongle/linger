use crate::{config::Config, repositories::Repositories, templates::Templates};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: Config,
    pub db: SqlitePool,
    pub templates: Templates,
    pub repositories: Repositories,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let templates = Templates::load(&config.templates)?;
        let db = crate::db::connect(&config.database).await?;
        let repositories = Repositories::new(db.clone());
        Ok(Self(Arc::new(AppStateInner {
            config,
            db,
            templates,
            repositories,
        })))
    }
    pub fn config(&self) -> &Config {
        &self.0.config
    }
    pub fn db(&self) -> &SqlitePool {
        &self.0.db
    }
    pub fn templates(&self) -> &Templates {
        &self.0.templates
    }
    pub fn repositories(&self) -> &Repositories {
        &self.0.repositories
    }
}
