use crate::{
    app::{constant::AUTHORIZATION_BEARER_PREFIX, lazy::AUTH_TOKEN},
    common::model::error::ChatError,
};
use axum::{
    Json,
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};

// 管理员认证中间件函数
pub async fn admin_auth_middleware(request: Request<Body>, next: Next) -> Response {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX));

    match auth_header {
        Some(token) if token == AUTH_TOKEN.as_str() => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        )
            .into_response(),
    }
}
