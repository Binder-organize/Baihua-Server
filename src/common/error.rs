use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorType {
    // JSON deserialization error.
    #[error("Invalid JSON format: {0}")]
    JsonRejectionError(String),

    // Validation error.
    #[error("Validation failed: {0}")]
    ValidationError(String),

    // User registration error.
    #[error("User registration failed: {0}")]
    RegistrationError(String),

    // Internal server error.
    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorType {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorType::JsonRejectionError(_) => StatusCode::BAD_REQUEST,
            ErrorType::ValidationError(_) => StatusCode::BAD_REQUEST,
            ErrorType::RegistrationError(_) => StatusCode::BAD_REQUEST,
            ErrorType::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    fn error_code(&self) -> &'static str {
        match self {
            ErrorType::JsonRejectionError(_) => "JSON_REJECTION_ERROR",
            ErrorType::ValidationError(_) => "VALIDATION_ERROR",
            ErrorType::RegistrationError(_) => "REGISTRATION_ERROR",
            ErrorType::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

// Convert ErrorType to axum response.
impl IntoResponse for ErrorType {
    fn into_response(self) -> axum::response::Response {
        let code = self.error_code().to_string();
        let message = self.to_string();
        let response = ErrorResponse { code, message };
        (self.status_code(), Json(response)).into_response()
    }
}

// Convert anyhow::Error to ErrorType.
impl From<anyhow::Error> for ErrorType {
    fn from(err: anyhow::Error) -> Self {
        ErrorType::InternalError(err.to_string())
    }
}