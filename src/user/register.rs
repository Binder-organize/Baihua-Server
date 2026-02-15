use axum::{
    extract::rejection::JsonRejection,
    Json,
};
use crate::common::error::ErrorType;
use crate::user::{NewUser, User};

pub async fn register(
    user: Result<Json<NewUser>, JsonRejection>,
) -> Result<Json<serde_json::Value>, ErrorType> {
    // Parse JSON and extract user data.
    let Json(new_user) = user.map_err(|err| {
        ErrorType::JsonRejectionError(err.to_string())
    })?;

    let user_created = User::new(NewUser {
        username: new_user.username,
        email: new_user.email,
        password: new_user.password,
    })?;

    Ok(Json(serde_json::json!({
        "message": "User registered successfully.",
        "user": {
            "username": user_created.username,
            "email": user_created.email,
            "created_at": user_created.created_at
        }
    })))
}