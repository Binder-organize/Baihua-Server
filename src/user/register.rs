use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::user::{NewUser, User};

pub async fn register(
    user: Result<Json<NewUser>, JsonRejection>,
) -> impl IntoResponse {
    match user {
        Ok(Json(user)) => {
            let new_user = NewUser {
                username: user.username,
                email: user.email,
                password: user.password,
            };

            match User::new(new_user) {
                Ok(user_created) => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({
                        "message": "User registered successfully.",
                        "user": {
                            "username": user_created.username,
                            "email": user_created.email,
                            "created_at": user_created.created_at
                        }
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": format!("Registration failed: {}.", e)
                    })),
                )
                    .into_response(),
            }
        }
        Err(err) => {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid JSON format: {}", err),
                })),
            )
                .into_response()
        }
    }
}