use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{ErrorDetails, ERROR_AUTH};

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    /// No Authorization header present
    MissingToken,
    /// Authorization header format is invalid (not "Bearer <token>")
    InvalidFormat,
    /// API key is invalid or expired
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let message = match self {
            AuthError::MissingToken => "Missing Authorization header",
            AuthError::InvalidFormat => {
                "Invalid Authorization header format. Expected: Bearer <token>"
            }
            AuthError::InvalidToken => "Invalid or expired API key",
        };

        let error_details = ErrorDetails {
            code: ERROR_AUTH,
            message: message.to_string(),
            data: None,
        };

        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "jsonrpc": "2.0",
                "error": error_details,
            })),
        )
            .into_response()
    }
}
