use confy::ConfyError;
use rusqlite;
use std::env::VarError;

/// Custom error type for the application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from SQLite database operations.
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// Error from configuration operations.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfyError),
    /// Error from environment variables.
    #[error("Environment variable error: {0}")]
    Env(#[from] VarError),
    /// Error for missing contact.
    #[error("No contact specified")]
    NoContact,
    /// Generic error with message.
    #[error("{0}")]
    Generic(String),
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;
