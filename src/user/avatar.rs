use super::profile;
use crate::ServerState;
use crate::common::StandardResponse;
use crate::common::error::ErrorResponse;
use crate::middleware::authenticate::AuthenticatedUser;
use axum::Extension;
use axum::extract::multipart::{Field, Multipart};
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::Row;
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

const AVATAR_URL_PREFIX: &str = "/static/avatars/";

fn extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

// Upload an avatar image file, store it on the local filesystem and point
// the user's avatar column at the public static URL serving that file.
pub async fn upload_avatar(
    State(state): State<Arc<ServerState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> Result<StandardResponse, ErrorResponse> {
    // Locate the 'file' part among the multipart fields. The field must be
    // processed inside the loop because a Field borrows the Multipart stream
    // mutably for its whole lifetime.
    while let Some(mut field) = multipart.next_field().await.map_err(|err| {
        ErrorResponse::BadRequest(format!("Failed to read multipart field: {}.", err))
    })? {
        if field.name() == Some("file") {
            return process_avatar_field(&state, &auth_user, &mut field).await;
        }
    }

    Err(ErrorResponse::Validation(
        "Upload field 'file' is required.".to_string(),
    ))
}

async fn process_avatar_field(
    state: &Arc<ServerState>,
    auth_user: &AuthenticatedUser,
    field: &mut Field<'_>,
) -> Result<StandardResponse, ErrorResponse> {
    let content_type = match field.content_type() {
        Some(content_type) => content_type,
        None => {
            return Err(ErrorResponse::Validation(
                "Avatar must be a JPEG, PNG, GIF or WebP image.".to_string(),
            ));
        }
    };
    let extension = match extension_for_content_type(content_type) {
        Some(extension) => extension,
        None => {
            return Err(ErrorResponse::Validation(
                "Avatar must be a JPEG, PNG, GIF or WebP image.".to_string(),
            ));
        }
    };

    // Stream the field in chunks so oversized uploads are rejected while
    // reading instead of buffering the whole body before the size check.
    let mut bytes: Vec<u8> = Vec::new();
    let mut total: usize = 0;
    loop {
        let chunk = field.chunk().await.map_err(|err| {
            ErrorResponse::BadRequest(format!("Failed to read multipart field: {}.", err))
        })?;
        let Some(chunk) = chunk else {
            break;
        };
        total += chunk.len();
        if total > state.configuration.avatar.max_bytes as usize {
            return Err(ErrorResponse::PayloadTooLarge(format!(
                "Avatar file exceeds the maximum allowed size of {} bytes.",
                state.configuration.avatar.max_bytes
            )));
        }
        bytes.extend_from_slice(&chunk);
    }

    if bytes.is_empty() {
        return Err(ErrorResponse::Validation(
            "Avatar file cannot be empty.".to_string(),
        ));
    }

    let filename = format!("{}.{}", Uuid::now_v7(), extension);

    // A previously uploaded avatar is removed only after the new file is
    // safely written, so a failed upload never leaves the user without one.
    let previous_avatar = sqlx::query("SELECT avatar FROM users WHERE id = $1")
        .bind(auth_user.user_id)
        .fetch_one(&state.pool)
        .await?
        .get::<Option<String>, _>("avatar");

    let old_filename = previous_avatar
        .as_deref()
        .and_then(|avatar| avatar.strip_prefix(AVATAR_URL_PREFIX))
        .map(str::to_string);

    tokio::fs::write(state.avatars_directory.join(&filename), &bytes)
        .await
        .map_err(|err| {
            ErrorResponse::InternalError(format!("Failed to save avatar file: {}.", err))
        })?;

    let avatar_url = format!("{}{}", AVATAR_URL_PREFIX, filename);
    sqlx::query("UPDATE users SET avatar = $1 WHERE id = $2")
        .bind(&avatar_url)
        .bind(auth_user.user_id)
        .execute(&state.pool)
        .await?;

    if let Some(old_filename) = old_filename
        && let Err(err) = tokio::fs::remove_file(state.avatars_directory.join(&old_filename)).await
    {
        warn!(
            "Failed to delete the previous avatar file '{}': {}.",
            old_filename, err
        );
    }

    profile::respond_with_full_user(
        state,
        auth_user,
        "Avatar uploaded successfully.".to_string(),
    )
    .await
}

// Serve an uploaded avatar file by its unguessable UUID filename. Public
// because <img> tags cannot attach Authorization headers.
pub async fn serve_avatar_file(
    State(state): State<Arc<ServerState>>,
    Path(filename): Path<String>,
) -> Result<Response, ErrorResponse> {
    lazy_static! {
        static ref AVATAR_FILENAME_REGEX: Regex = Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.(jpg|png|gif|webp)$"
        )
        .expect("Avatar filename regex is a compile-time constant");
    }

    // The strict pattern also rules out ".." path traversal segments.
    let Some(captures) = AVATAR_FILENAME_REGEX.captures(&filename) else {
        return Err(ErrorResponse::NotFound(
            "Avatar file not found.".to_string(),
        ));
    };

    let content_type = match &captures[1] {
        "jpg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => {
            return Err(ErrorResponse::NotFound(
                "Avatar file not found.".to_string(),
            ));
        }
    };

    let bytes = match tokio::fs::read(state.avatars_directory.join(&filename)).await {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(ErrorResponse::NotFound(
                "Avatar file not found.".to_string(),
            ));
        }
        Err(err) => {
            return Err(ErrorResponse::InternalError(format!(
                "Failed to read avatar file: {}.",
                err
            )));
        }
    };

    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}
