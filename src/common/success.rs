use crate::common::{ApiResponse, StandardResponse};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::Value;

pub struct SuccessResponse {
    status: StatusCode,
    messages: String,
    data: Value,
}

impl SuccessResponse {
    pub fn new(status: StatusCode, messages: String, data: Value) -> Self {
        Self {
            status,
            messages,
            data,
        }
    }
}

impl ApiResponse for SuccessResponse {
    fn to_response(self) -> StandardResponse {
        StandardResponse::success(self.status, self.messages, self.data)
    }
}

impl IntoResponse for SuccessResponse {
    fn into_response(self) -> axum::response::Response {
        let response = self.to_response();
        (response.status, Json(response.response)).into_response()
    }
}
