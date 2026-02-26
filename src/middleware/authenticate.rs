use crate::authenticate::jsonwebtoken::{extract_token_from_header, validate_token};
use crate::common::error::ErrorResponse;
use axum::http::HeaderMap;
use axum::{extract::Request, middleware::Next, response::Response};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub access_token: String,
}

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
    let claims = validate_token(token)?;

    let user_id = claims.sub;

    let mut request = request;
    request.extensions_mut().insert(AuthenticatedUser {
        user_id,
        access_token: token.to_string(),
    });

    Ok(next.run(request).await)
}
