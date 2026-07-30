use crate::common::StandardResponse;
use crate::{ServerState, common::error::ErrorResponse};
use axum::{extract::State, http::StatusCode};
use serde_json::json;
use std::sync::Arc;

pub async fn health_check(
    State(state): State<Arc<ServerState>>,
) -> Result<StandardResponse, ErrorResponse> {
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|error| {
            tracing::error!("Health check failed: {}", error);
            ErrorResponse::InternalError("Database connection failed.".to_string())
        })?;

    Ok(StandardResponse::success(
        StatusCode::OK,
        "Service is healthy.".to_string(),
        json!({"status": "ok"}),
    ))
}
