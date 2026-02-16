use crate::common::error::ErrorType;
use crate::common::response::Response;
use crate::user::{User, UserRegister};
use axum::{Json, extract::rejection::JsonRejection, http::StatusCode};
use serde_json::json;

pub async fn register(
    user: Result<Json<UserRegister>, JsonRejection>,
) -> Result<Response, ErrorType> {
    // Parse JSON and extract user data.
    let Json(new_user) = user.map_err(|error| ErrorType::JsonRejection(error.to_string()))?;

    let user_created = User::new(UserRegister {
        username: new_user.username,
        email: new_user.email,
        password: new_user.password,
    })?;

    Ok(Response::new(
        StatusCode::CREATED,
        json!({
            "message": "User registered successfully.",
            "user": {
                "username": user_created.username,
                "email": user_created.email,
                "created_at": user_created.created_at
            }
        }),
    ))
}
