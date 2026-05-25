pub mod repository;

use crate::{
    email::Email,
    error::{AppError, AppResult},
    state::AppState,
    view::View,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Form, Path, State},
    http::header,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use rand::rngs::OsRng;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_form).post(login))
        .route("/logout", post(logout))
        .route("/register", get(register_form).post(register))
        .route("/email/verify/:token", get(verify_email))
        .route("/password/forgot", get(forgot_form).post(forgot))
        .route("/password/reset/:token", get(reset_form))
        .route("/password/reset", post(reset))
}

#[derive(Deserialize)]
struct AuthForm {
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct EmailForm {
    email: String,
}
#[derive(Deserialize)]
struct ResetForm {
    token: String,
    password: String,
}

async fn login_form(view: View) -> AppResult<impl IntoResponse> {
    view.render("pages/auth/login.html", json!({}))
}
async fn register_form(view: View) -> AppResult<impl IntoResponse> {
    view.render("pages/auth/register.html", json!({}))
}
async fn forgot_form(view: View) -> AppResult<impl IntoResponse> {
    view.render("pages/auth/password/forgot.html", json!({}))
}
async fn reset_form(Path(token): Path<String>, view: View) -> AppResult<impl IntoResponse> {
    view.render("pages/auth/password/reset.html", json!({"token": token}))
}

async fn register(
    State(state): State<AppState>,
    Form(form): Form<AuthForm>,
) -> AppResult<impl IntoResponse> {
    let password_hash = hash_password(&form.password)?;
    let token = uuid::Uuid::new_v4().to_string();
    state
        .repositories()
        .users()
        .create(&form.email, &password_hash, Some(&hash_token(&token)))
        .await?;
    let _ = Email::new(state.clone())
        .send_verify(&form.email, &token)
        .await;
    Ok(Redirect::to("/auth/login"))
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<AuthForm>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = state
        .repositories()
        .users()
        .find_by_email(&form.email)
        .await?
    else {
        return Err(AppError::Unauthorized);
    };
    verify_password(&form.password, &user.password_hash)?;
    let sid = uuid::Uuid::new_v4().to_string();
    let csrf = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO sessions (id, user_id, csrf_token) VALUES (?, ?, ?)")
        .bind(&sid)
        .bind(user.id)
        .bind(&csrf)
        .execute(state.db())
        .await?;
    let jar = jar
        .add(
            Cookie::build((state.config().session.cookie_name.clone(), sid))
                .path("/")
                .http_only(true)
                .build(),
        )
        .add(
            Cookie::build(("csrf_token", csrf))
                .path("/")
                .http_only(true)
                .build(),
        );
    Ok((jar, Redirect::to("/")))
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<impl IntoResponse> {
    if let Some(c) = jar.get(&state.config().session.cookie_name) {
        let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(c.value())
            .execute(state.db())
            .await;
    }
    Ok((
        [(
            header::SET_COOKIE,
            format!("{}=; Path=/; Max-Age=0", state.config().session.cookie_name),
        )],
        Redirect::to("/"),
    ))
}

async fn verify_email(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> AppResult<impl IntoResponse> {
    state
        .repositories()
        .users()
        .verify_email(&hash_token(&token))
        .await?;
    Ok(Redirect::to("/auth/login"))
}

async fn forgot(
    State(state): State<AppState>,
    Form(form): Form<EmailForm>,
) -> AppResult<impl IntoResponse> {
    if let Some(user) = state
        .repositories()
        .users()
        .find_by_email(&form.email)
        .await?
    {
        let token = uuid::Uuid::new_v4().to_string();
        state
            .repositories()
            .users()
            .set_reset_token(user.id, &hash_token(&token))
            .await?;
        let _ = Email::new(state.clone())
            .send_reset(&form.email, &token)
            .await;
    }
    Ok(Redirect::to("/auth/login"))
}

async fn reset(
    State(state): State<AppState>,
    Form(form): Form<ResetForm>,
) -> AppResult<impl IntoResponse> {
    state
        .repositories()
        .users()
        .reset_password(&hash_token(&form.token), &hash_password(&form.password)?)
        .await?;
    Ok(Redirect::to("/auth/login"))
}

fn hash_password(password: &str) -> AppResult<String> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|e| AppError::BadRequest(e.to_string()))?
        .to_string())
}
fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(hash).map_err(|e| AppError::BadRequest(e.to_string()))?,
        )
        .map_err(|_| AppError::Unauthorized)
}
fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
