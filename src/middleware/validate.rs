use crate::common::error::ErrorResponse;
use crate::infrastructure::environment::Environment;
use axum::body::Body as HttpBody;
use axum::http::{HeaderMap, Request};
use axum::{middleware::Next, response::Response};
use serde_json::Value;

const MAX_BODY_SIZE_PRODUCTION: usize = 1024 * 1024; // 1 MB
const MAX_BODY_SIZE_DEVELOPMENT: usize = 10 * 1024 * 1024; // 10 MB

pub async fn validate_user(
    headers: HeaderMap,
    request: Request<HttpBody>,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Err(ErrorResponse::Validation(
            "Content-Type must be 'application/json'.".to_string(),
        ));
    }

    let max_size = if Environment::from_env().is_production() {
        MAX_BODY_SIZE_PRODUCTION
    } else {
        MAX_BODY_SIZE_DEVELOPMENT
    };

    let (parts, body) = request.into_parts();

    let body_bytes = axum::body::to_bytes(body, max_size).await.map_err(|_| {
        ErrorResponse::BadRequest(format!("Request body too large (max {} bytes).", max_size))
    })?;

    let json_value: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ErrorResponse::Json(format!("Invalid JSON format: {}.", e)))?;

    let username = json_value
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorResponse::Validation("Username is required.".to_string()))?;

    let email = json_value
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorResponse::Validation("Email is required.".to_string()))?;

    if username.is_empty() && email.is_empty() {
        return Err(ErrorResponse::Validation(
            "Username or email cannot be empty.".to_string(),
        ));
    }

    let request = Request::from_parts(parts, HttpBody::from(body_bytes));

    Ok(next.run(request).await)
}
