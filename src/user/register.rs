use crate::ServerState;
use crate::common::StandardResponse;
use crate::common::error::ErrorResponse;
use crate::user::{UserRegister, new_user};
use axum::extract::State;
use axum::{Json, extract::rejection::JsonRejection, http::StatusCode};
use serde_json::json;
use std::sync::Arc;

pub async fn register(
    State(state): State<Arc<ServerState>>,
    user_new: Result<Json<UserRegister>, JsonRejection>,
) -> Result<StandardResponse, ErrorResponse> {
    // Parse JSON and extract user data.
    let Json(user) = user_new.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    let user_created = new_user(user, &state.pool).await?;

    Ok(StandardResponse::success(
        StatusCode::CREATED,
        "User created successfully".to_string(),
        json!({
            "user": user_created
        }),
    ))
}
