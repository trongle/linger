use crate::{error::AppResult, view::View};
use axum::{routing::get, Router};
use serde_json::json;

pub fn routes() -> Router<crate::state::AppState> {
    Router::new().route("/", get(index))
}
async fn index(view: View) -> AppResult<impl axum::response::IntoResponse> {
    view.render("pages/home/index.html", json!({}))
}
