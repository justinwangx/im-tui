use std::error::Error as StdError;
use std::fmt;

/// Custom error type for the application.
#[derive(Debug)]
pub enum Error {
    /// Error from SQLite database operations.
    Database(rusqlite::Error),
    /// Error from configuration operations.
    Config(confy::ConfyError),
    /// Error from environment variables.
    Env(std::env::VarError),
    /// Error for missing contact.
    NoContact,
    /// Generic error with message.
    Generic(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::Config(e) => write!(f, "Configuration error: {}", e),
            Error::Env(e) => write!(f, "Environment error: {}", e),
            Error::NoContact => write!(
                f,
                "No contact configured. Please set one using: gf --set <contact>"
            ),
            Error::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Database(e) => Some(e),
            Error::Config(e) => Some(e),
            Error::Env(e) => Some(e),
            Error::NoContact => None,
            Error::Generic(_) => None,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::Database(error)
    }
}

impl From<confy::ConfyError> for Error {
    fn from(error: confy::ConfyError) -> Self {
        Error::Config(error)
    }
}

impl From<std::env::VarError> for Error {
    fn from(error: std::env::VarError) -> Self {
        Error::Env(error)
    }
}

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;
