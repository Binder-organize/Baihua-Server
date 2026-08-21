use axum::http::{HeaderName, HeaderValue};
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{Instrument, info_span};
use uuid::Uuid;

// Get the request ID and track it.
pub async fn tracing(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let span = info_span!("request", request_id = %request_id);

    let mut response = next.run(request).instrument(span).await;

    // Echo the request ID back so clients can correlate this response with
    // their request when reporting an issue.
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }

    response
}
