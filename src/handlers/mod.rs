pub mod create;
pub mod retrieve;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("File operation error: {0}")]
    FileOperation(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Failed to parse form: {0}")]
    Form(String),
}

impl axum::response::IntoResponse for BlogError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", self),
        )
            .into_response()
    }
}
