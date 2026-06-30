use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{Instrument, info_span};
use uuid::Uuid;

// Get the request ID and track it.
pub async fn tracing(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let span = info_span!("request", request_id = %request_id);

    next.run(request).instrument(span.clone()).await
}
