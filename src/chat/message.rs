use crate::ServerState;
use crate::chat::is_room_member;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::middleware::authenticate::AuthenticatedUser;
use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, extract::rejection::JsonRejection};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<Uuid>,
}

pub async fn send_message(
    State(state): State<Arc<ServerState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(room_id): Path<Uuid>,
    body: Result<Json<SendMessageRequest>, JsonRejection>,
) -> Result<SuccessResponse, ErrorResponse> {
    let Json(request) = body.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    if !is_room_member(&state.pool, room_id, auth_user.user_id).await? {
        return Err(ErrorResponse::Forbidden(
            "You are not a member of this room.".to_string(),
        ));
    }

    let message_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO messages (id, room_id, sender_id, content, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(message_id)
    .bind(room_id)
    .bind(auth_user.user_id)
    .bind(&request.content)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        error!("Failed to insert message: {}", error);
        ErrorResponse::InternalError("Failed to send message.".to_string())
    })?;

    Ok(SuccessResponse::new(
        StatusCode::CREATED,
        "Message sent successfully.".to_string(),
        json!({
            "id": message_id,
            "room_id": room_id,
            "sender_id": auth_user.user_id,
            "content": request.content,
            "created_at": now.to_rfc3339(),
        }),
    ))
}

pub async fn get_messages(
    State(state): State<Arc<ServerState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(room_id): Path<Uuid>,
    Query(params): Query<GetMessagesQuery>,
) -> Result<SuccessResponse, ErrorResponse> {
    if !is_room_member(&state.pool, room_id, auth_user.user_id).await? {
        return Err(ErrorResponse::Forbidden(
            "You are not a member of this room.".to_string(),
        ));
    }

    let limit = params.limit.unwrap_or(50).min(100);

    // Keyset pagination: use (created_at, id) composite to guarantee deterministic ordering
    // even when two messages share the same created_at timestamp.
    let rows = if let Some(before_id) = params.before {
        sqlx::query(
            "SELECT id, room_id, sender_id, content, created_at FROM messages \
             WHERE room_id = $1 \
               AND (created_at, id) < (SELECT created_at, id FROM messages WHERE id = $2) \
             ORDER BY created_at DESC, id DESC LIMIT $3",
        )
        .bind(room_id)
        .bind(before_id)
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, room_id, sender_id, content, created_at FROM messages \
             WHERE room_id = $1 \
             ORDER BY created_at DESC, id DESC LIMIT $2",
        )
        .bind(room_id)
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(|error| {
        error!("Failed to get messages: {}", error);
        ErrorResponse::InternalError("Failed to get messages.".to_string())
    })?;

    let has_more = rows.len() > limit as usize;
    let visible = rows.iter().take(limit as usize);

    let messages: Vec<serde_json::Value> = visible
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "room_id": row.get::<Uuid, _>("room_id"),
                "sender_id": row.get::<Uuid, _>("sender_id"),
                "content": row.get::<String, _>("content"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
            })
        })
        .collect();

    let next_cursor = if has_more {
        let last_idx = (limit as usize).saturating_sub(1);
        rows.get(last_idx).map(|row| row.get::<Uuid, _>("id"))
    } else {
        None
    };

    Ok(SuccessResponse::new(
        StatusCode::OK,
        "Messages retrieved successfully.".to_string(),
        json!({
            "messages": messages,
            "has_more": has_more,
            "next_cursor": next_cursor,
        }),
    ))
}
