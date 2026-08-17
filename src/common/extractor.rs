use axum::Json;
use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use serde::de::DeserializeOwned;

use crate::common::error::ErrorResponse;

// JSON body extractor that converts axum rejections into ErrorResponse.
// Removes per-handler `map_err` boilerplate and routes every JSON
// parsing failure through the unified error channel.
pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ErrorResponse;

    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(map_json_rejection)?;
        Ok(JsonBody(value))
    }
}

fn map_json_rejection(rejection: JsonRejection) -> ErrorResponse {
    match rejection {
        JsonRejection::BytesRejection(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            ErrorResponse::PayloadTooLarge("The request body exceeds the allowed size.".to_string())
        }
        other => ErrorResponse::Json(other.to_string()),
    }
}

// todo extract database errors
