use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

use crate::ServerState;
use crate::common::error::ErrorType;
use crate::common::response::Response;
use crate::user::{User, UserLogin};

#[axum::debug_handler]
pub async fn login(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserLogin>, JsonRejection>,
) -> Result<Response, ErrorType> {
    let Json(user_login) = user.map_err(|e| ErrorType::JsonRejection(e.to_string()))?;

    let user_opt = User::find_user_by_username(&user_login.username, &state.pool).await?;
    let user = user_opt
        .ok_or_else(|| ErrorType::Validation("Invalid username or password.".to_string()))?;

    if !user
        .verify_password(
            user_login.password.clone(),
            &user_login.username,
            &state.pool,
        )
        .await
    {
        return Err(ErrorType::Validation(
            "Invalid username or password.".to_string(),
        ));
    }

    info!(
        "User logged in: {}, id is {}",
        user_login.username,
        user.id.to_string()
    );

    Ok(Response::new(
        StatusCode::OK,
        json!(
            {
                "message": "User logged in successfully.",
                "user": user
            }
        ),
    ))
}
