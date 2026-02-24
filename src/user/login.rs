use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

use crate::ServerState;
use crate::common::error::ErrorType;
use crate::common::response::ApiResponse;
use crate::user::{User, UserLogin};
use crate::authenticate::jsonwebtoken::generate_token;

pub async fn login(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserLogin>, JsonRejection>,
) -> Result<ApiResponse, ErrorType> {
    let Json(user_login) = user.map_err(|error| ErrorType::Json(error.to_string()))?;

    let user_output = User::find_user_by_username(&user_login.username, &state.pool).await?;
    let user = user_output
        .ok_or_else(|| ErrorType::Authentication("Invalid username or password.".to_string()))?;

    if !User::verify_password(&(user_login.password), &(user_login.username), &state.pool).await? {
        return Err(ErrorType::Authentication(
            "Invalid username or password.".to_string(),
        ));
    }

    // Generate token.
    let token = generate_token(&(user.id.to_string())).await?;

    info!(
        "User logged in: {}, id: {}, jsonwebtoken: {}.",
        user_login.username,
        user.id.to_string(),
        token
    );

    Ok(ApiResponse::new(
        StatusCode::OK,
        "The user is logged in successfully.".to_string(),
        json!({
            "user": user,
            "token": token
        }),
    ))
}
