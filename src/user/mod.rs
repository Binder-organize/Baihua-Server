mod register;

use anyhow::Result;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    username: String,
    email: String,
    // Password encryption is done on the client side.
    password: String,
    // Optional and repeatable user nickname.
    nickname: Option<String>,
    phone_number: Option<String>,
    created_at: String,
    is_active: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewUser {
    username: String,
    email: String,
    password: String,
}

impl User {
    pub fn new(new_user: NewUser) -> Result<User> {
        if new_user.username.is_empty() || new_user.email.is_empty() || new_user.password.is_empty()
        {
            return Err(anyhow::anyhow!("Invalid user data.").context(StatusCode::BAD_REQUEST));
        }

        // todo Database read and write sections.

        info!("New user: {}  is created.", new_user.username);
        Ok(User {
            username: new_user.username,
            email: new_user.email,
            password: new_user.password,
            nickname: None,
            phone_number: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_active: true,
        })
    }
}

pub fn router() -> axum::Router {
    axum::Router::new().route("/register", axum::routing::post(register::register))
}
