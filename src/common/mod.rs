pub mod error;
pub mod extractor;
pub mod success;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct StandardResponseBody {
    pub response_id: String,
    pub code: String,
    pub message: String,
    pub data: Option<Value>,
}

pub struct StandardResponse {
    pub status: StatusCode,
    pub body: StandardResponseBody,
}

impl StandardResponse {
    pub fn success(status_code: StatusCode, message: String, data: Value) -> Self {
        let body = StandardResponseBody {
            response_id: Uuid::now_v7().to_string(),
            code: "SUCCESS".to_string(),
            message,
            data: Some(data),
        };

        Self {
            status: status_code,
            body,
        }
    }

    pub fn error(status_code: StatusCode, error_code: String, message: String) -> Self {
        let body = StandardResponseBody {
            response_id: Uuid::now_v7().to_string(),
            code: error_code,
            message,
            data: None,
        };

        Self {
            status: status_code,
            body,
        }
    }
}

impl IntoResponse for StandardResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
