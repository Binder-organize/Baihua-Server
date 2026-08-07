use crate::ServerState;
use crate::common::StandardResponse;
use crate::common::error::ErrorResponse;
use crate::common::extractor::JsonBody;
use crate::user::{UserRegister, new_user};
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

pub async fn register(
    State(state): State<Arc<ServerState>>,
    JsonBody(user): JsonBody<UserRegister>,
) -> Result<StandardResponse, ErrorResponse> {
    let user_created = new_user(user, &state.pool).await?;

    Ok(StandardResponse::success(
        StatusCode::CREATED,
        "User created successfully".to_string(),
        json!({
            "user": user_created
        }),
    ))
}
