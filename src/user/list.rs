use crate::ServerState;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::user::User;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::error;

// Get the user list.
pub async fn list_users(
    State(state): State<Arc<ServerState>>,
) -> Result<SuccessResponse, ErrorResponse> {
    let rows = sqlx::query(
        r#"SELECT id, username, email, nickname, phone_number, created_at, is_active
           FROM users WHERE is_active = true ORDER BY created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        error!("Failed to list users: {}", error);
        ErrorResponse::InternalError("Failed to list users.".to_string())
    })?;

    let users: Vec<User> = rows
        .iter()
        .map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            nickname: row.get("nickname"),
            phone_number: row.get("phone_number"),
            created_at: row.get("created_at"),
            is_active: row.get("is_active"),
        })
        .collect();

    Ok(SuccessResponse::new(
        StatusCode::OK,
        "Users retrieved successfully.".to_string(),
        json!({ "users": users }),
    ))
}
