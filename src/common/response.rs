use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct ApiResponse {
    status: StatusCode,
    messages: String,
    data: Json<Value>,
    id: String,
}

impl ApiResponse {
    pub fn new(status: StatusCode, messages: String, data: Value) -> Self {
        Self {
            status,
            messages,
            data: Json(data),
            id: Uuid::now_v7().to_string(),
        }
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        let body = json!({
            "id": self.id,
            "status": self.status.as_u16(),
            "messages": self.messages,
            "data": *self.data,
        });

        (self.status, Json(body)).into_response()
    }
}
