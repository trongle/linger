use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub email_verification_token_hash: Option<String>,
    pub email_verification_sent_at: Option<String>,
    pub email_verified_at: Option<String>,
    pub password_reset_token_hash: Option<String>,
    pub password_reset_sent_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}
impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn create(
        &self,
        email: &str,
        password_hash: &str,
        verify_hash: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query("INSERT INTO users (email, password_hash, email_verification_token_hash, email_verification_sent_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)").bind(email).bind(password_hash).bind(verify_hash).execute(&self.pool).await?;
        Ok(res.last_insert_rowid())
    }
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ? AND deleted_at IS NULL")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }
    pub async fn verify_email(&self, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET email_verified_at = CURRENT_TIMESTAMP, email_verification_token_hash = NULL WHERE email_verification_token_hash = ?").bind(token_hash).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn set_reset_token(&self, id: i64, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password_reset_token_hash = ?, password_reset_sent_at = CURRENT_TIMESTAMP WHERE id = ?").bind(token_hash).bind(id).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn reset_password(
        &self,
        token_hash: &str,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password_hash = ?, password_reset_token_hash = NULL, password_reset_sent_at = NULL WHERE password_reset_token_hash = ?").bind(password_hash).bind(token_hash).execute(&self.pool).await?;
        Ok(())
    }
}
