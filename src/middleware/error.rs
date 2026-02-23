use crate::common::error::ApiError;
use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::error;
use uuid::Uuid;

// Panic error handler.
// Theoretically, the business logic layer should not throw a panic.
pub async fn panic(request: Request, next: Next) -> Response {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| next.run(request)));

    match result {
        Ok(response) => response.await,
        Err(_) => {
            let error = ApiError {
                code: "SERVER_CRASHES".to_string(),
                message: "An unexpected error occurred.".to_string(),
                uuid: Uuid::now_v7().to_string(),
            };
            error!(
                "The server used 'panic!', returned an 'HTTP 500' error, ID: {}",
                error.uuid
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

// Not found(404) error handler.
pub async fn not_found(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status() == StatusCode::NOT_FOUND {
        let error = ApiError {
            code: "NOT_FOUND_ERROR".to_string(),
            message: "The requested resource was not found.".to_string(),
            uuid: Uuid::now_v7().to_string(),
        };
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }

    response
}
