use thiserror::Error;

#[derive(Error, Debug)]
/// Error relating to file-history
pub enum HistoryError {
    /// Represents std::io::Error
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Represents rusqlite::Error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Represents a generic error
    #[error("{0}")]
    Generic(String),
}
