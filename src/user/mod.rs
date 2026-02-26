mod login;
mod register;

use crate::ServerState;
use crate::common::error::ErrorResponse;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::{self, Row};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    // Primary key.
    pub id: Uuid,
    pub username: String,
    pub email: String,
    // Optional and repeatable user nickname.
    pub nickname: Option<String>,
    pub phone_number: Option<String>,
    // UTC datetime.
    #[serde_as(as = "DisplayFromStr")]
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

impl UserRegister {
    pub async fn validate(&self) -> Result<(), ErrorResponse> {
        // Validate the regular expression of the mailbox.
        lazy_static! {
            static ref EMAIL_REGEX: Result<Regex, String> = Regex::new(
                r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])*(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])*)*$"
            ).map_err(|error| error.to_string());
        }

        // Validate username, email, and password.
        if self.username.is_empty() || self.email.is_empty() || self.password.is_empty() {
            return Err(ErrorResponse::Validation(
                "Username, email, and password cannot be empty.".to_string(),
            ));
        }

        // Email regex.
        match EMAIL_REGEX.as_ref() {
            Ok(regex) => {
                if !regex.is_match(&self.email) {
                    return Err(ErrorResponse::Validation(
                        "Invalid email format. Please provide a valid email address.".to_string(),
                    ));
                }
            }
            Err(error_msg) => {
                error!("Regex compilation failed: {}", error_msg);
                return Err(ErrorResponse::InternalError(
                    "Failed to compile email regex.".to_string(),
                ));
            }
        }

        // Check the mailbox length.
        if self.email.len() > 254 {
            return Err(ErrorResponse::Validation(
                "Email address is too long (maximum 254 characters).".to_string(),
            ));
        }

        // Check username.
        if self.username.len() <= 3 || self.username.len() >= 40 {
            return Err(ErrorResponse::Validation(
                "Username must be between 4 and 40 characters long.".to_string(),
            ));
        }

        // An easter egg.
        if self.username == "Gavin" || self.username == "gavin" {
            return Err(ErrorResponse::Validation(
                "Username cannot be 'Gavin' or 'gavin'.".to_string(),
            ));
        }

        if self.email == "gav.zheng@outlook.com" {
            return Err(ErrorResponse::Validation(
                "Email cannot be 'gav.zheng@outlook.com'.".to_string(),
            ));
        }

        // todo Complete the email verification function.

        Ok(())
    }
}

pub async fn new_user(
    new_user: UserRegister,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<User, ErrorResponse> {
    // Validate the user.
    new_user.validate().await?;

    // Check if username or email already exists.
    let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
        .bind(&new_user.username)
        .bind(&new_user.email)
        .fetch_optional(pool)
        .await
        .map_err(|error| {
            ErrorResponse::InternalError(format!("Database query failed: {}", error))
        })?;

    if existing_user.is_some() {
        return Err(ErrorResponse::Validation(
            "Username or email already exists.".to_string(),
        ));
    }

    // Generate UUID and current time.
    let uuid = Uuid::now_v7();
    let created_at = Utc::now();

    // Hash password.
    let password_hashed = hash(new_user.password, DEFAULT_COST)
        .map_err(|e| ErrorResponse::InternalError(format!("Hash password failed: {}.", e)))?;

    // Insert user into database.
    sqlx::query(
        "INSERT INTO users (id, username, email, password, created_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(uuid)
    .bind(&new_user.username)
    .bind(&new_user.email)
    .bind(&password_hashed)
    .bind(created_at)
    .bind(true)
    .execute(pool)
    .await
    .map_err(|e| {
        ErrorResponse::InternalError(format!("Failed to insert user: {}.", e))
    })?;

    info!(
        "New user: {} is created, id is: {}.",
        new_user.username, &uuid
    );

    Ok(User {
        id: uuid,
        username: new_user.username,
        email: new_user.email,
        nickname: None,
        phone_number: None,
        created_at,
        is_active: true,
    })
}

// Find user by username.
pub async fn find_user_by_username(
    username: &str,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Option<User>, ErrorResponse> {
    let row = sqlx::query(
        r#"SELECT id, username, email, password, nickname, phone_number, created_at, is_active
           FROM users WHERE username = $1"#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErrorResponse::InternalError(format!("Database query failed: {}.", e)))?;

    match row {
        Some(row) => Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            nickname: row.get("nickname"),
            phone_number: row.get("phone_number"),
            created_at: row.get("created_at"),
            is_active: row.get("is_active"),
        })),
        None => Ok(None),
    }
}

// Find user password.
// If it is not necessary, it will not be read.
async fn find_user_password(
    username: &str,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Option<String>, ErrorResponse> {
    let row = sqlx::query("SELECT password FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| ErrorResponse::InternalError(format!("Database query failed: {}.", e)))?;

    match row {
        Some(row) => Ok(Some(row.get("password"))),
        None => Ok(None),
    }
}

// Verify password.
pub async fn verify_password(
    password: &str,
    username: &str,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<bool, ErrorResponse> {
    match find_user_password(username, pool).await {
        Ok(Some(hashed_password)) => verify(password, &hashed_password).map_err(|error| {
            ErrorResponse::BadRequest(format!("Failed to verify password: {}.", error))
        }),
        Ok(None) => Err(ErrorResponse::BadRequest(
            "Incorrect username or password.".to_string(),
        )),
        Err(error) => Err(ErrorResponse::BadRequest(format!(
            "Failed to find user password: {}.",
            error
        ))),
    }
}

pub fn router(state: Arc<ServerState>) -> axum::Router {
    axum::Router::new()
        .route("/register", axum::routing::post(register::register))
        .route("/login", axum::routing::post(login::login))
        .with_state(state)
}
