use crate::ServerState;
use crate::common::error::ErrorType;
use crate::common::response::Response;
use crate::user::{User, UserRegister};
use axum::extract::State;
use axum::{Json, extract::rejection::JsonRejection, http::StatusCode};
use serde_json::json;
use std::sync::Arc;

pub async fn register(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserRegister>, JsonRejection>,
) -> Result<Response, ErrorType> {
    // Parse JSON and extract user data.
    let Json(new_user) = user.map_err(|error| ErrorType::JsonRejection(error.to_string()))?;

    let user_created = User::new(new_user, &state.pool).await?;

    Ok(Response::new(
        StatusCode::CREATED,
        json!({
            "message": "User registered successfully.",
            "user": {
                "id": user_created.id.to_string(),
                "username": user_created.username,
                "email": user_created.email,
                "created_at": user_created.created_at.to_string(),
            }
        }),
    ))
}
