use std::sync::Arc;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn require_auth(
    State(expected_token): State<Arc<String>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("X-Auth-Token")
        .and_then(|v| v.to_str().ok());

    match token {
        Some(t) if t == expected_token.as_str() => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
