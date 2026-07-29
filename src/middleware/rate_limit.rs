use crate::ServerState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub(crate) struct SlidingWindowRateLimiter {
    max_requests: u32,
    window_secs: u64,
    inner: RwLock<HashMap<IpAddr, Vec<Instant>>>,
}

impl SlidingWindowRateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            inner: RwLock::new(HashMap::new()),
        }
    }

    // Returns true if the request should be allowed, false if rate limited.
    pub(crate) async fn allow(&self, ip: IpAddr) -> bool {
        let mut map = self.inner.write().await;
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let entries = map.entry(ip).or_default();

        // Prune timestamps outside the window.
        entries.retain(|&t| now.duration_since(t) < window);

        if entries.len() >= self.max_requests as usize {
            return false;
        }

        entries.push(now);
        true
    }
}

fn extract_client_ip(request: &Request) -> Option<IpAddr> {
    // Priority 1: X-Forwarded-For (standard reverse proxy header).
    if let Some(value) = request.headers().get("x-forwarded-for")
        && let Ok(value) = value.to_str()
        && let Some(ip_str) = value.split(',').next().map(|s| s.trim())
        && let Ok(ip) = ip_str.parse::<IpAddr>()
    {
        return Some(ip);
    }

    // Priority 2: X-Real-IP (common nginx header).
    if let Some(value) = request.headers().get("x-real-ip")
        && let Ok(value) = value.to_str()
        && let Ok(ip) = value.parse::<IpAddr>()
    {
        return Some(ip);
    }

    None
}

pub async fn rate_limit_login(
    state: State<Arc<ServerState>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(ip) = extract_client_ip(&request)
        && !state.login_rate_limiter.allow(ip).await
    {
        let response = crate::common::StandardResponse::error(
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_ERROR".to_string(),
            "Too many login attempts. Please try again later.".to_string(),
        );
        return (response.status, axum::Json(response.response)).into_response();
    }

    next.run(request).await
}

pub async fn rate_limit_register(
    state: State<Arc<ServerState>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(ip) = extract_client_ip(&request)
        && !state.register_rate_limiter.allow(ip).await
    {
        let response = crate::common::StandardResponse::error(
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_ERROR".to_string(),
            "Too many registration attempts. Please try again later.".to_string(),
        );
        return (response.status, axum::Json(response.response)).into_response();
    }

    next.run(request).await
}
