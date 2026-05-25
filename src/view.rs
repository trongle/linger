use crate::{error::AppResult, state::AppState};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderMap},
    response::{Html, IntoResponse},
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct View {
    pub state: AppState,
    pub headers: HeaderMap,
    pub csrf_token: String,
}

impl View {
    pub fn is_htmx(&self) -> bool {
        self.headers.get("HX-Request").is_some()
    }
    pub fn take_flash(&self) -> Option<String> {
        None
    }
    pub fn render(&self, template: &str, data: Value) -> AppResult<impl IntoResponse> {
        let mut ctx = serde_json::Map::new();
        ctx.insert("app".into(), json!({"name": self.state.config().app.name, "base_url": self.state.config().app.base_url}));
        ctx.insert("csrf_token".into(), json!(self.csrf_token));
        ctx.insert("is_htmx".into(), json!(self.is_htmx()));
        if let Value::Object(map) = data {
            for (k, v) in map {
                ctx.insert(k, v);
            }
        }
        Ok(Html(
            self.state
                .templates()
                .render(template, Value::Object(ctx))?,
        ))
    }
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for View {
    type Rejection = crate::AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let State(state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| crate::AppError::Unauthorized)?;
        let jar = CookieJar::from_headers(&parts.headers);
        let csrf_token = jar
            .get("csrf_token")
            .map(|c| c.value().to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Ok(Self {
            state,
            headers: parts.headers.clone(),
            csrf_token,
        })
    }
}
