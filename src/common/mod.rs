pub mod error;
pub mod success;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub trait ApiResponse {
    fn to_response(self) -> StandardResponse;
}

#[derive(Serialize, Deserialize)]
pub struct StandardResponseBody {
    pub response_id: String,
    pub error_code: String,
    pub message: String,
    pub data: Option<Value>,
}

pub struct StandardResponse {
    pub status: StatusCode,
    pub response: StandardResponseBody,
}

impl StandardResponse {
    pub fn success(status_code: StatusCode, message: String, data: Value) -> Self {
        let response = StandardResponseBody {
            response_id: Uuid::now_v7().to_string(),
            error_code: "OK".to_string(),
            message,
            data: Some(data),
        };

        Self {
            status: status_code,
            response,
        }
    }

    pub fn error(status_code: StatusCode, error_code: String, message: String) -> Self {
        let response = StandardResponseBody {
            response_id: Uuid::now_v7().to_string(),
            error_code,
            message,
            data: None,
        };

        Self {
            status: status_code,
            response,
        }
    }
}
