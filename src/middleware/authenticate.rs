use crate::authenticate::jsonwebtoken::{extract_token_from_header, validate_token};
use crate::common::error::ErrorResponse;
use axum::http::HeaderMap;
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::error;

// todo remove it.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub access_token: String,
}

#[allow(dead_code)]
pub async fn authenticate(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let authenticate_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ErrorResponse::Authentication(
            "Missing authorization header.".to_string(),
        ))?;

    let token = extract_token_from_header(authenticate_header)?;

    let secret = std::env::var("JWT_SECRET");
    if secret.is_err() {
        error!("Missing JWT_SECRET environment variable.");

        return Err(ErrorResponse::InternalError(
            "Missing JWT_SECRET environment variable.".to_string(),
        ));
    }

    let claims = validate_token(token, &(secret.unwrap()))?;

    let user_id = claims.sub;

    let mut request = request;
    request.extensions_mut().insert(AuthenticatedUser {
        user_id,
        access_token: token.to_string(),
    });

    Ok(next.run(request).await)
}
