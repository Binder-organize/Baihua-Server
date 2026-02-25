use crate::ServerState;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::user::{User, UserRegister};
use axum::extract::State;
use axum::{Json, extract::rejection::JsonRejection, http::StatusCode};
use serde_json::json;
use std::sync::Arc;

pub async fn register(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserRegister>, JsonRejection>,
) -> Result<SuccessResponse, ErrorResponse> {
    // Parse JSON and extract user data.
    let Json(new_user) = user.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    let user_created = User::new(new_user, &state.pool).await?;

    Ok(SuccessResponse::new(
        StatusCode::CREATED,
        "User created successfully".to_string(),
        json!({
            "message": "User created successfully",
            "user": user_created
        }),
    ))
}
