use std::borrow::Cow;

use super::ErrorResponse;

pub enum ChatError {
    ModelNotSupported(String),
    EmptyMessages,
    NoTokens,
    RequestFailed(String),
    Unauthorized,
}

impl ChatError {
    pub fn to_json(&self) -> ErrorResponse {
        let (error, message) = match self {
            ChatError::ModelNotSupported(model) => (
                "model_not_supported",
                format!("Model '{model}' is not supported"),
            ),
            ChatError::EmptyMessages => (
                "empty_messages",
                "Message array cannot be empty".to_string(),
            ),
            ChatError::NoTokens => ("no_tokens", "No available tokens".to_string()),
            ChatError::RequestFailed(err) => ("request_failed", format!("Request failed: {err}")),
            ChatError::Unauthorized => ("unauthorized", "Invalid authorization token".to_string()),
        };

        ErrorResponse {
            status: super::ApiStatus::Error,
            code: None,
            error: Some(Cow::Borrowed(error)),
            message: Some(Cow::Owned(message)),
        }
    }
}
