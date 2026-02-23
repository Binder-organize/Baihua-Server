use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

// fixme just a demo
pub async fn security(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    response
        .headers_mut()
        .insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    response.headers_mut().insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    response.headers_mut().insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'"),
    );

    response.headers_mut().insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}
