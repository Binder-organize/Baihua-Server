use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub uuid: String,
}

#[derive(Error, Debug)]
pub enum ErrorType {
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
}

impl ErrorType {
    // Get HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorType::Json(_) => StatusCode::BAD_REQUEST,
            ErrorType::Validation(_) => StatusCode::BAD_REQUEST,
            ErrorType::BadRequest(_) => StatusCode::BAD_REQUEST,
            ErrorType::Authentication(_) => StatusCode::UNAUTHORIZED,
            ErrorType::Forbidden(_) => StatusCode::FORBIDDEN,
            ErrorType::Database(_) | ErrorType::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    // Get error code.
    fn error_code(&self) -> &'static str {
        match self {
            ErrorType::Authentication(_) => "AUTHENTICATION_ERROR",
            ErrorType::Forbidden(_) => "FORBIDDEN_ERROR",
            ErrorType::Json(_) => "INVALID_JSON_ERROR",
            ErrorType::Validation(_) => "VALIDATION_ERROR",
            ErrorType::InternalError(_) => "INTERNAL_SERVER_ERROR",
            ErrorType::BadRequest(_) => "BAD_REQUEST_ERROR",
            ErrorType::Database(_) => "DATABASE_ERROR",
        }
    }
}

impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ApiError {
            code: self.error_code().to_string(),
            message: self.to_string(),
            uuid: Uuid::now_v7().to_string(),
        };

        // Log error.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            warn!(
                "The server returns a error: HTTP status code: 'INTERNAL_SERVER_ERROR', error code: '{}', message: '{}', ID: '{}'.",
                self.error_code(),
                error_response.message,
                error_response.uuid
            );
        } else {
            info!(
                "The server returns a error: HTTP status code: '{}', error code: '{}', message: '{}', ID: '{}'.",
                status,
                self.error_code(),
                error_response.message,
                error_response.uuid
            );
        }

        (status, Json(error_response)).into_response()
    }
}

// Error type is converted to Response.
impl From<ErrorType> for Response {
    fn from(error: ErrorType) -> Self {
        error.into_response()
    }
}
