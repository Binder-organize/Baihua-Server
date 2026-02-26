use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

use crate::ServerState;
use crate::authenticate::jsonwebtoken::generate_token;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::user::{UserLogin, find_user_by_username, verify_password};

pub async fn login(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserLogin>, JsonRejection>,
) -> Result<SuccessResponse, ErrorResponse> {
    let Json(user_login) = user.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    let user_output = find_user_by_username(&user_login.username, &state.pool).await?;
    let user = user_output.ok_or_else(|| {
        ErrorResponse::Authentication("Invalid username or password.".to_string())
    })?;

    if !verify_password(&(user_login.password), &(user_login.username), &state.pool).await? {
        return Err(ErrorResponse::Authentication(
            "Invalid username or password.".to_string(),
        ));
    }

    // Generate token.
    let token = generate_token(
        state.configure.user.jsonwebtoken_expiration_hours,
        &user.username,
    )
    .await?;

    info!(
        "User logged in: {}, id: {}, jsonwebtoken: {}.",
        user_login.username,
        user.id.to_string(),
        token
    );

    Ok(SuccessResponse::new(
        StatusCode::OK,
        "User logged in successfully.".to_string(),
        json!({
            "token": token,
            "user": user,
        }),
    ))
}
