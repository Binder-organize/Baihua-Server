use crate::ServerState;
use crate::authenticate::jsonwebtoken::validate_token;
use crate::chat::is_room_member;
use crate::common::error::ErrorResponse;
use crate::user::find_user_by_id;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, warn};
use uuid::Uuid;

// Client -> Server
const WS_TYPING: &str = "typing";
const WS_PONG: &str = "pong";

// Server -> Client
const WS_USER_ONLINE: &str = "user_online";
const WS_USER_OFFLINE: &str = "user_offline";
const WS_TYPING_INDICATOR: &str = "typing";
const WS_NEW_MESSAGE: &str = "new_message";
const WS_MESSAGE_SENT: &str = "message_sent";
const WS_ERROR: &str = "error";

// Client -> Server
const WS_SEND_MESSAGE: &str = "send_message";

#[derive(Debug, Deserialize)]
pub struct WebsocketQuery {
    pub token: String,
}

// HTTP handler that upgrades to WebSocket after JWT validation.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebsocketQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let claims = validate_token(&params.token, &state.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        error!("JWT sub claim is not a valid UUID: {}", claims.sub);
        ErrorResponse::Authentication("Invalid token.".to_string())
    })?;

    let user = find_user_by_id(user_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorResponse::Authentication("User not found or inactive.".to_string()))?;

    if !user.is_active {
        return Err(ErrorResponse::Authentication(
            "User is inactive.".to_string(),
        ));
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user)))
}

// Main WebSocket lifecycle handler.
async fn handle_socket(socket: WebSocket, state: Arc<ServerState>, user: crate::user::User) {
    let manager = &state.connection_manager;

    // Presence: mark user online.
    let just_came_online = manager.user_connected(user.id);

    // Query all rooms the user belongs to.
    let user_room_ids = get_user_room_ids(&state.pool, user.id).await;

    if just_came_online {
        let online_msg = json!({
            "type": WS_USER_ONLINE,
            "data": {
                "user_id": user.id,
                "username": user.username,
            }
        })
        .to_string();
        for &room_id in &user_room_ids {
            manager.broadcast(room_id, &online_msg);
        }
    }

    // Per-connection channels.
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<String>();

    // Split the WebSocket into sender + receiver halves.
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Track which rooms this connection is subscribed to.
    let mut subscribed_rooms: HashSet<Uuid> = HashSet::new();

    // Auto-subscribe: connect → immediately subscribe to all rooms.
    for &room_id in &user_room_ids {
        subscribe_to_room(user.id, room_id, &state, &msg_tx, &mut subscribed_rooms);
    }

    // Send a "connected" confirmation.
    let connected_msg = json!({
        "type": "connected",
        "data": {
            "user_id": user.id,
            "rooms": user_room_ids,
        }
    })
    .to_string();
    let _ = msg_tx.send(connected_msg);

    // Heartbeat: send WebSocket protocol-level PING every 30 seconds.
    // The client library auto-responds with PONG at the frame level.
    // If the send fails, the connection is dead and we break.
    let mut heartbeat = interval(Duration::from_secs(30));
    heartbeat.tick().await; // skip the immediate first tick

    // Main event loop
    loop {
        tokio::select! {
            // Outgoing: forward internal channel messages to WebSocket.
            msg = msg_rx.recv() => {
                match msg {
                    Some(text) => {
                        if ws_sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }

            // Incoming: messages from WebSocket client.
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match handle_incoming(
                            &state, &user, &text,
                        ).await {
                            Ok(Some(response)) => {
                                let _ = msg_tx.send(response);
                            }
                            Ok(None) => {}
                            Err(error) => {
                                let err_msg = json!({
                                    "type": WS_ERROR,
                                    "data": { "message": error.to_string() }
                                }).to_string();
                                let _ = msg_tx.send(err_msg);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }

            // Heartbeat: send PING every 30s. Client library auto-replies PONG.
            // Failing to send means the connection (or TCP) is dead.
            _ = heartbeat.tick() => {
                if ws_sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup
    // subscribed_rooms and their forward tasks are dropped here implicitly.

    let fully_offline = manager.user_disconnected(user.id);
    if fully_offline {
        let offline_msg = json!({
            "type": WS_USER_OFFLINE,
            "data": {
                "user_id": user.id,
                "username": user.username,
            }
        })
        .to_string();
        for &room_id in &user_room_ids {
            manager.broadcast(room_id, &offline_msg);
        }
    }
}

// Subscribe a connection to a room's broadcast channel.
// Spawns a background task that forwards messages from the room's
// broadcast channel to the connection's internal message channel.
// The task exits automatically when the user leaves the room
// (via `ConnectionManager::cancel_subscription`).
fn subscribe_to_room(
    user_id: Uuid,
    room_id: Uuid,
    state: &Arc<ServerState>,
    msg_tx: &mpsc::UnboundedSender<String>,
    subscribed_rooms: &mut HashSet<Uuid>,
) {
    if subscribed_rooms.contains(&room_id) {
        return;
    }

    let mut rx = state.connection_manager.subscribe(room_id);
    let mut cancel_rx = state
        .connection_manager
        .register_subscription(user_id, room_id);
    let forward_tx = msg_tx.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_rx.changed() => {
                    // User left this room; stop forwarding.
                    break;
                }
                msg = rx.recv() => {
                    match msg {
                        Ok(text) => {
                            if forward_tx.send(text).is_err() {
                                break;
                            }
                        }
                        Err(RecvError::Lagged(n)) => {
                            warn!(
                                "forward task lagged by {n} messages for user {user_id} in room {room_id}; continuing"
                            );
                            // Receiver is still valid — skipped messages are gone,
                            // but future messages will still arrive.
                        }
                        Err(RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    subscribed_rooms.insert(room_id);
}

// Process a JSON message received from the client.
// Returns Ok(None) for messages that need no response,
// Ok(Some(response)) for ack messages to send back,
// or Err(error) for errors to send back.
async fn handle_incoming(
    state: &Arc<ServerState>,
    user: &crate::user::User,
    text: &str,
) -> Result<Option<String>, ErrorResponse> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| ErrorResponse::Json(format!("Invalid WS message JSON: {}", e)))?;

    let msg_type = value.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
        ErrorResponse::Validation("Missing 'type' field in WS message.".to_string())
    })?;

    match msg_type {
        WS_TYPING => {
            let room_id_str = value
                .get("room_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ErrorResponse::Validation("Missing 'room_id' in typing.".to_string())
                })?;
            let room_id = Uuid::parse_str(room_id_str)
                .map_err(|_| ErrorResponse::Validation("Invalid room_id UUID.".to_string()))?;

            let typing_msg = json!({
                "type": WS_TYPING_INDICATOR,
                "data": {
                    "room_id": room_id,
                    "user_id": user.id,
                    "username": user.username,
                    "typing": true,
                }
            })
            .to_string();
            state.connection_manager.broadcast(room_id, &typing_msg);

            Ok(None)
        }

        WS_PONG => Ok(None),

        WS_SEND_MESSAGE => {
            let data = value.get("data").ok_or_else(|| {
                ErrorResponse::Validation("Missing 'data' field in send_message.".to_string())
            })?;

            let room_id_str = data
                .get("room_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ErrorResponse::Validation("Missing 'room_id' in send_message data.".to_string())
                })?;
            let room_id = Uuid::parse_str(room_id_str)
                .map_err(|_| ErrorResponse::Validation("Invalid room_id UUID.".to_string()))?;

            let content = data
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ErrorResponse::Validation("Missing 'content' in send_message data.".to_string())
                })?;

            validate_message_content(content)?;

            if !is_room_member(&state.pool, room_id, user.id).await? {
                return Err(ErrorResponse::Forbidden(
                    "You are not a member of this room.".to_string(),
                ));
            }

            let message_id = Uuid::now_v7();
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO messages (id, room_id, sender_id, content, created_at) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(message_id)
            .bind(room_id)
            .bind(user.id)
            .bind(content)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|error| {
                error!("Failed to insert message: {}", error);
                ErrorResponse::InternalError("Failed to send message.".to_string())
            })?;

            let ws_message = json!({
                "type": WS_NEW_MESSAGE,
                "data": {
                    "id": message_id,
                    "room_id": room_id,
                    "sender_id": user.id,
                    "content": content,
                    "created_at": now.to_rfc3339(),
                }
            })
            .to_string();
            state.connection_manager.broadcast(room_id, &ws_message);

            let ack = json!({
                "type": WS_MESSAGE_SENT,
                "data": {
                    "id": message_id,
                    "room_id": room_id,
                    "sender_id": user.id,
                    "content": content,
                    "created_at": now.to_rfc3339(),
                }
            })
            .to_string();

            Ok(Some(ack))
        }

        _ => Err(ErrorResponse::Validation(format!(
            "Unknown WS message type: '{}'.",
            msg_type
        ))),
    }
}

// Query all room IDs the user is a member of.
async fn get_user_room_ids(pool: &sqlx::PgPool, user_id: Uuid) -> Vec<Uuid> {
    let result =
        sqlx::query_scalar::<_, Uuid>("SELECT room_id FROM room_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await;

    result.unwrap_or_else(|error| {
        error!("Failed to query user rooms: {}", error);
        vec![]
    })
}

// Validate chat message content.
// Rejects empty/whitespace-only content and content exceeding 64 KB.
fn validate_message_content(content: &str) -> Result<(), ErrorResponse> {
    if content.trim().is_empty() {
        return Err(ErrorResponse::Validation(
            "Message content cannot be empty.".to_string(),
        ));
    }

    if content.len() > 65536 {
        return Err(ErrorResponse::Validation(
            "Message content exceeds 65536 bytes.".to_string(),
        ));
    }

    Ok(())
}
