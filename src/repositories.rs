use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Repositories {
    pool: SqlitePool,
}
impl Repositories {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub fn users(&self) -> crate::auth::repository::UserRepository {
        crate::auth::repository::UserRepository::new(self.pool.clone())
    }
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
