use crate::ServerState;
use crate::common::error::ErrorResponse;
use crate::common::success::SuccessResponse;
use crate::middleware::authenticate::AuthenticatedUser;
use crate::user::find_user_by_username;
use axum::Extension;
use axum::extract::State;
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
pub struct CreateRoomRequest {
    pub username: String,
}

pub async fn create_or_get_room(
    State(state): State<Arc<ServerState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    body: Result<Json<CreateRoomRequest>, JsonRejection>,
) -> Result<SuccessResponse, ErrorResponse> {
    let Json(request) = body.map_err(|error| ErrorResponse::Json(error.to_string()))?;

    // Find target user by username.
    let target_user = find_user_by_username(&request.username, &state.pool)
        .await?
        .ok_or(ErrorResponse::BadRequest(
            "Target user not found.".to_string(),
        ))?;

    // Don't allow creating a room with yourself.
    if target_user.id == auth_user.user_id {
        return Err(ErrorResponse::BadRequest(
            "Cannot create a room with yourself.".to_string(),
        ));
    }

    // Check if a private room already exists between these two users.
    let existing = sqlx::query(
        "SELECT r.id, r.name, r.created_by, r.created_at, r.is_group \
         FROM rooms r \
         INNER JOIN room_members m1 ON r.id = m1.room_id AND m1.user_id = $1 \
         INNER JOIN room_members m2 ON r.id = m2.room_id AND m2.user_id = $2 \
         WHERE r.is_group = false",
    )
    .bind(auth_user.user_id)
    .bind(target_user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        error!("Failed to check existing room: {}", error);
        ErrorResponse::InternalError("Failed to check existing room.".to_string())
    })?;

    if let Some(row) = existing {
        let room_id: Uuid = row.get("id");
        let members = vec![auth_user.user_id, target_user.id];
        return Ok(SuccessResponse::new(
            StatusCode::OK,
            "Room already exists.".to_string(),
            json!({
                "id": room_id,
                "name": Option::<String>::None,
                "created_by": row.get::<Uuid, _>("created_by"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
                "is_group": row.get::<bool, _>("is_group"),
                "members": members
            }),
        ));
    }

    // Create a new room.
    let room_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO rooms (id, name, created_by, created_at, is_group) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(room_id)
    .bind(Option::<String>::None)
    .bind(auth_user.user_id)
    .bind(now)
    .bind(false)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        error!("Failed to create room: {}", error);
        ErrorResponse::InternalError("Failed to create room.".to_string())
    })?;

    // Add both users as members.
    sqlx::query("INSERT INTO room_members (room_id, user_id, joined_at) VALUES ($1, $2, $3)")
        .bind(room_id)
        .bind(auth_user.user_id)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(|error| {
            error!("Failed to add member to room: {}", error);
            ErrorResponse::InternalError("Failed to add room member.".to_string())
        })?;

    sqlx::query("INSERT INTO room_members (room_id, user_id, joined_at) VALUES ($1, $2, $3)")
        .bind(room_id)
        .bind(target_user.id)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(|error| {
            error!("Failed to add member to room: {}", error);
            ErrorResponse::InternalError("Failed to add room member.".to_string())
        })?;

    Ok(SuccessResponse::new(
        StatusCode::CREATED,
        "Room created successfully.".to_string(),
        json!({
            "id": room_id,
            "name": Option::<String>::None,
            "created_by": auth_user.user_id,
            "created_at": now.to_rfc3339(),
            "is_group": false,
            "members": vec![auth_user.user_id, target_user.id]
        }),
    ))
}

pub async fn list_rooms(
    State(state): State<Arc<ServerState>>,
    request: axum::extract::Request,
) -> Result<SuccessResponse, ErrorResponse> {
    let auth_user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or(ErrorResponse::Authentication(
            "Not authenticated.".to_string(),
        ))?;

    let room_rows = sqlx::query(
        "SELECT r.id, r.name, r.created_by, r.created_at, r.is_group \
         FROM rooms r \
         INNER JOIN room_members rm ON r.id = rm.room_id \
         WHERE rm.user_id = $1 \
         ORDER BY r.created_at DESC",
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        error!("Failed to list rooms: {}", error);
        ErrorResponse::InternalError("Failed to list rooms.".to_string())
    })?;

    let mut rooms = Vec::new();
    for row in &room_rows {
        let room_id: Uuid = row.get("id");

        let member_rows = sqlx::query("SELECT user_id FROM room_members WHERE room_id = $1")
            .bind(room_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|error| {
                error!("Failed to get room members: {}", error);
                ErrorResponse::InternalError("Failed to get room members.".to_string())
            })?;

        let members: Vec<Uuid> = member_rows.iter().map(|row| row.get("user_id")).collect();

        rooms.push(json!({
            "id": room_id,
            "name": row.get::<Option<String>, _>("name"),
            "created_by": row.get::<Uuid, _>("created_by"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
            "is_group": row.get::<bool, _>("is_group"),
            "members": members,
        }));
    }

    Ok(SuccessResponse::new(
        StatusCode::OK,
        "Rooms listed successfully.".to_string(),
        json!({ "rooms": rooms }),
    ))
}
