mod message;
mod room;

use crate::ServerState;
use crate::common::error::ErrorResponse;
use axum::Router;
use axum::routing::get;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

// Check if it is a room that exists.
#[allow(dead_code)]
pub async fn is_room_member(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool, ErrorResponse> {
    let row = sqlx::query("SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2")
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| {
            error!("Failed to check room membership: {}", error);
            ErrorResponse::InternalError("Failed to check room membership.".to_string())
        })?;

    Ok(row.is_some())
}

pub fn router(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route(
            "/rooms",
            get(room::list_rooms).post(room::create_or_get_room),
        )
        .route(
            "/rooms/{room_id}/messages",
            get(message::get_messages).post(message::send_message),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::authenticate::authenticate,
        ))
}
