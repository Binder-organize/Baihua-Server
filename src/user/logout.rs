use crate::ServerState;
use crate::common::StandardResponse;
use crate::common::error::ErrorResponse;
use crate::middleware::authenticate::AuthenticatedUser;
use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tracing::info;

// Log the caller out of every device by bumping token_version so that all
// previously issued JWTs fail the version check.
pub async fn logout(
    State(state): State<Arc<ServerState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<StandardResponse, ErrorResponse> {
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE id = $1")
        .bind(auth_user.user_id)
        .execute(&state.pool)
        .await?;

    info!("User {} logged out of all devices.", auth_user.user_id);

    Ok(StandardResponse::success(
        StatusCode::OK,
        "Logged out successfully.".to_string(),
        json!(null),
    ))
}
