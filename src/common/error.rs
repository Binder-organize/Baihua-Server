use crate::common::StandardResponse;
use crate::infrastructure::environment::Environment;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum ErrorResponse {
    #[error("Invalid JSON format: {0}")]
    Json(String),

    #[error("Verification failed: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Request information error: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Insufficient permissions: {0}")]
    Forbidden(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Request timed out: {0}")]
    RequestTimeout(String),

    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    #[error("Request body too large: {0}")]
    PayloadTooLarge(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl ErrorResponse {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponse::Json(_) => StatusCode::BAD_REQUEST,
            ErrorResponse::Validation(_) => StatusCode::BAD_REQUEST,
            ErrorResponse::BadRequest(_) => StatusCode::BAD_REQUEST,
            ErrorResponse::Authentication(_) => StatusCode::UNAUTHORIZED,
            ErrorResponse::Forbidden(_) => StatusCode::FORBIDDEN,
            ErrorResponse::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            ErrorResponse::RequestTimeout(_) => StatusCode::REQUEST_TIMEOUT,
            ErrorResponse::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            ErrorResponse::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorResponse::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ErrorResponse::NotFound(_) => StatusCode::NOT_FOUND,
            ErrorResponse::Conflict(_) => StatusCode::CONFLICT,
            ErrorResponse::Database(_) | ErrorResponse::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            ErrorResponse::Authentication(_) => "AUTHENTICATION_ERROR",
            ErrorResponse::Forbidden(_) => "FORBIDDEN_ERROR",
            ErrorResponse::Json(_) => "INVALID_JSON_ERROR",
            ErrorResponse::Validation(_) => "VALIDATION_ERROR",
            ErrorResponse::InternalError(_) => "INTERNAL_SERVER_ERROR",
            ErrorResponse::BadRequest(_) => "BAD_REQUEST_ERROR",
            ErrorResponse::Database(_) => "DATABASE_ERROR",
            ErrorResponse::TooManyRequests(_) => "RATE_LIMIT_ERROR",
            ErrorResponse::RequestTimeout(_) => "REQUEST_TIMEOUT_ERROR",
            ErrorResponse::MethodNotAllowed(_) => "METHOD_NOT_ALLOWED_ERROR",
            ErrorResponse::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE_ERROR",
            ErrorResponse::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE_ERROR",
            ErrorResponse::NotFound(_) => "NOT_FOUND_ERROR",
            ErrorResponse::Conflict(_) => "CONFLICT_ERROR",
        }
    }

    // Message shown to the client.
    // In production, internal and database errors return a generic message
    // to avoid leaking implementation details.
    fn client_message(&self) -> String {
        match self {
            ErrorResponse::InternalError(_) | ErrorResponse::Database(_) => {
                if Environment::from_environment().is_production() {
                    "An internal error occurred. Please try again later.".to_string()
                } else {
                    self.to_string()
                }
            }
            _ => self.to_string(),
        }
    }

    fn to_response(&self) -> StandardResponse {
        StandardResponse::error(
            self.status_code(),
            self.error_code().to_string(),
            self.client_message(),
        )
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let response = self.to_response();

        // Log error.
        if response.status.is_server_error() {
            warn!(
                "Server returned an error: HTTP status code: 'INTERNAL_SERVER_ERROR', code: '{}', message: '{}', ID: '{}'.",
                response.body.code, response.body.message, response.body.response_id
            );
        } else {
            info!(
                "Server returned an error: HTTP status code: '{}', code: '{}', message: '{}', ID: '{}'.",
                response.status,
                response.body.code,
                response.body.message,
                response.body.response_id
            );
        }

        (response.status, Json(response.body)).into_response()
    }
}
/*
// Maybe is useless.
// Error type is converted to Response.
impl From<ErrorResponse> for Response {
    fn from(error: ErrorResponse) -> Self {
        error.into_response()
    }
}
*/
