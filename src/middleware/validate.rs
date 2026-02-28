use crate::common::error::ErrorResponse;
use axum::body::Body as HttpBody;
use axum::http::{HeaderMap, Request};
use axum::{middleware::Next, response::Response};
use serde_json::Value;

pub async fn validate_user(
    headers: HeaderMap,
    request: Request<HttpBody>,
    next: Next,
) -> Result<Response, ErrorResponse> {
    // Check Content-Type.
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Err(ErrorResponse::Validation(
            "Content-Type must be 'application/json'.".to_string(),
        ));
    }

    let (parts, body) = request.into_parts();

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| ErrorResponse::BadRequest(format!("Failed to read request body: {}.", e)))?;

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
        return Err(ErrorResponse::Validation("Username or email cannot be empty.".to_string()))
    }

    let request = Request::from_parts(parts, HttpBody::from(body_bytes));

    Ok(next.run(request).await)
}
