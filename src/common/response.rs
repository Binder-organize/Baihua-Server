use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub struct Response {
    pub status: StatusCode,
    pub body: Json<serde_json::Value>,
}

impl Response {
    pub fn new(status: StatusCode, body: serde_json::Value) -> Self {
        Self {
            status,
            body: Json(body),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.body).into_response()
    }
}
