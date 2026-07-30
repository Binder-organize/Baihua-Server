use crate::common::StandardResponse;
use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::error;

// Panic error handler.
// Theoretically, the business logic layer should not throw a panic.
pub async fn panic(request: Request, next: Next) -> Response {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| next.run(request)));

    match result {
        Ok(response) => response.await,
        Err(_) => {
            let error = StandardResponse::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_CRASHES".to_string(),
                "The server encountered an error.".to_string(),
            );

            error!(
                "The server used 'panic!', returned an 'HTTP 500' error, ID: {}",
                error.body.response_id
            );

            (error.status, Json(error.body)).into_response()
        }
    }
}

// Not found(404) error handler.
pub async fn not_found(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status() == StatusCode::NOT_FOUND {
        let error = StandardResponse::error(
            StatusCode::NOT_FOUND,
            "NOT_FOUND_ERROR".to_string(),
            "The requested resource was not found.".to_string(),
        );

        return (error.status, Json(error.body)).into_response();
    }

    response
}
