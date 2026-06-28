use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new("INVALID_INPUT", message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new("UNAVAILABLE", message)
    }

    pub fn command_failed(message: impl Into<String>) -> Self {
        Self::new("COMMAND_FAILED", message)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::new("IO_ERROR", value.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::new("DATABASE_ERROR", value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::new("JSON_ERROR", value.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(value: tauri::Error) -> Self {
        Self::new("TAURI_ERROR", value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
