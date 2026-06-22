use crate::ServerState;
use crate::authenticate::jsonwebtoken::generate_token;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::user::{UserLogin, find_user_with_password};
use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::State, http::StatusCode};
use bcrypt::verify;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

pub async fn login(
    State(state): State<Arc<ServerState>>,
    user: Result<Json<UserLogin>, JsonRejection>,
) -> Result<SuccessResponse, ErrorResponse> {
    let Json(user_login) = user.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    let Some((user, password_hash)) =
        find_user_with_password(&user_login.username, &state.pool).await?
    else {
        return Err(ErrorResponse::Authentication(
            "Invalid username or password.".to_string(),
        ));
    };

    if !verify(&user_login.password, &password_hash).map_err(|error| {
        error!("Password verification failed: {}", error);
        ErrorResponse::BadRequest("Failed to verify credentials.".to_string())
    })? {
        return Err(ErrorResponse::Authentication(
            "Invalid username or password.".to_string(),
        ));
    }

    let token = generate_token(
        &state.jwt_secret,
        state.configure.user.jsonwebtoken_expiration_hours,
        &user.username,
    )
    .await?;

    if state.environment.is_production() {
        // Production environments should not expose JWT tokens.
        info!(
            "User logged in: {}, id: {}.",
            user_login.username,
            user.id.to_string()
        );
    } else {
        info!(
            "User logged in: {}, id: {}, token: {}.",
            user_login.username,
            user.id.to_string(),
            token
        );
    }

    Ok(SuccessResponse::new(
        StatusCode::OK,
        "User logged in successfully.".to_string(),
        json!({
            "token": token,
            "user": user,
        }),
    ))
}
