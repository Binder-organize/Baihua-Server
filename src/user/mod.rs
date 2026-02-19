mod login;
mod register;

use crate::ServerState;
use crate::common::error::ErrorType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::{self, Row};
use std::sync::Arc;
use tracing::info;
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

// todo these two structs can be optimized.
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

impl User {
    pub async fn new(
        new_user: UserRegister,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<User, ErrorType> {
        // Validate username, email, and password.
        if new_user.username.is_empty() || new_user.email.is_empty() || new_user.password.is_empty()
        {
            return Err(ErrorType::Validation(
                "Username, email, and password cannot be empty.".to_string(),
            ));
        }

        // Validate email format.
        // todo This verification is not rigorous.
        if !new_user.email.contains('@') || !new_user.email.contains('.') {
            return Err(ErrorType::Validation("Invalid email format.".to_string()));
        }

        // Check if username or email already exists.
        let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
            .bind(&new_user.username)
            .bind(&new_user.email)
            .fetch_optional(pool)
            .await
            .map_err(|e| ErrorType::InternalError(format!("Database query failed: {}", e)))?;

        if existing_user.is_some() {
            return Err(ErrorType::Validation(
                "Username or email already exists.".to_string(),
            ));
        }

        // Generate UUID and current time.
        let uuid = Uuid::now_v7();
        let created_at = Utc::now();

        // todo hash password

        // Insert user into database
        sqlx::query(
            "INSERT INTO users (id, username, email, password, created_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(uuid)
        .bind(&new_user.username)
        .bind(&new_user.email)
        .bind(&new_user.password)
        .bind(created_at)
        .bind(true)
        .execute(pool)
        .await
        .map_err(|e| {
            ErrorType::InternalError(format!("Failed to insert user: {}.", e))
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
    ) -> Result<Option<User>, ErrorType> {
        let row = sqlx::query(
            r#"SELECT id, username, email, password, nickname, phone_number, created_at, is_active
               FROM users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| ErrorType::InternalError(format!("Database query failed: {}.", e)))?;

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
    pub async fn find_user_password(
        username: &str,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Option<String>, ErrorType> {
        let row = sqlx::query("SELECT password FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(|e| ErrorType::InternalError(format!("Database query failed: {}.", e)))?;

        match row {
            Some(row) => Ok(Some(row.get("password"))),
            None => Ok(None),
        }
    }

    pub async fn verify_password(
        &self,
        password: String,
        username: &str,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> bool {
        match Self::find_user_password(username, pool).await {
            Ok(Some(stored_password)) => stored_password == password,
            _ => false,
        }
    }
}

pub fn router(state: Arc<ServerState>) -> axum::Router {
    axum::Router::new()
        .route("/register", axum::routing::post(register::register))
        .route("/login", axum::routing::post(login::login))
        .with_state(state)
}
