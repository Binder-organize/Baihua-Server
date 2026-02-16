use crate::common::error::ErrorType;
use crate::common::response::Response;
use crate::user::UserLogin;
use axum::{Json, extract::rejection::JsonRejection, http::StatusCode};
use serde_json::json;
use tracing::info;

pub async fn login(user: Result<Json<UserLogin>, JsonRejection>) -> Result<Response, ErrorType> {
    let Json(user_login) = user.map_err(|error| ErrorType::JsonRejection(error.to_string()))?;

    // todo check user.

    info!("Login: {}", user_login.username);
    Ok(Response::new(
        StatusCode::OK,
        json!({ "message": "Login successful" }),
    ))
}
