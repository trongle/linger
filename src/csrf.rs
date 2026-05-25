use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

pub fn skip<H>(handler: H) -> H {
    handler
}

pub async fn middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let unsafe_method = matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    let jar = CookieJar::from_headers(req.headers());
    let cookie_token = jar.get("csrf_token").map(|c| c.value().to_string());
    if unsafe_method {
        let header_token = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok());
        if cookie_token.as_deref().is_none() || header_token != cookie_token.as_deref() {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    let token = cookie_token.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut res = next.run(req).await;
    if !res.headers().contains_key(header::SET_COOKIE) {
        let cookie = Cookie::build(("csrf_token", token))
            .path("/")
            .same_site(SameSite::Lax)
            .http_only(true)
            .build();
        res.headers_mut()
            .append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    }
    Ok(res)
}
