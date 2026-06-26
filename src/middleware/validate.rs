use crate::common::error::ErrorResponse;
use crate::infrastructure::environment::Environment;
use axum::body::Body as HttpBody;
use axum::body::Bytes;
use axum::http;
use axum::http::{HeaderMap, Request};
use axum::{middleware::Next, response::Response};
use serde_json::Value;

const MAX_BODY_SIZE_PRODUCTION: usize = 1024 * 1024; // 1 MB
const MAX_BODY_SIZE_DEVELOPMENT: usize = 10 * 1024 * 1024; // 10 MB

async fn validate_json_body(
    headers: &HeaderMap,
    request: Request<HttpBody>,
) -> Result<(http::request::Parts, Bytes, Value), ErrorResponse> {
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
    } else if Environment::from_env().is_development() {
        MAX_BODY_SIZE_DEVELOPMENT
    } else {
        unreachable!()
    };

    let (parts, body) = request.into_parts();

    let body_bytes = axum::body::to_bytes(body, max_size).await.map_err(|_| {
        ErrorResponse::BadRequest(format!("Request body too large (max {} bytes).", max_size))
    })?;

    let json_value: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ErrorResponse::Json(format!("Invalid JSON format: {}.", e)))?;

    Ok((parts, body_bytes, json_value))
}

pub async fn validate_register(
    headers: HeaderMap,
    request: Request<HttpBody>,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let (parts, body_bytes, json_value) = validate_json_body(&headers, request).await?;

    json_value
        .get("username")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ErrorResponse::Validation("username is required.".to_string()))?;

    json_value
        .get("email")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ErrorResponse::Validation("email is required.".to_string()))?;

    json_value
        .get("password")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ErrorResponse::Validation("password is required.".to_string()))?;

    let request = Request::from_parts(parts, HttpBody::from(body_bytes));
    Ok(next.run(request).await)
}

pub async fn validate_login(
    headers: HeaderMap,
    request: Request<HttpBody>,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let (parts, body_bytes, json_value) = validate_json_body(&headers, request).await?;

    json_value
        .get("username")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ErrorResponse::Validation("username is required.".to_string()))?;

    json_value
        .get("password")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ErrorResponse::Validation("password is required.".to_string()))?;

    let request = Request::from_parts(parts, HttpBody::from(body_bytes));
    Ok(next.run(request).await)
}
