use crate::common::error::ErrorResponse;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

pub async fn generate_token(
    secret: &str,
    expiration_hours: u32,
    user_id: &str,
) -> Result<String, ErrorResponse> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiration_hours as i64))
        .ok_or_else(|| {
            warn!("Error calculating token expiration time.");
            ErrorResponse::InternalError("Failed to calculate token expiration time.".to_string())
        })?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        iat: Utc::now().timestamp(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|error| {
        warn!("Error generating token: {:?}.", error);
        ErrorResponse::InternalError(format!("Failed to generate token: {}.", error))
    })
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, ErrorResponse> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
    .map_err(|error| {
        warn!("Token validation failed: {}.", error);
        ErrorResponse::Authentication("Invalid or expired token.".to_string())
    })
}

pub fn extract_token_from_header(auth_header: &str) -> Result<&str, ErrorResponse> {
    if auth_header.is_empty() {
        return Err(ErrorResponse::Authentication(
            "Authorization header is missing.".to_string(),
        ));
    }

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorResponse::Authentication(
            "Authorization header must start with 'Bearer '.".to_string(),
        ));
    }

    let token = auth_header.trim_start_matches("Bearer ");
    if token.is_empty() {
        return Err(ErrorResponse::Authentication(
            "Token is missing in Authorization header.".to_string(),
        ));
    }

    Ok(token)
}
